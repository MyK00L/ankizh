use crate::anki::*;
use enum_dispatch::enum_dispatch;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

fn catch_unwind_silent<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
    f: F,
) -> std::thread::Result<R> {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(f);
    std::panic::set_hook(prev_hook);
    result
}
pub fn process_pinyin(s: &str) -> String {
    let s = prettify_pinyin::prettify(s);
    catch_unwind_silent(|| {
        let parser = pinyin_parser::PinyinParser::new()
            .preserve_spaces(false)
            .preserve_punctuations(true)
            .with_strictness(pinyin_parser::Strictness::Loose)
            .preserve_miscellaneous(true);
        parser
            .parse(&s)
            .reduce(|acc, s| acc + &s)
            .unwrap_or_default()
    })
    .unwrap_or(s)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Definition {
    pub pinyin: Option<String>,
    pub english: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Priority {
    pub val: NotNan<f32>,
    pub max: NotNan<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CharWriting {
    Strokes(Vec<String>),
    Char(char),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordEntry {
    pub id: String,
    pub pinyin: Vec<String>,
    pub definitions: Vec<Definition>,
    pub freq: Vec<NotNan<f32>>,
    pub hsk_lev: Option<u8>,
    pub dependencies: Vec<EntryId>,
    pub writing: Vec<CharWriting>,
    pub traditional: Option<String>,
    pub audio_file: Option<std::path::PathBuf>,
    pub examples: Vec<Triplet>,
}
impl WordEntry {
    pub fn from_id(id: String) -> Self {
        WordEntry {
            id: id.clone(),
            pinyin: vec![],
            definitions: vec![],
            freq: vec![],
            hsk_lev: None,
            dependencies: vec![],
            writing: id.chars().map(CharWriting::Char).collect(),
            traditional: None,
            audio_file: None,
            examples: vec![],
        }
    }

    pub fn total_priority(&self) -> NotNan<f32> {
        let freq: NotNan<f32> = self.freq.iter().sum();
        let hsk_lev = self.hsk_lev.unwrap_or(10);

        let hp = NotNan::new((10 - hsk_lev) as f32 / 10f32).unwrap();
        let fp = (NotNan::new(freq.log2()).unwrap() + NotNan::new(16f32).unwrap())
            .max(NotNan::new(0f32).unwrap())
            / 16f32;

        hp * 0.5 + fp * 0.5
    }
    fn merge_inner(&mut self, mut o: Self) {
        assert_eq!(self.id, o.id);
        for py in o.pinyin {
            let py = process_pinyin(&py);
            if !self.pinyin.contains(&py) {
                self.pinyin.push(py);
            }
        }
        for dp in o.dependencies {
            if !self.dependencies.contains(&dp) {
                self.dependencies.push(dp);
            }
        }
        self.definitions.append(&mut o.definitions);
        self.freq.append(&mut o.freq);

        for (a, b) in self.writing.iter_mut().zip(o.writing.into_iter()) {
            if let CharWriting::Char(_) = a {
                *a = b;
            }
        }

        self.traditional = self.traditional.take().or(o.traditional);
        self.audio_file = self.audio_file.take().or(o.audio_file);
        self.hsk_lev = self.hsk_lev.take().or(o.hsk_lev);
    }
    fn is_missing_some_writing(&self) -> bool {
        self.writing.len() != self.id.chars().count()
            || self
                .writing
                .iter()
                .any(|x| matches!(x, CharWriting::Char(_)))
    }
    fn is_missing_all_writing(&self) -> bool {
        !self
            .writing
            .iter()
            .any(|x| matches!(x, CharWriting::Strokes(_)))
    }
}
impl Entry for WordEntry {
    fn priority(&self) -> NotNan<f32> {
        self.total_priority()
    }
    fn into_note(self, idx: usize) -> genanki_rs::Note {
        genanki_rs::Note::new(
            WORD_MODEL.clone(),
            vec![
                &format!("{}", idx),
                &self.id,
                &format!("{:?}", self.pinyin),
                &format!("{:?}", self.definitions),
                &format!("{:?}", self.writing),
                &format!("{:?}", self.traditional),
                &format!("{:?}", self.examples),
                &format!("{:?}", self.hsk_lev),
                &format!("{:?}", self.audio_file),
                &format!("{:?}", "extra"),
            ],
        )
        .unwrap()
    }
    fn id(&self) -> EntryId {
        EntryId::Word(self.id.clone())
    }
    fn dependencies(&self) -> Vec<EntryId> {
        let mut deps = self.dependencies.clone();
        if self.id.chars().count() > 1 {
            for c in self.id.chars() {
                let cs: EntryId = EntryId::Word(c.into());
                if !deps.contains(&cs) {
                    deps.push(cs);
                }
            }
        }
        deps
    }
    fn merge(&mut self, o: CommonEntry) {
        match o {
            CommonEntry::WordEntry(we) => self.merge_inner(we),
            _ => unreachable!(),
        }
    }
    fn compact_display(&self) -> String {
        if self.is_missing_some_writing() {
            format!("W {}:{} !MissingWriting!", self.id, self.priority())
        } else {
            format!("W {}:{}", self.id, self.priority())
        }
    }
    fn to_delete(&self) -> bool {
        ((!self.id.chars().any(is_good_cjk))
            || if self.id.chars().count() == 1 {
                (self.is_missing_all_writing() && self.definitions.is_empty())
            } else {
                self.definitions.is_empty()
            })
            && self.hsk_lev.is_none()
    }
}
pub fn is_good_cjk(c: char) -> bool {
    let cp: u32 = c.into();
    (0x4E00..=0x9FFF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp)
        || (0x20000..=0x2A6DF).contains(&cp)
        || (0x2A700..=0x2B73F).contains(&cp)
        || (0x2B740..=0x2B81F).contains(&cp)
        || (0x2B820..=0x2CEAF).contains(&cp)
        || (0x2CEB0..=0x2EBEF).contains(&cp)
        || (0x2EBF0..=0x2EE5F).contains(&cp)
        || (0x2F800..=0x2FA1F).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0x2F800..=0x2FA1F).contains(&cp)
        || (0x2E80..=0x2EFF).contains(&cp)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyllableEntry {
    /// pinyin
    pub id: String,
    pub audio_file: std::path::PathBuf,
}
impl Entry for SyllableEntry {
    fn priority(&self) -> NotNan<f32> {
        NotNan::new(10f32).unwrap()
    }
    fn into_note(self, idx: usize) -> genanki_rs::Note {
        todo!()
    }
    fn id(&self) -> EntryId {
        EntryId::Syllable(self.id.clone())
    }
    fn dependencies(&self) -> Vec<EntryId> {
        vec![]
    }
    fn merge(&mut self, _o: CommonEntry) {
        unimplemented!()
    }
    fn compact_display(&self) -> String {
        format!("S {}:{}", self.id, self.priority())
    }
    fn to_delete(&self) -> bool {
        false
    }
}

use std::sync::LazyLock;
pub static JIEBA: LazyLock<jieba_rs::Jieba> = LazyLock::new(jieba_rs::Jieba::new);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Triplet {
    pub zh: String,
    pub en: String,
    pub py: String,
}
impl Triplet {
    fn dependencies(&self) -> Vec<EntryId> {
        let words = JIEBA.cut(&self.zh, false);
        words
            .into_iter()
            .map(|x| EntryId::Word(x.to_owned()))
            .collect()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrammarEntry {
    pub id: String,
    pub structure: Triplet,
    pub example: Triplet,
    pub hsk_lev: Option<u8>,
    /// 0 for first entry, 1 for last entry
    pub hsk_sublev: Option<f32>,
}
impl Entry for GrammarEntry {
    fn priority(&self) -> NotNan<f32> {
        let hsk_lev = self.hsk_lev.unwrap_or(10);
        let hp = NotNan::new((10 - hsk_lev) as f32 / 10f32).unwrap();
        let fp = 1f32 - self.hsk_sublev.unwrap_or(1f32);
        hp * 0.5 + hp * fp * 0.25
    }
    fn into_note(self, idx: usize) -> genanki_rs::Note {
        todo!()
    }
    fn id(&self) -> EntryId {
        EntryId::Grammar(self.id.clone())
    }
    fn dependencies(&self) -> Vec<EntryId> {
        let mut a = self.structure.dependencies();
        a.extend(self.example.dependencies());
        a
    }
    fn merge(&mut self, o: CommonEntry) {
        unimplemented!()
    }
    fn compact_display(&self) -> String {
        format!("G {}:{}", self.example.zh, self.priority())
    }
    fn to_delete(&self) -> bool {
        false
    }
}

#[allow(clippy::enum_variant_names)]
#[enum_dispatch(Entry)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommonEntry {
    WordEntry,
    SyllableEntry,
    GrammarEntry,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntryId {
    Word(String),
    Syllable(String),
    Grammar(String),
}
#[enum_dispatch]
pub trait Entry {
    /// Higher priority means it should come earlier in the deck
    fn priority(&self) -> NotNan<f32>;
    fn into_note(self, idx: usize) -> genanki_rs::Note;
    fn id(&self) -> EntryId;
    fn dependencies(&self) -> Vec<EntryId>;
    fn merge(&mut self, o: CommonEntry);
    fn compact_display(&self) -> String;
    fn to_delete(&self) -> bool;
}

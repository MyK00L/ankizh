use crate::pinyin_type::{CapPinyin, Pinyin};
use crate::utils::*;
use enum_dispatch::enum_dispatch;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Definition {
    pub pinyin: Option<CapPinyin>,
    pub english: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Priority {
    pub val: NotNan<f32>,
    pub max: NotNan<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stroke {
    pub path: String,
    pub start: (i32, i32),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CharWriting {
    Strokes(Vec<Stroke>),
    Char(char),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordEntry {
    pub id: String,
    pub pinyin: Vec<Pinyin>,
    pub definitions: Vec<Definition>,
    pub simple_definitions: Vec<String>,
    pub freq: Vec<NotNan<f32>>,
    pub hsk_lev: Option<u8>,
    pub dependencies: Vec<EntryId>,
    pub writing: Vec<CharWriting>,
    pub traditional: Option<String>,
    pub audio_file: Option<std::path::PathBuf>,
    pub examples: Vec<Triplet>,
    pub extra: Vec<String>,
}
impl WordEntry {
    pub fn from_id(id: String) -> Self {
        WordEntry {
            id: id.clone(),
            pinyin: vec![],
            definitions: vec![],
            simple_definitions: vec![],
            freq: vec![],
            hsk_lev: None,
            dependencies: vec![],
            writing: id.chars().map(CharWriting::Char).collect(),
            traditional: None,
            audio_file: None,
            examples: vec![],
            extra: vec![],
        }
    }
    pub fn first_definition(&self) -> Option<String> {
        let py: CapPinyin = self.pinyin[0].clone().into();
        self.definitions
            .iter()
            .find_map(|x| {
                if x.pinyin.as_ref() == Some(&py) {
                    x.english[0]
                        .as_str()
                        .split(';')
                        .next()
                        .map(|x| x.to_owned())
                } else {
                    None
                }
            })
            .or_else(|| {
                if self.definitions.is_empty() {
                    None
                } else {
                    self.definitions[0].english[0]
                        .as_str()
                        .split(';')
                        .next()
                        .map(|x| x.to_owned())
                }
            })
    }
    pub fn simple_english(&self) -> Option<String> {
        let first = self.first_definition();
        let defs =
            first
                .iter()
                .chain(self.simple_definitions.iter())
                .fold(String::new(), |acc, e| {
                    let sep = if !acc.is_empty() { " | " } else { "" };
                    acc + sep + e
                });
        let defs = defs.trim();
        if defs.is_empty() {
            None
        } else {
            let num = self.id.chars().count();
            Some(format!("({}) {}", num, defs))
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
        self.simple_definitions.append(&mut o.simple_definitions);
        self.freq.append(&mut o.freq);
        self.examples.append(&mut o.examples);
        self.extra.append(&mut o.extra);

        for (a, b) in self.writing.iter_mut().zip(o.writing.into_iter()) {
            if let CharWriting::Char(_) = a {
                *a = b;
            }
        }

        self.traditional = self.traditional.take().or(o.traditional);
        self.audio_file = self.audio_file.take().or(o.audio_file);
        if let (Some(hska), Some(hskb)) = (self.hsk_lev, o.hsk_lev) {
            self.hsk_lev = Some(hska.min(hskb));
        } else {
            self.hsk_lev = self.hsk_lev.take().or(o.hsk_lev);
        }
    }
    fn is_missing_some_writing(&self) -> bool {
        self.writing.len() != self.id.chars().count()
            || self
                .writing
                .iter()
                .any(|x| matches!(x, CharWriting::Char(_)))
    }
}
impl Entry for WordEntry {
    fn priority(&self) -> NotNan<f32> {
        self.total_priority()
    }
    fn into_note(self, idx: usize) -> genanki_rs::Note {
        crate::anki::word_entry_to_note(self, idx)
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
        ((!self.id.chars().any(is_good_cjk)) || self.definitions.is_empty())
            && self.hsk_lev.is_none()
    }
    fn media(&self) -> Vec<String> {
        self.audio_file
            .iter()
            .map(|x| x.as_os_str().to_str().unwrap().to_owned())
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyllableEntry {
    /// pinyin
    pub id: Pinyin,
    pub audio_file: std::path::PathBuf,
}
impl Entry for SyllableEntry {
    fn priority(&self) -> NotNan<f32> {
        NotNan::new(10f32).unwrap()
    }
    fn into_note(self, idx: usize) -> genanki_rs::Note {
        crate::anki::syllable_entry_to_note(self, idx)
    }
    fn id(&self) -> EntryId {
        EntryId::Syllable(self.id.to_string())
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
    fn media(&self) -> Vec<String> {
        vec![self.audio_file.as_os_str().to_str().unwrap().to_owned()]
    }
}

use std::sync::LazyLock;
pub static JIEBA: LazyLock<jieba_rs::Jieba> = LazyLock::new(jieba_rs::Jieba::new);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Triplet {
    pub zh: String,
    pub en: String,
    pub py: Pinyin,
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
        crate::anki::grammar_entry_to_note(self, idx)
    }
    fn id(&self) -> EntryId {
        EntryId::Grammar(self.id.clone())
    }
    fn dependencies(&self) -> Vec<EntryId> {
        let mut a = self.structure.dependencies();
        a.extend(self.example.dependencies());
        a
    }
    fn merge(&mut self, _o: CommonEntry) {
        unimplemented!()
    }
    fn compact_display(&self) -> String {
        format!("G {}:{}", self.example.zh, self.priority())
    }
    fn to_delete(&self) -> bool {
        false
    }
    fn media(&self) -> Vec<String> {
        vec![]
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
impl std::fmt::Display for EntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EntryId::Word(x) | EntryId::Syllable(x) | EntryId::Grammar(x) => write!(f, "{}", x),
        }
    }
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
    fn media(&self) -> Vec<String>;
}

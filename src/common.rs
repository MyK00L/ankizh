use enum_dispatch::enum_dispatch;
use ordered_float::NotNan;
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

#[derive(Clone, Debug, Default)]
pub struct Definition {
    pub pinyin: Option<String>,
    pub english: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Priority {
    pub val: NotNan<f32>,
    pub max: NotNan<f32>,
}

#[derive(Clone, Debug)]
pub enum CharWriting {
    Strokes(Vec<String>),
    Char(char),
}

#[derive(Clone, Debug, Default)]
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
}
impl WordEntry {
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
        if !(self.writing.is_empty() || o.writing.is_empty()) {
            eprintln!("{:?} {:?}", self, o.clone());
        }
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
        assert!(self.writing.is_empty() || o.writing.is_empty());
        self.writing.append(&mut o.writing);

        self.traditional = self.traditional.take().or(o.traditional);
        self.audio_file = self.audio_file.take().or(o.audio_file);
        self.hsk_lev = self.hsk_lev.take().or(o.hsk_lev);
    }
}
impl Entry for WordEntry {
    fn priority(&self) -> NotNan<f32> {
        self.total_priority()
    }
    fn into_note(self, model: genanki_rs::Model) -> genanki_rs::Note {
        todo!()
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
}

#[derive(Clone, Debug)]
pub struct SyllableEntry {}
impl Entry for SyllableEntry {
    fn priority(&self) -> NotNan<f32> {
        todo!()
    }
    fn into_note(self, model: genanki_rs::Model) -> genanki_rs::Note {
        todo!()
    }
    fn id(&self) -> EntryId {
        todo!()
    }
    fn dependencies(&self) -> Vec<EntryId> {
        todo!()
    }
    fn merge(&mut self, o: CommonEntry) {
        todo!()
    }
}
#[derive(Clone, Debug)]
pub struct GrammarEntry {}
impl Entry for GrammarEntry {
    fn priority(&self) -> NotNan<f32> {
        todo!()
    }
    fn into_note(self, model: genanki_rs::Model) -> genanki_rs::Note {
        todo!()
    }
    fn id(&self) -> EntryId {
        todo!()
    }
    fn dependencies(&self) -> Vec<EntryId> {
        todo!()
    }
    fn merge(&mut self, o: CommonEntry) {
        todo!()
    }
}

#[allow(clippy::enum_variant_names)]
#[enum_dispatch(Entry)]
#[derive(Debug, Clone)]
pub enum CommonEntry {
    WordEntry,
    SyllableEntry,
    GrammarEntry,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EntryId {
    Word(String),
    Syllable(String),
    Grammar(String),
}
#[enum_dispatch]
pub trait Entry {
    /// Higher priority means it should come earlier in the deck
    fn priority(&self) -> NotNan<f32>;
    fn into_note(self, model: genanki_rs::Model) -> genanki_rs::Note;
    fn id(&self) -> EntryId;
    fn dependencies(&self) -> Vec<EntryId>;
    fn merge(&mut self, o: CommonEntry);
}

use ordered_float::NotNan;
use std::cmp::Ordering;

pub fn process_pinyin(s: &str) -> String {
    let s = prettify_pinyin::prettify(s);
    let parser = pinyin_parser::PinyinParser::new()
        .preserve_spaces(false)
        .preserve_punctuations(true)
        .with_strictness(pinyin_parser::Strictness::Loose)
        .preserve_miscellaneous(true);
    parser
        .parse(&s)
        .reduce(|acc, s| acc + &s)
        .unwrap_or_default()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Definition {
    pub pinyin: Option<String>,
    pub english: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Priority {
    pub val: NotNan<f32>,
    pub max: NotNan<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharWriting {
    Strokes(Vec<String>),
    Char(char),
}

#[derive(Clone, Debug, Default, Eq)]
pub struct Entry {
    pub id: String,
    pub pinyin: Vec<String>,
    pub definitions: Vec<Definition>,
    pub freq: Vec<NotNan<f32>>,
    pub hsk_lev: Option<u8>,
    pub dependencies: Vec<String>,
    pub writing: Vec<CharWriting>,
    pub traditional: Option<String>,
    pub audio_file: Option<std::path::PathBuf>,
}
impl Entry {
    pub fn total_priority(&self) -> NotNan<f32> {
        let freq: NotNan<f32> = self.freq.iter().sum();
        let hsk_lev = self.hsk_lev.unwrap_or(10);

        let hp = NotNan::new((10 - hsk_lev) as f32 / 10f32).unwrap();
        let fp = (NotNan::new(freq.log2()).unwrap() + NotNan::new(16f32).unwrap())
            .max(NotNan::new(0f32).unwrap())
            / 16f32;

        hp * 0.5 + fp * 0.5
    }
    pub fn merge_add(&mut self, mut o: Entry) {
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
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.total_priority().cmp(&other.total_priority())
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.total_priority().eq(&other.total_priority())
    }
}

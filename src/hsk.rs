use crate::common::*;
use std::fs::File;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct HskEntry {
    #[allow(unused)]
    tr: String,
    zh: String,
    pinyin: String,
    #[allow(unused)]
    def: String,
    #[serde(default)]
    level: Option<u8>,
}
impl From<HskEntry> for WordEntry {
    fn from(h: HskEntry) -> Self {
        let mut w = WordEntry::from_id(h.zh);
        w.hsk_lev = h.level;
        w.pinyin = vec![h.pinyin.into()];
        w
    }
}

fn get_hsk(filename: &str, level: u8) -> impl Iterator<Item = HskEntry> {
    let file = File::open(filename).unwrap();
    let reader = std::io::BufReader::new(file);
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').has_headers(false).from_reader(reader);
    let v: Vec<_> = rdr.deserialize::<HskEntry>().map(move |r| {
        let mut e = r.unwrap();
        e.level = Some(level);
        e
    }).collect();
    v.into_iter()
}

pub fn get_hsks() -> impl Iterator<Item = CommonEntry> {
    let h1 = get_hsk("res/HSK-3.0/HSK List (Meaning)/HSK 1.tsv", 1);
    let h2 = get_hsk("res/HSK-3.0/HSK List (Meaning)/HSK 2.tsv", 2);
    let h3 = get_hsk("res/HSK-3.0/HSK List (Meaning)/HSK 3.tsv", 3);
    let h4 = get_hsk("res/HSK-3.0/HSK List (Meaning)/HSK 4.tsv", 4);
    let h5 = get_hsk("res/HSK-3.0/HSK List (Meaning)/HSK 5.tsv", 5);
    let h6 = get_hsk("res/HSK-3.0/HSK List (Meaning)/HSK 6.tsv", 6);
    let h789 = get_hsk("res/HSK-3.0/HSK List (Meaning)/HSK 7-9.tsv", 7);

    h1.chain(h2)
        .chain(h3)
        .chain(h4)
        .chain(h5)
        .chain(h6)
        .chain(h789)
        .map(WordEntry::from)
        .map(CommonEntry::from)
}

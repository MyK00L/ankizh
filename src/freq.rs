use crate::common::*;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct FreqRecord {
    #[serde(rename = "Word")]
    word: String,
    #[serde(rename = "W.million")]
    wm: NotNan<f32>,
    #[serde(rename = "Dominant.PoS")]
    pos: String,
}
impl From<FreqRecord> for WordEntry {
    fn from(r: FreqRecord) -> Self {
        let freq = r.wm / 1000000f32;
        let mut w = WordEntry::from_id(r.word);
        w.freq = vec![freq];
        w
    }
}

pub fn get_records() -> impl Iterator<Item = CommonEntry> {
    let file = std::fs::File::open("res/SUBTLEX-CH.txt").unwrap();
    let reader = std::io::BufReader::new(file);
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(reader);
    let v: Vec<CommonEntry> = rdr
        .deserialize::<FreqRecord>()
        .map(|r| r.unwrap())
        .map(WordEntry::from)
        .map(CommonEntry::from)
        .collect();
    v.into_iter()
}

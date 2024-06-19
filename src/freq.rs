use crate::common::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct FreqRecord {
    #[serde(rename = "Word")]
    word: String,
    #[serde(rename = "W.million")]
    wm: f32,
    #[serde(rename = "Dominant.PoS")]
    pos: String,
}
impl From<FreqRecord> for Entry {
    fn from(r: FreqRecord) -> Self {
        let freq = r.wm / 1000000f32;
        let score = (freq.log2() + 16f32).max(0f32);

        Self {
            id: r.word,
            priority: vec![Priority {
                val: score,
                max: 16f32,
            }],
            ..Default::default()
        }
    }
}

pub fn get_records() -> Vec<FreqRecord> {
    let file = std::fs::File::open("res/SUBTLEX-CH.txt").unwrap();
    let reader = std::io::BufReader::new(file);
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(reader);
    let mut ans = vec![];
    for result in rdr.deserialize() {
        let record: FreqRecord = result.unwrap();
        ans.push(record);
    }
    ans
}

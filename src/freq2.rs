use crate::common::*;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone)]
pub struct FreqRecord {
    id: String,
    freq: NotNan<f32>,
}
impl From<FreqRecord> for WordEntry {
    fn from(r: FreqRecord) -> Self {
        let mut w = WordEntry::from_id(r.id);
        w.freq = vec![r.freq];
        w
    }
}

pub fn get_records() -> impl Iterator<Item = CommonEntry> {
    let file = std::fs::File::open("res/zh_cn_50k.txt").unwrap();
    let reader = std::io::BufReader::new(file).lines().map_while(Result::ok);
    let a: Vec<(String, u32)> = reader
        .map(|x| {
            let mut s = x.split_whitespace();
            (
                s.next().unwrap().to_owned(),
                u32::from_str(s.next().unwrap()).unwrap(),
            )
        })
        .collect();
    let tot: u32 = a.iter().map(|x| x.1).sum();
    a.into_iter()
        .map(move |x| FreqRecord {
            id: x.0,
            freq: NotNan::new(x.1 as f32 / tot as f32).unwrap(),
        })
        .map(WordEntry::from)
        .map(CommonEntry::from)
}

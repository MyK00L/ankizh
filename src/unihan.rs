use crate::common::*;
use crate::pinyin_type::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    #[serde(rename = "char")]
    utf8: char,
    // ucn: String,
    k_definition: String,
    k_mandarin: String,
    /*
    k_semantic_variant: String,
    k_simplified_variant: String,
    k_specialized_semantic_variant: String,
    k_spoofing_variant: String,
    k_traditional_variant: String,
    k_z_variant: String
    */
}
impl From<Record> for WordEntry {
    fn from(r: Record) -> Self {
        let mut w = Self::from_id(r.utf8.into());
        if !r.k_definition.trim().is_empty() {
            w.definitions = vec![Definition {
                pinyin: None,
                english: r
                    .k_definition
                    .split(";")
                    .map(|x| x.trim().to_owned())
                    .collect(),
            }];
        }
        w.pinyin = r.k_mandarin.split_whitespace().map(Pinyin::from).collect();
        w
    }
}

pub fn get_records() -> impl Iterator<Item = CommonEntry> {
    let file = std::fs::File::open("res/unihan.csv").unwrap();
    let reader = std::io::BufReader::new(file);
    let mut rdr = csv::ReaderBuilder::new().from_reader(reader);
    let v: Vec<CommonEntry> = rdr
        .deserialize::<Record>()
        .map(|r| r.unwrap())
        .map(WordEntry::from)
        .map(CommonEntry::from)
        .collect();
    v.into_iter()
}

use crate::common::*;
use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CedictEntry {
    pub simplified: String,
    pub traditional: String,
    pub pinyin: Vec<String>,
    pub definitions: IndexMap<String, String>,
}
impl From<CedictEntry> for WordEntry {
    fn from(o: CedictEntry) -> Self {
        let mut w = WordEntry::from_id(o.simplified);
        w.traditional = Some(o.traditional);
        w.pinyin = o.pinyin.iter().map(|x| process_pinyin(x)).collect();
        w.definitions = o
            .definitions
            .iter()
            .map(|x| Definition {
                pinyin: Some(x.0.clone()),
                english: x
                    .1
                    .split(';')
                    .map(|x| x.trim())
                    .filter(|x| !x.is_empty())
                    .map(|x| x.to_owned())
                    .collect(),
            })
            .collect();
        w
    }
}

pub fn get_cedict() -> impl Iterator<Item = CommonEntry> {
    let file = std::fs::File::open("res/all_cedict.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let hm: IndexMap<String, CedictEntry> = serde_json::from_reader(reader).unwrap();
    hm.into_values().map(WordEntry::from).map(CommonEntry::from)
}

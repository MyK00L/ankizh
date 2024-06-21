use crate::common::*;
use std::io::BufRead;

#[derive(Debug)]
pub struct CedictEntry {
    pub simplified: String,
    pub traditional: String,
    pub pinyin: String,
    pub definitions: Vec<String>,
}
impl From<&str> for CedictEntry {
    // regex sux
    fn from(s: &str) -> Self {
        let (tr, s) = s.split_once(' ').unwrap();
        let (zh, s) = s.split_once(' ').unwrap();
        assert!(&s[..1] == "[");
        let (py, s) = s[1..].split_once(']').unwrap();
        assert!(&s[..2] == " /");
        let mut d = vec![];

        let mut r = &s[2..];
        while let Some((ds, x)) = r.split_once('/') {
            r = x;
            d.push(ds.to_owned());
        }
        Self {
            simplified: zh.into(),
            traditional: tr.into(),
            pinyin: process_pinyin(py),
            definitions: d,
        }
    }
}
impl From<CedictEntry> for WordEntry {
    fn from(o: CedictEntry) -> Self {
        let mut w = WordEntry::from_id(o.simplified);
        w.traditional = Some(o.traditional);
        w.pinyin = vec![o.pinyin.clone()];
        w.definitions = vec![Definition {
            pinyin: Some(o.pinyin),
            english: o.definitions,
        }];
        w
    }
}

pub fn get_cedict() -> impl Iterator<Item = CommonEntry> {
    let file = std::fs::File::open("res/cedict_1_0_ts_utf-8_mdbg.txt").unwrap();

    let lines = std::io::BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter(|s| {
            if let Some(x) = s.trim().chars().next() {
                x != '#'
            } else {
                false
            }
        });

    lines
        .map(|x| CedictEntry::from(x.as_str()))
        .map(WordEntry::from)
        .map(CommonEntry::from)
}

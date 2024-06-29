use crate::common::*;
use crate::utils::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use crate::pinyin_type::Pinyin;

#[derive(Deserialize)]
struct AllsetlearningEntry {
    pinyin: String,
    #[allow(unused)]
    count: u32,
}

pub fn get() -> impl Iterator<Item = CommonEntry> {
    let file = File::open("res/allsetlearning_grammar_keywords.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let a: HashMap<String, AllsetlearningEntry> = serde_json::from_reader(reader).unwrap();
    a.into_iter()
        .map(|(k, v)| {
            let mut we = WordEntry::from_id(k.clone());
            we.pinyin = v.pinyin.split(',').map(Pinyin::from).collect();
            let url = url::Url::parse(&format!(
                r#"https://resources.allsetlearning.com/chinese/grammar/{}"#,
                penc(&k)
            ))
            .unwrap();
            we.extra
                .push(format!(r#"<a href="{}">asl grammar points</a>"#, url));
            we
        })
        .map(CommonEntry::from)
}

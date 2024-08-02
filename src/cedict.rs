use crate::common::*;
use crate::pinyin_type::*;
use regex::Regex;
use std::io::BufRead;
use std::sync::LazyLock;

pub static RERE: LazyLock<[Regex; 8]> = LazyLock::new(|| {
    [
        Regex::new(r#"used in (?:\S)*\[.*?\](?:\(.*\))?(?: and (?:\S)*\[.*?\](?:\(.*\))?)*"#).unwrap(),
        Regex::new(r#"old variant of (?:\S)*\[.*?\](?:\(.*\))?"#).unwrap(),
        Regex::new(r#"erhua variant of (?:\S)*\[.*?\](?:\(.*\))?"#).unwrap(),
        Regex::new(r#"variant of (?:\S)*\[.*?\](?:\(.*\))?"#).unwrap(),
        Regex::new(r#"see (?:\S)*\[.*?\](?:\(.*\))?"#).unwrap(),
        Regex::new(r#"see also (?:\S)*\[.*?\](?:\(.*\))?"#).unwrap(),
        Regex::new(r#",? ?occurring in.*etc"#).unwrap(),
        Regex::new(r#"\[.*?\]"#).unwrap(),
    ]
});

fn simplify_def(ss: &str, blacklist: &[char]) -> Option<String> {
    let mut ans: String = ss.trim().to_owned();
    for re in RERE.iter() {
        ans = re.replace_all(&ans, "").trim().to_owned();
    }
    ans = ans.as_str().replace(blacklist, "〇");
    let ans = ans.trim_matches(|c: char| c.is_whitespace() || c == ',' || c==';');
    if ans.is_empty() {
        None
    } else {
        Some(ans.to_owned())
    }
}

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
            if let Some(sd) = simplify_def(ds, &tr.chars().chain(zh.chars()).collect::<Vec<_>>()) {
                d.push(sd);
            }
        }
        Self {
            simplified: zh.into(),
            traditional: tr.into(),
            pinyin: py.to_owned(),
            definitions: d,
        }
    }
}
impl From<CedictEntry> for WordEntry {
    fn from(o: CedictEntry) -> Self {
        let mut w = WordEntry::from_id(o.simplified);
        w.traditional = Some(o.traditional);
        w.pinyin = vec![Pinyin::from(&o.pinyin)];
        w.definitions = if o.definitions.is_empty() {
            vec![]
        } else {
            vec![Definition {
                pinyin: Some(CapPinyin::from(o.pinyin)),
                english: o.definitions,
            }]
        };
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

use crate::common::*;
use crate::pinyin_type::*;
use serde::Deserialize;
use std::io::BufRead;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PinyinFreq {
    pinyin: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Component {
    character: char,
    #[serde(default)]
    is_glyph_changed: bool,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Word {
    word: String,
    gloss: String,
}
impl From<Word> for Triplet {
    fn from(word: Word) -> Self {
        Self {
            zh: word.word.clone(),
            en: word.gloss,
            py: Pinyin::from_hanzi(word.word),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Dong {
    #[serde(rename = "char")]
    utf8: Option<char>,
    simp: Option<char>,
    #[serde(default)]
    pinyin_frequencies: Vec<PinyinFreq>,
    #[serde(default)]
    components: Option<Vec<Component>>,
    #[serde(default)]
    top_words: Vec<Word>,
    #[allow(unused)]
    gloss: Option<String>,
    #[allow(unused)]
    original_meaning: Option<String>,
}
impl From<Dong> for WordEntry {
    fn from(dong: Dong) -> Self {
        let id = dong.simp.unwrap_or(dong.utf8.unwrap_or_default()).into();
        let mut ans = Self::from_id(id);
        ans.pinyin = dong
            .pinyin_frequencies
            .into_iter()
            .map(|x| Pinyin::from(x.pinyin))
            .collect();
        ans.dependencies = dong
            .components
            .iter()
            .flatten()
            .filter_map(|x| {
                if x.is_glyph_changed {
                    None
                } else {
                    Some(EntryId::Word(x.character.into()))
                }
            })
            .collect();
        ans.examples = dong.top_words.into_iter().map(Triplet::from).collect();
        ans
    }
}

pub fn get() -> impl Iterator<Item = CommonEntry> {
    let file = std::fs::File::open("res/dictionary_char_2024-06-17.jsonl").unwrap();
    let reader = std::io::BufReader::new(file);
    reader
        .lines()
        .map_while(Result::ok)
        .map(|line| {
            let x = serde_json::from_str::<Dong>(&line);
            if x.is_err() {
                eprintln!("{}", line);
                eprintln!("{:?}", x);
            }
            x.unwrap()
        })
        .map(WordEntry::from)
        .map(CommonEntry::from)
}

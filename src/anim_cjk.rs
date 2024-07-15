use crate::common::*;
use crate::utils::*;
use serde::Deserialize;

#[derive(Default, Clone, Debug, Deserialize)]
pub struct GraphicsEntry {
    pub character: char,
    pub strokes: Vec<String>,
    #[allow(unused)]
    medians: Vec<Vec<(i32, i32)>>, // TODO: use this for automatic checking if character was drawn correctly?
}
impl From<GraphicsEntry> for WordEntry {
    fn from(o: GraphicsEntry) -> Self {
        let mut w = WordEntry::from_id(o.character.into());
        w.writing = vec![CharWriting::Strokes(
            o.strokes
                .into_iter()
                .zip(o.medians.into_iter())
                .map(|(s, m)| Stroke {
                    path: s,
                    start: m[0],
                })
                .collect(),
        )];
        w
    }
}

#[derive(Default, Clone, Debug, Deserialize)]
pub struct DictionaryEntry {
    pub character: char,
    pub decomposition: String,
    #[allow(unused)]
    pub radical: String,
}
impl From<DictionaryEntry> for WordEntry {
    fn from(o: DictionaryEntry) -> Self {
        let radical_deps: Vec<EntryId> = o
            .decomposition
            .chars()
            .filter(|x| is_radical(*x))
            .map(Into::<String>::into)
            .map(EntryId::Word)
            .collect();
        let mut w = WordEntry::from_id(o.character.into());
        w.dependencies = radical_deps;
        w
    }
}

use std::fs::File;
use std::io::{self, BufRead};

pub fn parse_graphics_zh_hans() -> impl Iterator<Item = CommonEntry> {
    let file = File::open("res/graphicsZhHans.txt").unwrap();
    let lines = io::BufReader::new(file).lines().map_while(Result::ok);
    lines
        .map(|x| serde_json::from_str::<GraphicsEntry>(&x).unwrap())
        .map(WordEntry::from)
        .map(CommonEntry::from)
}
pub fn parse_dictionary_zh_hans() -> impl Iterator<Item = CommonEntry> {
    let file = File::open("res/dictionaryZhHans.txt").unwrap();
    let lines = io::BufReader::new(file).lines().map_while(Result::ok);
    lines
        .map(|x| serde_json::from_str::<DictionaryEntry>(&x).unwrap())
        .map(WordEntry::from)
        .map(CommonEntry::from)
}

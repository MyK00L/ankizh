use crate::common::*;
use serde::Deserialize;

#[derive(Default, Clone, Debug, Deserialize)]
pub struct GraphicsEntry {
    pub character: char,
    pub strokes: Vec<String>,
    medians: Vec<Vec<(f32, f32)>>,
}
impl From<GraphicsEntry> for Entry {
    fn from(o: GraphicsEntry) -> Self {
        Self {
            id: o.character.into(),
            writing: vec![CharWriting::Strokes(o.strokes)],
            ..Default::default()
        }
    }
}

fn is_radical(c: char) -> bool {
    let c32 = c as u32;
    //(0x2f00..0x2fe0).contains(&c) || (0x2e80..0x2f00).contains(&c)
    !c.is_ascii() && !(0x2ff0..0x3000).contains(&c32)
}

#[derive(Default, Clone, Debug, Deserialize)]
pub struct DictionaryEntry {
    pub character: char,
    pub set: Vec<String>,
    pub decomposition: String,
}
impl From<DictionaryEntry> for Entry {
    fn from(o: DictionaryEntry) -> Self {
        let radicals: Vec<String> = o
            .decomposition
            .chars()
            .filter(|x| is_radical(*x))
            .map(Into::<String>::into)
            .collect();
        let hsk_priority = match o.set[0].as_str() {
            "hsk1" => 9,
            "hsk2" => 8,
            "hsk3" => 7,
            "hsk4" => 6,
            "hsk5" => 5,
            "hsk6" => 4,
            "hsk7" => 3,
            "hsk8" => 2,
            _ => 0,
        };
        let pr = hsk_priority as f32 * 6f32;
        Self {
            id: o.character.into(),
            dependencies: radicals,
            priority: vec![Priority {
                val: pr,
                max: 9f32 * 6f32,
            }],
            ..Default::default()
        }
    }
}

use std::fs::File;
use std::io::{self, BufRead};

pub fn parse_graphics_zh_hans() -> Vec<GraphicsEntry> {
    let file = File::open("res/graphicsZhHans.txt").unwrap();
    let lines = io::BufReader::new(file).lines().map_while(Result::ok);
    let mut ans = Vec::<GraphicsEntry>::new();
    for line in lines {
        ans.push(serde_json::from_str(&line).unwrap());
    }
    ans
}
pub fn parse_dictionary_zh_hans() -> Vec<DictionaryEntry> {
    let file = File::open("res/dictionaryZhHans.txt").unwrap();
    let lines = io::BufReader::new(file).lines().map_while(Result::ok);
    let mut ans = Vec::<DictionaryEntry>::new();
    for line in lines {
        ans.push(serde_json::from_str(&line).unwrap());
    }
    ans
}

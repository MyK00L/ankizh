use serde::{Serialize, Deserialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum EtymologyType {
    #[default]
    #[serde(rename = "ideographic")]
    Ideographic,
    #[serde(rename = "pictographic")]
    Pictographic,
    #[serde(rename = "pictophonetic")]
    Pictophonetic,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Etymology {
    r#type: EtymologyType,
    hint: Option<String>,
    phonetic: Option<String>,
    semantic: Option<String>
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct DictionaryEntry {
    character: char,
    definition: Option<String>,
    pinyin: Vec<String>,
    decomposition: String,
    etymology: Option<Etymology>,
    radical: char,
    matches: Vec<Option<Vec<usize> > >,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct GraphicsEntry {
    pub character: char,
    strokes: Vec<String>,
    medians: Vec<Vec<(f32,f32)> >,
}

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn parse_graphics() -> Vec<GraphicsEntry> {
    let file = File::open("res/makemeahanzi/graphics.txt").unwrap();
    let lines = io::BufReader::new(file).lines().flatten();
    let mut ans = Vec::<GraphicsEntry>::new();
    for line in lines {
        ans.push(serde_json::from_str(&line).unwrap());
    }
    ans
}

pub fn parse_dictionary() -> Vec<DictionaryEntry> {
    let file = File::open("res/makemeahanzi/dictionary.txt").unwrap();
    let lines = io::BufReader::new(file).lines().flatten();
    let mut ans = Vec::<DictionaryEntry>::new();
    for line in lines {
        ans.push(serde_json::from_str(&line).unwrap());
    }
    ans
}


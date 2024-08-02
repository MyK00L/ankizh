use crate::common::*;
use crate::pinyin_type::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::BufRead;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum ComponentType {
    Meaning,
    Sound,
    Iconic,
    Remnant,
    Simplified,
    Deleted,
    Distinguishing,
    Unknown,
}

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
    #[serde(rename = "type")]
    ctype: Vec<ComponentType>,
    #[serde(skip)]
    is_bad: bool,
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

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
struct Statistics {
    #[serde(default)]
    top_words: Vec<Word>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Dong {
    #[serde(rename = "char")]
    utf8: Option<char>,
    simp: Option<char>,
    #[serde(default)]
    statistics: Statistics,
    #[serde(default)]
    trad_variants: Vec<char>,
    #[serde(default)]
    pinyin_frequencies: Vec<PinyinFreq>,
    #[serde(default)]
    components: Option<Vec<Component>>,
    #[allow(unused)]
    gloss: Option<String>,
    #[allow(unused)]
    original_meaning: Option<String>,
}
impl Dong {
    fn get_gloss(&self) -> Option<String> {
        self.gloss.clone().and_then(|g| {
            let mut ans = String::new();
            let mut np = 0;
            for c in g.chars() {
                if c == '(' {
                    np += 1;
                }

                if np == 0 {
                    ans.push(c);
                }

                if c == ')' {
                    np -= 1;
                }
            }
            let ans = ans.trim_matches(|c: char| c.is_whitespace() || c == ',' || c == ';');
            if ans.is_empty() {
                None
            } else {
                Some(ans.to_owned())
            }
        })
    }
    fn get_glosses(&self) -> Vec<String> {
        let gloss = self.get_gloss();
        match gloss {
            None => vec![],
            Some(g) => g.split(';').map(|x| x.trim().to_owned()).collect(),
        }
    }
    fn get_original(&self) -> Option<String> {
        self.original_meaning.clone().and_then(|om| {
            let og = om.trim();
            if og.is_empty() {
                None
            } else {
                Some(og.to_owned())
            }
        })
    }
}
impl From<Dong> for WordEntry {
    fn from(dong: Dong) -> Self {
        let id = dong.simp.unwrap_or(dong.utf8.unwrap_or_default()).into();
        let mut ans = Self::from_id(id);
        ans.dependencies = dong
            .components
            .iter()
            .flatten()
            .filter_map(|x| {
                if x.is_bad {
                    None
                } else {
                    Some(EntryId::Word(x.character.into()))
                }
            })
            .collect();

        let glosses = dong.get_glosses();
        let glosses_is_empty = glosses.is_empty();
        let original = dong.get_original();
        if !glosses_is_empty || original.is_some() {
            ans.definitions = vec![Definition {
                pinyin: None,
                english: glosses
                    .into_iter()
                    .chain(original.map(|x| "original: ".to_owned() + &x).into_iter())
                    .collect(),
            }];
        }
        if !glosses_is_empty {
            let fd = ans.definitions[0].english[0].clone();
            ans.simple_definitions = vec![fd.split_once(',').map(|x| x.0.to_owned()).unwrap_or(fd)];
        }
        ans.examples = dong
            .statistics
            .top_words
            .into_iter()
            .take(3)
            .map(Triplet::from)
            .collect();
        ans.pinyin = dong
            .pinyin_frequencies
            .into_iter()
            .map(|x| Pinyin::from(x.pinyin))
            .collect();
        ans
    }
}

pub fn get() -> impl Iterator<Item = CommonEntry> {
    let file = std::fs::File::open("res/dictionary_char_2024-06-17.jsonl").unwrap();
    let reader = std::io::BufReader::new(file);
    let mut dongs: Vec<Dong> = reader
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
        .collect();
    let comphm: HashMap<char, Vec<char>> = dongs
        .iter()
        .filter(|x| x.utf8.is_some())
        .map(|dong| {
            (
                dong.utf8.unwrap(),
                dong.components
                    .iter()
                    .flatten()
                    .map(|x| x.character)
                    .collect(),
            )
        })
        .collect();
    for dong in dongs.iter_mut() {
        if let Some(ref mut comps) = &mut dong.components {
            let tcomps: Vec<char> = dong
                .trad_variants
                .iter()
                .filter_map(|x| comphm.get(x))
                .flatten()
                .copied()
                .collect();
            for comp in comps.iter_mut() {
                if comp.is_glyph_changed
                    || comp.ctype.contains(&ComponentType::Deleted)
                    || (comp.ctype.contains(&ComponentType::Simplified)
                        && tcomps.contains(&comp.character))
                {
                    comp.is_bad = true;
                }
            }
        }
    }

    dongs
        .into_iter()
        .map(WordEntry::from)
        .map(CommonEntry::from)
}

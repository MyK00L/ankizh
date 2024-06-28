use crate::common::*;
use crate::pinyin_type::*;
use crate::utils::*;
use itertools::Itertools;
use ordered_float::NotNan;
use serde::Deserialize;

#[derive(Deserialize)]
struct TatoebaRecord {
    zh_id: u64,
    zh: String,
    #[allow(unused)]
    en_id: u64,
    en: String,
}

#[derive(Clone, Debug)]
struct Example {
    tokens: Vec<String>,
    zh: String,
    en: String,
}
impl From<Example> for Triplet {
    fn from(e: Example) -> Self {
        let epy = Pinyin::from_hanzi(&e.zh);
        Self {
            zh: e.zh,
            en: e.en,
            py: epy,
        }
    }
}
impl From<TatoebaRecord> for Example {
    fn from(tr: TatoebaRecord) -> Self {
        let tokenizable: String = tr
            .zh
            .chars()
            .map(|x| if is_good_cjk(x) { x } else { ' ' })
            .collect();
        Self {
            tokens: JIEBA
                .cut(&tokenizable, false)
                .into_iter()
                .map(|x| x.trim().to_owned())
                .filter(|x| !x.is_empty())
                .collect(),
            zh: tr.zh,
            en: tr.en,
        }
    }
}

fn get_records() -> Vec<Example> {
    let file = std::fs::File::open("res/tatoeba-zh-en.tsv").unwrap();
    let reader = std::io::BufReader::new(file);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .quoting(false)
        .from_reader(reader);
    let records: Vec<TatoebaRecord> = rdr
        .deserialize::<TatoebaRecord>()
        .map(|r| r.unwrap())
        .collect();
    records
        .into_iter()
        .rev()
        .unique_by(|x| x.zh_id)
        .filter(|x| x.zh.chars().count() < 27 && x.en.chars().count() < 61)
        .map(Example::from)
        .collect()
}

fn length_bonus(s: &str) -> NotNan<f32> {
    let l = s.chars().filter(|x| is_good_cjk(*x)).count() as f32;
    const GL: f32 = 7f32;
    const GR: f32 = 14f32;
    NotNan::new(if l < GL {
        l / GL
    } else if l > GR {
        2f32.powf(GR - l)
    } else {
        1f32
    })
    .unwrap()
}

use std::collections::HashMap;
pub fn add_examples(v: &mut [CommonEntry]) {
    // build tatoeba records trie
    let records = get_records();
    let mut trie = ptrie::Trie::new();
    for (i, record) in records.iter().enumerate() {
        for (st, _) in record.zh.char_indices() {
            trie.insert(record.zh[st..].bytes(), i);
        }
    }

    // build hashmap for priorities
    let it = v.iter().filter_map(|x| match x {
        CommonEntry::WordEntry(inner) => Some(inner),
        _ => None,
    });
    let mut hm = HashMap::<String, (usize, NotNan<f32>)>::new();
    for (i, entry) in it.enumerate() {
        hm.insert(entry.id.clone(), (i, entry.priority()));
    }

    // highest priority from a point onwards
    let it = v.iter().filter_map(|x| match x {
        CommonEntry::WordEntry(inner) => Some(inner),
        _ => None,
    });
    let mut maxpr: Vec<_> = it
        .rev()
        .map(|x| x.priority())
        .scan(NotNan::new(0f32).unwrap(), |state, x| {
            *state = (*state).max(x);
            Some(*state)
        })
        .collect();
    maxpr.reverse();

    let it = v.iter_mut().filter_map(|x| match x {
        CommonEntry::WordEntry(inner) => Some(inner),
        _ => None,
    });

    for (thisord, (i, thispr)) in it.zip(maxpr.into_iter()).enumerate() {
        i.examples = trie
            .find_postfixes(i.id.bytes())
            .into_iter()
            .unique()
            .map(|x| &records[*x])
            .k_largest_by_key(3, |x| -> NotNan<f32> {
                let lb = length_bonus(&x.zh);
                let cb = NotNan::new(if x.tokens.contains(&i.id) {
                    1.0f32
                } else {
                    0.0f32
                })
                .unwrap();

                let um: NotNan<f32> = x
                    .tokens
                    .iter()
                    .map(|x| {
                        hm.get(x)
                            .cloned()
                            .unwrap_or((usize::MAX, NotNan::new(0f32).unwrap()))
                    })
                    .map(|(ord, pr)| {
                        if ord <= thisord || pr > thispr {
                            NotNan::new(0.2f32).unwrap()
                        } else {
                            (pr - thispr) * NotNan::new(4f32).unwrap()
                        }
                    })
                    .sum();

                NotNan::new(3f32).unwrap() * lb + NotNan::new(5f32).unwrap() * cb + um
            })
            .cloned()
            .map(Triplet::from)
            .collect();
    }
}

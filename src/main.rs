#![feature(path_file_prefix)]
mod allsetlearning;
mod anim_cjk;
mod anki;
mod audio;
mod cedict;
mod common;
mod freq;
mod freq2;
mod hsk;
mod lp_grammar;
mod pinyin_type;
mod tatoeba;
mod utils;

use crate::pinyin_type::*;
use anki::*;
use common::*;
use genanki_rs::*;
use ordered_float::NotNan;
use utils::*;

const MAX_ENTRIES: usize = 20000;
const MIN_PRIORITY: f32 = 0.19f32;

use std::collections::{HashMap, HashSet};
fn process_entries() -> Vec<CommonEntry> {
    let ag = anim_cjk::parse_graphics_zh_hans();
    let al = allsetlearning::get();
    let ad = anim_cjk::parse_dictionary_zh_hans();
    let cd = cedict::get_cedict();
    let fr = freq::get_records();
    let f2 = freq2::get_records();
    let wa = audio::get_word_audios();
    let sa = audio::get_syllable_audios(); //.take(2);
    let hs = hsk::get_hsks();
    let lg = lp_grammar::get_records();

    let mut hm = HashMap::<EntryId, CommonEntry>::new();
    for e in hs
        .chain(al)
        .chain(ag)
        .chain(ad)
        .chain(cd)
        .chain(fr)
        .chain(f2)
        .chain(wa)
        .chain(sa)
        .chain(lg)
    {
        if let Some(hme) = hm.get_mut(&e.id()) {
            hme.merge(e);
        } else {
            hm.insert(e.id(), e);
        }
    }

    {
        // add writings
        let keys: Vec<_> = hm
            .keys()
            .filter(|k| {
                if let EntryId::Word(w) = k {
                    w.chars().count() > 1
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        for key in keys {
            if let EntryId::Word(ref w) = key {
                let wr = w
                    .chars()
                    .map(|c| {
                        hm.get(&EntryId::Word(c.into()))
                            .map(|x| {
                                if let CommonEntry::WordEntry(w) = x {
                                    w.writing[0].clone()
                                } else {
                                    unreachable!()
                                }
                            })
                            .unwrap_or(CharWriting::Char(c))
                    })
                    .collect();

                if let CommonEntry::WordEntry(w) = hm.get_mut(&key).unwrap() {
                    w.writing = wr;
                }
            }
        }
    }
    // add definitions to some single-character entries from unicode names
    // and pinyin to words missing them
    for (_, entry) in hm.iter_mut() {
        if let CommonEntry::WordEntry(w) = entry {
            if w.pinyin.is_empty() {
                w.pinyin.push(Pinyin::from_hanzi(&w.id));
            }
            if w.id.chars().count() == 1 && w.definitions.is_empty() {
                let c = w.id.chars().next().unwrap();
                let name = unicode_names2::name(c).unwrap().to_string();
                if !name.contains(&format!("{:X}", c as u32)) {
                    w.definitions.push(Definition {
                        pinyin: None,
                        english: vec![name],
                    });
                }
            }
        }
    }
    hm.retain(|_k, v| !v.to_delete());

    let mut ordered: Vec<(NotNan<f32>, EntryId)> = hm
        .iter()
        .map(|(k, v)| (v.priority(), k.clone()))
        .filter(|(p, _k)| *p >= NotNan::new(MIN_PRIORITY).unwrap())
        .collect();
    ordered.sort_by_key(|e| e.0);
    ordered = ordered.into_iter().rev().take(MAX_ENTRIES).rev().collect();

    let mut ans = vec![];
    let mut done = HashSet::<EntryId>::new();
    let mut stack: Vec<Vec<EntryId>> = vec![ordered.into_iter().map(|(_p, k)| k).collect()];
    while !stack.is_empty() && ans.len() < MAX_ENTRIES {
        while stack.last().is_some_and(|x| x.is_empty()) {
            stack.pop();
        }
        //eprintln!("{:?}", &stack[1..]);
        if let Some(lv) = stack.last_mut() {
            if let Some(eid) = lv.last().cloned() {
                let e = hm.get(&eid).unwrap().clone();
                if !done.contains(&e.id()) {
                    let mut deps: Vec<EntryId> = e
                        .dependencies()
                        .into_iter()
                        .filter(|x| !done.contains(x) && hm.contains_key(x))
                        .collect();
                    if deps.is_empty() {
                        done.insert(e.id().clone());
                        ans.push(e);
                        lv.pop();
                        //eprintln!("made {}", eid);
                    } else {
                        deps.sort_by_cached_key(|a| hm.get(a).unwrap().priority());
                        //eprintln!("to make a {}:{} i need {:?}", eid, e.total_priority(), deps);
                        stack.push(deps);
                    }
                } else {
                    lv.pop();
                }
            }
        }
    }

    tatoeba::add_examples(&mut ans);
    ans
}

/// Prettifies output, used for debugging purposes
#[allow(unused)]
fn cache_entries() {
    let entries = process_entries();
    let file = std::fs::File::create("out/cache.bin").unwrap();
    let writer = std::io::BufWriter::new(file);
    bincode::serialize_into(writer, &entries).unwrap();
}
#[allow(unused)]
fn get_cached_entries() -> Vec<CommonEntry> {
    let file = std::fs::File::open("out/cache.bin").unwrap();
    let reader = std::io::BufReader::new(file);
    bincode::deserialize_from(reader).unwrap()
}
#[allow(unused)]
fn debug_entries(entries: Vec<CommonEntry>) {
    for entry in entries {
        println!("{}", entry.compact_display());
    }
}

fn main() {
    //cache_entries();return;
    let entries = get_cached_entries();
    //let entries = process_entries();
    //debug_entries(entries);return;

    let media: Vec<String> = entries.iter().flat_map(|x| x.media()).collect();

    let mut guids = HashSet::<String>::new();
    for entry in entries.iter() {
        let guid = guid_for(entry.id());
        if !guids.insert(guid) {
            panic!("GUID collision");
        }
    }

    let notes = entries
        .into_iter()
        .enumerate()
        .map(|(idx, x)| x.into_note(idx));

    let mut deck = Deck::new(DECK_ID, "zh", "zh");
    for note in notes {
        deck.add_note(note);
    }

    let mut package = Package::new(vec![deck], media.iter().map(|x| x.as_str()).collect()).unwrap();

    let file = std::fs::File::create("out/test.apkg").unwrap();
    let writer = std::io::BufWriter::new(file);
    package.write(writer).unwrap();
}

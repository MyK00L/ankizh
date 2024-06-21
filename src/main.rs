#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused)]
#![feature(path_file_prefix)]
mod anim_cjk;
mod anki;
mod audio;
mod cedict;
mod common;
mod freq;
mod freq2;
mod hsk;
mod lp_grammar;
mod tatoeba;

use anki::*;
use common::*;
use genanki_rs::*;
use std::io::Write;

const MAX_ENTRIES: usize = 32;

use std::collections::{HashMap, HashSet};
fn process_entries() -> Vec<CommonEntry> {
    let ag = anim_cjk::parse_graphics_zh_hans();
    let ad = anim_cjk::parse_dictionary_zh_hans();
    let cd = cedict::get_cedict();
    let fr = freq::get_records();
    let f2 = freq2::get_records();
    let wa = audio::get_word_audios();
    //let sa = audio::get_syllable_audios();
    let hs = hsk::get_hsks();
    let lg = lp_grammar::get_records();

    let mut hm = HashMap::<EntryId, CommonEntry>::new();
    for e in ag
        .chain(ad)
        .chain(cd)
        .chain(fr)
        .chain(f2)
        .chain(wa)
        //.chain(sa)
        .chain(hs)
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
    for (_, entry) in hm.iter_mut() {
        if let CommonEntry::WordEntry(w) = entry {
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

    let mut entries: Vec<CommonEntry> = hm.values().cloned().collect();

    entries.sort_by_cached_key(|e| e.priority());

    entries = entries.into_iter().rev().take(MAX_ENTRIES).rev().collect();

    let mut ans = vec![];
    let mut done = HashSet::<EntryId>::new();
    let mut stack: Vec<Vec<EntryId>> = vec![entries.iter().map(|x| x.id()).collect()];
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

    assert_eq!(ans.len(), entries.len());

    tatoeba::add_examples(&mut ans);

    ans
}

fn main() {
    let entries: Vec<CommonEntry> = if false {
        let mut entries = process_entries();
        let file = std::fs::File::create("out/cache.ron").unwrap();
        let writer = std::io::BufWriter::new(file);
        ron::ser::to_writer_pretty(writer, &entries, ron::ser::PrettyConfig::default()).unwrap();
        return;
        unreachable!();
    } else {
        let file = std::fs::File::open("out/cache.ron").unwrap();
        let reader = std::io::BufReader::new(file);
        ron::de::from_reader(reader).unwrap()
    };

    let media: Vec<String> = entries.iter().flat_map(|x| x.media()).collect();

    let notes = entries
        .into_iter()
        //TODO: remove filter
        .filter(|x| matches!(x, CommonEntry::WordEntry(_)))
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

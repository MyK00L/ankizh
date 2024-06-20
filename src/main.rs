#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused)]
#![feature(path_file_prefix)]
mod anim_cjk;
mod audio;
mod cedict;
mod common;
mod freq;
mod freq2;
mod hsk;
mod lp_grammar;

use common::*;
use genanki_rs::*;

const OPTIONS_JS: &str = include_str!("../anki-canvas/options.js");
const FRONT_JS: &str = include_str!("../anki-canvas/front.js");
const BACK_JS: &str = include_str!("../anki-canvas/back.js");

fn main_model() -> Model {
    const MODEL_ID: i64 = 7568361786070221454;

    let template_writing = {
        let front_inner = r#"<p>{{reading}}</p><div id="ac-front"></div>"#;
        let back_inner = r#"<div id="ac-back"></div>{{writing}}{{utf8}}{{reading}}{{meaning}}"#;
        let front = format!(
            "<script>{}</script>{}<script>{}</script>",
            OPTIONS_JS, front_inner, FRONT_JS
        );
        let back = format!(
            "<script>{}</script>{}<script>{}</script>",
            OPTIONS_JS, back_inner, BACK_JS
        );
        Template::new("zh_writing").qfmt(&front).afmt(&back)
    };

    Model::new(
        MODEL_ID,
        "hanziM",
        vec![
            Field::new("sort_field"),
            Field::new("utf8"),
            Field::new("meaning"),
            Field::new("reading"),
            Field::new("writing"),
            Field::new("unique"),
        ],
        vec![template_writing],
    )
}

use std::collections::{HashMap, HashSet};
fn process_entries() -> Vec<CommonEntry> {
    let ag = anim_cjk::parse_graphics_zh_hans();
    let ad = anim_cjk::parse_dictionary_zh_hans();
    let cd = cedict::get_cedict();
    let fr = freq::get_records();
    let f2 = freq2::get_records();
    let wa = audio::get_word_audios();
    let hs = hsk::get_hsks();
    let lg = lp_grammar::get_records();

    let mut hm = HashMap::<EntryId, CommonEntry>::new();
    for e in ag
        .chain(ad)
        .chain(cd)
        .chain(fr)
        .chain(f2)
        .chain(wa)
        .chain(hs)
        .chain(lg)
    {
        if let Some(hme) = hm.get_mut(&e.id()) {
            hme.merge(e);
        } else {
            hm.insert(e.id(), e);
        }
    }

    // TODO: character-based writing for single-character entries
    // TODO: writing for multiple-character entries
    {
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

    let mut entries: Vec<CommonEntry> = hm.values().cloned().collect();
    entries.sort_by_cached_key(|e| e.priority());

    let mut ans = vec![];
    let mut done = HashSet::<EntryId>::new();
    let mut stack: Vec<Vec<EntryId>> = vec![entries.iter().map(|x| x.id()).collect()];
    while !stack.is_empty() {
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

    if ans.len() != entries.len() {
        eprintln!("NOT EVERYTHING INCLUDED?");
    }

    ans
}

fn main() {
    let entries = process_entries();
    eprintln!("entries.len: {}", entries.len());
    for entry in entries.into_iter().take(10000) {
        println!("{}", entry.compact_display());
    }

    return;

    const DECK_ID: i64 = 9030804782668984910;
    let main_model = main_model();
    let test_note = Note::new(main_model, vec!["行","go","xíng","<img src='https://raw.githubusercontent.com/parsimonhi/animCJK/master/svgsZhHans/34892.svg'></img>","false"]).unwrap();

    let mut deck = Deck::new(DECK_ID, "zh", "zh");
    deck.add_note(test_note);
    deck.write_to_file("test.apkg").unwrap();
}

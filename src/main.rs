mod anim_cjk;
mod cedict;
mod common;
mod freq;

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

    let main_model = Model::new(
        MODEL_ID,
        "hanziM",
        vec![
            Field::new("utf8"),
            Field::new("meaning"),
            Field::new("reading"),
            Field::new("writing"),
            Field::new("unique"),
        ],
        vec![template_writing],
    );

    main_model
}

use std::collections::{HashMap, HashSet};
fn process_entries() -> Vec<Entry> {
    let ag = anim_cjk::parse_graphics_zh_hans()
        .into_iter()
        .map(Entry::from);
    let ad = anim_cjk::parse_dictionary_zh_hans()
        .into_iter()
        .map(Entry::from);
    let cd = cedict::get_cedict().into_iter().map(Entry::from);
    let fe = freq::get_records().into_iter().map(Entry::from);

    let mut hm = HashMap::<String, Entry>::new();
    for e in ag.chain(ad).chain(cd).chain(fe) {
        let hme = hm.entry(e.id.clone()).or_default();
        hme.id = e.id.clone();
        hme.merge_add(e);
    }

    // add character-based writing for single character entries
    for (key, val) in hm
        .iter_mut()
        .filter(|(key, val)| key.chars().count() > 1 && val.writing.is_empty())
    {
        let c = key.chars().next().unwrap();
        val.writing.push(CharWriting::Char(c));
    }

    // add stuff to multi-character words from single character words
    {
        let keys = hm.keys().cloned().collect::<Vec<_>>();
        for key in keys.into_iter().filter(|x| x.chars().count() > 1) {
            for c in key.clone().chars() {
                let cs: String = c.into();
                if !hm.contains_key(&cs) {
                    let val = hm.get_mut(&key).unwrap();
                    val.writing.push(CharWriting::Char(c));
                    continue;
                }

                let mut writing = hm.get(&cs).unwrap().writing.clone();
                let val = hm.get_mut(&key).unwrap();

                if !val.dependencies.contains(&cs) {
                    val.dependencies.push(cs);
                }
                //TODO: writing might be empty (or not exist) :)
                val.writing.append(&mut writing);
            }
        }
    }

    let mut entries: Vec<Entry> = hm.values().cloned().collect();
    entries.sort();
    let mut ans = vec![];
    let mut done = HashSet::<String>::new();
    let mut stack: Vec<Vec<String>> = vec![entries.iter().map(|x| x.id.clone()).collect()];
    while !stack.is_empty() {
        while stack.last().is_some_and(|x| x.is_empty()) {
            stack.pop();
        }
        if let Some(lv) = stack.last_mut() {
            if let Some(eid) = lv.last().cloned() {
                let e = hm.get(&eid).unwrap().clone();
                if !done.contains(&e.id) {
                    let mut deps: Vec<String> = e
                        .dependencies
                        .iter()
                        .filter(|x| !done.contains(*x) && hm.contains_key(*x))
                        .cloned()
                        .collect();
                    if deps.is_empty() {
                        done.insert(e.id.clone());
                        ans.push(e);
                        lv.pop();
                    } else {
                        deps.sort_by(|a, b| hm.get(a).unwrap().cmp(hm.get(b).unwrap()));
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

    /*
    let mut outd = HashMap::<String, u32>::new();
    let mut revdep = HashMap::<String, Vec<String>>::new();
    let mut pq = BinaryHeap::<Entry>::new();
    for (key, val) in hm.iter() {
        let mut dnum = 0u32;
        for dep in val.dependencies.iter() {
            if !hm.contains_key(dep) {
                eprintln!("no dep {}", dep);
            } else {
                revdep.entry(dep.clone()).or_default().push(key.clone());
                dnum += 1;
            }
        }
        outd.insert(key.clone(), dnum);
        if dnum == 0 {
            pq.push(val.clone());
        }
    }

    let mut ans = Vec::new();
    while let Some(entry) = pq.pop() {
        if let Some(rdeps) = revdep.get(&entry.id) {
            for rdep in rdeps {
                let dnum = outd.get_mut(rdep).unwrap();
                *dnum -= 1;
                if *dnum == 0 {
                    pq.push(hm.get(rdep).unwrap().clone());
                }
            }
        }
        ans.push(entry);
    }
    */

    /*
    for (key, dnum) in outd.iter() {
        if *dnum != 0 {
            eprintln!("not included: {} {}", key, dnum);
        }
    }
    */
}

fn main() {
    let entries = process_entries();
    for entry in entries {
        println!("[{}:{}]", entry.id, entry.total_priority());
    }

    return;

    const DECK_ID: i64 = 9030804782668984910;
    let main_model = main_model();
    let test_note = Note::new(main_model, vec!["行","go","xíng","<img src='https://raw.githubusercontent.com/parsimonhi/animCJK/master/svgsZhHans/34892.svg'></img>","false"]).unwrap();

    let mut deck = Deck::new(DECK_ID, "zh", "zh");
    deck.add_note(test_note);
    deck.write_to_file("test.apkg").unwrap();
}

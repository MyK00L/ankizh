use crate::common::*;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Clone)]
pub struct HskEntry {
    id: String,
    level: u8,
}
impl From<HskEntry> for WordEntry {
    fn from(h: HskEntry) -> Self {
        Self {
            id: h.id,
            hsk_lev: Some(h.level),
            ..Default::default()
        }
    }
}

fn get_hsk(filename: &str, lev: u8) -> impl Iterator<Item = HskEntry> {
    let file = File::open(filename).unwrap();
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .map(move |w| HskEntry {
            id: w.trim().to_owned(),
            level: lev,
        })
}

pub fn get_hsks() -> impl Iterator<Item = CommonEntry> {
    let h1 = get_hsk("res/HSK-3.0/HSK List/HSK 1.txt", 1);
    let h2 = get_hsk("res/HSK-3.0/HSK List/HSK 2.txt", 2);
    let h3 = get_hsk("res/HSK-3.0/HSK List/HSK 3.txt", 3);
    let h4 = get_hsk("res/HSK-3.0/HSK List/HSK 4.txt", 4);
    let h5 = get_hsk("res/HSK-3.0/HSK List/HSK 5.txt", 5);
    let h6 = get_hsk("res/HSK-3.0/HSK List/HSK 6.txt", 6);
    let h789 = get_hsk("res/HSK-3.0/HSK List/HSK 7-9.txt", 7);

    h1.chain(h2)
        .chain(h3)
        .chain(h4)
        .chain(h5)
        .chain(h6)
        .chain(h789)
        .map(WordEntry::from)
        .map(CommonEntry::from)
}

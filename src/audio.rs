use crate::common::*;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AudioPath {
    id: String,
    path: PathBuf,
}
impl From<AudioPath> for WordEntry {
    fn from(a: AudioPath) -> Self {
        let mut w = WordEntry::from_id(a.id);
        w.audio_file = Some(a.path);
        w
    }
}
impl From<AudioPath> for SyllableEntry {
    fn from(a: AudioPath) -> Self {
        Self {
            id: a.id.into(),
            audio_file: a.path,
        }
    }
}

pub fn get_syllable_audios() -> impl Iterator<Item = CommonEntry> {
    let mut ans = vec![];
    for entry in std::fs::read_dir("res/audio-cmn/64k/syllabs").unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let name = path.file_prefix().unwrap().to_str().unwrap();
            if name.contains('_') {
                continue;
            }
            let id = name[4..].to_owned();
            ans.push(AudioPath { id, path });
        }
    }
    ans.into_iter()
        .map(SyllableEntry::from)
        .map(CommonEntry::from)
}

pub fn get_word_audios() -> impl Iterator<Item = CommonEntry> {
    let mut ans = vec![];
    for entry in std::fs::read_dir("res/audio-cmn/64k/hsk").unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let name = path.file_prefix().unwrap().to_str().unwrap();
            if name.contains('_') {
                continue;
            }
            let id = name[4..].to_owned();
            ans.push(AudioPath { id, path });
        }
    }
    ans.into_iter().map(WordEntry::from).map(CommonEntry::from)
}

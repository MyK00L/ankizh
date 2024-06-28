use crate::common::*;
use crate::utils::*;
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
    for entry in std::fs::read_dir("res/audio2").unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let name = path.file_prefix().unwrap().to_str().unwrap();
            let id = name.split_once('_').unwrap().0.to_string();
            ans.push(SyllableEntry::from(AudioPath { id, path }));
        }
    }
    ans.sort_by_key(|s| guid_for(s.id()));
    ans.into_iter().map(CommonEntry::from)
}

#[allow(unused)]
pub fn get_syllable_audios_old() -> impl Iterator<Item = CommonEntry> {
    panic!("Use get_syllable_audios for better quality audio or remove this line");
    let mut ans = vec![];
    for entry in std::fs::read_dir("res/audio-cmn/64k/syllabs").unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let name = path.file_prefix().unwrap().to_str().unwrap();
            if name.contains('_') {
                continue;
            }
            let id = name[4..].to_owned();
            ans.push(SyllableEntry::from(AudioPath { id, path }));
        }
    }
    ans.sort_by_key(|s| guid_for(s.id()));
    ans.into_iter().map(CommonEntry::from)
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

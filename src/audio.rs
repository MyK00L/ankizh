use crate::common::*;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AudioPath {
    id: String,
    path: PathBuf,
}
impl From<AudioPath> for Entry {
    fn from(a: AudioPath) -> Self {
        Self {
            id: a.id,
            audio_file: Some(a.path),
            ..Default::default()
        }
    }
}

pub fn get_word_audios() -> Vec<AudioPath> {
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
    ans
}
use crate::common::*;
use ordered_float::NotNan;
use pinyin::ToPinyin;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Deserialize, Debug, Clone)]
struct GrammarRecord {
    id: u32,
    code: String,
    structure: String,
    #[allow(unused)]
    pattern: String,
    pinyin: String,
    english: String,
    #[allow(unused)]
    review: String,
    example: String,
    #[serde(rename = "exampleTranslation")]
    example_translation: String,
    #[allow(unused)]
    url: String,
    _hsk_lev: Option<u8>,
    _hsk_sublev: Option<f32>,
}
impl From<GrammarRecord> for GrammarEntry {
    fn from(gr: GrammarRecord) -> Self {
        let epy: String = gr
            .example
            .as_str()
            .to_pinyin()
            .flatten()
            .map(|x| x.with_tone().to_string())
            .fold(String::new(), |acc, e| acc + &e);
        let epy = process_pinyin(&epy);
        Self {
            id: gr.id.to_string(),
            structure: Triplet {
                zh: gr.structure,
                py: gr.pinyin,
                en: gr.english,
            },
            example: Triplet {
                zh: gr.example,
                en: gr.example_translation,
                py: epy,
            },
            hsk_lev: gr._hsk_lev,
            hsk_sublev: gr._hsk_sublev,
        }
    }
}

pub fn get_records() -> impl Iterator<Item = CommonEntry> {
    let file = std::fs::File::open("res/lp_grammar.csv").unwrap();
    let reader = std::io::BufReader::new(file);
    let mut rdr = csv::ReaderBuilder::new().from_reader(reader);
    let mut ans = vec![];
    let mut nhsk = [0u32; 11];
    for result in rdr.deserialize() {
        let mut record: GrammarRecord = result.unwrap();
        record._hsk_lev =
            Some(u8::from_str(record.code.split(&['.', '-']).next().unwrap()).unwrap());
        nhsk[record._hsk_lev.unwrap() as usize] += 1;
        ans.push(record);
    }

    let mut ihsk = [0u32; 11];
    for record in ans.iter_mut() {
        let hl = record._hsk_lev.unwrap() as usize;
        record._hsk_sublev = Some(ihsk[hl] as f32 / (nhsk[hl] - 1) as f32);
        ihsk[hl] += 1;
    }

    ans.into_iter()
        .map(GrammarEntry::from)
        .map(CommonEntry::from)
}

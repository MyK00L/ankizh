use crate::common::*;
use ordered_float::NotNan;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct GrammarRecord {
    id: u32,
    code: String,
    structure: String,
    pattern: String,
    pinyin: String,
    english: String,
    review: String,
    example: String,
    #[serde(rename = "exampleTranslation")]
    example_translation: String,
    url: String,
}

pub fn get_records() -> Vec<GrammarRecord> {
    let file = std::fs::File::open("res/lp_grammar.csv").unwrap();
    let reader = std::io::BufReader::new(file);
    let mut rdr = csv::ReaderBuilder::new().from_reader(reader);
    let mut ans = vec![];
    for result in rdr.deserialize() {
        let record: GrammarRecord = result.unwrap();
        ans.push(record);
    }
    ans
}

use crate::common::*;
use serde::Deserialize;

#[derive(Default, Clone, Debug, Deserialize)]
pub struct GraphicsEntry {
    pub character: char,
    pub strokes: Vec<String>,
    #[allow(unused)]
    medians: Vec<Vec<(f32, f32)>>, // TODO: use this for automatic checking if character was drawn correctly?
}
impl From<GraphicsEntry> for WordEntry {
    fn from(o: GraphicsEntry) -> Self {
        let mut w = WordEntry::from_id(o.character.into());
        w.writing = vec![CharWriting::Strokes(o.strokes)];
        w
    }
}

const RADICALS: &str = "⺀⺄⺆⺈⺊⺌⺍⺕⺗⺤⺥⺧⺪⺫⺮⺲⺳⺶⺷⺺⺻⺼⺾⺿⻂⻆⻇⻊⻌⻍⻎⻏⻖⻣⼎㇇㔾㣺䒑一丨丬丶丷丿乀乁么乙乚乛乾亅二亠人亻儿兀兒入八冂冖冫几凵刀刂力勹匕匚匸十卜卤卩厂厶又口囗土士夂夊夕大女子宀寸小尢尣尸屍屮山巛巜川工己已巳巾干幹幺广廠廣廴廾弋弓彐彑彡彳心忄戈戶户戸手扌支攴攵文斉斗斤方无旡日曰月木朩杀欠止歹歺殳毋母比毛氏气氣水氵氺火灬爪爫父爻爿片牙牛牜犬犭玄玉玊王瓜瓦甘生用田电疋疒癶白皮皿目矛矢石示礻禸禾穴立竹米糸糹纟缶网罒罓羊羽老耂而耒耳聿肀肉臣自至臼舌舛舟艮色艸艹虍虫蟲血行衣衤襾西覀見见角言訁讠谷豆豕豸貝贝赤走足身車车辛辰辵辶邑酉釆里金釒钅長镸长門门阜阝隶隹雨靑青非面靣革韋韦韭音頁页風风飛飞食飠饣首香馬马骨高髙髟鬥鬯鬲鬼魚鱼鳥鸟鹵鹿麥麦麻黃黄黍黑黹黽黾鼎鼓鼠鼻齊齐齒齿龍龙龜龟龠龰龵𠃊𠃋𠃌𠃍𠃑𠄌𠄎𠆢𠘨𠤎𡿨𤣩𤴔𥫗𦍌𦥑𧘇𧾷𫶧𭕄";
fn is_radical(c: char) -> bool {
    let c32 = c as u32;
    RADICALS.contains(c) || (0x2f00..0x2fe0).contains(&c32) || (0x2e80..0x2f00).contains(&c32)

    // !c.is_ascii() && !(0x2ff0..0x3000).contains(&c32)
}

#[derive(Default, Clone, Debug, Deserialize)]
pub struct DictionaryEntry {
    pub character: char,
    pub decomposition: String,
    #[allow(unused)]
    pub radical: String,
}
impl From<DictionaryEntry> for WordEntry {
    fn from(o: DictionaryEntry) -> Self {
        let radical_deps: Vec<EntryId> = o
            .decomposition
            .chars()
            .filter(|x| is_radical(*x))
            .map(Into::<String>::into)
            .map(EntryId::Word)
            .collect();
        let mut w = WordEntry::from_id(o.character.into());
        w.dependencies = radical_deps;
        w
    }
}

use std::fs::File;
use std::io::{self, BufRead};

pub fn parse_graphics_zh_hans() -> impl Iterator<Item = CommonEntry> {
    let file = File::open("res/graphicsZhHans.txt").unwrap();
    let lines = io::BufReader::new(file).lines().map_while(Result::ok);
    lines
        .map(|x| serde_json::from_str::<GraphicsEntry>(&x).unwrap())
        .map(WordEntry::from)
        .map(CommonEntry::from)
}
pub fn parse_dictionary_zh_hans() -> impl Iterator<Item = CommonEntry> {
    let file = File::open("res/dictionaryZhHans.txt").unwrap();
    let lines = io::BufReader::new(file).lines().map_while(Result::ok);
    lines
        .map(|x| serde_json::from_str::<DictionaryEntry>(&x).unwrap())
        .map(WordEntry::from)
        .map(CommonEntry::from)
}

use crate::common::*;
use const_format::concatcp;
use genanki_rs::*;
use html_escape::encode_safe;
use std::sync::LazyLock;

pub const DECK_ID: i64 = 9030804782668984910;

const PRE_HTML_COMMON: &str = r#"<span lang="zh-Hans">"#;
const POST_HTML_COMMON: &str = r#"<p style="display:none;">{{sort_field}}</p></span>"#;
const POST_HTML_COMMON_NO_SORTFIELD: &str = r#"</span>"#;

const CSS_COMMON: &str = r#"
body {
    font-size: 2em;
}
h1,h2,h3,h4,#ac-back,#ac-front,.tc {
    text-align: center;
}
h2,h3,h4 {
    font-weight: 600;
}
ol {
    list-style-type: none;
}
.def {
    margin-left: 0.5em;
    margin-top:0.2em;
    padding: 0.1em 0.5em;
    border-style:solid;
    border-width:thin;
    border-radius: 5px;
    display: inline-block;
}
.charvg {
    display: inline-block;
    width: 3em;
    height: 3em;
    font-size: 3em;
    border: solid;
    border-width: thin;
}
"#;

pub static WORD_MODEL: LazyLock<Model> = LazyLock::new(|| {
    const BACK_COMMON: &str = r#"
    <h1>{{word}}</h1>
    <h2>{{pinyin}}</h2>
    <h4>{{traditional}}</h4>
    <hr>
    <ol>{{definitions}}</ol>
    <hr>
    <ul>{{examples}}</ul>
    <hr>
    <p>HSK: {{hsk}}</p>
    <details>
        <summary>Extra</summary>
        {{extra}}
    </details>
    {{#audio}}<p class="tc">{{audio}}</p>{{/audio}}
    "#;

    const MODEL_ID: i64 = 7568361786070221454;

    let template_writing = {
        const OPTIONS_JS: &str = concatcp!(
            "<script>",
            include_str!("../anki-canvas/options.js"),
            "</script>"
        );
        const FRONT_JS: &str = concatcp!(
            "<script>",
            include_str!("../anki-canvas/front.js"),
            "</script>"
        );
        const BACK_JS: &str = concatcp!(
            "<script>",
            include_str!("../anki-canvas/back.js"),
            "</script>"
        );
        const FRONT_INNER: &str = r#"
        <div id="ac-front"></div>
        {{#audio}}
            <h3 class="tc">{{audio}}</h3>
            <h3 class="tc">{{hint:pinyin}}</h3>
        {{/audio}}
        {{^audio}}
            <h3 class="tc">{{pinyin}}</h3>
        {{/audio}}
        {{hint:definitions}}
        "#;
        const BACK_INNER: &str = concatcp!(
            r#"
        <div id="ac-back"></div>
        <br/>
        {{writing}}
        "#,
            BACK_COMMON
        );

        const FRONT: &str = concatcp!(
            PRE_HTML_COMMON,
            OPTIONS_JS,
            FRONT_INNER,
            FRONT_JS,
            POST_HTML_COMMON
        );
        const BACK: &str = concatcp!(
            PRE_HTML_COMMON,
            OPTIONS_JS,
            BACK_INNER,
            BACK_JS,
            POST_HTML_COMMON
        );
        Template::new("zh_word_writing").qfmt(FRONT).afmt(BACK)
    };

    let template_meaning = {
        const FRONT_INNER: &str = r#"
        <h2>What are the <b style="color:red;">meanings</b>?</h2>
        {{#audio}}
            <h2>{{audio}}</h2>
            <h2>{{hint:pinyin}}</h2>
        {{/audio}}
        {{^audio}}
            <h2>{{pinyin}}</h2>
        {{/audio}}
        <h2>{{hint:word}}</h2>
        "#;
        const BACK_INNER: &str = BACK_COMMON;

        const FRONT: &str = concatcp!(PRE_HTML_COMMON, FRONT_INNER, POST_HTML_COMMON);
        const BACK: &str = concatcp!(PRE_HTML_COMMON, BACK_INNER, POST_HTML_COMMON);
        Template::new("zh_word_meaning").qfmt(FRONT).afmt(BACK)
    };
    let template_reading = {
        const FRONT_INNER: &str = r#"
        <h2>What are the <b style="color:red;">readings</b>?</h2>
        <h1>{{word}}</h1>
        "#;
        const BACK_INNER: &str = BACK_COMMON;
        const FRONT: &str = concatcp!(PRE_HTML_COMMON, FRONT_INNER, POST_HTML_COMMON);
        const BACK: &str = concatcp!(PRE_HTML_COMMON, BACK_INNER, POST_HTML_COMMON);
        Template::new("zh_word_reading").qfmt(FRONT).afmt(BACK)
    };
    let template_recalling = {
        const FRONT_INNER: &str = r#"
        <h2>How to say <b style="color:red;">{{english_single}}</b> in chinese?</h2>
        "#;
        const BACK_INNER: &str = BACK_COMMON;
        const FRONT: &str = concatcp!(PRE_HTML_COMMON, FRONT_INNER, POST_HTML_COMMON_NO_SORTFIELD,);
        const BACK: &str = concatcp!(PRE_HTML_COMMON, BACK_INNER, POST_HTML_COMMON);
        Template::new("zh_word_recalling").qfmt(FRONT).afmt(BACK)
    };

    Model::new_with_options(
        MODEL_ID,
        "HanziWord",
        vec![
            Field::new("sort_field"),
            Field::new("word"),
            Field::new("english_single"),
            Field::new("pinyin"),
            Field::new("definitions"),
            Field::new("writing"),
            Field::new("traditional"),
            Field::new("examples"),
            Field::new("hsk"),
            Field::new("audio"),
            Field::new("extra"),
        ],
        vec![
            template_meaning,
            template_reading,
            template_writing,
            template_recalling,
        ],
        Some(CSS_COMMON),
        None,
        None,
        None,
        None,
    )
});
fn anki_canvas_contrast(idx: usize) -> String {
    fn rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
        let i = (h * 6f64).floor();
        let f = h * 6f64 - i;
        let p = v * (1f64 - s);
        let q = v * (1f64 - f * s);
        let t = v * (1f64 - (1f64 - f) * s);
        let (mut r, mut g, mut b) = match i as u32 % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            5 => (v, p, q),
            _ => unreachable!(),
        };
        r *= 255f64;
        g *= 255f64;
        b *= 255f64;
        r = r.floor();
        g = g.floor();
        b = b.floor();
        (r as u8, g as u8, b as u8)
    }
    let s = 0.95f64;
    let v = 0.75f64;
    let angle = 0.618033988749895f64;
    let h = idx as f64 / angle;
    let (r, g, b) = rgb(h, s, v);
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}
fn svg_from_strokes(strokes: Vec<Stroke>, i0: usize) -> String {
    format!(
        r#"<svg class="charvg" viewbox="0 0 1024 1024"><g transform="scale(1, -1) translate(0, -900)">{}</g>{}</svg>"#,
        strokes
            .iter()
            .enumerate()
            .map(|(i, stroke)| {
                format!(
                    r#"<path d="{}" fill="{}"></path>"#,
                    stroke.path,
                    anki_canvas_contrast(i0 + i),
                )
            })
            .fold(String::new(), |acc, e| acc + &e),
        strokes
            .iter()
            .enumerate()
            .map(|(i, stroke)| {
                format!(
                    r#"<text x="{}" y="{}" stroke="white" fill="black">{}</text>"#,
                    stroke.start.0,
                    900 - stroke.start.1,
                    i0 + i + 1
                )
            })
            .fold(String::new(), |acc, e| acc + &e)
    )
}
fn html_from_char_writing(w: CharWriting, i0: usize) -> String {
    match w {
        CharWriting::Strokes(strokes) => svg_from_strokes(strokes, i0),
        CharWriting::Char(c) => format!(r#"<span class="charvg">{}</span>"#, c),
    }
}
fn html_from_writing(w: Vec<CharWriting>) -> String {
    let mut cw = Vec::<String>::with_capacity(w.len());
    let mut i0 = 0usize;
    for c in w.into_iter() {
        let ns = match &c {
            CharWriting::Strokes(strokes) => strokes.len(),
            CharWriting::Char(_c) => 0,
        };
        cw.push(html_from_char_writing(c, i0));
        i0 += ns;
        if ns == 0 {
            i0 = 0;
        }
    }
    format!(r#"<p class="tc">{}</p>"#, cw.join(""))
}
pub fn word_entry_to_note(we: WordEntry, idx: usize) -> Note {
    Note::new(
        WORD_MODEL.clone(),
        vec![
            // sort_field
            &format!("{:08}", idx),
            // word
            &encode_safe(&we.id),
            // english_single
            &we.hsk_lev
                .and_then(|hsk| {
                    if hsk < 7 {
                        we.first_definition().map(|x| encode_safe(&x).to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_default(),
            // pinyin
            &we.pinyin
                .iter()
                .map(|x| encode_safe(&x.to_string()).to_string())
                .fold(String::new(), |acc, e| {
                    let sep = if !acc.is_empty() { ", " } else { "" };
                    acc + sep + &e
                }),
            // definitions
            &we.definitions
                .into_iter()
                .map(|x| {
                    format!(
                        "<li><b>{}</b>: {}</li>",
                        &encode_safe(&x.pinyin.unwrap_or_default().to_string()),
                        x.english
                            .iter()
                            .map(|x| format!(r#"<span class="def">{}</span>"#, encode_safe(x)))
                            .fold(String::new(), |acc, e| acc + &e)
                    )
                })
                .fold(String::new(), |acc, e| acc + &e),
            // writing
            &html_from_writing(we.writing),
            // traditional
            &encode_safe(&we.traditional.unwrap_or_default()),
            // examlpes
            &we.examples
                .into_iter()
                .map(|x| {
                    format!(
                        "<li><ruby>{}<rt>{}</rt></ruby><br/>{}</li>",
                        encode_safe(&x.zh),
                        encode_safe(&x.py.to_string()),
                        encode_safe(&x.en)
                    )
                })
                .fold(String::new(), |acc, e| acc + &e),
            // hsk
            &we.hsk_lev
                .map(|x| x.to_string())
                .unwrap_or(String::from("no")),
            // audio
            &we.audio_file
                .map(|x| format!("[sound:{}]", x.file_name().unwrap().to_str().unwrap()))
                .unwrap_or_default(),
            // extra
            "",
        ],
    )
    .unwrap()
}

pub static SYLLABLE_MODEL: LazyLock<Model> = LazyLock::new(|| {
    const MODEL_ID: i64 = 4446634760010140961;
    let template_listening = {
        const FRONT_INNER: &str =
            r#"<h2>What is the <b style="color:red">pinyin</b>?</h2><h1>{{audio}}</h1>"#;
        const BACK_INNER: &str = r#"<h1>{{audio}}</h1><h1>{{pinyin}}</h1>"#;
        const FRONT: &str = concatcp!(PRE_HTML_COMMON, FRONT_INNER, POST_HTML_COMMON);
        const BACK: &str = concatcp!(PRE_HTML_COMMON, BACK_INNER, POST_HTML_COMMON);
        Template::new("zh_syllable_listening")
            .qfmt(FRONT)
            .afmt(BACK)
    };
    Model::new_with_options(
        MODEL_ID,
        "HanziSyllable",
        vec![
            Field::new("sort_field"),
            Field::new("pinyin"),
            Field::new("audio"),
        ],
        vec![template_listening],
        Some(CSS_COMMON),
        None,
        None,
        None,
        None,
    )
});
pub fn syllable_entry_to_note(se: SyllableEntry, idx: usize) -> Note {
    Note::new(
        SYLLABLE_MODEL.clone(),
        vec![
            // sort_field
            &format!("{:08}", idx),
            // pinyin
            &encode_safe(&se.id.to_string()),
            &format!(
                "[sound:{}]",
                se.audio_file.file_name().unwrap().to_str().unwrap()
            ),
        ],
    )
    .unwrap()
}

pub static GRAMMAR_MODEL: LazyLock<Model> = LazyLock::new(|| {
    const MODEL_ID: i64 = -284526913684597160;
    const BACK_COMMON: &str = r#"<h3><ruby>{{szh}}<rt>{{spy}}</rt></ruby></h3><h3>{{sen}}</h3><hr><h3><ruby>{{ezh}}<rt>{{epy}}</rt></ruby><h3>{{een}}</h3><p>HSK: {{hsk}}</p>"#;
    let template_zhen = {
        const FRONT_INNER: &str = r#"<h2>What is the <b style="color:red">grammatical structure</b> in english?</h2><h1>{{szh}}</h1>"#;
        const BACK_INNER: &str = BACK_COMMON;
        const FRONT: &str = concatcp!(PRE_HTML_COMMON, FRONT_INNER, POST_HTML_COMMON);
        const BACK: &str = concatcp!(PRE_HTML_COMMON, BACK_INNER, POST_HTML_COMMON);
        Template::new("zh_grammar_zhen").qfmt(FRONT).afmt(BACK)
    };
    let template_enzh = {
        const FRONT_INNER: &str = r#"<h2>What is the <b style="color:red">grammatical structure</b> in chinese?</h2><h1>{{sen}}</h1>"#;
        const BACK_INNER: &str = BACK_COMMON;
        const FRONT: &str = concatcp!(PRE_HTML_COMMON, FRONT_INNER, POST_HTML_COMMON);
        const BACK: &str = concatcp!(PRE_HTML_COMMON, BACK_INNER, POST_HTML_COMMON);
        Template::new("zh_grammar_enzh").qfmt(FRONT).afmt(BACK)
    };
    Model::new_with_options(
        MODEL_ID,
        "HanziGrammar",
        vec![
            Field::new("sort_field"),
            Field::new("szh"),
            Field::new("sen"),
            Field::new("spy"),
            Field::new("ezh"),
            Field::new("een"),
            Field::new("epy"),
            Field::new("hsk"),
        ],
        vec![template_zhen, template_enzh],
        Some(CSS_COMMON),
        None,
        None,
        None,
        None,
    )
});
pub fn grammar_entry_to_note(ge: GrammarEntry, idx: usize) -> Note {
    Note::new(
        GRAMMAR_MODEL.clone(),
        vec![
            // sort_field
            &format!("{:08}", idx),
            // szh
            &encode_safe(&ge.structure.zh),
            // sen
            &encode_safe(&ge.structure.en),
            // spy
            &encode_safe(&ge.structure.py.to_string()),
            // ezh
            &encode_safe(&ge.example.zh),
            // een
            &encode_safe(&ge.example.en),
            // epy
            &encode_safe(&ge.example.py.to_string()),
            // hsk
            &ge.hsk_lev
                .map(|x| x.to_string())
                .unwrap_or(String::from("no")),
        ],
    )
    .unwrap()
}

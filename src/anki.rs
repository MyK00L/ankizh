use crate::common::*;
use const_format::concatcp;
use genanki_rs::*;
use std::sync::LazyLock;

pub const DECK_ID: i64 = 9030804782668984910;

const PRE_HTML_COMMON: &str = r#"<span lang="zh-Hans">"#;
const POST_HTML_COMMON: &str = r#"<p style="display:none;">{{sort_field}}</p></span>"#;

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
rb {
    font-size: 0.5em;
}
ol {
    list-style-type: none;
}
.cw,svg {
    display: inline-block;
    width: 3em;
    height: 3em;
    font-size: 3em;
}
"#;

pub static WORD_MODEL: LazyLock<Model> = LazyLock::new(|| {
    const BACK_COMMON: &str = r#"
    <h1>{{word}}</h1>
    <h2>{{pinyin}}</h2>
    <h4>{{traditional}}</h4>
    <ol>{{definitions}}</ol>
    <ul>{{examples}}</ul>
    <p>HSK: {{#hsk}}{{hsk}}{{/hsk}}{{^hsk}}no{{/hsk}}</p>
    <details>
        <summary>Extra</summary>
        {{extra}}
    </details>
    {{#audio}}<p class=tc>{{audio}}</p>{{/audio}}
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
            <p class="tc">{{audio}}</p>
            <p class="tc">{{hint:pinyin}}</p>
        {{/audio}}
        {{^audio}}
            <p class="tc">{{pinyin}}</p>
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
        Template::new("zh_writing").qfmt(FRONT).afmt(BACK)
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
        Template::new("zh_meaning").qfmt(FRONT).afmt(BACK)
    };
    let template_reading = {
        const FRONT_INNER: &str = r#"
        <h2>What are the <b style="color:red;">readings</b>?</h2>
        <h1>{{word}}</h1>
        "#;
        const BACK_INNER: &str = BACK_COMMON;
        const FRONT: &str = concatcp!(PRE_HTML_COMMON, FRONT_INNER, POST_HTML_COMMON);
        const BACK: &str = concatcp!(PRE_HTML_COMMON, BACK_INNER, POST_HTML_COMMON);
        Template::new("zh_reading").qfmt(FRONT).afmt(BACK)
    };

    Model::new_with_options(
        MODEL_ID,
        "hanziM",
        vec![
            Field::new("sort_field"),
            Field::new("word"),
            Field::new("pinyin"),
            Field::new("definitions"),
            Field::new("writing"),
            Field::new("traditional"),
            Field::new("examples"),
            Field::new("hsk"),
            Field::new("audio"),
            Field::new("extra"),
        ],
        vec![template_writing, template_meaning, template_reading],
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
fn svg_from_strokes(strokes: Vec<String>, i0: usize) -> String {
    format!(
        r#"<svg viewbox="0 0 1024 1024"><g transform="scale(1, -1) translate(0, -900)">{}</g></svg>"#,
        strokes
            .iter()
            .enumerate()
            .map(|(i, stroke)| format!(
                r#"<path d="{}" fill="{}"></path>"#,
                stroke,
                anki_canvas_contrast(i0 + i)
            ))
            .fold(String::new(), |acc, e| acc + &e)
    )
}
fn html_from_char_writing(w: CharWriting, i0: usize) -> String {
    match w {
        CharWriting::Strokes(strokes) => svg_from_strokes(strokes, i0),
        CharWriting::Char(c) => format!(r#"<span class="cw">{}</span>"#, c),
    }
}
fn html_from_writing(w: Vec<CharWriting>) -> String {
    let mut cw = Vec::<String>::with_capacity(w.len());
    let mut i0 = 0usize;
    for c in w.into_iter() {
        let ns = match &c {
            CharWriting::Strokes(strokes) => strokes.len(),
            CharWriting::Char(_c) => 1,
        };
        cw.push(html_from_char_writing(c, i0));
        i0 += ns;
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
            &we.id,
            // pinyin
            &we.pinyin.join(", "),
            // definitions
            &we.definitions
                .into_iter()
                .map(|x| {
                    format!(
                        "<li><b>{}</b>: {}</li>",
                        x.pinyin.unwrap_or_default(),
                        x.english.join("; ")
                    )
                })
                .fold(String::new(), |acc, e| acc + &e),
            // writing
            &html_from_writing(we.writing),
            // traditional
            &we.traditional.unwrap_or_default(),
            // examlpes
            &we.examples
                .into_iter()
                .map(|x| {
                    format!(
                        "<li><ruby>{}<rt>{}</rt></ruby><br/>{}</li>",
                        x.zh, x.py, x.en
                    )
                })
                .fold(String::new(), |acc, e| acc + &e),
            // hsk
            &we.hsk_lev.map(|x| x.to_string()).unwrap_or_default(),
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

use const_format::concatcp;
use genanki_rs::*;
use std::sync::LazyLock;

pub const DECK_ID: i64 = 9030804782668984910;

pub static WORD_MODEL: LazyLock<Model> = LazyLock::new(|| {
    const BACK_COMMON: &str = r#"
    {{#audio}}{{audio}}{{/audio}}
    {{word}}
    {{pinyin}}
    {{traditional}}
    {{definitions}}
    {{examples}}
    <details>
        <summary>Extra</summary>
        {{extra}}
    </details>
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
            {{audio}}
            {{hint:pinyin}}
        {{/audio}}
        {{^audio}}
            {{pinyin}}
        {{/audio}}
        {{hint:definitions}}
        "#;
        const BACK_INNER: &str = concatcp!(
            r#"
        <div id="ac-back"></div>
        {{writing}}
        "#,
            BACK_COMMON
        );

        const FRONT: &str = concatcp!(OPTIONS_JS, FRONT_INNER, FRONT_JS);
        const BACK: &str = concatcp!(OPTIONS_JS, BACK_INNER, BACK_JS);
        Template::new("zh_writing").qfmt(FRONT).afmt(BACK)
    };

    Model::new(
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
        vec![template_writing],
    )
});

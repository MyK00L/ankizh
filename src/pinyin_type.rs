use core::fmt;
use pinyin::ToPinyin;
use serde::{Deserialize, Serialize};

fn catch_unwind_silent<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
    f: F,
) -> std::thread::Result<R> {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(f);
    std::panic::set_hook(prev_hook);
    result
}
fn process_pinyin(s: &str) -> String {
    let s = prettify_pinyin::prettify(s);
    catch_unwind_silent(|| {
        let parser = pinyin_parser::PinyinParser::new()
            .preserve_spaces(false)
            .preserve_punctuations(true)
            .with_strictness(pinyin_parser::Strictness::Loose)
            .preserve_miscellaneous(true);
        parser
            .parse(&s)
            .reduce(|acc, s| acc + " " + &s)
            .unwrap_or_default()
    })
    .unwrap_or(s)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Pinyin(String);
impl Pinyin {
    pub fn from_hanzi<S: AsRef<str>>(s: S) -> Self {
        let spy = s
            .as_ref()
            .to_pinyin()
            .flatten()
            .map(|x| x.with_tone().to_string())
            .fold(String::new(), |acc, e| acc + &e);
        Self::from(spy)
    }
}
impl fmt::Display for Pinyin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl<S: AsRef<str>> From<S> for Pinyin {
    fn from(s: S) -> Self {
        Self(process_pinyin(s.as_ref()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct CapPinyin {
    py: String,
    cap: bool,
}
impl CapPinyin {
    pub fn from_hanzi<S: AsRef<str>>(s: S) -> Self {
        let spy = s
            .as_ref()
            .to_pinyin()
            .flatten()
            .map(|x| x.with_tone().to_string())
            .fold(String::new(), |acc, e| acc + &e);
        Self::from(spy)
    }
}
impl<S: AsRef<str>> From<S> for CapPinyin {
    fn from(s: S) -> Self {
        Self {
            py: process_pinyin(s.as_ref()),
            cap: s
                .as_ref()
                .chars()
                .next()
                .map(|x| x.is_uppercase())
                .unwrap_or(false),
        }
    }
}
impl fmt::Display for CapPinyin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.cap {
            write!(
                f,
                "{}{}",
                self.py.chars().next().unwrap().to_uppercase(),
                self.py.chars().skip(1).collect::<String>()
            )
        } else {
            write!(f, "{}", self.py)
        }
    }
}

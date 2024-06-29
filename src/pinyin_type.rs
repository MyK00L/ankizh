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
    let s = s.trim();
    let s = {
        let mut ans = String::new();
        let mut lastnum = false;
        for c in s.chars() {
            if c.is_alphabetic() && lastnum {
                ans.push(' ');
            }
            ans.push(c);
            lastnum = c.is_numeric();
        }
        ans
    };
    let s = prettify_pinyin::prettify(&s);
    let s = s.trim();
    catch_unwind_silent(|| {
        let parser = pinyin_parser::PinyinParser::new()
            .preserve_spaces(true)
            .preserve_punctuations(true)
            .with_strictness(pinyin_parser::Strictness::Loose)
            .preserve_miscellaneous(true);
        parser.parse(s).fold(String::new(), |acc, s| {
            let st = s.trim();
            let sep = if acc.is_empty() || st.is_empty() {
                ""
            } else {
                " "
            };
            acc + sep + st
        })
    })
    .unwrap_or(s.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Pinyin(String);
impl Pinyin {
    /// Use sparingly, it's not very accurate
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
    pub fn is_capitalized(&self) -> bool {
        self.cap
    }
}
impl From<Pinyin> for CapPinyin {
    fn from(p: Pinyin) -> Self {
        Self {
            py: p.0,
            cap: false,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    fn test_pyfrom(a: &str, b: &str) {
        assert_eq!(Pinyin::from(&a).to_string(), b.to_string());
    }
    #[test]
    fn py() {
        test_pyfrom("wo3bu2zhi1dao english", "wǒ bú zhī dao english");
        test_pyfrom("wo3bu2zhi1dao5 english", "wǒ bú zhī dao english");
        test_pyfrom("wo3 bu2 zhi1 dao english", "wǒ bú zhī dao english");
        test_pyfrom("wǒ bú zhī dao english", "wǒ bú zhī dao english");
        test_pyfrom("wǒbúzhīdao english", "wǒ bú zhī dao english");
    }
}

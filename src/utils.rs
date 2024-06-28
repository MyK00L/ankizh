use core::hash::{Hash, Hasher};
use siphasher::sip::SipHasher;

pub fn is_good_cjk(c: char) -> bool {
    let cp: u32 = c.into();
    (0x4E00..=0x9FFF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp)
        || (0x20000..=0x2A6DF).contains(&cp)
        || (0x2A700..=0x2B73F).contains(&cp)
        || (0x2B740..=0x2B81F).contains(&cp)
        || (0x2B820..=0x2CEAF).contains(&cp)
        || (0x2CEB0..=0x2EBEF).contains(&cp)
        || (0x2EBF0..=0x2EE5F).contains(&cp)
        || (0x2F800..=0x2FA1F).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0x2F800..=0x2FA1F).contains(&cp)
        || (0x2E80..=0x2EFF).contains(&cp)
}

const BASE91_TABLE: [char; 91] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '!', '#', '$', '%', '&', '(', ')', '*', '+', ',', '-', '.', '/', ':',
    ';', '<', '=', '>', '?', '@', '[', ']', '^', '_', '`', '{', '|', '}', '~',
];
fn base91_encode(mut x: u64) -> String {
    let mut rv_reversed = vec![];
    while x != 0 {
        rv_reversed.push(BASE91_TABLE[(x % 91) as usize]);
        x /= 91;
    }
    rv_reversed.into_iter().rev().collect()
}
pub fn guid_for<T: Hash>(v: T) -> String {
    let mut h = SipHasher::new();
    v.hash(&mut h);
    let u: u64 = h.finish();
    base91_encode(u)
}

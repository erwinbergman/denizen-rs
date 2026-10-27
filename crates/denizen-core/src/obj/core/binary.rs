//! `BinaryTag`: a sequence of bytes, identified as `binary@<hex>`.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use std::any::Any;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BinaryTag {
    pub bytes: Vec<u8>,
    pub prefix: String,
}

impl BinaryTag {
    pub fn new(bytes: Vec<u8>) -> Self {
        BinaryTag { bytes, prefix: "binary".to_string() }
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 7 && s[..7].eq_ignore_ascii_case("binary@") {
            s = &s[7..];
        }
        crate::util::hex_decode(s).map(BinaryTag::new)
    }
}

impl ObjectTagImpl for BinaryTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("binary@{}", crate::util::hex_encode(&self.bytes))
    }
    fn object_type_name(&self) -> &'static str {
        "BinaryTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

pub fn register_tags() {
    let p = get_processor("BinaryTag");

    p.register_tag("ElementTag", "length", |_a, o| {
        o.downcast_ref::<BinaryTag>().map(|b| ObjectTag::new(crate::obj::ElementTag::from_int(b.bytes.len() as i64)))
    }, true, &[]);

    p.register_tag("ElementTag", "to_hex", |_a, o| {
        o.downcast_ref::<BinaryTag>().map(|b| ObjectTag::new(crate::obj::ElementTag::new(crate::util::hex_encode(&b.bytes))))
    }, true, &[]);

    p.register_tag("ElementTag", "to_base64", |_a, o| {
        o.downcast_ref::<BinaryTag>().map(|b| ObjectTag::new(crate::obj::ElementTag::new(base64_encode(&b.bytes))))
    }, true, &[]);
}

fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { CHARS[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[(n & 63) as usize] as char } else { '=' });
    }
    out
}

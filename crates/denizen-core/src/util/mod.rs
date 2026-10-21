//! Core utility helpers mirroring `com.denizenscript.denizencore.utilities.CoreUtilities`.

use std::time::{SystemTime, UNIX_EPOCH};

/// Split a string around a single character, keeping empty entries (Java semantics).
pub fn split(s: &str, c: char) -> Vec<String> {
    s.split(c).map(|x| x.to_string()).collect()
}

/// Split with a maximum number of resulting pieces (Java semantics: max is the number of parts).
pub fn split_max(s: &str, c: char, max: usize) -> Vec<String> {
    if max == 0 {
        return Vec::new();
    }
    s.splitn(max, c).map(|x| x.to_string()).collect()
}

/// Join pieces with a separator.
pub fn join<S: AsRef<str>>(parts: &[S], sep: &str) -> String {
    parts.iter().map(|p| p.as_ref()).collect::<Vec<_>>().join(sep)
}

/// ASCII-only lowercase, matching Denizen's fast-path behavior.
pub fn to_lower(s: &str) -> String {
    s.to_ascii_lowercase()
}

/// ASCII-only uppercase.
pub fn to_upper(s: &str) -> String {
    s.to_ascii_uppercase()
}

/// ASCII case-insensitive equality (length-sensitive).
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.eq_ignore_ascii_case(b)
}

/// Whether a string contains a char.
pub fn contains(s: &str, c: char) -> bool {
    s.contains(c)
}

/// Concatenate pieces with a separator.
pub fn concat(parts: &[String], sep: &str) -> String {
    parts.join(sep)
}

/// Replace all occurrences of `find` with `swap`.
pub fn replace(original: &str, find: &str, swap: &str) -> String {
    if find.is_empty() {
        return original.to_string();
    }
    original.replace(find, swap)
}

/// Remove NBSP characters.
pub fn clear_nbsps(input: &str) -> String {
    input.replace('\u{00A0}', "")
}

/// Truncate a float to a clean string, matching Denizen's formatting (no trailing `.0`).
pub fn double_to_string(d: f64) -> String {
    if d.is_nan() {
        return "NaN".to_string();
    }
    if d.is_infinite() {
        return if d > 0.0 { "Infinity".to_string() } else { "-Infinity".to_string() };
    }
    if d == d.trunc() && d.abs() < 1e15 {
        return format!("{}", d as i64);
    }
    let mut s = format!("{}", d);
    if s.contains('e') || s.contains('E') {
        s = format!("{:.6}", d);
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

/// Convert a float to a clean string.
pub fn float_to_clean_string(f: f32) -> String {
    double_to_string(f as f64)
}

/// Format a number as money (two decimal places, with grouping disabled to match core).
pub fn format_money(amount: f64) -> String {
    let neg = amount < 0.0;
    let cents = (amount.abs() * 100.0).round() as i64;
    let whole = cents / 100;
    let frac = cents % 100;
    let mut out = String::new();
    let whole_str = whole.to_string();
    for (i, ch) in whole_str.chars().enumerate() {
        if i > 0 && (whole_str.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    format!("{}{}.{:02}", if neg { "-" } else { "" }, out, frac)
}

/// Split a string into lines by a maximum character count.
pub fn split_lines_by_character_count(s: &str, length: usize) -> String {
    if length == 0 {
        return s.to_string();
    }
    let mut result = String::new();
    let mut count = 0;
    for word in s.split(' ') {
        if count + word.len() + 1 > length {
            result.push('\n');
            count = 0;
        } else if count > 0 {
            result.push(' ');
            count += 1;
        }
        result.push_str(word);
        count += word.len();
    }
    result
}

/// Fix newline characters into list separation (core behavior).
pub fn fix_newlines_to_list_separation(list: &mut Vec<String>) {
    let mut out = Vec::new();
    for entry in list.drain(..) {
        for part in entry.split('\n') {
            out.push(part.to_string());
        }
    }
    *list = out;
}

/// Levenshtein distance.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// Closest option from a list by Levenshtein distance.
pub fn get_closest_option(strs: &[String], opt: &str) -> String {
    let mut best: Option<(usize, &String)> = None;
    for s in strs {
        let d = levenshtein_distance(s, opt);
        if best.map(|(bd, _)| d < bd).unwrap_or(true) {
            best = Some((d, s));
        }
    }
    best.map(|(_, s)| s.clone()).unwrap_or_default()
}

/// Parse a UUID with or without dashes; returns canonical dashed form.
pub fn try_parse_uuid(input: &str) -> Option<String> {
    let clean: String = input.chars().filter(|c| *c != '-').collect();
    if clean.len() != 32 || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!(
        "{}-{}-{}-{}-{}",
        &clean[0..8],
        &clean[8..12],
        &clean[12..16],
        &clean[16..20],
        &clean[20..32]
    ))
}

/// Hex encode bytes.
pub fn hex_encode(value: &[u8]) -> String {
    let mut s = String::with_capacity(value.len() * 2);
    for b in value {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Hex decode a string.
pub fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = (bytes[i] as char).to_digit(16)?;
        let lo = (bytes[i + 1] as char).to_digit(16)?;
        out.push((hi * 16 + lo) as u8);
    }
    Some(out)
}

/// Current unix millis.
pub fn current_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Monotonic millis (approximate via process uptime).
pub fn monotonic_millis() -> i64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let start = START.get_or_init(std::time::Instant::now);
    start.elapsed().as_millis() as i64
}

/// Compute an MD5 hex digest. Small self-contained implementation (RFC 1321).
pub fn md5_hex(input: &[u8]) -> String {
    let mut ctx = Md5::new();
    ctx.update(input);
    hex_encode(&ctx.finish())
}

struct Md5 {
    state: [u32; 4],
    len: u64,
    buf: [u8; 64],
    buflen: usize,
}

impl Md5 {
    fn new() -> Self {
        Md5 { state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476], len: 0, buf: [0; 64], buflen: 0 }
    }

    fn update(&mut self, data: &[u8]) {
        self.len += data.len() as u64;
        for &b in data {
            self.buf[self.buflen] = b;
            self.buflen += 1;
            if self.buflen == 64 {
                let block = self.buf;
                self.process(&block);
                self.buflen = 0;
            }
        }
    }

    fn process(&mut self, block: &[u8; 64]) {
        const S: [u32; 64] = [
            7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20,
            5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
            6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
        ];
        const K: [u32; 64] = [
            0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
            0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
            0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
            0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
            0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
            0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
            0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
            0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
            0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
            0xeb86d391,
        ];
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes([block[i * 4], block[i * 4 + 1], block[i * 4 + 2], block[i * 4 + 3]]);
        }
        let (mut a, mut b, mut c, mut d) = (self.state[0], self.state[1], self.state[2], self.state[3]);
        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | (!b & d), i),
                16..=31 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let tmp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(K[i])
                    .wrapping_add(m[g])
                    .rotate_left(S[i]),
            );
            a = tmp;
        }
        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
    }

    fn finish(mut self) -> Vec<u8> {
        let bit_len = self.len.wrapping_mul(8);
        self.update(&[0x80]);
        while self.buflen != 56 {
            self.update(&[0]);
        }
        let mut pad = [0u8; 8];
        pad.copy_from_slice(&bit_len.to_le_bytes());
        // manually append without affecting len semantics
        for &b in pad.iter() {
            self.buf[self.buflen] = b;
            self.buflen += 1;
            if self.buflen == 64 {
                let block = self.buf;
                self.process(&block);
                self.buflen = 0;
            }
        }
        let mut out = Vec::with_capacity(16);
        for s in self.state {
            out.extend_from_slice(&s.to_le_bytes());
        }
        out
    }
}

/// Roman numerals.
pub fn to_roman(mut num: i64) -> String {
    if num <= 0 {
        return num.to_string();
    }
    const VALS: &[(i64, &str)] = &[
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"), (50, "L"),
        (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
    ];
    let mut out = String::new();
    for &(v, s) in VALS {
        while num >= v {
            out.push_str(s);
            num -= v;
        }
    }
    out
}

/// Parse a roman numeral (returns None if invalid).
pub fn from_roman(s: &str) -> Option<i64> {
    fn val(c: char) -> Option<i64> {
        Some(match c.to_ascii_uppercase() {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None,
        })
    }
    let chars: Vec<char> = s.chars().collect();
    let mut total = 0;
    for i in 0..chars.len() {
        let v = val(chars[i])?;
        if i + 1 < chars.len() && v < val(chars[i + 1])? {
            total -= v;
        } else {
            total += v;
        }
    }
    Some(total)
}

/// Split a string into lines preserving trailing empty lines semantics.
pub fn split_lines(s: &str) -> Vec<String> {
    s.split('\n').map(|l| l.trim_end_matches('\r').to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md5_known() {
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(md5_hex(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(md5_hex(b"The quick brown fox jumps over the lazy dog"), "9e107d9d372bb6826bd81d3542a419d6");
    }

    #[test]
    fn roman_roundtrip() {
        assert_eq!(to_roman(1994), "MCMXCIV");
        assert_eq!(from_roman("MCMXCIV"), Some(1994));
    }

    #[test]
    fn doubles() {
        assert_eq!(double_to_string(1.0), "1");
        assert_eq!(double_to_string(1.5), "1.5");
    }
}

pub mod yaml;
pub mod debug;

pub use debug::{debug_log, strip_colors, set_debug_enabled, is_debug_enabled, set_log_sink};

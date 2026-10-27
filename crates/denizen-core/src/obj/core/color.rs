//! `ColorTag`: an RGB color, identified as `color@r,g,b`.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use std::any::Any;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorTag {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub prefix: &'static str,
}

impl ColorTag {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        ColorTag { red: r, green: g, blue: b, prefix: "color" }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let h = hex.trim_start_matches('#');
        if h.len() != 6 {
            return None;
        }
        Some(ColorTag::new(
            u8::from_str_radix(&h[0..2], 16).ok()?,
            u8::from_str_radix(&h[2..4], 16).ok()?,
            u8::from_str_radix(&h[4..6], 16).ok()?,
        ))
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 6 && s[..6].eq_ignore_ascii_case("color@") {
            s = &s[6..];
        }
        if s.starts_with('#') {
            return ColorTag::from_hex(s);
        }
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() == 3 {
            return Some(ColorTag::new(
                parts[0].trim().parse().ok()?,
                parts[1].trim().parse().ok()?,
                parts[2].trim().parse().ok()?,
            ));
        }
        // Named colors.
        match s.to_lowercase().as_str() {
            "red" => Some(ColorTag::new(255, 0, 0)),
            "green" => Some(ColorTag::new(0, 255, 0)),
            "blue" => Some(ColorTag::new(0, 0, 255)),
            "white" => Some(ColorTag::new(255, 255, 255)),
            "black" => Some(ColorTag::new(0, 0, 0)),
            "yellow" => Some(ColorTag::new(255, 255, 0)),
            "cyan" | "aqua" => Some(ColorTag::new(0, 255, 255)),
            "magenta" | "pink" => Some(ColorTag::new(255, 0, 255)),
            "orange" => Some(ColorTag::new(255, 165, 0)),
            "purple" => Some(ColorTag::new(128, 0, 128)),
            "gray" | "grey" => Some(ColorTag::new(128, 128, 128)),
            _ => None,
        }
    }
}

impl ObjectTagImpl for ColorTag {
    fn prefix(&self) -> String {
        self.prefix.to_string()
    }
    fn identify(&self) -> String {
        format!("color@{},{},{}", self.red, self.green, self.blue)
    }
    fn identify_simple(&self) -> String {
        self.to_hex()
    }
    fn object_type_name(&self) -> &'static str {
        "ColorTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(*self)
    }
}

pub fn register_tags() {
    let p = get_processor("ColorTag");
    p.register_tag("ElementTag", "red", |_a, o| o.downcast_ref::<ColorTag>().copied().map(|c| ObjectTag::new(crate::obj::ElementTag::from_int(c.red as i64))), true, &[]);
    p.register_tag("ElementTag", "green", |_a, o| o.downcast_ref::<ColorTag>().copied().map(|c| ObjectTag::new(crate::obj::ElementTag::from_int(c.green as i64))), true, &[]);
    p.register_tag("ElementTag", "blue", |_a, o| o.downcast_ref::<ColorTag>().copied().map(|c| ObjectTag::new(crate::obj::ElementTag::from_int(c.blue as i64))), true, &[]);
    p.register_tag("ElementTag", "hex", |_a, o| o.downcast_ref::<ColorTag>().copied().map(|c| ObjectTag::new(crate::obj::ElementTag::new(c.to_hex()))), true, &[]);
    p.register_tag("ElementTag", "rgb", |_a, o| {
        o.downcast_ref::<ColorTag>().copied().map(|c| ObjectTag::new(crate::obj::ElementTag::new(format!("{}, {}, {}", c.red, c.green, c.blue))))
    }, true, &[]);
}

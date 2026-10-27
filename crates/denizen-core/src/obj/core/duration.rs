//! `DurationTag`: a span of time, with a short form (eg `2m`) and a formatted form.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use std::any::Any;

/// A duration measured in seconds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DurationTag {
    pub seconds: f64,
    pub prefix: &'static str,
}

impl DurationTag {
    pub fn from_seconds(s: f64) -> Self {
        DurationTag { seconds: s, prefix: "duration" }
    }

    pub fn from_millis(ms: f64) -> Self {
        DurationTag { seconds: ms / 1000.0, prefix: "duration" }
    }

    pub fn as_seconds(&self) -> f64 {
        self.seconds
    }

    pub fn as_millis(&self) -> f64 {
        self.seconds * 1000.0
    }

    pub fn as_ticks(&self) -> f64 {
        self.seconds * 20.0
    }

    /// Parse a duration string like `5s`, `2m`, `1h`, `3d`, or `12.5`.
    pub fn from_identify(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        let lower = s.to_lowercase();
        let last = lower.chars().last()?;
        let (numeric, mult) = match last {
            'y' => (&lower[..lower.len() - 1], 60.0 * 60.0 * 24.0 * 365.0),
            'w' => (&lower[..lower.len() - 1], 60.0 * 60.0 * 24.0 * 7.0),
            'd' => (&lower[..lower.len() - 1], 60.0 * 60.0 * 24.0),
            'h' => (&lower[..lower.len() - 1], 60.0 * 60.0),
            'm' => (&lower[..lower.len() - 1], 60.0),
            's' => (&lower[..lower.len() - 1], 1.0),
            't' => (&lower[..lower.len() - 1], 0.05),
            _ => (lower.as_str(), 1.0),
        };
        if numeric.is_empty() {
            return None;
        }
        let num = numeric.parse::<f64>().ok()?;
        Some(DurationTag::from_seconds(num * mult))
    }

    /// Short identify form, eg `2m` or `90s`.
    pub fn identify_short(&self) -> String {
        let s = self.seconds;
        if s.abs() < 1e-9 {
            return "0s".to_string();
        }
        let choices: &[(f64, &str)] = &[
            (60.0 * 60.0 * 24.0 * 365.0, "y"),
            (60.0 * 60.0 * 24.0 * 7.0, "w"),
            (60.0 * 60.0 * 24.0, "d"),
            (60.0 * 60.0, "h"),
            (60.0, "m"),
            (1.0, "s"),
        ];
        for &(unit, suffix) in choices {
            if s.abs() >= unit {
                let v = s / unit;
                if (v - v.trunc()).abs() < 1e-9 {
                    return format!("{}{}", v.trunc() as i64, suffix);
                }
                return format!("{}{}", crate::util::double_to_string((v * 100.0).round() / 100.0), suffix);
            }
        }
        format!("{}t", (s * 20.0).round() as i64)
    }

    /// Human-readable formatted form.
    pub fn formatted(&self) -> String {
        let mut rem = self.seconds;
        let mut parts = Vec::new();
        let units: &[(f64, &str)] = &[
            (60.0 * 60.0 * 24.0 * 365.0, "year"),
            (60.0 * 60.0 * 24.0 * 7.0, "week"),
            (60.0 * 60.0 * 24.0, "day"),
            (60.0 * 60.0, "hour"),
            (60.0, "minute"),
            (1.0, "second"),
        ];
        for &(unit, name) in units {
            let count = (rem / unit).floor();
            if count >= 1.0 {
                rem -= count * unit;
                let label = if count == 1.0 { name.to_string() } else { format!("{}s", name) };
                parts.push(format!("{} {}", count as i64, label));
            }
        }
        if parts.is_empty() {
            return "0 seconds".to_string();
        }
        parts.join(", ")
    }
}

impl ObjectTagImpl for DurationTag {
    fn prefix(&self) -> String {
        self.prefix.to_string()
    }
    fn identify(&self) -> String {
        self.identify_short()
    }
    fn object_type_name(&self) -> &'static str {
        "DurationTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(*self)
    }
    fn debuggable(&self) -> String {
        format!("{} <GR>({})", self.identify(), self.formatted())
    }
}

/// Register DurationTag tags.
pub fn register_tags() {
    let p = get_processor("DurationTag");

    p.register_tag("ElementTag", "seconds", |_a, o| {
        o.to_duration().map(|d| ObjectTag::new(crate::obj::ElementTag::from_double(d.seconds)))
    }, true, &[]);

    p.register_tag("ElementTag", "milliseconds", |_a, o| {
        o.to_duration().map(|d| ObjectTag::new(crate::obj::ElementTag::from_double(d.as_millis())))
    }, true, &[]);

    p.register_tag("ElementTag", "ticks", |_a, o| {
        o.to_duration().map(|d| ObjectTag::new(crate::obj::ElementTag::from_double(d.as_ticks())))
    }, true, &[]);

    p.register_tag("ElementTag", "formatted", |_a, o| {
        o.to_duration().map(|d| ObjectTag::new(crate::obj::ElementTag::new(d.formatted())))
    }, true, &[]);

    p.register_tag("ElementTag", "as_seconds", |_a, o| {
        o.to_duration().map(|d| ObjectTag::new(crate::obj::ElementTag::from_double(d.seconds)))
    }, true, &[]);

    p.register_tag_with_param("DurationTag", "ElementTag", "plus", |_a, o, param| {
        let d = o.to_duration()?;
        let add = DurationTag::from_identify(&param.as_element().value)?;
        Some(ObjectTag::new(DurationTag::from_seconds(d.seconds + add.seconds)))
    }, true, &["add"]);

    p.register_tag_with_param("DurationTag", "ElementTag", "minus", |_a, o, param| {
        let d = o.to_duration()?;
        let sub = DurationTag::from_identify(&param.as_element().value)?;
        Some(ObjectTag::new(DurationTag::from_seconds(d.seconds - sub.seconds)))
    }, true, &["subtract"]);
}

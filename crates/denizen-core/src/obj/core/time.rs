//! `TimeTag`: a calendar date/time, identified as `time@<epoch_millis>`.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use std::any::Any;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TimeTag {
    /// Milliseconds since the Unix epoch.
    pub millis: i64,
    pub prefix: &'static str,
}

impl TimeTag {
    pub fn from_millis(ms: i64) -> Self {
        TimeTag { millis: ms, prefix: "time" }
    }

    pub fn now() -> Self {
        TimeTag::from_millis(crate::util::current_time_millis())
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 5 && s[..5].eq_ignore_ascii_case("time@") {
            s = &s[5..];
        }
        s.parse::<i64>().ok().map(TimeTag::from_millis)
    }

    /// Days since the epoch (1970-01-01) for the stored instant (UTC).
    fn days_and_secs(&self) -> (i64, i64) {
        let secs = self.millis.div_euclid(1000);
        (secs.div_euclid(86400), secs.rem_euclid(86400))
    }

    /// Convert to a calendar date (year, month, day) using the civil-from-days algorithm.
    pub fn year_month_day(&self) -> (i64, u32, u32) {
        let (days, _) = self.days_and_secs();
        civil_from_days(days)
    }

    pub fn hour_minute_second(&self) -> (u32, u32, u32) {
        let (_, secs) = self.days_and_secs();
        ((secs / 3600) as u32, ((secs % 3600) / 60) as u32, (secs % 60) as u32)
    }

    pub fn day_of_week(&self) -> u32 {
        let (days, _) = self.days_and_secs();
        // 1970-01-01 was a Thursday (4).
        ((days + 4).rem_euclid(7)) as u32
    }

    pub fn month_name(&self) -> &'static str {
        const MONTHS: [&str; 12] = [
            "January", "February", "March", "April", "May", "June", "July", "August", "September",
            "October", "November", "December",
        ];
        let (_, m, _) = self.year_month_day();
        MONTHS[(m - 1) as usize]
    }

    pub fn day_name(&self) -> &'static str {
        const DAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
        DAYS[self.day_of_week() as usize]
    }

    /// Formats the time using a Java `SimpleDateFormat`-like pattern subset.
    pub fn format(&self, pattern: &str) -> String {
        let (y, mo, d) = self.year_month_day();
        let (h, mi, se) = self.hour_minute_second();
        let mut out = String::new();
        let mut chars = pattern.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                'y' => {
                    let mut n = 1;
                    while chars.peek() == Some(&'y') {
                        chars.next();
                        n += 1;
                    }
                    if n <= 2 {
                        out.push_str(&format!("{:02}", y % 100));
                    } else {
                        out.push_str(&format!("{:04}", y));
                    }
                }
                'M' => {
                    let mut n = 1;
                    while chars.peek() == Some(&'M') {
                        chars.next();
                        n += 1;
                    }
                    match n {
                        1 => out.push_str(&format!("{}", mo)),
                        2 => out.push_str(&format!("{:02}", mo)),
                        3 => out.push_str(&self.month_name()[..3]),
                        _ => out.push_str(self.month_name()),
                    }
                }
                'd' => {
                    let mut n = 1;
                    while chars.peek() == Some(&'d') {
                        chars.next();
                        n += 1;
                    }
                    if n == 1 {
                        out.push_str(&format!("{}", d));
                    } else {
                        out.push_str(&format!("{:02}", d));
                    }
                }
                'H' => out.push_str(&format!("{:02}", h)),
                'm' => out.push_str(&format!("{:02}", mi)),
                's' => out.push_str(&format!("{:02}", se)),
                'E' => {
                    let mut n = 1;
                    while chars.peek() == Some(&'E') {
                        chars.next();
                        n += 1;
                    }
                    if n <= 3 {
                        out.push_str(&self.day_name()[..3]);
                    } else {
                        out.push_str(self.day_name());
                    }
                }
                '\'' => {
                    while let Some(&nc) = chars.peek() {
                        chars.next();
                        if nc == '\'' {
                            break;
                        }
                        out.push(nc);
                    }
                }
                other => out.push(other),
            }
        }
        out
    }
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

impl ObjectTagImpl for TimeTag {
    fn prefix(&self) -> String {
        self.prefix.to_string()
    }
    fn identify(&self) -> String {
        format!("time@{}", self.millis)
    }
    fn identify_simple(&self) -> String {
        self.format("yyyy-MM-dd HH:mm:ss")
    }
    fn object_type_name(&self) -> &'static str {
        "TimeTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(*self)
    }
}

pub fn register_tags() {
    let p = get_processor("TimeTag");
    p.register_tag("ElementTag", "year", |_a, o| o.downcast_ref::<TimeTag>().copied().map(|t| ObjectTag::new(crate::obj::ElementTag::from_int(t.year_month_day().0))), true, &[]);
    p.register_tag("ElementTag", "month", |_a, o| o.downcast_ref::<TimeTag>().copied().map(|t| ObjectTag::new(crate::obj::ElementTag::from_int(t.year_month_day().1 as i64))), true, &[]);
    p.register_tag("ElementTag", "day", |_a, o| o.downcast_ref::<TimeTag>().copied().map(|t| ObjectTag::new(crate::obj::ElementTag::from_int(t.year_month_day().2 as i64))), true, &[]);
    p.register_tag("ElementTag", "hour", |_a, o| o.downcast_ref::<TimeTag>().copied().map(|t| ObjectTag::new(crate::obj::ElementTag::from_int(t.hour_minute_second().0 as i64))), true, &[]);
    p.register_tag("ElementTag", "minute", |_a, o| o.downcast_ref::<TimeTag>().copied().map(|t| ObjectTag::new(crate::obj::ElementTag::from_int(t.hour_minute_second().1 as i64))), true, &[]);
    p.register_tag("ElementTag", "second", |_a, o| o.downcast_ref::<TimeTag>().copied().map(|t| ObjectTag::new(crate::obj::ElementTag::from_int(t.hour_minute_second().2 as i64))), true, &[]);
    p.register_tag("ElementTag", "millis", |_a, o| o.downcast_ref::<TimeTag>().copied().map(|t| ObjectTag::new(crate::obj::ElementTag::from_int(t.millis))), true, &[]);
    p.register_tag("ElementTag", "month_name", |_a, o| o.downcast_ref::<TimeTag>().copied().map(|t| ObjectTag::new(crate::obj::ElementTag::new(t.month_name()))), true, &[]);
    p.register_tag("ElementTag", "day_name", |_a, o| o.downcast_ref::<TimeTag>().copied().map(|t| ObjectTag::new(crate::obj::ElementTag::new(t.day_name()))), true, &[]);
    p.register_tag_with_param("ElementTag", "ElementTag", "format", |_a, o, param| {
        o.downcast_ref::<TimeTag>().copied().map(|t| ObjectTag::new(crate::obj::ElementTag::new(t.format(&param.as_element().value))))
    }, true, &[]);
}

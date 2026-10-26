//! Date and time fields (`a:fld type="datetime…"`): the text each field type shows for a moment,
//! and the local clock.
//!
//! The field types and their formats follow ECMA-376 Part 1, §21.1.2.2.4 (`fld`): `datetime1`
//! to `datetime13`, written as in PowerPoint's English (United States) Date and Time list.
//! `datetime` and `datetimeFigureOut` show the language's short date. Numeric dates follow the
//! run's language; formats with month or day names, or AM/PM, are computed for English and
//! Vietnamese (`vi`: day/month/year, "Thứ Hai, 26 tháng 10 năm 2026", 24-hour time), so other
//! languages keep the text the file was saved with.

/// A local date and time: what the clock read, or a fixed moment (tests, deterministic output).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTime {
    pub year: i32,
    /// 1–12.
    pub month: u32,
    /// 1–31.
    pub day: u32,
    /// 0–23.
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl DateTime {
    pub fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Self {
        DateTime { year, month, day, hour, minute, second }
    }

    /// The local date and time now (the user's time zone); `None` where there is no clock (wasm).
    pub fn now_local() -> Option<DateTime> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use chrono::{Datelike, Timelike};
            let n = chrono::Local::now();
            Some(DateTime::new(n.year(), n.month(), n.day(), n.hour(), n.minute(), n.second()))
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }

    /// A real calendar date and time of day.
    fn is_valid(&self) -> bool {
        let leap = self.year % 4 == 0 && (self.year % 100 != 0 || self.year % 400 == 0);
        let days = match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return false,
        };
        (1..=9999).contains(&self.year) && (1..=days).contains(&self.day) && self.hour < 24 && self.minute < 60 && self.second < 61
    }

    /// Day of the week, 0 = Sunday (days since 1970-01-01, Howard Hinnant's algorithm).
    fn weekday(&self) -> usize {
        let (y, m, d) = (i64::from(self.year), i64::from(self.month), i64::from(self.day));
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        let days = era * 146_097 + doe - 719_468;
        (days + 4).rem_euclid(7) as usize
    }
}

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const DAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
/// Vietnamese weekday names, Sunday first (as [`DAYS`]).
const VI_DAYS: [&str; 7] = ["Chủ Nhật", "Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy"];

/// Is field `kind` a date/time field?
pub fn is_date_field(kind: &str) -> bool {
    kind.starts_with("datetime")
}

/// The text date/time field `kind` shows at `now` in a run of language `lang` (BCP 47, e.g.
/// `en-US`). `None` when `kind` isn't a date/time type this knows, `now` isn't a real date, or
/// the format needs names in a language other than English: the field keeps its saved text.
pub fn field_text(kind: &str, lang: Option<&str>, now: DateTime) -> Option<String> {
    if !now.is_valid() {
        return None;
    }
    let lang = lang.map(str::trim).filter(|l| !l.is_empty()).unwrap_or("en-US").to_ascii_lowercase();
    if lang == "vi" || lang.starts_with("vi-") || lang.starts_with("vi_") {
        return vietnamese(kind, now);
    }
    let english = lang == "en" || lang.starts_with("en-");
    let month = MONTHS.get(now.month.checked_sub(1)? as usize)?;
    let mon = month.get(..3)?;
    let day = DAYS.get(now.weekday())?;
    let (d, y) = (now.day, now.year);
    let yy = y.rem_euclid(100);
    let h12 = match now.hour % 12 {
        0 => 12,
        h => h,
    };
    let ampm = if now.hour < 12 { "AM" } else { "PM" };
    let (mi, s) = (now.minute, now.second);
    let short = short_date(&lang, now);
    let named = |text: String| english.then_some(text);
    match kind {
        "datetime" | "datetimeFigureOut" | "datetime1" => Some(short),
        "datetime2" => named(format!("{day}, {month} {d}, {y}")),
        "datetime3" => named(format!("{d} {month} {y}")),
        "datetime4" => named(format!("{month} {d}, {y}")),
        "datetime5" => named(format!("{d}-{mon}-{yy:02}")),
        "datetime6" => named(format!("{month} {yy:02}")),
        "datetime7" => named(format!("{mon}-{yy:02}")),
        "datetime8" => named(format!("{short} {h12}:{mi:02} {ampm}")),
        "datetime9" => named(format!("{short} {h12}:{mi:02}:{s:02} {ampm}")),
        "datetime10" => Some(format!("{}:{mi:02}", now.hour)),
        "datetime11" => Some(format!("{}:{mi:02}:{s:02}", now.hour)),
        "datetime12" => named(format!("{h12}:{mi:02} {ampm}")),
        "datetime13" => named(format!("{h12}:{mi:02}:{s:02} {ampm}")),
        _ => None,
    }
}

/// Vietnamese: `dd/MM/yyyy`, names written out ("Thứ Hai, 26 tháng 10 năm 2026"), `HH:mm` 24-hour
/// time. The types with an English abbreviation (`datetime7`, "Oct-26") or a 12-hour clock
/// (`datetime9`, `datetime12`, `datetime13`) have no Vietnamese form.
fn vietnamese(kind: &str, now: DateTime) -> Option<String> {
    let (d, m, y) = (now.day, now.month, now.year);
    let yy = y.rem_euclid(100);
    let short = format!("{d:02}/{m:02}/{y}");
    let hm = format!("{:02}:{:02}", now.hour, now.minute);
    let day = VI_DAYS.get(now.weekday())?;
    Some(match kind {
        "datetime" | "datetimeFigureOut" | "datetime1" => short,
        "datetime2" => format!("{day}, {d} tháng {m} năm {y}"),
        "datetime3" => format!("ngày {d} tháng {m} năm {y}"),
        "datetime4" => format!("{d} tháng {m}, {y}"),
        "datetime5" => format!("{d:02}/{m:02}/{yy:02}"),
        "datetime6" => format!("tháng {m}/{y}"),
        "datetime8" => format!("{short} {hm}"),
        "datetime10" => hm,
        "datetime11" => format!("{hm}:{:02}", now.second),
        _ => return None,
    })
}

/// The short (numeric) date of language `lang` (lower case), US order unless the language is
/// known to write its dates another way.
fn short_date(lang: &str, t: DateTime) -> String {
    let (y, m, d) = (t.year, t.month, t.day);
    let primary = lang.split(['-', '_']).next().unwrap_or("");
    match lang {
        "en-gb" | "en-au" | "en-nz" | "en-ie" | "en-in" => return format!("{d:02}/{m:02}/{y}"),
        "en-ca" | "fr-ca" | "en-za" => return format!("{y}-{m:02}-{d:02}"),
        _ => {}
    }
    match primary {
        "fr" | "es" | "it" | "pt" | "el" | "ca" | "vi" => format!("{d:02}/{m:02}/{y}"),
        "de" | "ru" | "pl" | "cs" | "sk" | "tr" | "fi" | "nb" | "nn" | "no" | "da" | "uk" | "ro" => format!("{d:02}.{m:02}.{y}"),
        "nl" => format!("{d}-{m}-{y}"),
        "ja" | "zh" => format!("{y}/{m}/{d}"),
        "ko" | "sv" | "lt" => format!("{y}-{m:02}-{d:02}"),
        _ => format!("{m}/{d}/{y}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRI: DateTime = DateTime { year: 2026, month: 10, day: 9, hour: 20, minute: 5, second: 7 };

    #[test]
    fn english_formats_of_every_type() {
        let f = |k: &str| field_text(k, Some("en-US"), FRI);
        assert_eq!(f("datetime1").as_deref(), Some("10/9/2026"));
        assert_eq!(f("datetime2").as_deref(), Some("Friday, October 9, 2026"));
        assert_eq!(f("datetime3").as_deref(), Some("9 October 2026"));
        assert_eq!(f("datetime4").as_deref(), Some("October 9, 2026"));
        assert_eq!(f("datetime5").as_deref(), Some("9-Oct-26"));
        assert_eq!(f("datetime6").as_deref(), Some("October 26"));
        assert_eq!(f("datetime7").as_deref(), Some("Oct-26"));
        assert_eq!(f("datetime8").as_deref(), Some("10/9/2026 8:05 PM"));
        assert_eq!(f("datetime9").as_deref(), Some("10/9/2026 8:05:07 PM"));
        assert_eq!(f("datetime10").as_deref(), Some("20:05"));
        assert_eq!(f("datetime11").as_deref(), Some("20:05:07"));
        assert_eq!(f("datetime12").as_deref(), Some("8:05 PM"));
        assert_eq!(f("datetime13").as_deref(), Some("8:05:07 PM"));
        assert_eq!(f("datetimeFigureOut").as_deref(), Some("10/9/2026"));
        assert_eq!(f("datetime").as_deref(), Some("10/9/2026"));
        // No language: US English. Midnight and noon are 12 on the clock.
        assert_eq!(field_text("datetime12", None, DateTime::new(2024, 2, 29, 0, 0, 0)).as_deref(), Some("12:00 AM"));
        assert_eq!(field_text("datetime12", None, DateTime::new(2024, 2, 29, 12, 30, 0)).as_deref(), Some("12:30 PM"));
        assert_eq!(field_text("datetime2", None, DateTime::new(2000, 1, 1, 0, 0, 0)).as_deref(), Some("Saturday, January 1, 2000"));
    }

    #[test]
    fn language_orders_numeric_dates_and_keeps_names_english_only() {
        assert_eq!(field_text("datetime1", Some("en-GB"), FRI).as_deref(), Some("09/10/2026"));
        assert_eq!(field_text("datetimeFigureOut", Some("de-DE"), FRI).as_deref(), Some("09.10.2026"));
        assert_eq!(field_text("datetime1", Some("ja-JP"), FRI).as_deref(), Some("2026/10/9"));
        assert_eq!(field_text("datetime1", Some("sv-SE"), FRI).as_deref(), Some("2026-10-09"));
        assert_eq!(field_text("datetime10", Some("fr-FR"), FRI).as_deref(), Some("20:05"));
        // Month names in another language: not computed (the field keeps its saved text).
        assert_eq!(field_text("datetime2", Some("de-DE"), FRI), None);
        assert_eq!(field_text("datetime12", Some("fr-FR"), FRI), None);
        assert_eq!(field_text("datetime4", Some("en-GB"), FRI).as_deref(), Some("October 9, 2026"));
    }

    /// Vietnamese: day/month/year, names in Vietnamese ("Thứ Hai", "tháng 10"), 24-hour time with
    /// no AM/PM; the formats with an English abbreviation or a 12-hour clock aren't offered.
    #[test]
    fn vietnamese_formats_of_every_type() {
        let mon = DateTime::new(2026, 10, 26, 11, 37, 55);
        let f = |k: &str| field_text(k, Some("vi-VN"), mon);
        assert_eq!(f("datetime1").as_deref(), Some("26/10/2026"));
        assert_eq!(f("datetime2").as_deref(), Some("Thứ Hai, 26 tháng 10 năm 2026"));
        assert_eq!(f("datetime3").as_deref(), Some("ngày 26 tháng 10 năm 2026"));
        assert_eq!(f("datetime4").as_deref(), Some("26 tháng 10, 2026"));
        assert_eq!(f("datetime5").as_deref(), Some("26/10/26"));
        assert_eq!(f("datetime6").as_deref(), Some("tháng 10/2026"));
        assert_eq!(f("datetime7"), None);
        assert_eq!(f("datetime8").as_deref(), Some("26/10/2026 11:37"));
        assert_eq!(f("datetime9"), None);
        assert_eq!(f("datetime10").as_deref(), Some("11:37"));
        assert_eq!(f("datetime11").as_deref(), Some("11:37:55"));
        assert_eq!(f("datetime12"), None);
        assert_eq!(f("datetime13"), None);
        assert_eq!(f("datetimeFigureOut").as_deref(), Some("26/10/2026"));
        // "vi" alone, any case; single digits, the evening and every weekday name.
        let early = DateTime::new(2026, 3, 1, 20, 5, 7);
        assert_eq!(field_text("datetime2", Some("VI"), early).as_deref(), Some("Chủ Nhật, 1 tháng 3 năm 2026"));
        assert_eq!(field_text("datetime8", Some("vi"), early).as_deref(), Some("01/03/2026 20:05"));
        assert_eq!(field_text("datetime10", Some("vi-VN"), DateTime::new(2026, 3, 1, 8, 5, 0)).as_deref(), Some("08:05"));
        let names: Vec<String> = (26..=32)
            .map(|d| {
                field_text("datetime2", Some("vi-VN"), DateTime::new(2026, 10 + d / 32, if d > 31 { 1 } else { d }, 0, 0, 0)).unwrap_or_default()
            })
            .collect();
        let days: Vec<&str> = names.iter().filter_map(|n| n.split(',').next()).collect();
        assert_eq!(days, ["Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy", "Chủ Nhật"]);
    }

    #[test]
    fn unknown_types_and_impossible_dates_give_nothing() {
        assert_eq!(field_text("slidenum", None, FRI), None);
        assert_eq!(field_text("datetime14", None, FRI), None);
        for bad in [
            DateTime::new(2026, 0, 9, 0, 0, 0),
            DateTime::new(2026, 13, 9, 0, 0, 0),
            DateTime::new(2026, 2, 29, 0, 0, 0),
            DateTime::new(2026, 4, 31, 0, 0, 0),
            DateTime::new(2026, 1, 1, 24, 0, 0),
            DateTime::new(i32::MIN, 1, 1, 0, 0, 0),
            DateTime::new(i32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX),
        ] {
            assert_eq!(field_text("datetime2", None, bad), None, "{bad:?}");
        }
        assert!(is_date_field("datetimeFigureOut") && !is_date_field("slidenum"));
    }

    #[test]
    fn the_clock_reads_a_real_date() {
        if let Some(now) = DateTime::now_local() {
            assert!(field_text("datetime1", None, now).is_some(), "{now:?}");
        }
    }
}

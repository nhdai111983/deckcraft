//! NexusOffice: how numbers and lengths read in the interface's language.
//!
//! English keeps upstream's look exactly (inches, a decimal point, upstream's parsing). Vietnamese
//! (vi-VN) shows lengths in centimetres and numbers with a decimal comma ("2,54 cm", "0,75 pt",
//! "12,5%"), and reads typed numbers with either a comma or a point. Only what is shown and typed
//! changes: the presentation keeps its points and EMU, and commands get the same numbers.
//!
//! Every widget that shows a number goes through here: [`drag`] and [`slider`] for egui's
//! `DragValue`/`Slider` (a custom formatter and parser), [`crate::widgets::spinner`] for the
//! ribbon's and panes' number boxes, and the dialogs' text fields ([`format_length`],
//! [`parse_length`], [`format_number`], [`parse_number`]).
//!
//! Typed text is untrusted: nothing here panics, and a number that isn't one (empty, letters,
//! "1,2,3", NaN, infinity, absurdly long input) is `None`.
//!
//! Two readings of a point in Vietnamese, on purpose:
//! - **Data** (Chart Data cells: [`parse_grouped`], [`format_grouped`]) follows vi-VN strictly: a
//!   point groups thousands and a comma is the decimal ("1.500.000", "1.234,5"; "1.234" is 1234).
//! - **Lengths, sizes, percents and the like** ([`parse_number`], [`parse_measure`],
//!   [`parse_length`]) take a point *or* a comma as the decimal separator ("2.54" = "2,54"). This
//!   is a deliberate exception: these fields never reach a thousand in practice (a slide is at most
//!   142 cm), and people type "2.5" as often as "2,5", so a point read as thousands would turn
//!   "1.5 cm" into 15 cm.

use std::borrow::Cow;
use std::ops::RangeInclusive;

use super::Language;

/// Points in an inch (the presentation's unit is the point).
pub const PT_PER_INCH: f64 = 72.0;
/// Centimetres in an inch.
pub const CM_PER_INCH: f64 = 2.54;
/// Typed numbers longer than this are not numbers (nobody types 65 digits into a size box).
const MAX_INPUT: usize = 64;

/// The unit lengths are shown in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LengthUnit {
    /// Upstream's inches (English).
    Inch,
    /// Centimetres (Vietnamese).
    Cm,
}

impl LengthUnit {
    /// The unit `language` shows lengths in.
    pub fn of(language: Language) -> Self {
        match language {
            Language::En => LengthUnit::Inch,
            Language::Vi => LengthUnit::Cm,
        }
    }

    /// Points in one of this unit.
    pub fn points(self) -> f64 {
        match self {
            LengthUnit::Inch => PT_PER_INCH,
            LengthUnit::Cm => PT_PER_INCH / CM_PER_INCH,
        }
    }

    /// `pt` points in this unit.
    pub fn from_points(self, pt: f64) -> f64 {
        pt / self.points()
    }

    /// `v` of this unit in points.
    pub fn to_points(self, v: f64) -> f64 {
        v * self.points()
    }

    /// The unit's symbol as a label shows it ("in", "cm").
    pub fn symbol(self) -> &'static str {
        match self {
            LengthUnit::Inch => "in",
            LengthUnit::Cm => "cm",
        }
    }
}

/// The decimal separator of `language`.
pub fn decimal_separator(language: Language) -> char {
    match language {
        Language::En => '.',
        Language::Vi => ',',
    }
}

/// A number formatted the upstream way (decimal point, no grouping) in `language`: the point
/// becomes a comma in Vietnamese. Upstream never groups thousands in these fields, so the only
/// point in `text` is the decimal one.
pub fn localize(language: Language, text: &str) -> Cow<'_, str> {
    match language {
        Language::En => Cow::Borrowed(text),
        Language::Vi if text.contains('.') => Cow::Owned(text.replace('.', ",")),
        Language::Vi => Cow::Borrowed(text),
    }
}

/// `v` with at most `max_decimals` decimals (trailing zeros dropped) in `language`: "2,54", "12,5",
/// "3". Not a finite number: "0".
pub fn format_number(language: Language, v: f64, max_decimals: usize) -> String {
    if !v.is_finite() {
        return "0".into();
    }
    let s = format!("{v:.prec$}", prec = max_decimals.min(12));
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.') } else { s.as_str() };
    // "-0" after rounding a tiny negative.
    let s = if s == "-0" { "0" } else { s };
    localize(language, s).into_owned()
}

/// `pt` points as a length in `language`'s unit, two decimals at most: "1" (inch) in English,
/// "2,54" (cm) in Vietnamese. The number only: the label names the unit.
pub fn format_length(language: Language, pt: f64) -> String {
    format_number(language, LengthUnit::of(language).from_points(pt), 2)
}

/// A typed number in `language`. English is upstream's `str::parse` (finite only). Vietnamese takes
/// an optional sign, digits and at most one decimal separator, a comma or a point ("2,5", "2.5",
/// ",5", "-1"): anything else is `None`.
pub fn parse_number(language: Language, text: &str) -> Option<f64> {
    match language {
        Language::En => text.trim().parse::<f64>().ok().filter(|v| v.is_finite()),
        Language::Vi if text.len() > MAX_INPUT => None,
        Language::Vi => parse_decimal(text),
    }
}

/// Digits with one optional ',' or '.' and an optional sign; finite.
fn parse_decimal(text: &str) -> Option<f64> {
    let (negative, digits) = split_sign(text.trim());
    let digits = digits.trim_start();
    let mut plain = String::with_capacity(digits.len() + 1);
    if negative {
        plain.push('-');
    }
    let (mut seen_digit, mut seen_separator) = (false, false);
    for c in digits.chars() {
        match c {
            '0'..='9' => {
                seen_digit = true;
                plain.push(c);
            }
            ',' | '.' if !seen_separator => {
                seen_separator = true;
                plain.push('.');
            }
            _ => return None,
        }
    }
    if !seen_digit {
        return None;
    }
    plain.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// A typed measurement: the number with a unit after it taken off (`%`, `°`, `pt`, `cm`, `in`, `"`,
/// `px`, `s`, spaces), parsed in `language`.
pub fn parse_measure(language: Language, text: &str) -> Option<f64> {
    let (number, _) = split_unit(text)?;
    parse_number(language, number)
}

/// A typed length in points. A unit typed after the number wins ("1 in", "2,54cm", "10 mm",
/// "72 pt", `1"`); a bare number is in `language`'s unit (inches in English, centimetres in
/// Vietnamese).
pub fn parse_length(language: Language, text: &str) -> Option<f64> {
    let (number, unit) = split_unit(text)?;
    let v = parse_number(language, number)?;
    let pt = match unit {
        "in" | "\"" | "inch" | "inches" => v * PT_PER_INCH,
        "cm" => v * PT_PER_INCH / CM_PER_INCH,
        "mm" => v * PT_PER_INCH / CM_PER_INCH / 10.0,
        "pt" => v,
        "" => LengthUnit::of(language).to_points(v),
        _ => return None,
    };
    Some(pt).filter(|p| p.is_finite())
}

/// `text` split into its number and the unit typed after it (lower case, "" for none).
fn split_unit(text: &str) -> Option<(&str, &'static str)> {
    if text.len() > MAX_INPUT {
        return None;
    }
    let t = text.trim();
    const UNITS: [&str; 11] = ["inches", "inch", "in", "\"", "cm", "mm", "pt", "px", "%", "°", "s"];
    let lower = t.to_ascii_lowercase();
    for unit in UNITS {
        if lower.ends_with(unit) {
            let cut = t.len().checked_sub(unit.len())?;
            // Only cut at a character boundary ("°" is two bytes; lower case keeps byte lengths
            // for ASCII and leaves other characters as they are).
            let number = t.get(..cut)?;
            return Some((number.trim_end(), unit));
        }
    }
    Some((t, ""))
}

/// A spinner's text ([`crate::widgets::spinner`]), how typed text reads back (in the caller's unit) and its step,
/// in `language`. English is upstream's. Vietnamese shows a decimal comma and reads a comma or a
/// point; a length in inches (suffix `"`) shows in centimetres and steps by 0.1 cm.
pub fn spinner(language: Language, value: f64, step: f64, suffix: &str) -> (String, fn(&str) -> Option<f64>, f64) {
    fn upstream(text: &str) -> Option<f64> {
        let num: String = text.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect();
        num.parse::<f64>().ok()
    }
    fn vi_number(text: &str) -> Option<f64> {
        parse_measure(Language::Vi, text)
    }
    fn vi_inches(text: &str) -> Option<f64> {
        parse_length(Language::Vi, text).map(|pt| pt / PT_PER_INCH)
    }
    match language {
        Language::En => (crate::widgets::fmt_num(value, suffix), upstream, step),
        Language::Vi if suffix == "\"" => {
            let cm = format_number(language, value * CM_PER_INCH, 2);
            (format!("{cm} cm"), vi_inches, step / CM_PER_INCH)
        }
        Language::Vi => (localize(language, &crate::widgets::fmt_num(value, suffix)).into_owned(), vi_number, step),
    }
}

/// A number typed as **data** (Chart Data cells), in `language`. English is upstream's parse.
/// Vietnamese follows vi-VN: a point groups thousands, a comma is the decimal. With a comma, the
/// points before it must group thousands ("1.234,5"); without one, points that group thousands are
/// thousands ("1.234", "1.500.000") and a single other point is a decimal point ("1.5", "0.75",
/// "0.123": a first group starting with 0 never groups thousands).
/// Anything else ("1,2,3", "12.34,5", "1,234.5", letters) is `None`: the caller tells the user
/// instead of dropping the value.
pub fn parse_grouped(language: Language, text: &str) -> Option<f64> {
    match language {
        Language::En => text.trim().parse::<f64>().ok().filter(|v| v.is_finite()),
        Language::Vi if text.len() > MAX_INPUT => None,
        Language::Vi => {
            let t = text.trim();
            let (negative, body) = split_sign(t);
            let (int, frac) = match body.split_once(',') {
                Some((i, f)) => (i, Some(f)),
                None => (body, None),
            };
            let int = if int.contains('.') {
                if groups_thousands(int) {
                    int.replace('.', "")
                } else if frac.is_none() && int.matches('.').count() == 1 {
                    return parse_decimal(t);
                } else {
                    return None;
                }
            } else {
                int.to_string()
            };
            let frac = frac.unwrap_or("");
            let digits = |s: &str| s.chars().all(|c| c.is_ascii_digit());
            if !digits(&int) || !digits(frac) || (int.is_empty() && frac.is_empty()) {
                return None;
            }
            let plain = format!(
                "{}{}.{}",
                if negative { "-" } else { "" },
                if int.is_empty() { "0" } else { int.as_str() },
                if frac.is_empty() { "0" } else { frac }
            );
            plain.parse::<f64>().ok().filter(|v| v.is_finite())
        }
    }
}

/// "1.234", "12.345.678": one to three digits not starting with 0, then groups of exactly three.
/// "0.123" is not thousands (it reads as a decimal).
fn groups_thousands(s: &str) -> bool {
    let mut parts = s.split('.');
    let first_ok = parts.next().is_some_and(|p| (1..=3).contains(&p.len()) && !p.starts_with('0') && p.chars().all(|c| c.is_ascii_digit()));
    let mut rest = 0usize;
    for p in parts {
        if p.len() != 3 || !p.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        rest += 1;
    }
    first_ok && rest > 0
}

/// A sign at the start of `t` (`-`, `−`, `+`) and what follows it.
fn split_sign(t: &str) -> (bool, &str) {
    if let Some(rest) = t.strip_prefix('-').or_else(|| t.strip_prefix('\u{2212}')) {
        (true, rest)
    } else if let Some(rest) = t.strip_prefix('+') {
        (false, rest)
    } else {
        (false, t)
    }
}

/// A data number as a cell shows it: upstream's text in English; in Vietnamese thousands grouped
/// with points and a decimal comma ("1.500.000", "1.234,5"), every digit kept so
/// [`parse_grouped`] reads back the same value. Not a finite number: "".
pub fn format_grouped(language: Language, v: f64) -> String {
    if language == Language::En {
        return v.to_string();
    }
    if !v.is_finite() {
        return String::new();
    }
    // `Display` for f64 is the shortest text that reads back exactly, never in exponent form.
    let s = v.to_string();
    let (negative, body) = split_sign(&s);
    let (int, frac) = body.split_once('.').unwrap_or((body, ""));
    let mut out = String::with_capacity(s.len() + s.len() / 3 + 1);
    if negative {
        out.push('-');
    }
    let n = int.len();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (n - i) % 3 == 0 {
            out.push('.');
        }
        out.push(c);
    }
    if !frac.is_empty() {
        out.push(',');
        out.push_str(frac);
    }
    out
}

/// egui's own number text for `value` in `range` decimals, in `language`.
pub fn format_in_range(language: Language, value: f64, range: RangeInclusive<usize>) -> String {
    let s = egui::emath::format_with_decimals_in_range(value, range);
    localize(language, &s).into_owned()
}

/// A `DragValue` that shows and reads numbers in the language of the frame `ui` draws: unchanged in
/// English, a decimal comma (and a comma or point typed) in Vietnamese.
pub fn drag<'a>(ui: &egui::Ui, w: egui::DragValue<'a>) -> egui::DragValue<'a> {
    match super::current(ui.ctx()) {
        Language::En => w,
        language => w.custom_formatter(move |n, range| format_in_range(language, n, range)).custom_parser(move |s| parse_measure(language, s)),
    }
}

/// A `Slider` whose value box shows and reads numbers in the language of the frame `ui` draws (see
/// [`drag`]).
pub fn slider<'a>(ui: &egui::Ui, w: egui::Slider<'a>) -> egui::Slider<'a> {
    match super::current(ui.ctx()) {
        Language::En => w,
        language => w.custom_formatter(move |n, range| format_in_range(language, n, range)).custom_parser(move |s| parse_measure(language, s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VI: Language = Language::Vi;
    const EN: Language = Language::En;

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn vietnamese_shows_centimetres_with_a_decimal_comma() {
        assert_eq!(format_length(VI, 72.0), "2,54");
        assert_eq!(format_length(EN, 72.0), "1");
        assert_eq!(format_length(EN, 54.0), "0.75");
        assert_eq!(format_length(VI, 0.0), "0");
        assert_eq!(format_number(VI, 0.75, 2), "0,75");
        assert_eq!(format_number(VI, 12.5, 1), "12,5");
        assert_eq!(format_number(EN, 12.5, 1), "12.5");
        assert_eq!(format_number(VI, 3.0, 2), "3");
        assert_eq!(format_number(VI, -0.001, 2), "0");
        assert_eq!(format_number(VI, f64::NAN, 2), "0");
        assert_eq!(format_number(VI, f64::INFINITY, 2), "0");
        assert_eq!(format_number(VI, 1e300, 2).chars().filter(|c| *c == ',').count(), 0, "no grouping, no decimals left");
        assert_eq!(localize(VI, "0.0%"), "0,0%");
        assert_eq!(localize(EN, "0.0%"), "0.0%");
        assert_eq!(format_in_range(VI, 0.25, 0..=2), "0,25");
        assert_eq!(LengthUnit::of(VI).symbol(), "cm");
        assert_eq!(LengthUnit::of(EN).symbol(), "in");
    }

    #[test]
    fn vietnamese_reads_a_comma_or_a_point() {
        assert_eq!(parse_number(VI, "2,54"), Some(2.54));
        assert_eq!(parse_number(VI, "2.54"), Some(2.54));
        assert_eq!(parse_number(VI, " -1,5 "), Some(-1.5));
        assert_eq!(parse_number(VI, "\u{2212}1,5"), Some(-1.5));
        assert_eq!(parse_number(VI, "+3"), Some(3.0));
        assert_eq!(parse_number(VI, ",5"), Some(0.5));
        assert_eq!(parse_number(VI, "5,"), Some(5.0));
        assert_eq!(parse_measure(VI, "0,75 pt"), Some(0.75));
        assert_eq!(parse_measure(VI, "12,5%"), Some(12.5));
        assert_eq!(parse_measure(VI, "45°"), Some(45.0));
        // English stays upstream's parse.
        assert_eq!(parse_number(EN, "2.54"), Some(2.54));
        assert_eq!(parse_number(EN, "2,54"), None);
        assert_eq!(parse_number(EN, "1e3"), Some(1000.0));
    }

    #[test]
    fn lengths_round_trip_through_the_text_in_both_languages() {
        for pt in [0.0, 1.0, 7.2, 36.0, 54.0, 72.0, 100.5, 540.0, 960.0, 4032.0, -18.0] {
            for lang in [EN, VI] {
                let shown = format_length(lang, pt);
                let back = parse_length(lang, &shown).unwrap_or(f64::NAN);
                // Two decimals of the unit: within half a hundredth of a centimetre or inch.
                let tolerance = LengthUnit::of(lang).points() * 0.005 + 1e-9;
                assert!((back - pt).abs() <= tolerance, "{lang:?} {pt} pt shows {shown:?}, reads back {back}");
            }
        }
    }

    #[test]
    fn a_typed_unit_wins_over_the_languages_unit() {
        assert!(parse_length(VI, "1 in").is_some_and(|p| near(p, 72.0)));
        assert!(parse_length(VI, "1\"").is_some_and(|p| near(p, 72.0)));
        assert!(parse_length(VI, "2,54cm").is_some_and(|p| near(p, 72.0)));
        assert!(parse_length(VI, "25,4 mm").is_some_and(|p| near(p, 72.0)));
        assert!(parse_length(VI, "72 pt").is_some_and(|p| near(p, 72.0)));
        assert!(parse_length(VI, "2,54").is_some_and(|p| near(p, 72.0)));
        assert!(parse_length(EN, "1").is_some_and(|p| near(p, 72.0)));
        assert!(parse_length(EN, "2.54 CM").is_some_and(|p| near(p, 72.0)));
        assert_eq!(parse_length(VI, "3 px"), None, "pixels are not a length here");
    }

    #[test]
    fn spinners_show_and_read_the_languages_numbers() {
        let (text, read, step) = spinner(VI, 1.0, 0.1, "\"");
        assert_eq!(text, "2,54 cm");
        assert!(read("5,08 cm").is_some_and(|v| near(v, 2.0)), "centimetres back to the caller's inches");
        assert!(read("2 in").is_some_and(|v| near(v, 2.0)));
        assert!(near(step, 0.1 / CM_PER_INCH), "steps by 0.1 cm");
        let (text, read, step) = spinner(VI, 0.75, 0.25, " pt");
        assert_eq!(text, "0,75 pt");
        assert_eq!(read("1,5 pt"), Some(1.5));
        assert_eq!(read("1.5"), Some(1.5));
        assert_eq!(read("1,2,3"), None);
        assert!(near(step, 0.25));
        assert_eq!(spinner(VI, 12.5, 5.0, "%").0, "12,5%");
        // English is upstream's spinner.
        let (text, read, step) = spinner(EN, 1.0, 0.1, "\"");
        assert_eq!(text, "1\"");
        assert_eq!(read("2\""), Some(2.0));
        assert!(near(step, 0.1));
        assert_eq!(spinner(EN, 0.75, 0.25, " pt").0, "0.75 pt");
    }

    #[test]
    fn data_numbers_read_points_as_thousands_and_a_comma_as_the_decimal() {
        assert_eq!(parse_grouped(VI, "1.500.000"), Some(1_500_000.0));
        assert_eq!(parse_grouped(VI, "1.234,5"), Some(1234.5));
        assert_eq!(parse_grouped(VI, "1.234"), Some(1234.0));
        assert_eq!(parse_grouped(VI, "-1.234,5"), Some(-1234.5));
        assert_eq!(parse_grouped(VI, "12,5"), Some(12.5));
        assert_eq!(parse_grouped(VI, "1234,5"), Some(1234.5));
        assert_eq!(parse_grouped(VI, "1234"), Some(1234.0));
        // One point that doesn't group three digits is a decimal point.
        assert_eq!(parse_grouped(VI, "1.5"), Some(1.5));
        assert_eq!(parse_grouped(VI, "0.75"), Some(0.75));
        // A first group of "0" never groups thousands: "0.123" is a decimal, not 123.
        assert_eq!(parse_grouped(VI, "0.123"), Some(0.123));
        assert_eq!(parse_grouped(VI, "0.500"), Some(0.5));
        assert_eq!(parse_grouped(VI, "-0.123"), Some(-0.123));
        assert_eq!(parse_grouped(VI, "0,5"), Some(0.5));
        assert_eq!(parse_grouped(VI, "0.123.456"), None);
        assert_eq!(parse_grouped(VI, "0.123,5"), None);
        assert_eq!(parse_grouped(VI, "012.345"), Some(12.345), "a leading zero doesn't group thousands either");
        for bad in ["abc", "", " ", "1,2,3", "1.23.4", "12.34,5", "1,234.5", "1.234.5", "1..234", "NaN", "inf", "1e3", "--1", ",", "."] {
            assert_eq!(parse_grouped(VI, bad), None, "{bad:?}");
        }
        assert_eq!(parse_grouped(VI, &"9".repeat(400)), None);
        // English is upstream's parse.
        assert_eq!(parse_grouped(EN, "1234.5"), Some(1234.5));
        assert_eq!(parse_grouped(EN, "1,234.5"), None);
    }

    /// Split from the test above (upstream's cognitive-complexity limit).
    #[test]
    fn data_numbers_are_shown_grouped_and_read_back_exactly() {
        // Shown grouped, read back exactly.
        assert_eq!(format_grouped(VI, 1_500_000.0), "1.500.000");
        assert_eq!(format_grouped(VI, 1234.5), "1.234,5");
        assert_eq!(format_grouped(VI, -1234.5), "-1.234,5");
        assert_eq!(format_grouped(VI, 999.0), "999");
        assert_eq!(format_grouped(VI, -0.25), "-0,25");
        assert_eq!(format_grouped(EN, 1234.5), "1234.5");
        for v in [0.0, 1.0, -1.0, 0.1, 2.5, 999.999, 1000.0, 1234.5678, -98765.4321, 1e15, 123_456_789.25] {
            assert_eq!(parse_grouped(VI, &format_grouped(VI, v)), Some(v), "{v}");
        }
        assert_eq!(format_grouped(VI, f64::NAN), "");
    }

    #[test]
    fn hostile_input_is_not_a_number_and_never_panics() {
        let long = "9".repeat(400);
        let hostile = [
            "", " ", "abc", "1,2,3", "1.2.3", "1,2.3", ",", ".", "-", "+", "--1", "1-", "NaN", "nan", "inf", "-inf", "infinity", "1e999", "1e3",
            "0x10", "1 2", "١٢", "２", "°", "%", "cm", "\"", "in", "1 cmx", "é", "ệ", "1,5ệ", "\u{0}", "1\u{0}", &long, "🙂", "1🙂",
        ];
        for t in hostile {
            assert_eq!(parse_number(VI, t), None, "{t:?}");
            let _ = parse_number(EN, t);
            assert!(parse_measure(VI, t).is_none_or(f64::is_finite), "{t:?}");
            assert!(parse_length(VI, t).is_none_or(f64::is_finite), "{t:?}");
            assert!(parse_length(EN, t).is_none_or(f64::is_finite), "{t:?}");
        }
        // Huge but finite: a number, and a finite length.
        let big = "9".repeat(60);
        assert!(parse_number(VI, &big).is_some_and(f64::is_finite));
        assert!(parse_length(VI, &big).is_some_and(f64::is_finite));
        assert!(parse_length(VI, "1e308").is_none(), "letters are not a Vietnamese number");
        assert_eq!(parse_length(EN, "1e308 in"), None, "inches past f64 are not a length");
        // Every char boundary of a multi-byte unit is handled.
        for t in ["5°", "5 °", "°5", "5\u{301}"] {
            let _ = parse_measure(VI, t);
            let _ = parse_length(VI, t);
        }
    }
}

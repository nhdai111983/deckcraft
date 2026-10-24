//! Design tokens (light and dark, the NexusOffice palette with Nexus Slide's Amber, and the
//! user's accent colour) and fonts for the UI.

use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Stroke, Visuals};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Brightness {
    #[default]
    Light,
    Dark,
}

/// Every colour the chrome uses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub dark: bool,
    /// Window background (title bar, tab row, side panes).
    pub chrome: Color32,
    /// The ribbon card and popovers.
    pub card: Color32,
    pub card_border: Color32,
    /// Canvas around the slide.
    pub canvas: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_faint: Color32,
    pub border: Color32,
    pub separator: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub selected: Color32,
    /// The accent (tabs underline, selected thumbnail, primary buttons): Amber's ink, or the
    /// user's colour shaded to stay readable.
    pub accent: Color32,
    /// Text and icons drawn on an `accent` fill (white in light mode, dark in dark mode).
    pub accent_text: Color32,
    pub contextual: Color32,
    pub selection: Color32,
    pub handle_fill: Color32,
    pub handle_stroke: Color32,
    pub guide: Color32,
    pub caret: Color32,
    pub text_selection: Color32,
    pub field_bg: Color32,
    pub shadow: Color32,
}

impl Tokens {
    /// Nexus Slide light palette (NexusOffice design system): slate surfaces and the Amber app
    /// colour. The accent is Amber's ink shade [`APP_INK`], so white text on it and accent text on
    /// the bars reach WCAG 4.5:1 (Amber #F59E0B itself is 2.2:1 on white); contextual menus and
    /// the smart guides use the second colour, Rose.
    pub fn light() -> Self {
        Tokens {
            dark: false,
            chrome: Color32::from_rgb(0xF1, 0xF5, 0xF9),
            card: Color32::from_rgb(0xF8, 0xFA, 0xFC),
            card_border: Color32::from_rgb(0xE2, 0xE8, 0xF0),
            canvas: Color32::from_rgb(0xE9, 0xED, 0xF2),
            text: Color32::from_rgb(0x1E, 0x29, 0x3B),
            text_dim: Color32::from_rgb(0x47, 0x55, 0x69),
            text_faint: Color32::from_rgb(0x94, 0xA3, 0xB8),
            border: Color32::from_rgb(0xCB, 0xD5, 0xE1),
            separator: Color32::from_rgb(0xE2, 0xE8, 0xF0),
            hover: Color32::from_rgb(0xE9, 0xED, 0xF2),
            pressed: Color32::from_rgb(0xDC, 0xE3, 0xEA),
            selected: Color32::from_rgb(0xFD, 0xEA, 0xC9),
            accent: APP_INK,
            accent_text: Color32::WHITE,
            contextual: SECOND_INK,
            selection: Color32::from_rgb(0x64, 0x74, 0x8B),
            handle_fill: Color32::WHITE,
            handle_stroke: Color32::from_rgb(0x64, 0x74, 0x8B),
            guide: SECOND_COLOR,
            caret: Color32::BLACK,
            text_selection: Color32::from_rgba_unmultiplied(0x5A, 0x8F, 0xE0, 90),
            field_bg: Color32::WHITE,
            shadow: Color32::from_black_alpha(28),
        }
    }
    /// Nexus Slide dark palette: Obsidian #0F172A and slate surfaces, Amber itself as the accent
    /// with dark text on it (white on Amber would be 2.2:1), Rose 400 for contextual menus.
    pub fn dark() -> Self {
        Tokens {
            dark: true,
            chrome: Color32::from_rgb(0x0F, 0x17, 0x2A),
            card: Color32::from_rgb(0x1E, 0x29, 0x3B),
            card_border: Color32::from_rgb(0x33, 0x41, 0x55),
            canvas: Color32::from_rgb(0x02, 0x06, 0x17),
            text: Color32::from_rgb(0xF1, 0xF5, 0xF9),
            text_dim: Color32::from_rgb(0xA8, 0xB4, 0xC4),
            text_faint: Color32::from_rgb(0x64, 0x74, 0x8B),
            border: Color32::from_rgb(0x47, 0x55, 0x69),
            separator: Color32::from_rgb(0x33, 0x41, 0x55),
            hover: Color32::from_rgb(0x33, 0x41, 0x55),
            pressed: Color32::from_rgb(0x47, 0x55, 0x69),
            selected: Color32::from_rgb(0x56, 0x37, 0x04),
            accent: APP_COLOR,
            accent_text: Color32::from_rgb(0x0F, 0x17, 0x2A),
            contextual: Color32::from_rgb(0xFB, 0x71, 0x85),
            selection: Color32::from_rgb(0x94, 0xA3, 0xB8),
            handle_fill: Color32::WHITE,
            handle_stroke: Color32::from_rgb(0x64, 0x74, 0x8B),
            guide: Color32::from_rgb(0xFB, 0x71, 0x85),
            caret: Color32::BLACK,
            text_selection: Color32::from_rgba_unmultiplied(0x5A, 0x8F, 0xE0, 90),
            field_bg: Color32::from_rgb(0x02, 0x06, 0x17),
            shadow: Color32::from_black_alpha(90),
        }
    }
    pub fn for_brightness(b: Brightness) -> Self {
        match b {
            Brightness::Light => Self::light(),
            Brightness::Dark => Self::dark(),
        }
    }
    /// The palette in use: light or dark, with the user's accent colour ([`set_accent`]) if any.
    pub fn get(ctx: &egui::Context) -> Self {
        let dark = ctx.global_style().visuals.dark_mode;
        let t = if dark { Self::dark() } else { Self::light() };
        let Some(c) = accent(ctx) else { return t };
        // Every widget asks for the palette each frame: the shaded one is kept per (colour, theme).
        let cache = egui::Id::new("nexus_user_accent_tokens");
        if let Some((cc, cd, ct)) = ctx.data(|d| d.get_temp::<(Color32, bool, Tokens)>(cache))
            && cc == c
            && cd == dark
        {
            return ct;
        }
        let t = with_accent(t, c);
        ctx.data_mut(|d| d.insert_temp(cache, (c, dark, t)));
        t
    }
}

/// Nexus Slide's app colour, Amber (NexusOffice design system), and its ink shade for text and
/// filled controls with white text (5.0:1 on white, 4.6:1 on the bars). Never PowerPoint's orange.
pub const APP_COLOR: Color32 = Color32::from_rgb(0xF5, 0x9E, 0x0B);
pub const APP_INK: Color32 = Color32::from_rgb(0xB4, 0x53, 0x09);
/// Nexus Slide's second colour, Rose, and its ink shade for text on the light bars (5.7:1).
pub const SECOND_COLOR: Color32 = Color32::from_rgb(0xF4, 0x3F, 0x5E);
pub const SECOND_INK: Color32 = Color32::from_rgb(0xBE, 0x12, 0x3C);
/// The SonnyTech brand gradient (indigo → cyan → amber), a thin line under the menu bar.
pub const BRAND_GRADIENT: [Color32; 3] = [Color32::from_rgb(0x63, 0x66, 0xF1), Color32::from_rgb(0x06, 0xB6, 0xD4), APP_COLOR];

/// Accent presets offered in Preferences (the NexusOffice palette); any other colour comes from
/// the picker.
pub const ACCENT_PRESETS: [(&str, Color32); 6] = [
    ("Amber", APP_COLOR),
    ("Coral", SECOND_COLOR),
    ("Indigo", Color32::from_rgb(0x63, 0x66, 0xF1)),
    ("Violet", Color32::from_rgb(0x8B, 0x5C, 0xF6)),
    ("Sky", Color32::from_rgb(0x0E, 0xA5, 0xE9)),
    ("Emerald", Color32::from_rgb(0x10, 0xB9, 0x81)),
];

/// Where the user's accent colour lives: the egui context's memory, so every widget reads it
/// through [`Tokens::get`] and each window (and each test harness) has its own.
fn accent_id() -> egui::Id {
    egui::Id::new("nexus_user_accent")
}

/// The user's accent colour in this context, if any.
pub fn accent(ctx: &egui::Context) -> Option<Color32> {
    ctx.data(|d| d.get_temp::<Color32>(accent_id()))
}

/// Sets (or with `None` clears) the user's accent colour. Returns whether it changed; the caller
/// then re-applies the style ([`apply`] with [`with_accent`]).
pub fn set_accent(ctx: &egui::Context, c: Option<Color32>) -> bool {
    if accent(ctx) == c {
        return false;
    }
    ctx.data_mut(|d| match c {
        Some(c) => {
            d.insert_temp(accent_id(), c);
        }
        None => {
            d.remove::<Color32>(accent_id());
        }
    });
    ctx.request_repaint();
    true
}

/// `#RRGGBB` or `RRGGBB` → colour; anything else `None`.
pub fn parse_hex(s: &str) -> Option<Color32> {
    let h = s.trim();
    let h = h.strip_prefix('#').unwrap_or(h);
    if h.len() != 6 || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| h.get(i..i + 2).and_then(|x| u8::from_str_radix(x, 16).ok());
    Some(Color32::from_rgb(byte(0)?, byte(2)?, byte(4)?))
}

/// `#RRGGBB` for a colour.
pub fn to_hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

fn luminance(c: Color32) -> f32 {
    let ch = |v: u8| {
        let s = f32::from(v) / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
}

/// WCAG contrast ratio of two opaque colours (1 to 21).
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// `c` moved towards black (or white with `lighten`) in 2.5% steps until it reads against every
/// colour in `against` at `ratio`. Black and white end the walk, so it always stops.
pub fn shade_for(c: Color32, against: &[Color32], ratio: f32, lighten: bool) -> Color32 {
    let target = if lighten { 255.0 } else { 0.0 };
    let mix = |v: u8, t: f32| {
        let v = f32::from(v);
        (v + (target - v) * t).round().clamp(0.0, 255.0) as u8
    };
    for k in 0..=40u8 {
        let t = f32::from(k) * 0.025;
        let out = Color32::from_rgb(mix(c.r(), t), mix(c.g(), t), mix(c.b(), t));
        if against.iter().all(|b| contrast_ratio(out, *b) >= ratio) {
            return out;
        }
    }
    if lighten { Color32::WHITE } else { Color32::BLACK }
}

/// `c` mixed with white (`k` = how much of `c` stays).
fn tint(c: Color32, k: f32) -> Color32 {
    let m = |v: u8| (255.0 - (255.0 - f32::from(v)) * k).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(m(c.r()), m(c.g()), m(c.b()))
}

/// `c` mixed with black (`k` = how much of `c` stays).
fn shade(c: Color32, k: f32) -> Color32 {
    let m = |v: u8| (f32::from(v) * k).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(m(c.r()), m(c.g()), m(c.b()))
}

/// The palette with the user's accent colour `c` in place of Amber. The accent is shaded until
/// its own text (`accent_text`) and accent text on the bars, cards and fields read at WCAG 4.5:1;
/// toggled tools (`selected`) get a soft tint of it that keeps the interface text readable.
pub fn with_accent(mut t: Tokens, c: Color32) -> Tokens {
    let c = Color32::from_rgb(c.r(), c.g(), c.b());
    t.accent = shade_for(c, &[t.accent_text, t.chrome, t.card, t.field_bg], 4.5, t.dark);
    t.selected = if t.dark { shade(c, 0.35) } else { tint(c, 0.22) };
    t
}

/// Apply tokens to egui's style.
pub fn apply(ctx: &egui::Context, t: &Tokens) {
    let mut v = if t.dark { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = t.chrome;
    v.window_fill = t.card;
    v.extreme_bg_color = t.field_bg;
    v.faint_bg_color = t.hover;
    v.window_stroke = Stroke::new(1.0, t.card_border);
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    v.selection.bg_fill = t.accent.gamma_multiply(0.85);
    v.selection.stroke = Stroke::new(1.0, t.accent_text);
    v.hyperlink_color = t.accent;
    v.override_text_color = Some(t.text);
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = CornerRadius::same(5);
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.separator);
    v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.inactive.bg_fill = t.field_bg;
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.hovered.weak_bg_fill = t.hover;
    v.widgets.hovered.bg_fill = t.hover;
    v.widgets.hovered.bg_stroke = Stroke::NONE;
    v.widgets.active.weak_bg_fill = t.pressed;
    v.widgets.active.bg_fill = t.pressed;
    v.widgets.open.weak_bg_fill = t.pressed;
    v.window_shadow = egui::epaint::Shadow { offset: [0, 4], blur: 18, spread: 0, color: t.shadow };
    v.popup_shadow = egui::epaint::Shadow { offset: [0, 3], blur: 12, spread: 0, color: t.shadow };
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 4.0);
        s.spacing.button_padding = egui::vec2(6.0, 3.0);
        s.spacing.interact_size.y = 22.0;
        s.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
        s.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
        s.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(11.0));
        s.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(18.0));
    });
}

/// UI fonts: Inter (from craft-fonts) when embedded, a Japanese UI face for CJK, then egui's.
pub fn install_fonts(ctx: &egui::Context) {
    install_fonts_with(ctx, &[]);
}

/// [`install_fonts`], with `extra` faces from the font database after every interface font: the
/// installed or embedded faces that draw text the interface faces lack (Hangul, Simplified
/// Chinese, Devanagari…, #101). See [`crate::uifonts`].
pub fn install_fonts_with(ctx: &egui::Context, extra: &[std::sync::Arc<deckcraft_fonts::FontFace>]) {
    ctx.set_fonts(font_definitions(extra));
}

/// The interface font definitions; see [`install_fonts_with`].
pub fn font_definitions(extra: &[std::sync::Arc<deckcraft_fonts::FontFace>]) -> FontDefinitions {
    let mut defs = FontDefinitions::default();
    let mut names = vec![];
    for (i, f) in deckcraft_fonts::CRAFT_FONTS.iter().enumerate() {
        let wanted = (f.family == "Inter" && (f.style == "Regular" || f.style == "SemiBold" || f.style == "Bold"))
            || (f.family == "BIZ UDPGothic" && f.style == "Regular")
            || (f.family == "Noto Sans Arabic");
        if !wanted {
            continue;
        }
        let name = format!("craft-{i}-{}-{}", f.family, f.style);
        defs.font_data.insert(name.clone(), std::sync::Arc::new(FontData::from_static(f.bytes)));
        names.push((f.family, f.style, name));
    }
    let regular: Vec<String> = names.iter().filter(|(fam, st, _)| !(*fam == "Inter" && *st != "Regular")).map(|(_, _, n)| n.clone()).collect();
    if let Some(list) = defs.families.get_mut(&FontFamily::Proportional) {
        for (k, n) in regular.iter().enumerate() {
            list.insert(k, n.clone());
        }
    }
    // A bold family for headings and selected tabs.
    let mut bold: Vec<String> = names.iter().filter(|(fam, st, _)| *fam == "Inter" && *st == "SemiBold").map(|(_, _, n)| n.clone()).collect();
    bold.extend(defs.families.get(&FontFamily::Proportional).cloned().unwrap_or_default());
    defs.families.insert(FontFamily::Name("bold".into()), bold);
    add_fallback_faces(&mut defs, extra);
    // The Japanese interface face (the first one when Inter isn't embedded) hints its en and em
    // dashes, bars thinner than a pixel, down to nothing, so they drew as gaps in interface text;
    // unhinted, they draw.
    for (name, d) in defs.font_data.iter_mut() {
        if name.contains("BIZ UDPGothic") {
            let mut data = (**d).clone();
            data.tweak.hinting = Some(false);
            *d = std::sync::Arc::new(data);
        }
    }
    defs
}

/// Add `faces` after every other font of each interface family (the last fallbacks).
pub fn add_fallback_faces(defs: &mut FontDefinitions, faces: &[std::sync::Arc<deckcraft_fonts::FontFace>]) {
    for f in faces {
        let name = format!("fallback {} {} {}", f.family, f.style, f.id());
        if defs.font_data.contains_key(&name) {
            continue;
        }
        let mut data = FontData::from_owned(f.data().to_vec());
        data.index = f.index();
        defs.font_data.insert(name.clone(), std::sync::Arc::new(data));
        for list in defs.families.values_mut() {
            list.push(name.clone());
        }
    }
}

pub fn bold(size: f32) -> egui::FontId {
    egui::FontId::new(size, bold_family())
}

/// The interface's bold family (headings, the selected tab).
pub fn bold_family() -> FontFamily {
    FontFamily::Name("bold".into())
}

/// NexusOffice: `ours` added to the fonts a host window already has (another editor's in its
/// other tabs): our faces join the window's, our named families are set (they are this app's own:
/// `bold`), and our faces are appended to the window's proportional and monospace lists, so
/// another editor's text keeps its own faces and ours still finds a glyph.
pub fn merge_font_definitions(mut window: FontDefinitions, ours: FontDefinitions) -> FontDefinitions {
    for (name, data) in ours.font_data {
        window.font_data.entry(name).or_insert(data);
    }
    for (family, chain) in ours.families {
        match family {
            FontFamily::Name(_) => {
                window.families.insert(family, chain);
            }
            other => {
                let list = window.families.entry(other).or_default();
                for name in chain {
                    if !list.contains(&name) {
                        list.push(name);
                    }
                }
            }
        }
    }
    window
}

pub fn font(size: f32) -> egui::FontId {
    egui::FontId::proportional(size)
}

pub fn to_color32(c: deckcraft_color::Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

#[cfg(test)]
mod tests {
    /// Each character of `text` laid out in the interface fonts at interface sizes, and whether it
    /// has ink (a glyph image).
    fn drawn(text: &str) -> Vec<(char, f32, bool)> {
        let ctx = egui::Context::default();
        ctx.set_fonts(super::font_definitions(&[]));
        let mut o = ctx.run_ui(Default::default(), |_| {});
        o.textures_delta.clear();
        let mut out = vec![];
        ctx.fonts_mut(|f| {
            for size in [11.0, 12.0, 13.0, 16.0] {
                for family in [egui::FontFamily::Proportional, egui::FontFamily::Name("bold".into())] {
                    let g = f.layout_no_wrap(text.to_string(), egui::FontId::new(size, family), egui::Color32::WHITE);
                    for gl in g.rows.iter().flat_map(|r| &r.glyphs) {
                        out.push((gl.chr, size, gl.uv_rect.size.x > 0.0 && gl.uv_rect.size.y > 0.0));
                    }
                }
            }
        });
        let mut o = ctx.run_ui(Default::default(), |_| {});
        o.textures_delta.clear();
        out
    }

    #[test]
    fn thin_punctuation_is_drawn_in_interface_text() {
        // En and em dashes (thin bars) were hinted down to nothing in the Japanese interface face,
        // so notes and the outline showed gaps where they were.
        for (c, size, ink) in drawn("\u{2013}\u{2014}-_\u{2026}\u{2022}\u{00b7}\u{2018}\u{2019}\u{201c}\u{201d}=~") {
            assert!(ink, "{c:?} ({:04X}) draws nothing at {size} px", c as u32);
        }
    }
}

#[cfg(test)]
mod nexus_accent_tests {
    use super::*;

    /// The pairs of colours drawn as text on a fill (or a fill under text) that must read at 4.5:1.
    fn text_pairs(t: &Tokens) -> [(&'static str, Color32, Color32); 6] {
        [
            ("accent_text on accent", t.accent_text, t.accent),
            ("accent on chrome", t.accent, t.chrome),
            ("accent on card", t.accent, t.card),
            ("accent on field", t.accent, t.field_bg),
            ("text on selected", t.text, t.selected),
            ("contextual on chrome", t.contextual, t.chrome),
        ]
    }

    /// Every colour drawn as text or under text keeps 4.5:1 with any accent the user picks, in
    /// both themes.
    #[test]
    fn user_accents_stay_readable() {
        let mut picks: Vec<Color32> = ACCENT_PRESETS.iter().map(|p| p.1).collect();
        picks.extend([Color32::WHITE, Color32::BLACK, Color32::YELLOW, Color32::from_rgb(0, 0, 0x40), Color32::from_rgb(0x80, 0x80, 0x80)]);
        for r in (0..=255u16).step_by(51) {
            for g in (0..=255u16).step_by(51) {
                for b in (0..=255u16).step_by(51) {
                    picks.push(Color32::from_rgb(r as u8, g as u8, b as u8));
                }
            }
        }
        for c in picks {
            for base in [Tokens::light(), Tokens::dark()] {
                let t = with_accent(base, c);
                for (what, fg, bg) in text_pairs(&t) {
                    let r = contrast_ratio(fg, bg);
                    assert!(r >= 4.5, "{c:?} dark={}: {what} {r:.2}", t.dark);
                }
            }
        }
    }

    /// The built-in Nexus palettes meet the same bar; the app colour is Amber with its ink, and no
    /// Office brand colour is left in the chrome.
    #[test]
    fn nexus_palettes_are_readable_and_amber() {
        for t in [Tokens::light(), Tokens::dark()] {
            for (what, fg, bg) in text_pairs(&t) {
                assert!(contrast_ratio(fg, bg) >= 4.5, "dark={}: {what} {:.2}", t.dark, contrast_ratio(fg, bg));
            }
            assert!(contrast_ratio(t.text, t.chrome) >= 4.5 && contrast_ratio(t.text_dim, t.card) >= 4.5, "dark={}", t.dark);
            assert!(contrast_ratio(t.text_dim, t.hover) >= 4.5, "dark={}", t.dark);
            let office = [(0xB7, 0x47, 0x2A), (0x2B, 0x57, 0x9A), (0x21, 0x73, 0x46), (0xD9, 0x5A, 0x1A), (0xC4, 0x3E, 0x1C)];
            for c in [t.accent, t.contextual, t.guide, t.selected, t.chrome, t.card, t.canvas] {
                for (r, g, b) in office {
                    assert_ne!(c, Color32::from_rgb(r, g, b), "dark={}", t.dark);
                }
            }
        }
        assert_eq!(Tokens::light().accent, APP_INK);
        assert_eq!(Tokens::dark().accent, APP_COLOR);
        assert_eq!(to_hex(APP_COLOR), "#F59E0B");
        assert_eq!(to_hex(SECOND_COLOR), "#F43F5E");
        // Amber itself is too light for white text; its ink is not.
        assert!(contrast_ratio(Color32::WHITE, APP_COLOR) < 4.5);
        assert!(contrast_ratio(Color32::WHITE, APP_INK) >= 4.5);
    }

    #[test]
    fn hex_colours_parse_strictly() {
        assert_eq!(parse_hex("#f59e0b"), Some(APP_COLOR));
        assert_eq!(parse_hex(" F43F5E "), Some(SECOND_COLOR));
        for junk in ["", "#", "#12345", "#1234567", "zzzzzz", "#ÿÿÿ", "+12345", "#-12345", "##123456", "#12 456"] {
            assert_eq!(parse_hex(junk), None, "{junk:?}");
        }
        assert_eq!(to_hex(Color32::from_rgb(1, 2, 255)), "#0102FF");
    }

    /// The accent lives in the context: setting it changes the palette widgets read; clearing it
    /// goes back to Amber; another context keeps its own.
    #[test]
    fn the_accent_is_per_context() {
        let ctx = egui::Context::default();
        apply(&ctx, &Tokens::light());
        let other = egui::Context::default();
        apply(&other, &Tokens::light());
        assert!(set_accent(&ctx, Some(SECOND_COLOR)));
        assert!(!set_accent(&ctx, Some(SECOND_COLOR)), "no change");
        let t = Tokens::get(&ctx);
        assert_eq!(t, with_accent(Tokens::light(), SECOND_COLOR));
        assert_eq!(Tokens::get(&ctx), t, "the cached palette");
        assert_eq!(Tokens::get(&other), Tokens::light(), "another window keeps its own");
        assert!(set_accent(&ctx, None));
        assert_eq!(Tokens::get(&ctx), Tokens::light());
    }
}

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
    /// Text fields. They have no outline, so the fill alone must set them apart from the card.
    pub text_field: Color32,
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
    /// Unchecked checkbox and radio boxes and slider rails, drawn without an outline, so they
    /// must stand apart from the card and the chrome.
    pub control: Color32,
    /// [`Tokens::control`] under the pointer and pressed: a step darker each in light mode.
    pub control_hover: Color32,
    pub control_pressed: Color32,
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
            text_field: Color32::from_rgb(0xE5, 0xEA, 0xF0),
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
            // Slate 300 and two darker steps (upstream's grey ramp, in the Nexus slate).
            control: Color32::from_rgb(0xCB, 0xD5, 0xE1),
            control_hover: Color32::from_rgb(0xB8, 0xC4, 0xD2),
            control_pressed: Color32::from_rgb(0xA6, 0xB3, 0xC4),
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
            text_field: Color32::from_rgb(0x02, 0x06, 0x17),
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
            control: Color32::from_rgb(0x02, 0x06, 0x17),
            control_hover: Color32::from_rgb(0x33, 0x41, 0x55),
            control_pressed: Color32::from_rgb(0x47, 0x55, 0x69),
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
        let cache = egui::Id::unique("nexus_user_accent_tokens");
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
    egui::Id::unique("nexus_user_accent")
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
    v.text_edit_bg_color = Some(t.text_field);
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
    v.widgets.inactive.bg_fill = t.control;
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.hovered.weak_bg_fill = t.hover;
    v.widgets.hovered.bg_fill = t.control_hover;
    v.widgets.hovered.bg_stroke = Stroke::NONE;
    v.widgets.active.weak_bg_fill = t.pressed;
    v.widgets.active.bg_fill = t.control_pressed;
    v.widgets.open.weak_bg_fill = t.pressed;
    v.window_shadow = egui::epaint::Shadow { offset: [0, 4], blur: 18, spread: 0, color: t.shadow };
    v.popup_shadow = egui::epaint::Shadow { offset: [0, 3], blur: 12, spread: 0, color: t.shadow };
    // NexusOffice: inputs with a visible frame, the accent when focused (`nexus_dialog`).
    crate::nexus_dialog::input_visuals(&mut v, t);
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
            // Exact craft-fonts family names: the Korean face is "NanumGothic" (no space), and
            // the symbol faces are "Noto Sans Symbols" / "Noto Sans Symbols 2". A near-miss here
            // would leave Hangul and the ◉ laser toggle without a glyph. Hebrew backs the
            // Language dialog's native "עברית" label where no system font supplies it.
            || matches!(f.family, "Noto Sans Arabic" | "Noto Sans Hebrew" | "NanumGothic" | "Noto Sans Symbols" | "Noto Sans Symbols 2")
            // This opt-in native face follows the existing families. Keep per-character
            // fallback and Latin metrics; no Chinese payload enters the wasm allowlist.
            || (cfg!(not(target_arch = "wasm32")) && f.family == "LXGW WenKai Lite" && f.style == "Regular");
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
    // Hack is already bundled for Monospace and covers the interface's arrows and triangles.
    // Keep it after every proportional face so ordinary text metrics stay unchanged.
    if defs.font_data.contains_key("Hack") {
        for chain in defs.families.values_mut() {
            if !chain.iter().any(|name| name == "Hack") {
                chain.push("Hack".into());
            }
        }
    }
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

/// NexusOffice: whether [`bold_family`] has a semibold face (Inter SemiBold, from craft-fonts);
/// without one it is the regular face, and "Lưu thay đổi?" draws its title bolder itself.
pub fn has_semibold() -> bool {
    deckcraft_fonts::CRAFT_FONTS.iter().any(|f| f.family == "Inter" && f.style == "SemiBold")
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
    // The window's own text faces (the host's interface font), as the last fallbacks of our named
    // families: without Inter embedded, `bold` would otherwise end in egui's defaults, which have
    // no Vietnamese (Tệp drew as T□p).
    let window_text = window.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let our_text = ours.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for (family, chain) in ours.families {
        match family {
            FontFamily::Name(_) => {
                // The family's own faces (Inter SemiBold), then the window's text faces, then ours.
                let (own, rest): (Vec<String>, Vec<String>) = chain.into_iter().partition(|n| !our_text.contains(n));
                let mut merged = own;
                for name in window_text.iter().chain(&rest) {
                    if !merged.contains(name) {
                        merged.push(name.clone());
                    }
                }
                window.families.insert(family, merged);
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
    use super::*;

    /// Each character of `text` laid out in the interface fonts at interface sizes, and whether it
    /// has ink (a glyph image).
    fn drawn(text: &str) -> Vec<(char, f32, bool)> {
        let ctx = egui::Context::default();
        // A replacement glyph also gets a nonempty atlas rectangle. Fail before layout when no
        // font can draw a character, so `uv_rect` cannot mistake tofu for real ink.
        ctx.set_missing_glyph_policy(egui::MissingGlyphPolicy::Panic);
        ctx.set_fonts(super::font_definitions(&[]));
        let mut o = ctx.run_ui(Default::default(), |_| {});
        o.textures_delta.clear();
        let mut out = vec![];
        ctx.fonts_mut(|f| {
            for size in [11.0, 12.0, 13.0, 16.0] {
                for family in [egui::FontFamily::Proportional, egui::FontFamily::Name("bold".into())] {
                    let font = egui::FontId::new(size, family);
                    for c in text.chars() {
                        assert!(f.has_glyph(&font, c), "no font supplies {c:?} at {size} px in {font:?}");
                    }
                    let g = f.layout_no_wrap(text.to_string(), font, egui::Color32::WHITE);
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

    /// The UI font library supplies default text fonts and an explicit monochrome emoji face,
    /// independent of kittest defaults.
    #[test]
    fn the_ui_library_has_default_fonts_and_explicit_monochrome_emoji_for_tests() {
        // `egui`'s `default_fonts` is no longer a default feature and kittest's own dev
        // dependencies do not propagate. This library enables it (and the test build enables
        // `monochrome_emoji_fonts`) explicitly.
        let defs = FontDefinitions::default();
        assert!(defs.font_data.contains_key("Hack"), "default_fonts must be enabled by deckcraft-ui-egui itself");
        assert!(defs.font_data.contains_key("Ubuntu-Light"));
        assert!(defs.font_data.contains_key("NotoEmoji-Regular"), "monochrome_emoji_fonts is an explicit test feature");
        assert!(defs.families.contains_key(&FontFamily::Proportional));
        // The production font policy still owns the UI families: craft faces first, then egui's.
        let ours = super::font_definitions(&[]);
        assert!(ours.families.contains_key(&FontFamily::Name("bold".into())));
        assert!(ours.font_data.contains_key("Hack"));
    }

    /// When the Chinese face is present (required builds guarantee it), exactly one LXGW WenKai
    /// Lite Regular is embedded, and Chinese labels have glyph coverage and ink; builds without the
    /// optional face skip this test.
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn native_chinese_fallback_draws_initial_document_labels() {
        let faces: Vec<_> = deckcraft_fonts::CRAFT_FONTS.iter().filter(|f| f.family == "LXGW WenKai Lite").collect();
        if faces.is_empty() {
            assert!(option_env!("CRAFT_FONTS_REQUIRED").is_none(), "required native font input must include the reviewed Chinese fallback");
            eprintln!("optional Chinese input absent: bundled Chinese coverage was not qualified");
            return;
        }
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].style, "Regular");
        assert_eq!(faces[0].bytes.len(), 13_949_472);
        let face = skrifa::FontRef::new(faces[0].bytes).unwrap();
        use skrifa::MetadataProvider as _;
        assert!(face.charmap().map('简').is_some(), "the approved fallback supplies the original missing glyph");
        let defs = font_definitions(&[]);
        for family in [FontFamily::Proportional, FontFamily::Name("bold".into())] {
            let chain = &defs.families[&family];
            let chinese = chain.iter().position(|name| name.contains("LXGW WenKai Lite")).unwrap();
            let japanese = chain.iter().position(|name| name.contains("BIZ UDPGothic")).unwrap();
            assert!(japanese < chinese, "retain existing glyph selection and Latin metrics");
            assert_eq!(chain.iter().filter(|name| name.contains("LXGW WenKai Lite")).count(), 1);
        }
        let text = "简体中文演示文稿";
        let ctx = egui::Context::default();
        ctx.set_missing_glyph_policy(egui::MissingGlyphPolicy::Panic);
        ctx.set_fonts(defs);
        let mut output = ctx.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();
        ctx.fonts_mut(|fonts| {
            for family in [FontFamily::Proportional, FontFamily::Name("bold".into())] {
                let font = egui::FontId::new(13.0, family);
                for c in text.chars() {
                    assert!(fonts.has_glyph(&font, c), "the embedded fallback does not supply {c:?} in {font:?}");
                }
            }
        });
        let ink = drawn(text);
        assert_eq!(ink.len(), text.chars().count() * 4 * 2);
        for (c, size, visible) in ink {
            assert!(visible, "Chinese label {c:?} has no ink at {size} px");
        }
    }

    /// When the face is present (required builds guarantee it), the native Hebrew fallback is exactly
    /// one static Regular Noto Sans Hebrew face with glyph coverage and ink for language labels.
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn native_hebrew_fallback_is_one_regular_face_and_draws_language_labels() {
        let faces: Vec<_> = deckcraft_fonts::CRAFT_FONTS.iter().filter(|f| f.family == "Noto Sans Hebrew").collect();
        if faces.is_empty() {
            assert!(option_env!("CRAFT_FONTS_REQUIRED").is_none(), "required native font input must include Noto Sans Hebrew Regular");
            eprintln!("optional font input absent: native Hebrew coverage was not qualified");
            return;
        }
        assert_eq!(faces.len(), 1, "one static face, without variable named instances");
        assert_eq!(faces[0].style, "Regular");
        assert_eq!(faces[0].bytes.len(), 66_156, "use the approved full static Regular face");
        use skrifa::raw::{ReadError, TableProvider};
        let face = skrifa::FontRef::new(faces[0].bytes).unwrap();
        assert_eq!(face.os2().unwrap().us_weight_class(), 400, "the embedded bytes must be Regular, not a renamed Thin face");
        assert!(matches!(face.fvar(), Err(ReadError::TableIsMissing(_))), "embed one static face, not variable named instances");
        let defs = super::font_definitions(&[]);
        let names: Vec<_> = defs.font_data.keys().filter(|name| name.contains("Noto Sans Hebrew")).collect();
        assert_eq!(names.len(), 1);
        for family in [FontFamily::Proportional, FontFamily::Name("bold".into())] {
            assert_eq!(defs.families[&family].iter().filter(|name| *name == names[0]).count(), 1);
        }
        let ctx = egui::Context::default();
        ctx.set_fonts(defs);
        let mut output = ctx.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();
        ctx.fonts_mut(|fonts| {
            for family in [FontFamily::Proportional, FontFamily::Name("bold".into())] {
                let font = egui::FontId::new(13.0, family);
                for c in "עבריתשלוםdeck42".chars() {
                    assert!(fonts.has_glyph(&font, c), "missing language-label glyph {c:?}");
                }
            }
        });
        // This checks ink and coverage only, not bidi ordering or GPOS mark positioning.
        for (c, size, ink) in drawn("עבריתשלוםdeck42") {
            assert!(ink, "{c:?} has no ink at {size} px");
        }
    }

    /// Hebrew fallback preserves logical-order editing in mixed-direction text.
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn native_hebrew_fallback_keeps_mixed_text_editing_in_logical_order() {
        if !deckcraft_fonts::CRAFT_FONTS.iter().any(|f| f.family == "Noto Sans Hebrew") {
            assert!(option_env!("CRAFT_FONTS_REQUIRED").is_none(), "required native Hebrew face is absent");
            return;
        }
        let ctx = egui::Context::default();
        ctx.set_fonts(super::font_definitions(&[]));
        let id = egui::Id::unique("mixed-hebrew-label-edit");
        let mut text = "deck שלום 42".to_owned();
        let frame = |text: &mut String, events: Vec<egui::Event>| {
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500.0, 100.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    ui.memory_mut(|m| m.request_focus(id));
                    ui.add(egui::TextEdit::singleline(text).id(id));
                },
            );
            output.drop_without_applying_deltas();
        };
        frame(&mut text, vec![]);
        // Character offsets remain logical even though the Hebrew glyphs are drawn right-to-left.
        let mut state = egui::TextEdit::load_state(&ctx, id).unwrap();
        state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(7))));
        egui::TextEdit::store_state(&ctx, id, state);
        frame(&mut text, vec![egui::Event::Text("א".into())]);
        assert_eq!(text, "deck שלאום 42");
        let state = egui::TextEdit::load_state(&ctx, id).unwrap();
        assert_eq!(state.cursor.char_range().unwrap().primary.index.0, 8);
        frame(
            &mut text,
            vec![egui::Event::Key { key: egui::Key::Backspace, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }],
        );
        assert_eq!(text, "deck שלום 42");
        ctx.fonts_mut(|fonts| {
            let galley = fonts.layout_no_wrap(text.clone(), egui::FontId::proportional(13.0), egui::Color32::WHITE);
            let row = &galley.rows[0];
            let x = |c| row.glyphs.iter().find(|g| g.chr == c).unwrap().pos.x;
            assert!(x('ש') > x('ל') && x('ל') > x('ו') && x('ו') > x('ם'), "Hebrew paints in visual RTL order");
            assert!(x('4') < x('2'), "the adjacent number keeps its LTR order");
        });
        frame(&mut text, vec![]); // consume the atlas updates from the explicit layout above
    }

    #[test]
    fn light_checkboxes_and_slider_rails_stand_out_from_their_background() {
        // An unchecked checkbox and a slider rail are filled with the inactive widget fill and
        // have no outline, so in light mode they vanished into the white dialog (#766).
        let ctx = egui::Context::default();
        super::apply(&ctx, &super::Tokens::light());
        let v = ctx.global_style().visuals.clone();
        assert_eq!(v.widgets.inactive.bg_stroke, egui::Stroke::NONE, "the test assumes no outline");
        let fill = v.widgets.inactive.bg_fill;
        for (name, bg) in [("dialog", v.window_fill), ("panel", v.panel_fill)] {
            let diff = [fill.r().abs_diff(bg.r()), fill.g().abs_diff(bg.g()), fill.b().abs_diff(bg.b())];
            assert!(diff.iter().all(|d| *d >= 24), "control fill {fill:?} is too close to the {name} background {bg:?}");
        }
        // Hovering and pressing darken the control a step each, as they do buttons.
        let (hovered, active) = (v.widgets.hovered.bg_fill, v.widgets.active.bg_fill);
        assert!(fill.r() > hovered.r() && hovered.r() > active.r(), "idle {fill:?}, hovered {hovered:?}, active {active:?} don't darken in turn");
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

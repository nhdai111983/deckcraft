//! NexusOffice branding for the egui front end (Nexus Slide by SonnyTech), kept in one file so
//! upstream's files change as little as possible. The names and ids live in
//! [`deckcraft_engine::nexus`]; this module draws the mark and the About page.
//!
//! DeckCraft has no translation layer, so there is no single lookup to rename through: upstream's
//! interface strings that name the product are changed where they are written (the fork's
//! `nexus/BRANDING_PATCH.md` lists each file). Text from the user's presentations is never
//! rewritten.

use deckcraft_engine::nexus::{ABOUT_SUITE, APP_NAME, ATTRIBUTION, COPYRIGHT, UPSTREAM_COPYRIGHT};
use egui::{Color32, CornerRadius, Painter, Rect, Stroke, Ui, pos2, vec2};

use crate::theme::Tokens;

/// The Nexus Slide mark, drawn in code (SonnyTech, CC0, like `nexus/assets/nexus-slide.svg`): an
/// orange tile with a screen on a stand and an "N" with two amber dots.
pub fn paint_logo(p: &Painter, r: Rect) {
    let orange = Color32::from_rgb(0xC2, 0x41, 0x0C);
    let paper = Color32::from_rgb(0xF8, 0xFA, 0xFC);
    let amber = Color32::from_rgb(0xF5, 0x9E, 0x0B);
    let s = r.width() / 512.0;
    let at = |x: f32, y: f32| pos2(r.min.x + x * s, r.min.y + y * s);
    let rect = |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(at(x, y), vec2(w * s, h * s));
    let radius = |v: f32| CornerRadius::same((v * s).clamp(0.0, 255.0) as u8);
    p.rect_filled(r, radius(112.0), orange);
    p.rect_filled(rect(80.0, 120.0, 352.0, 232.0), radius(24.0), paper);
    p.rect_filled(rect(240.0, 352.0, 32.0, 48.0), CornerRadius::ZERO, paper);
    p.rect_filled(rect(176.0, 392.0, 160.0, 24.0), radius(12.0), paper);
    let n = [at(192.0, 304.0), at(192.0, 168.0), at(320.0, 304.0), at(320.0, 168.0)];
    p.add(egui::Shape::line(n.to_vec(), Stroke::new(36.0 * s, orange)));
    for c in [n[0], n[3]] {
        p.circle_filled(c, 26.0 * s, amber);
    }
}

/// About ▸ About: the mark, name, version, what it is, where the source is, the attribution to
/// upstream (text only, no ArtCraft logo or link) and the copyright notices.
pub fn about_ui(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    // NexusOffice: in the interface's language; the copyright notices stay as written.
    let lang = crate::i18n::current(ui.ctx());
    let vi = lang == crate::i18n::Language::Vi;
    ui.vertical_centered(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(64.0, 64.0), egui::Sense::hover());
        paint_logo(ui.painter(), r);
        ui.label(egui::RichText::new(APP_NAME).font(crate::theme::bold(22.0)));
        ui.label(egui::RichText::new(version_line(lang)).color(t.text_dim));
        ui.add_space(6.0);
        ui.label(crate::i18n::tl(ui, "Presentations and slide shows, in pure Rust."));
        ui.label(if vi { ABOUT_SUITE.1 } else { ABOUT_SUITE.0 });
        ui.label(crate::i18n::tl(ui, "An open-source, clean-room project. Not affiliated with Microsoft."));
        ui.add_space(6.0);
        let source = lang.fill("Source code: {link}", &[("link", "github.com/sonnytech-vn/deckcraft")]);
        ui.hyperlink_to(source, deckcraft_engine::links::GITHUB);
        ui.add_space(6.0);
        ui.label(egui::RichText::new(if vi { ATTRIBUTION.1 } else { ATTRIBUTION.0 }).size(11.0).color(t.text_dim));
        ui.label(egui::RichText::new(UPSTREAM_COPYRIGHT).size(11.0).color(t.text_faint));
        ui.label(egui::RichText::new(COPYRIGHT).size(11.0).color(t.text_faint));
    });
}

/// The About page's version line in `lang` ("Version 0.7.0", "Phiên bản 0.7.0"); upstream's
/// [`crate::dialogs::version_line`] with the word translated.
fn version_line(lang: crate::i18n::Language) -> String {
    let line = crate::dialogs::version_line();
    match line.strip_prefix("Version ") {
        Some(rest) => lang.fill("Version {v}", &[("v", rest)]),
        None => line,
    }
}

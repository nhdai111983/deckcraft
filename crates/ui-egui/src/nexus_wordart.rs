//! NexusOffice: the WordArt gallery (Insert › WordArt, Shape Format › WordArt Styles).
//!
//! A grid of tiles, four to a row, each showing "Aa" drawn by the slide renderer in that style
//! (fill, outline, shadow, glow, gradient, reflection exactly as the slide will show them), with
//! hover and focus states, the style's name as a tooltip in the interface language, and arrow-key
//! navigation (Enter or Space picks). The styles are data in the engine
//! ([`deckcraft_engine::nexus_wordart::STYLES`]); this file only draws them.

use std::collections::HashMap;

use deckcraft_engine::nexus_wordart::{self as wa, STYLES, WordArtStyle};
use deckcraft_model::text::TextBody;
use deckcraft_model::{Shape, ShapeId};
use egui::{Align2, Color32, CornerRadius, Id, Rect, Sense, Stroke, StrokeKind, TextureHandle, Ui, Vec2, vec2};
use serde_json::json;

use crate::SlideApp;
use crate::theme::{self, Tokens};

/// Tiles per row.
pub const COLUMNS: usize = 4;
/// A tile's size, points.
pub const TILE: Vec2 = vec2(72.0, 60.0);
const GAP: f32 = 6.0;

/// What picking a tile does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Insert › WordArt: a new WordArt in that style.
    Insert,
    /// Shape Format › WordArt Styles: the selected text or shapes take that style.
    Restyle,
}

impl Mode {
    fn command(self) -> &'static str {
        match self {
            Mode::Insert => "insert.wordArt",
            Mode::Restyle => "format.wordArtStyle",
        }
    }
}

/// The id of tile `i` of the gallery in `mode` (kittest and focus).
pub fn tile_id(mode: Mode, i: usize) -> Id {
    Id::new(("nexus_wordart_tile", mode, i))
}

/// A style's name in the interface language.
pub fn name(ctx: &egui::Context, s: &WordArtStyle) -> &'static str {
    match crate::i18n::current(ctx) {
        crate::i18n::Language::Vi => s.name.1,
        crate::i18n::Language::En => s.name.0,
    }
}

/// The gallery in a popup: picks run the mode's command (and close the popup).
pub fn menu(app: &mut SlideApp, ui: &mut Ui, mode: Mode) {
    if let Some(style) = gallery(app, ui, mode) {
        let mut p = json!({"style": style});
        if mode == Mode::Insert {
            // The new text in the interface language ("Nhập chữ của bạn vào đây").
            p["text"] = json!(crate::i18n::tr(ui.ctx(), wa::DEFAULT_TEXT.0));
        }
        let _ = app.run(mode.command(), p);
        ui.close();
    }
}

/// Draws the gallery; returns the id of the style picked this frame.
pub fn gallery(app: &mut SlideApp, ui: &mut Ui, mode: Mode) -> Option<&'static str> {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    ui.label(egui::RichText::new(crate::i18n::tl(ui, "WordArt Styles")).font(theme::font(12.0)).color(t.text_dim));
    ui.add_space(2.0);
    let current = match mode {
        Mode::Restyle => app.session.active().and_then(|d| d.selected_shapes().first().and_then(|s| wa::style_of(s))).map(|s| s.id),
        Mode::Insert => None,
    };
    let textures = tiles(app, &ctx);
    let rows = STYLES.len().div_ceil(COLUMNS);
    let size = vec2(COLUMNS as f32 * TILE.x + (COLUMNS as f32 - 1.0) * GAP, rows as f32 * TILE.y + (rows as f32 - 1.0).max(0.0) * GAP);
    let (area, _) = ui.allocate_exact_size(size, Sense::hover());
    let mut picked = None;
    let mut focus_to: Option<usize> = None;
    for (i, style) in STYLES.iter().enumerate() {
        let (r, c) = (i / COLUMNS, i % COLUMNS);
        let rect = Rect::from_min_size(area.min + vec2(c as f32 * (TILE.x + GAP), r as f32 * (TILE.y + GAP)), TILE);
        let id = tile_id(mode, i);
        let resp = ui.interact(rect, id, Sense::click());
        let focused = resp.has_focus();
        let on = current == Some(style.id);
        let p = ui.painter();
        // A tile is a little slide: white, whatever the interface's appearance.
        let border = if focused || resp.hovered() || on { t.accent } else { t.border };
        p.rect(rect, CornerRadius::same(4), Color32::WHITE, Stroke::new(if focused || on { 2.0 } else { 1.0 }, border), StrokeKind::Inside);
        if resp.hovered() && !focused {
            p.rect_filled(rect.shrink(2.0), CornerRadius::same(3), t.accent.gamma_multiply(0.08));
        }
        match textures.get(i).and_then(Option::as_ref) {
            Some(tex) => {
                let [w, h] = tex.size();
                let ppp = ctx.pixels_per_point().max(0.1);
                let img = vec2(w as f32 / ppp, h as f32 / ppp);
                let k = ((TILE.x - 6.0) / img.x.max(1.0)).min((TILE.y - 6.0) / img.y.max(1.0)).min(1.0);
                let r = Rect::from_center_size(rect.center(), img * k);
                p.image(tex.id(), r, Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
            }
            None => {
                p.text(rect.center(), Align2::CENTER_CENTER, "Aa", theme::font(22.0), Color32::from_rgb(0x0F, 0x17, 0x2A));
            }
        }
        if on {
            let dot = rect.right_top() + vec2(-8.0, 8.0);
            p.circle_filled(dot, 5.0, t.accent);
            p.text(dot, Align2::CENTER_CENTER, "✓", theme::font(8.0), t.accent_text);
        }
        let label = name(&ctx, style);
        resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, on, label));
        let resp = resp.on_hover_text(label);
        if resp.clicked() {
            picked = Some(style.id);
        }
        if focused {
            let (left, right, up, down) = ui.input(|inp| {
                (
                    inp.key_pressed(egui::Key::ArrowLeft),
                    inp.key_pressed(egui::Key::ArrowRight),
                    inp.key_pressed(egui::Key::ArrowUp),
                    inp.key_pressed(egui::Key::ArrowDown),
                )
            });
            focus_to = step(i, STYLES.len(), left, right, up, down);
        }
    }
    if let Some(to) = focus_to {
        ctx.memory_mut(|m| m.request_focus(tile_id(mode, to)));
    }
    picked
}

/// Where the focus goes from tile `i` of `n` on an arrow key (wrapping along the rows).
pub fn step(i: usize, n: usize, left: bool, right: bool, up: bool, down: bool) -> Option<usize> {
    if n == 0 || i >= n {
        return None;
    }
    let to = if left {
        if i == 0 { n - 1 } else { i - 1 }
    } else if right {
        (i + 1) % n
    } else if up {
        if i >= COLUMNS { i - COLUMNS } else { i }
    } else if down {
        if i + COLUMNS < n { i + COLUMNS } else { i }
    } else {
        return None;
    };
    (to != i).then_some(to)
}

/// The tile pictures, drawn once per theme fonts and screen density and cached in the context.
fn tiles(app: &SlideApp, ctx: &egui::Context) -> Vec<Option<TextureHandle>> {
    let ppp = ctx.pixels_per_point();
    let Some(st) = app.session.active() else { return vec![None; STYLES.len()] };
    let doc = &st.doc;
    let slide = st.current_slide().cloned().or_else(|| doc.slides.first().map(|s| s.as_ref().clone()));
    let Some(mut slide) = slide else { return vec![None; STYLES.len()] };
    let fonts = doc.masters.iter().map(|m| m.theme.fonts.major.latin.as_str()).collect::<Vec<_>>().join(",");
    let key = format!("{fonts}|{ppp:.2}");
    let cache_id = Id::new("nexus_wordart_tiles");
    let cached: Option<(String, Vec<Option<TextureHandle>>)> = ctx.data(|d| d.get_temp(cache_id));
    if let Some((k, v)) = cached
        && k == key
    {
        return v;
    }
    let mut out = Vec::with_capacity(STYLES.len());
    for style in STYLES {
        let sh = sample(style);
        let id = sh.id;
        slide.shapes = vec![sh];
        // Points → pixels: the sample box (120 × 100 pt) shown in a tile, at the screen's density.
        let scale = f64::from(ppp.max(0.5)) * 0.62;
        let tex = deckcraft_render::render_shape(doc, &slide, id, scale)
            .map(|img| crate::textures::upload(ctx, format!("nexus-wordart-{}", style.id), crate::textures::to_color_image(&img, false)));
        out.push(tex);
    }
    ctx.data_mut(|d| d.insert_temp(cache_id, (key, out.clone())));
    out
}

/// "Aa" in `style`, as the tile shows it: the WordArt's own font and look, with room under it
/// for a reflection.
fn sample(style: &WordArtStyle) -> Shape {
    let mut body = TextBody::from_text("Aa");
    body.body.wrap = Some(false);
    for para in &mut body.paragraphs {
        para.props.align = Some(deckcraft_model::text::Align::Center);
        for r in &mut para.runs {
            r.props.size = Some(40.0);
            r.props.font = Some("+mj-lt".into());
            style.apply(&mut r.props);
        }
    }
    Shape {
        id: ShapeId(1),
        xfrm: Some(deckcraft_geom::Xfrm::new(0.0, 0.0, 120.0, 100.0)),
        text_box: true,
        text: Some(body),
        geom: deckcraft_model::Geom::preset("rect"),
        ..Default::default()
    }
}

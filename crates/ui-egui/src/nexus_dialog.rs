//! NexusOffice: one look for every Nexus Slide dialog.
//!
//! - **Inputs** ([`input_visuals`], applied by [`crate::theme::apply`]): every text field, check
//!   box, radio button, combo box and button in dialogs and panes (the bars keep frameless buttons:
//!   [`chrome_style`]) has a visible 1 px neutral frame (3:1 against the field and
//!   the dialog), a darker one on hover and the accent when focused. egui draws a field's frame
//!   with the widget's `bg_stroke` and, focused, `selection.stroke`; the Nexus palette had set the
//!   first to nothing and the second to the accent's *text* colour (white in the light theme), so
//!   fields were white boxes on a near-white dialog.
//! - **The window** ([`window`]): auto-sized to its content (no fixed height), 16 px padding, a
//!   minimum width.
//! - **Sections** ([`section`]) in muted text, **hints** ([`hint`]) smaller.
//! - **List boxes** ([`list_box`]): framed, rows with hover and a full-width accent-tinted
//!   selection with inner padding.
//! - **The footer** ([`footer`], [`footer_with`]): right-aligned, secondary buttons ("Hủy") then
//!   the primary one ("OK", "Chèn"), all the same height and minimum width, in a row exactly as tall
//!   as the buttons. (A right-to-left layout centred vertically fills the whole height it's given,
//!   which made dialogs as tall as the screen allowed, with the buttons floating in the middle.)
//!
//! Spacing comes from [`GAP_S`], [`GAP_M`] and [`GAP_L`] (8, 12, 16 px).

use egui::{Align, Color32, CornerRadius, Layout, RichText, Sense, Stroke, Ui, Visuals, vec2};

use crate::i18n::tl;
use crate::theme::{self, Tokens, contrast_ratio};

/// Spacing tokens: between related controls, between groups, around the dialog.
pub const GAP_S: f32 = 8.0;
pub const GAP_M: f32 = 12.0;
pub const GAP_L: f32 = 16.0;
/// Footer buttons: height and minimum width.
pub const BUTTON_H: f32 = 28.0;
pub const BUTTON_W: f32 = 80.0;
/// A dialog's minimum content width.
pub const MIN_WIDTH: f32 = 360.0;
/// A list box row's height.
pub const ROW_H: f32 = 26.0;

/// The neutral frame of an input: 3:1 against the field and the dialog (WCAG non-text contrast).
pub fn input_border(t: &Tokens) -> Color32 {
    if t.dark { Color32::from_rgb(0x64, 0x74, 0x8B) } else { Color32::from_rgb(0x7C, 0x8B, 0xA0) }
}

/// The frame of a hovered input.
pub fn input_border_hover(t: &Tokens) -> Color32 {
    if t.dark { Color32::from_rgb(0x94, 0xA3, 0xB8) } else { Color32::from_rgb(0x47, 0x55, 0x69) }
}

fn blend(a: Color32, b: Color32, k: f32) -> Color32 {
    let m = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * k).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// The fill of a selected row (and of selected text in a field): a clear tint of the accent over
/// the field colour, as strong as keeps the interface text on it at 4.5:1. The field colour itself
/// meets that (the palettes guarantee it), so the walk always ends.
pub fn selected_row(t: &Tokens) -> Color32 {
    for step in 0..=22u8 {
        let k = 0.22 - f32::from(step) * 0.01;
        let fill = blend(t.field_bg, t.accent, k.max(0.0));
        if contrast_ratio(t.text, fill) >= 4.5 {
            return fill;
        }
    }
    t.field_bg
}

/// The frame of a focused input, which is also the colour of text selected in it: the accent,
/// darkened (lightened in the dark theme) until it reads at 4.5:1 on [`selected_row`], the field
/// and the dialog.
pub fn focus_color(t: &Tokens) -> Color32 {
    theme::shade_for(t.accent, &[selected_row(t), t.field_bg, t.card], 4.5, t.dark)
}

/// The input look on egui's visuals (called by [`crate::theme::apply`]).
pub fn input_visuals(v: &mut Visuals, t: &Tokens) {
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, input_border(t));
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, input_border_hover(t));
    // Focused fields are framed in the accent; selected text (and selected list rows) sits on a
    // tint of it.
    v.selection.stroke = Stroke::new(1.0, focus_color(t));
    v.selection.bg_fill = selected_row(t);
}

/// The title bar, ribbon and status bar keep egui's frameless buttons (a framed button is 2 px
/// larger, and the bars are laid out to the pixel): sets that look on `ui` and returns the style
/// to put back after them.
pub fn chrome_style(ui: &mut Ui) -> std::sync::Arc<egui::Style> {
    let before = ui.style().clone();
    let w = &mut ui.visuals_mut().widgets;
    w.inactive.bg_stroke = Stroke::NONE;
    w.hovered.bg_stroke = Stroke::NONE;
    before
}

/// A dialog window titled `title`, sized to its content. Its top edge stays put (centred
/// horizontally, a sixth of the way down, higher for a dialog too tall for that): a dialog that grows (a row added, another page of
/// options) grows downwards instead of re-centring, so nothing under the pointer moves.
pub fn window<'a>(ctx: &egui::Context, title: String, id: egui::Id) -> egui::Window<'a> {
    let frame = egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin::same(GAP_L as i8));
    // A tall dialog moves up as far as it needs to fit (its last height, from the previous frame).
    let screen = ctx.content_rect().height();
    let last = ctx.memory(|m| m.area_rect(id)).map_or(0.0, |r| r.height());
    let top = (screen / 6.0).clamp(16.0, 200.0).min(screen - last - GAP_L).max(GAP_L);
    egui::Window::new(title).id(id).collapsible(false).resizable(false).frame(frame).anchor(egui::Align2::CENTER_TOP, vec2(0.0, top))
}

/// A section label in muted text ("Định dạng có sẵn:").
pub fn section(ui: &mut Ui, text: &str) {
    let t = Tokens::get(ui.ctx());
    ui.label(RichText::new(tl(ui, text)).color(t.text_dim));
}

/// A smaller muted note under a control.
pub fn hint(ui: &mut Ui, text: &str) {
    let t = Tokens::get(ui.ctx());
    ui.add(egui::Label::new(RichText::new(tl(ui, text)).size(11.5).color(t.text_dim)).wrap());
}

/// The width of the label column of a form row ([`field_row`]) and of its field.
pub const LABEL_W: f32 = 200.0;
pub const FIELD_W: f32 = 180.0;

/// A form row: `label` (already in the interface language) left-aligned in a [`LABEL_W`] column,
/// then the control `add` draws in the width it is given. A label too long for the column goes on
/// its own line above the control. `indent` moves both right (a sub-option).
pub fn field_row(ui: &mut Ui, label: &str, indent: bool, add: impl FnOnce(&mut Ui, f32)) {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let color = ui.visuals().text_color();
    let text_w = ui.fonts_mut(|f| f.layout_no_wrap(label.to_string(), font, color).size().x);
    let pad = if indent { GAP_L } else { 0.0 };
    let spacing = ui.spacing().item_spacing.x;
    if text_w + spacing <= LABEL_W {
        ui.horizontal(|ui| {
            ui.add_space(pad);
            let r = ui.label(label);
            ui.add_space((LABEL_W - pad - r.rect.width() - spacing).max(0.0));
            add(ui, FIELD_W);
        });
    } else {
        ui.horizontal(|ui| {
            ui.add_space(pad);
            ui.label(label);
        });
        ui.horizontal(|ui| {
            ui.add_space(pad);
            add(ui, (LABEL_W + FIELD_W - pad).max(FIELD_W));
        });
        ui.add_space(2.0);
    }
}

/// A framed list of `rows` (already in the interface language), `selected` highlighted, as tall as
/// its rows up to `max_height` (it scrolls past that). Returns the row clicked this frame and
/// whether it was a double click.
pub fn list_box(ui: &mut Ui, id_salt: &str, rows: &[String], selected: Option<usize>, max_height: f32) -> (Option<usize>, bool) {
    let rows_h = rows.len() as f32 * ROW_H;
    list_box_sized(ui, id_salt, rows, selected, (rows_h.min(max_height), max_height), "")
}

/// [`list_box`] at least `heights.0` tall (a list that keeps its size as rows come and go), with
/// `empty` (muted) when there are no rows.
pub fn list_box_sized(
    ui: &mut Ui,
    id_salt: &str,
    rows: &[String],
    selected: Option<usize>,
    heights: (f32, f32),
    empty: &str,
) -> (Option<usize>, bool) {
    let t = Tokens::get(ui.ctx());
    let mut picked = None;
    let mut double = false;
    let (min_h, max_h) = (heights.0.max(0.0), heights.1.max(heights.0).max(0.0));
    egui::Frame::new()
        .fill(t.field_bg)
        .stroke(Stroke::new(1.0, input_border(&t)))
        .corner_radius(CornerRadius::same(5))
        .inner_margin(egui::Margin::same(4))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_salt(id_salt).max_height(max_h).min_scrolled_height(min_h).auto_shrink([false, true]).show(ui, |ui| {
                ui.set_min_height(min_h);
                ui.spacing_mut().item_spacing.y = 0.0;
                if rows.is_empty() && !empty.is_empty() {
                    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::hover());
                    ui.painter().text(
                        rect.left_center() + vec2(GAP_S, 0.0),
                        egui::Align2::LEFT_CENTER,
                        empty,
                        egui::FontId::proportional(13.0),
                        t.text_dim,
                    );
                }
                for (i, row) in rows.iter().enumerate() {
                    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::click());
                    let on = selected == Some(i);
                    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, row));
                    let fill = if on {
                        selected_row(&t)
                    } else if resp.hovered() {
                        t.hover
                    } else {
                        Color32::TRANSPARENT
                    };
                    if ui.is_rect_visible(rect) {
                        ui.painter().rect_filled(rect, CornerRadius::same(4), fill);
                        ui.painter().text(
                            rect.left_center() + vec2(GAP_S, 0.0),
                            egui::Align2::LEFT_CENTER,
                            row,
                            egui::FontId::proportional(13.0),
                            t.text,
                        );
                    }
                    if resp.clicked() {
                        picked = Some(i);
                    }
                    if resp.double_clicked() {
                        picked = Some(i);
                        double = true;
                    }
                }
            });
        });
    (picked, double)
}

/// What the footer's buttons did this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Footer {
    /// The primary button was clicked, or Enter pressed while it is enabled.
    pub primary: bool,
    /// The index (in `secondary`) of the secondary button clicked.
    pub secondary: Option<usize>,
}

/// The footer of a dialog: Hủy then `primary` ("OK", "Chèn"), right-aligned. Returns (OK, Cancel).
/// Enter is OK while `enabled`.
pub fn footer(ui: &mut Ui, primary: &str, enabled: bool) -> (bool, bool) {
    let f = footer_with(ui, &["Cancel"], primary, enabled);
    (f.primary, f.secondary == Some(0))
}

/// The footer with `secondary` buttons (left to right, the first is usually "Cancel") before the
/// primary one. Labels are English; they are shown in the interface language.
pub fn footer_with(ui: &mut Ui, secondary: &[&str], primary: &str, enabled: bool) -> Footer {
    let t = Tokens::get(ui.ctx());
    let mut out = Footer::default();
    ui.add_space(GAP_L);
    let size = vec2(BUTTON_W, BUTTON_H);
    ui.allocate_ui_with_layout(vec2(ui.available_width(), BUTTON_H), Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = GAP_S;
        let label = tl(ui, primary).into_owned();
        let button = egui::Button::new(RichText::new(label).color(t.accent_text)).fill(t.accent).stroke(Stroke::new(1.0, t.accent)).min_size(size);
        if ui.add_enabled(enabled, button).clicked() || (enabled && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
            out.primary = true;
        }
        for (i, s) in secondary.iter().enumerate().rev() {
            let label = tl(ui, s).into_owned();
            let button =
                egui::Button::new(RichText::new(label).color(t.text)).fill(t.field_bg).stroke(Stroke::new(1.0, input_border(&t))).min_size(size);
            if ui.add(button).clicked() {
                out.secondary = Some(i);
            }
        }
    });
    out
}

#[cfg(test)]
#[path = "nexus_dialog_tests.rs"]
mod tests;

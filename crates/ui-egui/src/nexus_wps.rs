//! NexusOffice layout in the style of WPS Office (its "Universal UI"; Henry's approved mockup,
//! 2026-10-24), the same layout as Nexus Docs' and Nexus Sheet's so the suite's tabs feel like one
//! product. Only the layout is borrowed: every icon is drawn in code (`icons.rs`) and the colours
//! are the app's own (`theme.rs`).
//!
//! There is no title bar: the NexusOffice window's tab strip (or the system's title bar in the
//! stand-alone app) names the presentation. The app draws two rows above the slides:
//!
//! - [`bar`] (36 pt): File (opens the File tab), the Quick Access Toolbar (Save, Undo, Redo and
//!   the user's own buttons), the ribbon tabs left aligned (» for the ones that don't fit); on the
//!   right the save state (Save and Save As in its menu), Search, Comments, Play from Start, Share
//!   and a "⋯" menu (light/dark, language, Options, About).
//! - [`strip`] (56 pt): the current tab's groups as one flat row, two rows of small buttons high (a
//!   large button is a small icon over one label line). The groups the window has no room for fold
//!   into a "…" panel at the end of the row instead of being cut off.
//!
//! The thumbnails, the slide, the notes and the status bar are upstream's. The groups are
//! upstream's too ([`crate::ribbon::tab_groups`]); [`group`], called by the ribbon's own group
//! helper, decides which of them each pass draws.

use egui::{Align2, Color32, Id, Rect, Response, Sense, Stroke, Ui, pos2, vec2};
use serde_json::json;

use crate::i18n::{self, Language};
use crate::icons::{self, Icon};
use crate::theme::{self, Tokens};
use crate::{SlideApp, widgets};

/// Height of the top row (File, Quick Access, tabs, Share).
pub const BAR_H: f32 = 36.0;
/// Height of the command strip under it: two rows of small buttons, or a large button's 20 px icon
/// over its label ([`widgets::COMPACT_H`] plus 3 pt above and below).
pub const STRIP_H: f32 = widgets::COMPACT_H + 6.0;
/// Height of the top row's buttons and search box.
const CONTROL_H: f32 = 28.0;
/// Width kept at the end of the strip for the "…" button.
const OVERFLOW_W: f32 = 30.0;
/// Width of the » button that holds the tabs the top row has no room for.
const MORE_TABS_W: f32 = 26.0;
/// Below this window width the top row drops the word "Comments" (the icon stays) and narrows the
/// search box, so more tabs fit.
const NARROW_W: f32 = 1200.0;

/// The File (Tệp) button.
pub fn file_button_id() -> Id {
    Id::new("nexus_wps_file")
}
/// The strip's "…" button, shown when groups don't fit.
pub fn overflow_button_id() -> Id {
    Id::new("nexus_wps_overflow")
}
/// The save state ("Saved" / "Not saved"), which opens Save and Save As.
pub fn save_state_id() -> Id {
    Id::new("nexus_wps_save_state")
}
/// The top row's "⋯" menu.
pub fn more_button_id() -> Id {
    Id::new("nexus_wps_more")
}
/// The search box.
pub fn search_id() -> Id {
    Id::new("nexus_wps_search")
}
/// The » button with the tabs that don't fit.
pub fn more_tabs_id() -> Id {
    Id::new("nexus_wps_more_tabs")
}
/// Share.
pub fn share_id() -> Id {
    Id::new("nexus_wps_share")
}

/// The tabs the top row offers now: upstream's ten, then the contextual ones the selection asks
/// for (Shape Format, Table Design…).
pub fn shown_tabs(app: &SlideApp) -> Vec<&'static str> {
    crate::ribbon::TABS.iter().copied().chain(crate::ribbon::contextual(app)).collect()
}

/// Draw the layout's top panels: the top row, and the command strip unless the ribbon is
/// collapsed (double-click a tab).
pub fn show(app: &mut SlideApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    // A contextual tab that went away (the selection left the table or picture) leaves for Home.
    let tabs = shown_tabs(app);
    if !tabs.contains(&app.ui.tab.as_str()) {
        app.ui.tab = "Home".into();
    }
    bar(app, ui, &t, &tabs);
    if !app.ui.ribbon_collapsed {
        strip(app, ui, &t);
    }
}

/// The File button's fill: the accent, faint, over the row (a little stronger under the pointer).
pub(crate) fn file_fill(t: &Tokens, hovered: bool) -> Color32 {
    t.chrome.lerp_to_gamma(t.accent, if hovered { 0.16 } else { 0.08 })
}

/// File's label on `fill`: the accent, darkened toward the body text as far as 4.5:1 needs.
pub(crate) fn file_ink(t: &Tokens, fill: Color32) -> Color32 {
    [0.0, 0.2, 0.4, 0.6, 0.8]
        .iter()
        .map(|k| t.accent.lerp_to_gamma(t.text, *k))
        .find(|ink| theme::contrast_ratio(*ink, fill) >= 4.5)
        .unwrap_or(t.text)
}

/// The colour "Saved" is written in: green that keeps 4.5:1 on the row in either theme.
fn saved_color(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(0x6E, 0xE7, 0xB7) } else { Color32::from_rgb(0x04, 0x78, 0x57) }
}

/// The top row.
fn bar(app: &mut SlideApp, ui: &mut Ui, t: &Tokens, tabs: &[&'static str]) {
    // Stand-alone on macOS the window draws into the title bar: the row sits beside the traffic
    // lights and moves the window. Embedded, the NexusOffice window's own tab strip does that.
    let mac = app.integrated_titlebar && !app.embedded;
    let height = if mac { BAR_H.max(crate::ribbon::MAC_TITLE_BAR) } else { BAR_H };
    let left = if mac { crate::ribbon::MAC_CONTENT_LEFT.clamp(0.0, f32::from(i8::MAX)) as i8 } else { 12 };
    egui::Panel::top("wps_bar")
        .exact_size(height)
        .frame(egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin { left, right: 10, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            let full = ui.max_rect();
            if !app.embedded {
                // The stand-alone window has no system title bar: the empty row moves it.
                let drag = ui.interact(full, ui.id().with("wps_drag"), Sense::click_and_drag());
                if drag.drag_started() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if drag.double_clicked() {
                    let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
                }
            }
            let narrow = ui.ctx().content_rect().width() < NARROW_W;
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                file_button(ui, app, t);
                ui.add_space(8.0);
                quick_access(app, ui);
                ui.add_space(6.0);
                let (d, _) = ui.allocate_exact_size(vec2(1.0, 20.0), Sense::hover());
                ui.painter().rect_filled(d, 0.0, t.border);
                ui.add_space(8.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                    more_menu(ui, app, t);
                    share_button(ui, app, t);
                    play_button(ui, app, t);
                    comments_button(ui, app, t, narrow);
                    search_box(ui, app, t, narrow);
                    ui.add_space(6.0);
                    save_state(ui, app, t);
                    ui.add_space(12.0);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                        tab_row(ui, app, t, tabs, full.height());
                    });
                });
            });
            ui.painter().hline(full.x_range(), full.max.y - 0.5, Stroke::new(1.0, t.separator));
        });
}

/// A button of the top row: an icon (and a label, unless `label` is empty) that names itself
/// `name` for screen readers and tests.
fn row_button(ui: &mut Ui, t: &Tokens, icon: Icon, label: &str, name: &str, enabled: bool, on: bool) -> Response {
    let font = theme::font(12.5);
    let tw = if label.is_empty() { 0.0 } else { ui.painter().layout_no_wrap(label.to_string(), font.clone(), t.text).size().x + 6.0 };
    let w = if label.is_empty() { CONTROL_H } else { 8.0 + 16.0 + tw + 8.0 };
    let (r, resp) = ui.allocate_exact_size(vec2(w, CONTROL_H), if enabled { Sense::click() } else { Sense::hover() });
    if on || (resp.hovered() && enabled) {
        ui.painter().rect_filled(r, 6.0, if on { t.selected } else { t.hover });
    }
    let ink = if enabled { t.text_dim } else { t.text_faint };
    let ix = if label.is_empty() { r.center().x } else { r.min.x + 16.0 };
    icons::paint(ui.painter(), Rect::from_center_size(pos2(ix, r.center().y), vec2(17.0, 17.0)), icon, ink, !enabled);
    if !label.is_empty() {
        ui.painter().text(pos2(r.min.x + 30.0, r.center().y), Align2::LEFT_CENTER, label, font, if enabled { t.text } else { t.text_faint });
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, name));
    resp
}

/// File (Tệp): the three bars and the word on a faint accent, opening the File tab as File does.
fn file_button(ui: &mut Ui, app: &mut SlideApp, t: &Tokens) {
    let lang = app.ui.language;
    let label = lang.tr("File");
    let font = theme::bold(13.0);
    let tw = ui.painter().layout_no_wrap(label.to_string(), font.clone(), t.accent).size().x;
    let (r, _) = ui.allocate_exact_size(vec2(10.0 + 16.0 + 6.0 + tw + 12.0, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, file_button_id(), Sense::click());
    let fill = file_fill(t, resp.hovered());
    let ink = file_ink(t, fill);
    ui.painter().rect_filled(r, 6.0, fill);
    icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 18.0, r.center().y), vec2(16.0, 16.0)), Icon::Hamburger, ink, false);
    ui.painter().text(pos2(r.min.x + 32.0, r.center().y), Align2::LEFT_CENTER, &label, font, ink);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
    if resp.on_hover_text(format!("{label} (Alt+F)")).clicked() {
        app.open_backstage(crate::backstage::Page::Home);
    }
}

/// The commands the top row always has (as WPS has): Save, Undo, Redo.
const FIXED_QAT: [&str; 3] = ["file.save", "edit.undo", "edit.redo"];
/// Upstream's starting toolbar has these too; the WPS row leaves them to File (Home), Share
/// (Export) and the Slide Show tab, so a new install shows Save, Undo and Redo only.
const UPSTREAM_DEFAULTS: [&str; 3] = ["app.home", "app.exportDialog", "show.fromStart"];

/// Save, Undo and Redo, then the buttons the user added to the Quick Access Toolbar, then its
/// Customize menu.
fn quick_access(app: &mut SlideApp, ui: &mut Ui) {
    let has_doc = app.session.active().is_some();
    for id in FIXED_QAT {
        let (icon, label) = match id {
            "file.save" => (Icon::Save, "Save"),
            "edit.undo" => (Icon::Undo, "Undo"),
            _ => (Icon::Redo, "Redo"),
        };
        let en = if id == "file.save" { has_doc } else { crate::ribbon::enabled(app, id) };
        // Named with its shortcut ("Lưu (Ctrl+S)"), as upstream's toolbar buttons are.
        let tip = crate::ribbon::tip(id, label);
        if widgets::icon_toggle(ui, icon, &tip, en, false).clicked() {
            if id == "file.save" {
                app.save();
            } else {
                let _ = app.run(id, json!({}));
            }
        }
    }
    let extra: Vec<crate::qat::QatItem> =
        app.ui.qat.iter().filter(|q| !FIXED_QAT.contains(&q.id.as_str()) && !UPSTREAM_DEFAULTS.contains(&q.id.as_str())).cloned().collect();
    crate::qat::show_items(app, ui, &extra);
}

/// Which tabs the row shows, given each tab's width and whether it must stay (the current tab and
/// contextual ones): all when they fit, else as many as fit in order with room kept for the »
/// button, the pinned ones first.
pub fn fit_tabs(widths: &[(f32, bool)], room: f32, more_w: f32) -> Vec<bool> {
    let w = |x: f32| if x.is_finite() { x.max(0.0) } else { 0.0 };
    if widths.iter().map(|(x, _)| w(*x)).sum::<f32>() <= room + 0.5 {
        return vec![true; widths.len()];
    }
    let mut left = room - more_w - widths.iter().filter(|(_, pinned)| *pinned).map(|(x, _)| w(*x)).sum::<f32>();
    widths
        .iter()
        .map(|(x, pinned)| {
            if *pinned {
                return true;
            }
            if w(*x) <= left + 0.5 {
                left -= w(*x);
                true
            } else {
                left = f32::NEG_INFINITY;
                false
            }
        })
        .collect()
}

/// The ribbon tabs, left aligned: the active one in the accent, bold, underlined; contextual ones
/// in the contextual colour. Tabs that don't fit go into a » menu.
fn tab_row(ui: &mut Ui, app: &mut SlideApp, t: &Tokens, tabs: &[&'static str], height: f32) {
    let lang = app.ui.language;
    let contextual = crate::ribbon::contextual(app);
    let labels: Vec<String> = tabs.iter().map(|tab| lang.tr(tab).into_owned()).collect();
    let widths: Vec<(f32, bool)> = tabs
        .iter()
        .zip(&labels)
        .map(|(tab, shown)| {
            let w = ui.painter().layout_no_wrap(shown.clone(), theme::bold(13.0), t.text).size().x + 20.0;
            (w, *tab == app.ui.tab || contextual.contains(tab))
        })
        .collect();
    let fits = fit_tabs(&widths, ui.available_width(), MORE_TABS_W);
    for (((tab, shown), (w, _)), fit) in tabs.iter().zip(&labels).zip(&widths).zip(&fits) {
        if !fit {
            continue;
        }
        let (r, resp) = ui.allocate_exact_size(vec2(*w, height), Sense::click());
        let active = app.ui.tab == *tab;
        resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, shown));
        if resp.hovered() && !active {
            ui.painter().rect_filled(r.shrink2(vec2(2.0, 6.0)), 6.0, t.hover);
        }
        let color = if active {
            t.accent
        } else if contextual.contains(tab) {
            t.contextual
        } else {
            t.text
        };
        ui.painter().text(r.center(), Align2::CENTER_CENTER, shown, if active { theme::bold(13.0) } else { theme::font(13.0) }, color);
        if active {
            ui.painter().rect_filled(Rect::from_min_max(pos2(r.min.x + 10.0, r.max.y - 2.0), pos2(r.max.x - 10.0, r.max.y)), 1.0, t.accent);
        }
        if resp.double_clicked() {
            app.ui.ribbon_collapsed = !app.ui.ribbon_collapsed;
        } else if resp.clicked() {
            app.ui.tab = tab.to_string();
            app.ui.ribbon_collapsed = false;
        }
    }
    let left_out: Vec<(&str, &str)> =
        tabs.iter().zip(&labels).zip(&fits).filter(|(_, fit)| !**fit).map(|((tab, shown), _)| (*tab, shown.as_str())).collect();
    if !left_out.is_empty() {
        let (r, _) = ui.allocate_exact_size(vec2(MORE_TABS_W, height), Sense::hover());
        let resp = ui.interact(r, more_tabs_id(), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(r.shrink2(vec2(0.0, 6.0)), 6.0, t.hover);
        }
        icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(15.0, 15.0)), Icon::ChevronDoubleRight, t.text_dim, false);
        let tip = lang.tr("More tabs");
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &tip));
        egui::Popup::menu(&resp).show(|ui| {
            for (tab, shown) in left_out {
                if ui.selectable_label(false, shown).clicked() {
                    app.ui.tab = tab.to_string();
                    app.ui.ribbon_collapsed = false;
                    ui.close();
                }
            }
        });
    }
}

/// The save state: "Saved" with a check, or "Not saved". A click opens Save and Save As.
fn save_state(ui: &mut Ui, app: &mut SlideApp, t: &Tokens) {
    let lang = app.ui.language;
    let saved = app.session.active().is_some_and(|d| !d.is_dirty() && d.path.is_some());
    let (text, color) = if saved { (lang.tr("Saved"), saved_color(t.dark)) } else { (lang.tr("Not saved"), t.text_dim) };
    let font = theme::font(12.0);
    let tw = ui.painter().layout_no_wrap(text.to_string(), font.clone(), color).size().x;
    let icon_w = if saved { 20.0 } else { 0.0 };
    let (r, _) = ui.allocate_exact_size(vec2(8.0 + icon_w + tw + 4.0 + 10.0 + 8.0, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, save_state_id(), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, 6.0, t.hover);
    }
    if saved {
        icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 16.0, r.center().y), vec2(15.0, 15.0)), Icon::Check, color, false);
    }
    let x = r.min.x + 8.0 + icon_w;
    ui.painter().text(pos2(x, r.center().y), Align2::LEFT_CENTER, &text, font, color);
    icons::paint(
        ui.painter(),
        Rect::from_center_size(pos2(x + tw + 9.0, r.center().y + 1.0), vec2(11.0, 11.0)),
        Icon::ChevronDown,
        t.text_dim,
        false,
    );
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &text));
    let has_doc = app.session.active().is_some();
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(200.0);
        if ui.add_enabled(has_doc, egui::Button::new(lang.tr("Save"))).clicked() {
            app.save();
            ui.close();
        }
        if ui.add_enabled(has_doc, egui::Button::new(lang.tr("Save As…"))).clicked() {
            app.open_backstage(crate::backstage::Page::SaveAs);
            ui.close();
        }
    });
}

/// The search box: a click (or its shortcut) opens the command search.
fn search_box(ui: &mut Ui, app: &mut SlideApp, t: &Tokens, narrow: bool) {
    let (r, _) = ui.allocate_exact_size(vec2(if narrow { CONTROL_H + 4.0 } else { 220.0 }, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, search_id(), Sense::click());
    ui.painter().rect(r, 8.0, if resp.hovered() { t.hover } else { t.field_bg }, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    let icon_x = if narrow { r.center().x } else { r.min.x + 16.0 };
    icons::paint(ui.painter(), Rect::from_center_size(pos2(icon_x, r.center().y), vec2(15.0, 15.0)), Icon::Search, t.text_dim, false);
    let lang = app.ui.language;
    let ph = lang.tr("Search commands and help");
    if !narrow {
        ui.painter().with_clip_rect(r.shrink2(vec2(8.0, 0.0))).text(
            pos2(r.min.x + 30.0, r.center().y),
            Align2::LEFT_CENTER,
            &ph,
            theme::font(12.0),
            t.text_dim,
        );
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &ph));
    let tip = crate::ribbon::tip("app.palette", &lang.tr("Search commands"));
    if resp.on_hover_text(tip).clicked() {
        app.palette = Some((String::new(), 0));
    }
}

/// Comments: the comments pane. Narrow windows show the icon only.
fn comments_button(ui: &mut Ui, app: &mut SlideApp, t: &Tokens, narrow: bool) {
    let label = app.ui.language.tr("Comments");
    let has_doc = app.session.active().is_some();
    let on = app.ui.pane.as_deref() == Some("comments");
    let resp = row_button(ui, t, Icon::CommentsBubble, if narrow { "" } else { &label }, &label, has_doc, on);
    if resp.on_hover_text(label.as_ref()).clicked() {
        let _ = app.run("view.pane", json!({"pane": "comments", "toggle": true}));
    }
}

/// Play from Start (F5), as WPS's presentation window keeps it in the top row.
fn play_button(ui: &mut Ui, app: &mut SlideApp, t: &Tokens) {
    let label = app.ui.language.tr("Start from Beginning");
    let has_doc = app.session.active().is_some();
    let resp = row_button(ui, t, Icon::PlayFromStart, "", &label, has_doc, false);
    if resp.on_hover_text(format!("{label} (F5)")).clicked() {
        app.start_show(app.show_start_index(), false);
    }
}

/// Share: the accent fill and its text colour; a menu of the copies to share (PDF, PowerPoint,
/// a picture of the slide, the outline), as upstream's File › Export has.
fn share_button(ui: &mut Ui, app: &mut SlideApp, t: &Tokens) {
    let lang = app.ui.language;
    let label = lang.tr("Share");
    let font = theme::bold(12.5);
    let tw = ui.painter().layout_no_wrap(label.to_string(), font.clone(), t.accent_text).size().x;
    let (r, _) = ui.allocate_exact_size(vec2(12.0 + 14.0 + 6.0 + tw + 12.0, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, share_id(), Sense::click());
    let fill = if resp.hovered() { t.accent.lerp_to_gamma(t.text, 0.15) } else { t.accent };
    // White (or dark) text on the hovered fill keeps its contrast: darken toward the text only in
    // light mode, where the text is dark and the label white.
    let fill = if theme::contrast_ratio(t.accent_text, fill) >= 4.5 { fill } else { t.accent };
    ui.painter().rect_filled(r, 8.0, fill);
    icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 19.0, r.center().y), vec2(15.0, 15.0)), Icon::Share, t.accent_text, false);
    ui.painter().text(pos2(r.min.x + 32.0, r.center().y), Align2::LEFT_CENTER, &label, font, t.accent_text);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
    let resp = resp.on_hover_text(lang.tr("Share: export or save a copy"));
    let has_doc = app.session.active().is_some();
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(260.0);
        for (text, fmt) in [
            ("Export as PDF…", "pdf"),
            ("Save a Copy as PowerPoint (.pptx)…", "pptx"),
            ("Export the Current Slide as a Picture (.png)…", "png"),
            ("Export the Outline (.txt)…", "outline"),
        ] {
            if ui.add_enabled(has_doc, egui::Button::new(lang.tr(text))).clicked() {
                app.export_as(fmt, json!({}));
                ui.close();
            }
        }
    });
}

/// "⋯": light or dark, the interface language, Options and About.
fn more_menu(ui: &mut Ui, app: &mut SlideApp, t: &Tokens) {
    let (r, _) = ui.allocate_exact_size(vec2(CONTROL_H, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, more_button_id(), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, 6.0, t.hover);
    }
    icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(18.0, 18.0)), Icon::More, t.text_dim, false);
    let lang = app.ui.language;
    let tip = lang.tr("More options");
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &tip));
    let resp = resp.on_hover_text(tip.as_ref());
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(220.0);
        let dark = app.ui.brightness == theme::Brightness::Dark;
        for (on, text) in [(false, "Light Mode"), (true, "Dark Mode")] {
            if ui.radio(dark == on, lang.tr(text)).clicked() {
                let _ = app.run("view.dark", json!({"on": on}));
            }
        }
        ui.separator();
        ui.label(egui::RichText::new(lang.tr("Interface language")).weak());
        for l in Language::ALL {
            if ui.radio(app.ui.language == l, l.name()).clicked() {
                app.ui.language = l;
            }
        }
        ui.separator();
        if ui.button(lang.tr("Options")).clicked() {
            let _ = app.run("app.preferences", json!({}));
            ui.close();
        }
        if ui.button(lang.tr("About Nexus Slide")).clicked() {
            let _ = app.run("app.about", json!({}));
            ui.close();
        }
    });
}

/// The command strip: the current tab's groups, then "…" with the ones that didn't fit.
fn strip(app: &mut SlideApp, ui: &mut Ui, t: &Tokens) {
    egui::Panel::top("ribbon")
        .exact_size(STRIP_H)
        .frame(
            egui::Frame::NONE
                .fill(t.chrome)
                .inner_margin(egui::Margin { left: 12, right: 8, top: 3, bottom: 3 })
                .stroke(Stroke::new(1.0, t.separator)),
        )
        .show(ui, |ui| {
            let key = Id::new("nexus_wps_group_widths").with((app.ui.tab.as_str(), app.ui.language));
            let known: Vec<f32> = ui.data(|d| d.get_temp(key)).unwrap_or_default();
            let full = ui.available_rect_before_wrap();
            let room = (full.width() - OVERFLOW_W).max(0.0);
            let shown = fit_groups(&known, room);
            let content = Rect::from_min_size(full.min, vec2(room, full.height()));
            // Salted with the tab: every tab's controls keep their own state (a stack's rows).
            let builder = egui::UiBuilder::new().id_salt(("wps_strip", app.ui.tab.as_str())).max_rect(content);
            let mut row = ui.new_child(builder.layout(egui::Layout::left_to_right(egui::Align::Min)));
            row.set_clip_rect(content.intersect(ui.clip_rect()));
            row.spacing_mut().item_spacing = vec2(2.0, 2.0);
            let gate = with_gate(ui.ctx(), Mode::Strip { shown }, || crate::ribbon::tab_groups(app, &mut row));
            let mut widths = known;
            widths.resize(gate.next, 0.0);
            for (w, m) in widths.iter_mut().zip(&gate.widths) {
                if let Some(m) = m {
                    *w = *m;
                }
            }
            if ui.data(|d| d.get_temp::<Vec<f32>>(key)).as_ref() != Some(&widths) {
                ui.data_mut(|d| d.insert_temp(key, widths));
                ui.ctx().request_repaint();
            }
            if shown >= gate.next {
                return;
            }
            let r = Rect::from_min_size(pos2(full.max.x - OVERFLOW_W + 4.0, full.min.y), vec2(OVERFLOW_W - 4.0, widgets::COMPACT_H));
            let resp = ui.interact(r, overflow_button_id(), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(r, 6.0, t.hover);
            }
            icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(20.0, 20.0)), Icon::More, t.text_dim, false);
            let tip = app.ui.language.tr("More commands");
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &tip));
            let resp = resp.on_hover_text(tip.as_ref());
            overflow_panel(app, ui, &resp, shown);
        });
}

/// Where the "…" panel was drawn last frame, while it is open: (its tab, its rect).
fn overflow_open_id() -> Id {
    Id::new("nexus_wps_overflow_open")
}

/// The "…" panel: the groups from `from` on, under the button. Not an egui popup — only one of
/// those can be open, and the groups' own menus (New Slide's layouts, Arrange…) are popups that
/// must open over it. It closes on Escape, on another tab, or on a click outside it (unless that
/// click was in one of the groups' menus).
fn overflow_panel(app: &mut SlideApp, ui: &mut Ui, button: &Response, from: usize) {
    let ctx = ui.ctx().clone();
    let mut open: Option<(String, Rect)> = ctx.data(|d| d.get_temp(overflow_open_id()));
    if button.clicked() {
        open = match open {
            Some(_) => None,
            None => Some((app.ui.tab.clone(), Rect::NOTHING)),
        };
    } else if let Some((tab, rect)) = &open {
        let outside =
            ctx.input(|i| i.pointer.primary_pressed() && i.pointer.interact_pos().is_some_and(|p| !rect.contains(p) && !button.rect.contains(p)));
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if *tab != app.ui.tab || escape || (outside && !egui::Popup::is_any_open(&ctx)) {
            open = None;
        }
    }
    let Some((tab, _)) = open else {
        ctx.data_mut(|d| d.remove::<(String, Rect)>(overflow_open_id()));
        return;
    };
    let area = egui::Area::new(overflow_open_id().with("area"))
        .order(egui::Order::Foreground)
        .pivot(Align2::RIGHT_TOP)
        .fixed_pos(button.rect.right_bottom() + vec2(0.0, 6.0))
        .constrain(true)
        .show(&ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                let builder = egui::UiBuilder::new().id_salt(("wps_overflow", tab.as_str())).layout(egui::Layout::left_to_right(egui::Align::Min));
                ui.scope_builder(builder, |ui| {
                    ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                    with_gate(&ctx, Mode::Menu { from }, || crate::ribbon::tab_groups(app, ui));
                });
            });
        });
    let rect = area.response.rect;
    ctx.data_mut(|d| d.insert_temp(overflow_open_id(), (tab, rect)));
}

/// Whether the "…" panel is open.
pub fn overflow_open(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<(String, Rect)>(overflow_open_id())).is_some()
}

/// How many groups, from the first, fit in `room` points given their widths last drawn: all of
/// them (`usize::MAX`) when they fit. A group not measured yet counts as fitting, so it is drawn
/// (clipped at worst) and measured once.
pub fn fit_groups(widths: &[f32], room: f32) -> usize {
    let mut x = 0.0;
    for (i, w) in widths.iter().enumerate() {
        x += if w.is_finite() { w.max(0.0) } else { 0.0 };
        if x > room + 0.5 {
            return i;
        }
    }
    usize::MAX
}

/// Which groups a pass over the tab draws.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    /// The strip: the first `shown` groups.
    Strip { shown: usize },
    /// The "…" panel: the groups from `from` on.
    Menu { from: usize },
}

/// A pass over the tab's groups: the mode, how many groups were met, and the width of each the
/// strip drew.
#[derive(Clone, Debug)]
struct Gate {
    mode: Mode,
    next: usize,
    widths: Vec<Option<f32>>,
    /// A stack in the group being drawn is still finding its layout, so the group's width this
    /// frame isn't its width: it is recorded as unknown (and the group drawn again).
    unsettled: bool,
}

fn gate_id() -> Id {
    Id::new("nexus_wps_gate")
}

/// Run `draw` with groups gated by `mode`; returns what the pass met.
fn with_gate(ctx: &egui::Context, mode: Mode, draw: impl FnOnce()) -> Gate {
    ctx.data_mut(|d| {
        d.insert_temp(gate_id(), Gate { mode, next: 0, widths: Vec::new(), unsettled: false });
        d.insert_temp(compact_id(), true);
    });
    draw();
    let gate = ctx.data_mut(|d| {
        d.remove::<bool>(compact_id());
        let gate = d.get_temp::<Gate>(gate_id());
        d.remove::<Gate>(gate_id());
        gate
    });
    gate.unwrap_or(Gate { mode, next: 0, widths: Vec::new(), unsettled: false })
}

fn compact_id() -> Id {
    Id::new("nexus_wps_compact")
}

/// Whether the ribbon being drawn is the WPS strip (two rows high: see [`widgets::COMPACT_H`]).
pub(crate) fn compact(ui: &Ui) -> bool {
    ui.data(|d| d.get_temp::<bool>(compact_id())).unwrap_or(false)
}

/// Whether a strip pass is deciding which groups to draw.
pub(crate) fn gated(ui: &Ui) -> bool {
    ui.data(|d| d.get_temp::<Gate>(gate_id()).is_some())
}

#[cfg(test)]
thread_local! {
    /// Groups drawn taller than the strip's two rows (test builds only): "tab #n: height".
    pub(crate) static TALL_GROUPS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A ribbon group in the WPS strip or its "…" panel: drawn when this pass draws it, then a
/// divider; the strip records its width so the groups that don't fit can fold away.
pub(crate) fn group(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    let Some(slot) = group_begin(ui) else { return };
    let t = Tokens::get(ui.ctx());
    let h = widgets::COMPACT_H;
    let start = ui.cursor().min;
    let content = ui.allocate_ui_with_layout(vec2(ui.available_width(), h), egui::Layout::left_to_right(egui::Align::Min), |ui| {
        ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
        add(ui);
    });
    #[cfg(test)]
    if content.response.rect.height() > h + 0.5 {
        let n = match slot {
            Slot::Strip(n) | Slot::Menu(n) => n,
        };
        TALL_GROUPS.with(|g| g.borrow_mut().push(format!("group #{n}: {:.1}", content.response.rect.height())));
    }
    #[cfg(not(test))]
    let _ = content;
    ui.add_space(5.0);
    let x = ui.cursor().min.x;
    ui.painter().line_segment([pos2(x, start.y + 6.0), pos2(x, start.y + h - 6.0)], Stroke::new(1.0, t.separator));
    ui.add_space(6.0);
    group_end(ui, slot, ui.cursor().min.x - start.x);
}

/// A stack of controls (upstream draws them one under another) in the WPS strip. Two rows fit as
/// they are. Three or more flow left to right in two rows (Cut and Copy, then Format Painter), as
/// narrow as two rows allow. What fits is found over a few frames — one column, then one row,
/// then from half that width up until the controls take two rows — and kept per language.
pub(crate) fn compact_stack<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    let key = ui.id().with(("wps_stack", ui.next_auto_id(), i18n::current(ui.ctx()).code()));
    let state: Option<StackFit> = ui.data(|d| d.get_temp(key));
    // Two rows run a point or two past the strip's content height at most; three can't fit.
    let two_rows = widgets::COMPACT_H + 8.0;
    let (layout, max) = match state {
        None | Some(StackFit::Column) => (egui::Layout::top_down(egui::Align::Min), None),
        Some(StackFit::MeasureRow) => (egui::Layout::left_to_right(egui::Align::Min), None),
        Some(StackFit::Rows { width, .. }) => (
            egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true),
            Some(Rect::from_min_size(ui.cursor().min, vec2(width.max(1.0), widgets::COMPACT_H))),
        ),
    };
    let mut builder = egui::UiBuilder::new().layout(layout);
    if let Some(max) = max {
        builder = builder.max_rect(max);
    }
    let r = ui.scope_builder(builder, |ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 2.0);
        add(ui)
    });
    let size = r.response.rect.size();
    if !size.x.is_finite() || !size.y.is_finite() {
        return r.inner;
    }
    let next = match state {
        None if size.y <= two_rows => Some(StackFit::Column),
        None => Some(StackFit::MeasureRow),
        Some(StackFit::MeasureRow) => Some(StackFit::Rows { total: size.x, width: (size.x / 2.0).ceil() }),
        // (A point past one row's width, so rounding never wraps its last control.)
        Some(StackFit::Rows { total, width }) if size.y > two_rows && width < total + 1.0 => {
            Some(StackFit::Rows { total, width: (width + (total * 0.15).max(12.0)).min(total + 1.0) })
        }
        _ => None,
    };
    if let Some(next) = next {
        ui.data_mut(|d| {
            d.insert_temp(key, next);
            if let Some(mut gate) = d.get_temp::<Gate>(gate_id()) {
                gate.unsettled = true;
                d.insert_temp(gate_id(), gate);
            }
        });
        ui.ctx().request_repaint();
    }
    r.inner
}

/// How a [`compact_stack`] lays its controls out.
#[derive(Clone, Copy, Debug, PartialEq)]
enum StackFit {
    /// One under another (they fit in two rows).
    Column,
    /// Being measured in one row.
    MeasureRow,
    /// Flowing in rows `width` wide; `total` is their width in one row.
    Rows { total: f32, width: f32 },
}

/// Where a group is being drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Slot {
    /// The WPS strip, as its group number `n`.
    Strip(usize),
    /// The WPS "…" panel, as its group number `n`.
    Menu(usize),
}

/// Called as a ribbon group starts: where it is drawn, or `None` when this pass leaves it out.
fn group_begin(ui: &Ui) -> Option<Slot> {
    ui.ctx().data_mut(|d| {
        let mut gate = d.get_temp::<Gate>(gate_id())?;
        let n = gate.next;
        gate.next = n.saturating_add(1);
        gate.unsettled = false;
        d.insert_temp(gate_id(), gate.clone());
        match gate.mode {
            Mode::Strip { shown } if n < shown => Some(Slot::Strip(n)),
            Mode::Menu { from } if n >= from => Some(Slot::Menu(n)),
            _ => None,
        }
    })
}

/// Called as a ribbon group ends, with the width it took (its divider included).
fn group_end(ui: &Ui, slot: Slot, width: f32) {
    let Slot::Strip(n) = slot else { return };
    ui.ctx().data_mut(|d| {
        let Some(mut gate) = d.get_temp::<Gate>(gate_id()) else { return };
        if gate.widths.len() <= n {
            gate.widths.resize(n.saturating_add(1), None);
        }
        let unsettled = std::mem::take(&mut gate.unsettled);
        if let Some(w) = gate.widths.get_mut(n) {
            // 0 = not known: the strip draws it (clipped at worst) and measures it again.
            *w = Some(if unsettled { 0.0 } else { width });
        }
        d.insert_temp(gate_id(), gate);
    });
}

#[cfg(test)]
#[path = "nexus_wps_tests.rs"]
mod tests;

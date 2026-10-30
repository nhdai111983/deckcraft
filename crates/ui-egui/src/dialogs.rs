//! Dialogs (Insert Table, Slide Size, Font, Paragraph, Tabs, Header & Footer, Hyperlink, Set Up Show, Zoom, …) and the
//! About window. The start screen is File › Home (`backstage.rs`).

use deckcraft_color::SchemeSlot;
use egui::{Color32, CornerRadius, Rect, Sense, Stroke, Ui, vec2};
use serde_json::{Value, json};

use crate::SlideApp;
use crate::i18n::units;
use crate::i18n::{Language, tl, tl_in};
use crate::theme::{self, Tokens};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Dialog {
    pub id: String,
    pub params: Value,
    /// Field values while the dialog is open.
    #[serde(skip)]
    pub fields: std::collections::HashMap<String, String>,
}

impl Dialog {
    pub fn new(id: &str) -> Self {
        Dialog { id: id.into(), params: Value::Null, fields: Default::default() }
    }
    pub fn modal(&self) -> bool {
        true
    }
    fn get(&mut self, k: &str, default: &str) -> String {
        self.fields.entry(k.into()).or_insert_with(|| default.to_string()).clone()
    }
}

fn title(id: &str) -> &'static str {
    match id {
        "table" => "Insert Table",
        "slideSize" => "Slide Size",
        "sizePosition" => "Size and Position",
        "headerFooter" => "Header and Footer",
        "hyperlink" => "Insert Hyperlink",
        "setupShow" => "Set Up Slide Show",
        "zoom" => "Zoom",
        "comment" => "New Comment",
        "altText" => "Alt Text",
        "outline" => "Slides from Outline",
        "renameSection" => "Rename Section",
        "reuseSlides" => "Reuse Slides",
        "renameShape" => "Rename",
        "renameLayout" => "Rename Layout",
        "paragraph" => "Paragraph",
        "tabs" => "Tabs",
        "font" => "Font",
        "about" => "About Nexus Slide",
        "preferences" => "Preferences",
        "autocorrect" => "AutoCorrect",
        "gridOptions" => "Grid Options",
        "columns" => "Columns",
        "export" => "Export",
        "symbol" => "Symbol",
        "photoAlbum" => "Photo Album",
        "replaceFonts" => "Replace Font",
        "equation" => "Equation",
        "dateTime" => "Date and Time",
        "spelling" => "Spelling",
        "pasteSpecial" => "Paste Special",
        "find" => "Find and Replace",
        "accessibility" => "Accessibility Checker",
        "language" => "Language",
        "customShow" => "Custom Shows",
        "chartData" => "Chart Data",
        "trim" => "Trim Media",
        "smartart" => "Choose a SmartArt Graphic",
        "bullets" => "Bullets and Numbering",
        "action" => "Action Settings",
        "quit" => deckcraft_engine::nexus::APP_NAME,
        "themeColors" => "Create New Theme Colors",
        "themeFonts" => "Create New Theme Fonts",
        _ => deckcraft_engine::nexus::APP_NAME,
    }
}

/// Render the active dialog, clearing it when dismissed and preserving or restoring application
/// state as its close path requires.
pub fn show(app: &mut SlideApp, ctx: &egui::Context) {
    let Some(mut d) = app.dialog.take() else { return };
    // NexusOffice: as a tab of the NexusOffice window, the prompt every Nexus app shows.
    if app.embedded && matches!(d.id.as_str(), "close" | "quit") {
        match embedded_save_prompt(app, ctx, &d) {
            Some(true) => {
                app.dialog = Some(d);
                return;
            }
            Some(false) => return,
            None => {}
        }
    }
    let mut open = true;
    let mut close = false;
    let lang = app.ui.language;
    // NexusOffice: titles in the interface's language; "Save changes?" names itself there.
    let shown = if lang != crate::i18n::Language::En && matches!(d.id.as_str(), "close" | "quit") {
        lang.tr("Save changes?").into_owned()
    } else {
        // A title can differ from the ribbon's word for the same thing ("Đoạn văn", the group "Đoạn").
        lang.lookup(&format!("dialog.{}", d.id)).map_or_else(|| lang.tr(title(&d.id)).into_owned(), str::to_string)
    };
    // NexusOffice: the dialog's numbers and lengths read in the interface's language (`lang_of`).
    d.fields.insert(LANG_FIELD.into(), lang.code().into());
    // NexusOffice: the shared dialog look (sized to the content, 16 px padding).
    let mut window = crate::nexus_dialog::window(ctx, shown, egui::Id::unique(("dialog", d.id.clone()))).open(&mut open);
    // Two columns of colour rows plus Save must fit the first frame (Theme Colors), as must Save
    // in Theme Fonts. A later resize clips the buttons (the window size sticks) and clicks miss them.
    if d.id == "themeColors" {
        window = window.default_size(vec2(760.0, 420.0)).min_width(720.0).min_height(420.0);
    } else if d.id == "themeFonts" {
        window = window.default_size(vec2(460.0, 320.0)).min_width(420.0).min_height(300.0);
    }
    window.show(ctx, |ui| {
        ui.set_min_width(match d.id.as_str() {
            "themeColors" => 700.0,
            "themeFonts" => 400.0,
            _ => crate::nexus_dialog::MIN_WIDTH,
        });
        if d.id == "themeColors" {
            ui.set_min_height(360.0);
        } else if d.id == "themeFonts" {
            ui.set_min_height(280.0);
        }
        close = body(app, ui, &mut d);
    });
    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    if open && !close && !escape {
        app.dialog = Some(d);
    } else if !close && d.id == "preferences" {
        // Esc or the window's close button: the same as Cancel.
        restore_preferences(app, &d);
    } else if app.dialog.is_none()
        && let Some(ret) = d.params.get("returnParams").filter(|v| v.is_object()).cloned()
    {
        // Tabs… opened from Paragraph: closing it (OK, Cancel, Esc or the window) returns there.
        let id = d.params.get("returnId").and_then(Value::as_str).unwrap_or("paragraph");
        let mut back = Dialog::new(id);
        back.params = ret;
        app.dialog = Some(back);
    }
}

/// NexusOffice: "Lưu thay đổi?" over the tab (`nexus_save_prompt`) for File › Close ("close")
/// and quitting ("quit"). Lưu, Không lưu and Hủy do what the stand-alone prompt's Save, Don't Save
/// and Cancel do. Returns whether the prompt stays open, or None when there is nothing to ask
/// about (the stand-alone code then finishes the close or the quit).
fn embedded_save_prompt(app: &mut SlideApp, ctx: &egui::Context, d: &Dialog) -> Option<bool> {
    use crate::nexus_save_prompt as sp;
    let quit = d.id == "quit";
    let names: Vec<String> = if quit {
        app.session.documents().iter().filter(|d| d.is_dirty()).map(|d| d.title()).collect()
    } else {
        app.session.active().map(|d| d.title()).into_iter().collect()
    };
    let lang = app.ui.language;
    let vi = lang == crate::i18n::Language::Vi;
    // Through the catalogue (the same words as `sp::question`), so the interface audit knows them.
    let question = match names.as_slice() {
        [] => return None,
        [one] => lang.fill("Save changes to “{name}” before closing?", &[("name", &sp::shorten_middle(one, 80))]),
        many => lang.fill("Save changes to {n} presentations before closing?", &[("n", &many.len().to_string())]),
    };
    let t = Tokens::get(ctx);
    let prompt = sp::Prompt { words: sp::words(vi), question: &question, note: None, error: None };
    let look = sp::Look {
        dark: t.dark,
        accent: t.accent,
        on_accent: t.accent_text,
        error: ctx.global_style().visuals.error_fg_color,
        title_font: theme::bold(sp::TITLE_SIZE),
        title_fake_bold: !theme::has_semibold(),
    };
    let Some(answer) = sp::show(ctx, egui::Id::unique("nexus_save_prompt"), app.drawn_in, &prompt, &look) else { return Some(true) };
    match (answer, quit) {
        (sp::Answer::Cancel, _) => {}
        (sp::Answer::DontSave, false) => {
            let _ = app.run("file.close", json!({}));
        }
        (sp::Answer::DontSave, true) => {
            let _ = app.session.execute("file.recovery.discard", &json!({}));
            app.quit_confirmed = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        (sp::Answer::Save, false) => {
            app.save();
            // In the browser a save is a download: the presentation stays "unsaved".
            let web = app.services.pick_save.is_none() && app.services.download.is_some();
            if web || app.session.active().is_some_and(|d| !d.is_dirty()) {
                let _ = app.run("file.close", json!({}));
            } else {
                // A presentation with no file waits for the host's Save As.
                app.close_after_pending_save();
            }
        }
        (sp::Answer::Save, true) => {
            if app.save_all() {
                app.quit_confirmed = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
    Some(false)
}

/// NexusOffice: the shared footer (Hủy, then the primary button, right-aligned; Enter is OK).
fn buttons(ui: &mut Ui, ok_label: &str) -> (bool, bool) {
    crate::nexus_dialog::footer(ui, ok_label, true)
}

/// Draw a labelled single-line field bound to `key`, falling back to `default`, and store the
/// edited value in the dialog.
fn field(ui: &mut Ui, d: &mut Dialog, label: &str, key: &str, default: &str) {
    let mut v = d.get(key, default);
    // NexusOffice: an indented label ("  Footer text:") keeps its indent in every language; labels
    // sit left-aligned in one column, and a label too long for it goes above its field
    // (`nexus_dialog::field_row`).
    let name = label.trim_start();
    let indent = label.len().saturating_sub(name.len()) > 0;
    let text = tl(ui, name).into_owned();
    crate::nexus_dialog::field_row(ui, &text, indent, |ui, width, label| {
        let _ = ui.add(egui::TextEdit::singleline(&mut v).desired_width(width).margin(vec2(6.0, 4.0)).min_size(vec2(0.0, 26.0))).labelled_by(label);
    });
    d.fields.insert(key.into(), v);
}

fn check(ui: &mut Ui, d: &mut Dialog, label: &str, key: &str, default: bool) -> bool {
    let mut v = d.get(key, if default { "1" } else { "0" }) == "1";
    ui.checkbox(&mut v, tl(ui, label));
    d.fields.insert(key.into(), if v { "1".into() } else { "0".into() });
    v
}

fn num(d: &mut Dialog, key: &str, default: f64) -> f64 {
    let lang = lang_of(d);
    units::parse_number(lang, d.get(key, "").trim().trim_end_matches(['"', '%', ' '])).unwrap_or(default)
}

/// NexusOffice: where `show` notes the interface language of the frame (numbers and lengths).
const LANG_FIELD: &str = "nx.lang";

/// NexusOffice: the interface language the dialog is drawn in (English until `show` notes it).
fn lang_of(d: &Dialog) -> Language {
    d.fields.get(LANG_FIELD).and_then(|c| Language::parse(c)).unwrap_or_default()
}

/// NexusOffice: typed `text` as upstream's parsers read it: as it is in English; in Vietnamese the
/// number it holds (a comma or a point) written with a point, or "" when it holds none.
fn std_number(lang: Language, text: &str) -> String {
    match lang {
        Language::En => text.to_string(),
        _ => units::parse_measure(lang, text).map(|v| v.to_string()).unwrap_or_default(),
    }
}

fn num_text(v: f64) -> String {
    crate::widgets::fmt_num(v, "")
}

fn field_of(d: &Dialog, key: &str) -> String {
    d.fields.get(key).cloned().unwrap_or_default()
}

fn parse_shown(s: &str) -> Option<f64> {
    let t = s.trim().trim_end_matches(['%', '"']).trim();
    if t.is_empty() {
        return None;
    }
    t.parse().ok().filter(|v: &f64| v.is_finite())
}

fn edited(d: &Dialog, key: &str) -> bool {
    field_of(d, key) != field_of(d, &format!("seed{key}"))
}

fn exact_of(d: &Dialog, key: &str) -> Option<f64> {
    parse_shown(&field_of(d, &format!("{key}Pt")))
}

/// Draw a labelled single-line text row bound to a dialog key and store the edited value.
fn text_row(ui: &mut Ui, d: &mut Dialog, label: &str, key: &str) {
    let mut v = field_of(d, key);
    ui.horizontal(|ui| {
        let l = ui.add_sized(vec2(148.0, 20.0), egui::Label::new(tl(ui, label)));
        ui.add(egui::TextEdit::singleline(&mut v).desired_width(96.0)).labelled_by(l.id);
    });
    d.fields.insert(key.into(), v);
}

use deckcraft_text::datetime::DATE_KINDS;

mod about;
mod alt_text;
mod body;
mod columns;
mod date_time;
mod equation;
mod export;
mod find;
mod font;
mod grid_options;
mod hyperlink;
mod language;
mod paragraph;
mod photo_album;
mod preferences;
mod size_position;
mod slide_size;
mod tabs;
mod theme_colors;
mod theme_fonts;
mod trim;
mod zoom;

pub use about::{about, paint_logo, version_line};
use body::body;
pub use date_time::date_default_lang;
#[cfg(test)]
use hyperlink::{
    CurrentLink, HyperlinkSeed, common_click, hyperlink_command, hyperlink_seed_of, hyperlink_should_apply, link_covering, seed_from, seed_of_link,
};
pub use language::selected_language;
use preferences::restore_preferences;
pub use zoom::parse_zoom_percent;

fn seed_once(d: &mut Dialog, key: &str, val: String) {
    d.fields.entry(key.to_string()).or_insert(val);
}

fn json_text(v: Option<&Value>, default: &str) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.as_f64().map(|x| crate::widgets::fmt_num(x, "")).unwrap_or_else(|| default.to_string()),
        Some(Value::Bool(true)) => "1".into(),
        Some(Value::Bool(false)) => "0".into(),
        _ => default.to_string(),
    }
}

fn color_label(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Object(o) => o.get("scheme").or_else(|| o.get("rgb")).and_then(Value::as_str).unwrap_or("").to_string(),
        _ => String::new(),
    }
}

/// Draw a uniquely identified accessible choice combo box and store the selected key;
/// the caller supplies the visible label.
fn choice(ui: &mut Ui, d: &mut Dialog, salt: &str, name: &str, key: &str, choices: &[(&str, &str)]) {
    let cur = d.get(key, "");
    let label = choices.iter().find(|(v, _)| *v == cur).map(|(_, l)| *l).unwrap_or(cur.as_str());
    // NexusOffice: the items in the interface's language (keyed by field and value where a word
    // differs by place: `align.left` "Căn trái"), worked out before the list opens.
    let shown: Vec<String> = choices.iter().map(|(v, l)| tl_in(ui, key, v, l)).collect();
    let selected = choices.iter().position(|(v, _)| *v == cur).and_then(|i| shown.get(i).cloned()).unwrap_or_else(|| tl(ui, label).into_owned());
    let combo = craft_ui::ComboBox::from_id_salt(salt).selected_text(selected).width(140.0).show_ui(ui, |ui| {
        for ((v, _), l) in choices.iter().zip(&shown) {
            if ui.selectable_label(cur == *v, l.as_str()).clicked() {
                d.fields.insert((*key).into(), (*v).into());
            }
        }
    });
    combo.response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ComboBox, true, name));
}

fn labeled_text(ui: &mut Ui, d: &mut Dialog, label: &str, key: &str, width: f32) -> String {
    let mut v = d.get(key, "");
    let l = ui.label(tl_in(ui, "field", key, label));
    ui.add(egui::TextEdit::singleline(&mut v).desired_width(width)).labelled_by(l.id);
    d.fields.insert(key.into(), v.clone());
    v
}

fn finite_pt(text: &str) -> Option<f64> {
    text.trim().trim_end_matches("pt").trim().parse::<f64>().ok().filter(|v| v.is_finite())
}

fn pt_text(v: Option<&Value>, default: f64) -> String {
    match v.and_then(Value::as_f64).filter(|n| n.is_finite()) {
        Some(n) => crate::widgets::fmt_num(n, ""),
        None => crate::widgets::fmt_num(default, ""),
    }
}

fn inches_text(lang: Language, v: Option<&Value>) -> String {
    let pt = v.and_then(Value::as_f64).filter(|n| n.is_finite()).unwrap_or(0.0);
    match lang {
        Language::En => crate::widgets::fmt_num(pt / 72.0, ""),
        _ => units::format_length(lang, pt),
    }
}

pub mod credits;
#[cfg(test)]
mod font_dialog_tests;
#[cfg(not(target_arch = "wasm32"))]
pub(super) mod updates_ui;

#[cfg(test)]
mod size_position_tests;

#[cfg(test)]
mod columns_dialog_tests;

#[cfg(test)]
mod photo_album_tests;

#[cfg(test)]
mod theme_colors_tests;

#[cfg(test)]
mod theme_fonts_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod spelling_tests;

#[cfg(test)]
mod paragraph_dialog_tests;

#[cfg(test)]
mod preferences_tests;

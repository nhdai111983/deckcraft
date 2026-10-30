//! The NexusOffice layout (Henry, 2026-10-23; ROADMAP F2.2 + F2.4): a menu bar and one row of
//! tools in place of the ribbon, like the WPS-style layout Nexus Docs and Nexus Sheet have.
//!
//! The menus are built from the command registry: every command goes under the ribbon tab and
//! group its location names (a third part becomes a submenu), and locations that name no ribbon
//! tab (Edit, Arrange, Selection Pane, Comments…) fold into the nearest menu, so nothing the
//! registry places is lost. Contextual menus (Shape Format, Picture Format, Table Design…) appear
//! while they apply, as the ribbon's contextual tabs do. A command that needs values runs through
//! the dialog the ribbon uses for it, a submenu of its choices, or a small form of its parameters.
//!
//! The tool row holds the everyday presentation tools; the tools that don't fit a narrow window
//! move into a "More tools" menu at its end. The ribbon's code stays as upstream wrote it and
//! draws when [`SlideApp::toolbar`] is off (upstream's own tests and builds).

use deckcraft_engine::CommandSpec;
use deckcraft_engine::cmd::{Param, parse_params};
use egui::{Align2, Color32, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::{Map, Value, json};

use crate::SlideApp;
use crate::icons::Icon;
use crate::ribbon;
use crate::theme::{self, Tokens};
use crate::widgets::{self, color_split, dropdown, icon_toggle, small_button_ex};

/// Height of the menu bar and of the tool row.
pub const MENU_H: f32 = 28.0;
pub const TOOLS_H: f32 = 36.0;

/// A command placed in the menus: its id and its registry location.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub id: &'static str,
    pub menu: &'static [&'static str],
}

/// UI commands (views, zoom, panes) the ribbon shows, placed as the ribbon places them, so the
/// menus offer them too. Engine commands come from the registry.
pub const UI_PLACES: &[(&str, &[&str])] = &[
    ("view.normal", &["View", "Presentation Views"]),
    ("view.outline", &["View", "Presentation Views"]),
    ("view.sorter", &["View", "Presentation Views"]),
    ("view.notesPage", &["View", "Presentation Views"]),
    ("view.reading", &["View", "Presentation Views"]),
    ("view.ruler", &["View", "Show"]),
    ("view.gridlines", &["View", "Show"]),
    ("view.guides", &["View", "Show"]),
    ("view.notes", &["View", "Show"]),
    ("view.gridOptions", &["View", "Show"]),
    ("view.zoomDialog", &["View", "Zoom"]),
    ("view.zoomIn", &["View", "Zoom"]),
    ("view.zoomOut", &["View", "Zoom"]),
    ("view.fit", &["View", "Zoom"]),
    ("view.grayscale", &["View", "Color/Grayscale"]),
    ("view.dark", &["View", "Color/Grayscale"]),
    ("view.newWindow", &["View", "Window"]),
    ("show.presenter", &["Slide Show", "Monitors"]),
    ("animation.preview", &["Animations", "Preview"]),
    ("review.thesaurusPane", &["Review", "Proofing"]),
];

/// The registry's commands and [`UI_PLACES`], for [`menus`].
pub fn placed(commands: &'static [CommandSpec]) -> Vec<Placed> {
    commands.iter().map(|c| Placed { id: c.id, menu: c.menu }).chain(UI_PLACES.iter().map(|(id, menu)| Placed { id, menu })).collect()
}

/// One entry of a menu group: a command, or a submenu (the location's third part, e.g.
/// Home › Slides › Section) of commands.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Command(&'static str),
    Sub(&'static str, Vec<&'static str>),
}

/// One menu of the menu bar: a ribbon tab and its commands, grouped by ribbon group (in registry
/// order) as `(group, entries)`; `""` is a command placed on the tab itself.
#[derive(Debug, Clone, PartialEq)]
pub struct Menu {
    pub tab: &'static str,
    pub groups: Vec<(&'static str, Vec<Entry>)>,
}

/// The menu a ribbon tab shows as: the contextual Layout tab (tables) is the Table Layout menu,
/// so it doesn't read like the slide's layout.
pub fn menu_name(tab: &str) -> &str {
    if tab == "Layout" { "Table Layout" } else { tab }
}

/// Where a registry location goes in the menus: `(menu, group, submenu)`. Locations that name a
/// pane, a dialog or an upstream menu rather than a ribbon tab fold into the nearest menu.
pub fn place(menu: &[&'static str]) -> Option<(&'static str, &'static str, Option<&'static str>)> {
    let first = *menu.first()?;
    let second = menu.get(1).copied();
    let third = menu.get(2).copied();
    let folded = |tab: &'static str, group: &'static str| Some((tab, group, second));
    match first {
        "Edit" => folded("Home", "Edit"),
        "Arrange" => folded("Home", "Arrange"),
        "Selection Pane" => folded("Home", "Selection Pane"),
        "Format" => folded("Home", "Paragraph"),
        "Comments" => folded("Review", "Comments"),
        "Tools" => folded("Review", "Proofing"),
        "Window" => folded("View", "Window"),
        "Format Background" => folded("Design", "Customize"),
        "Animation Pane" => folded("Animations", "Advanced Animation"),
        "Format Shape" => Some(("Shape Format", second.unwrap_or(""), third)),
        "Video Format" => Some(("Playback", second.unwrap_or(""), third)),
        "Layout" => Some(("Table Layout", second.unwrap_or(""), third)),
        tab => Some((tab, second.unwrap_or(""), third)),
    }
}

/// Groups in the order the ribbon shows them (registry order puts Editing before Font). Groups
/// not listed keep their registry order after these.
const GROUP_ORDER: &[&str] = &["Clipboard", "Slides", "Font", "Paragraph", "Drawing", "Editing", "Designer"];

/// The menus for `tabs` (File excluded: it opens the File page): every placed command whose
/// location goes to the tab ([`place`]) is in that tab's menu, under its group. Tabs with no
/// command are left out.
pub fn menus(items: &[Placed], tabs: &[&str]) -> Vec<Menu> {
    let mut out: Vec<Menu> = Vec::new();
    for &tab in tabs.iter().filter(|t| **t != "File") {
        let tab = menu_name(tab);
        if out.iter().any(|m| m.tab == tab) {
            continue;
        }
        let mut name: Option<&'static str> = None;
        let mut groups: Vec<(&'static str, Vec<Entry>)> = Vec::new();
        for it in items {
            let Some((t, group, sub)) = place(it.menu) else { continue };
            if t != tab {
                continue;
            }
            name.get_or_insert(t);
            if !groups.iter().any(|(g, _)| *g == group) {
                groups.push((group, Vec::new()));
            }
            let Some((_, entries)) = groups.iter_mut().find(|(g, _)| *g == group) else { continue };
            match sub {
                None => entries.push(Entry::Command(it.id)),
                Some(sub) => match entries.iter_mut().find(|e| matches!(e, Entry::Sub(s, _) if *s == sub)) {
                    Some(Entry::Sub(_, ids)) => ids.push(it.id),
                    _ => entries.push(Entry::Sub(sub, vec![it.id])),
                },
            }
        }
        if let Some(tab) = name {
            // The ribbon's order of groups; groups folded in from elsewhere follow.
            groups.sort_by_key(|(g, _)| GROUP_ORDER.iter().position(|o| o == g).unwrap_or(GROUP_ORDER.len()));
            out.push(Menu { tab, groups });
        }
    }
    out
}

/// The tabs whose menus the bar shows: the ribbon's tabs, then the contextual ones that apply now.
fn tab_names(app: &SlideApp) -> (Vec<&'static str>, Vec<&'static str>) {
    let contextual = ribbon::contextual(app);
    let mut tabs: Vec<&'static str> = ribbon::TABS.to_vec();
    tabs.extend(contextual.iter().copied());
    (tabs, contextual)
}

/// The menu bar and the tool row.
pub fn show(app: &mut SlideApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("nexus_menu")
        .exact_size(MENU_H)
        .frame(egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin { left: 6, right: 10, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            menu_bar(app, ui);
            brand_line(ui);
        });
    if app.ui.ribbon_collapsed {
        return;
    }
    egui::Panel::top("nexus_tools")
        .exact_size(TOOLS_H)
        .frame(egui::Frame::NONE.fill(t.card).inner_margin(egui::Margin { left: 8, right: 8, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            let r = ui.max_rect();
            ui.painter().line_segment([pos2(r.left() - 8.0, r.bottom()), pos2(r.right() + 8.0, r.bottom())], Stroke::new(1.0, t.card_border));
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = vec2(1.0, 0.0);
                tools(app, ui);
            });
        });
}

fn menu_bar(app: &mut SlideApp, ui: &mut Ui) {
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing = vec2(1.0, 0.0);
        // File opens the File page (Backstage), as the ribbon's File tab does.
        if menu_title(ui, "File", false).on_hover_text("File (Alt+F)").clicked() {
            app.open_backstage(crate::backstage::Page::Home);
        }
        let (tabs, contextual) = tab_names(app);
        let items = placed(deckcraft_engine::command_specs());
        for m in menus(&items, &tabs) {
            let ctx_tab = contextual.iter().any(|c| menu_name(c) == m.tab);
            let resp = menu_title(ui, m.tab, ctx_tab);
            egui::Popup::menu(&resp).show(|ui| {
                ui.set_min_width(220.0);
                menu_body(app, ui, &m);
            });
        }
    });
}

/// The SonnyTech gradient (indigo → cyan → amber), 2 pt under the menu bar.
fn brand_line(ui: &mut Ui) {
    let r = ui.max_rect();
    let line = Rect::from_min_max(pos2(r.min.x - 6.0, r.max.y - 2.0), pos2(r.max.x + 10.0, r.max.y));
    let [a, b, c] = theme::BRAND_GRADIENT;
    let mid = line.center().x;
    let mut mesh = egui::Mesh::default();
    for (x0, x1, c0, c1) in [(line.min.x, mid, a, b), (mid, line.max.x, b, c)] {
        let i = mesh.vertices.len() as u32;
        mesh.colored_vertex(pos2(x0, line.min.y), c0);
        mesh.colored_vertex(pos2(x1, line.min.y), c1);
        mesh.colored_vertex(pos2(x1, line.max.y), c1);
        mesh.colored_vertex(pos2(x0, line.max.y), c0);
        mesh.add_triangle(i, i + 1, i + 2);
        mesh.add_triangle(i, i + 2, i + 3);
    }
    ui.painter().add(egui::Shape::mesh(mesh));
}

/// A menu bar title: text that highlights on hover and while its menu is open; contextual menus
/// in the contextual colour, as the ribbon draws contextual tabs.
fn menu_title(ui: &mut Ui, text: &str, contextual: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let font = theme::font(13.0);
    let w = ui.painter().layout_no_wrap(text.to_string(), font.clone(), t.text).size().x + 16.0;
    let (r, resp) = ui.allocate_exact_size(vec2(w, MENU_H - 6.0), Sense::click());
    if resp.hovered() || egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&resp)) {
        ui.painter().rect_filled(r, 4.0, t.hover);
    }
    ui.painter().text(r.center(), Align2::CENTER_CENTER, text, font, if contextual { t.contextual } else { t.text });
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, text));
    resp
}

/// A menu's items: one submenu per ribbon group (a menu with a single group lists it directly).
fn menu_body(app: &mut SlideApp, ui: &mut Ui, m: &Menu) {
    let single = m.groups.len() == 1;
    for (group, entries) in &m.groups {
        if single || group.is_empty() {
            entry_items(app, ui, entries);
        } else {
            ui.menu_button(*group, |ui| {
                ui.set_min_width(220.0);
                entry_items(app, ui, entries);
            });
        }
    }
}

fn entry_items(app: &mut SlideApp, ui: &mut Ui, entries: &[Entry]) {
    for e in entries {
        match e {
            Entry::Command(id) => item(app, ui, id),
            Entry::Sub(name, ids) => {
                ui.menu_button(*name, |ui| {
                    ui.set_min_width(200.0);
                    for id in ids {
                        item(app, ui, id);
                    }
                });
            }
        }
    }
}

/// A command's label, shortcut and parameter doc (engine or UI command).
fn describe(id: &str) -> Option<(&'static str, Option<&'static str>, &'static str)> {
    if let Some(c) = deckcraft_engine::find_command(id) {
        return Some((c.label, c.shortcut, c.params));
    }
    crate::UI_COMMANDS.iter().find(|c| c.0 == id).map(|c| (c.1, c.2, c.3))
}

/// What a menu item does when picked.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Run this command with these parameters (the command itself, or the UI command or dialog
    /// the ribbon uses to ask for its values).
    Run(&'static str, Value),
    /// The command needs one value from a short list: a submenu of `(label, params)`.
    Choices(Vec<(String, Value)>),
    /// The command needs values with no dialog of their own: a small form of its parameters.
    Form,
}

/// The dialog or UI command the ribbon uses to ask for a command's values.
fn substitute(id: &str) -> Option<(&'static str, Value)> {
    let dialog = |d: &str| Some(("app.dialog", json!({ "id": d })));
    match id {
        "edit.pasteSpecial" => Some(("app.pasteSpecial", json!({}))),
        "text.find" => Some(("app.find", json!({}))),
        "text.replace" => Some(("app.replace", json!({}))),
        "insert.picture" => Some(("app.insertPictureDialog", json!({}))),
        "insert.audio" => Some(("app.insertAudioDialog", json!({}))),
        "insert.video" => Some(("app.insertVideoDialog", json!({}))),
        "slide.reuse" => Some(("app.reuseSlidesDialog", json!({}))),
        "design.browseTheme" => Some(("app.browseThemeDialog", json!({}))),
        "review.compare" => Some(("app.compareDialog", json!({}))),
        "picture.change" => Some(("app.changePictureDialog", json!({}))),
        "insert.table" => dialog("table"),
        "insert.hyperlink" => dialog("hyperlink"),
        "insert.symbol" => dialog("symbol"),
        "insert.smartArt" => dialog("smartart"),
        "design.slideSize" => dialog("slideSize"),
        "design.headerFooter" => dialog("headerFooter"),
        "comment.add" => dialog("comment"),
        "shape.altText" => dialog("altText"),
        "shape.action" => dialog("action"),
        "font.replace" => dialog("replaceFonts"),
        "slide.fromOutline" => dialog("outline"),
        "show.customShow" => dialog("customShow"),
        "show.setup" => dialog("setupShow"),
        "chart.data" => dialog("chartData"),
        _ => None,
    }
}

/// `justifyLow` → "Justify Low", `rotate90` → "Rotate90".
fn humanize(v: &str) -> String {
    let mut out = String::new();
    for (i, ch) in v.chars().enumerate() {
        if i == 0 {
            out.extend(ch.to_uppercase());
        } else {
            if ch.is_uppercase() {
                out.push(' ');
            }
            out.push(ch);
        }
    }
    out
}

/// The values a parameter's doc lists: `left|center|right` (identifiers only; `…` and remarks
/// end the list), `bool`, or a short `index: 0..3` range.
fn choices(p: &Param<'_>) -> Vec<(String, Value)> {
    let head = p.about.split_whitespace().next().unwrap_or("");
    if head == "bool" {
        return vec![("On".into(), json!({ p.name: true })), ("Off".into(), json!({ p.name: false }))];
    }
    if p.name == "index"
        && let Some((a, b)) = head.split_once("..")
        && let (Ok(a), Ok(b)) = (a.parse::<u32>(), b.parse::<u32>())
        && b >= a
        && b - a <= 9
    {
        return (a..=b).map(|n| (format!("{}", n.saturating_add(1)), json!({ p.name: n }))).collect();
    }
    let ident = |v: &&str| !v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric()) && v.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
    let vals: Vec<&str> = head.split('|').take_while(ident).collect();
    if vals.len() < 2 {
        return vec![];
    }
    vals.into_iter().map(|v| (humanize(v), json!({ p.name: v }))).collect()
}

/// How a menu runs command `id` with the parameter doc `params`.
pub fn action(id: &'static str, params: &str) -> Action {
    if let Some((ui_id, p)) = substitute(id) {
        return Action::Run(ui_id, p);
    }
    if !deckcraft_engine::cmd::needs_arguments(params) {
        return Action::Run(id, json!({}));
    }
    let ps = parse_params(params).unwrap_or_default();
    let required: Vec<&Param<'_>> = ps.iter().filter(|p| p.object == 0 && !p.optional).collect();
    if let [only] = required.as_slice() {
        let list = choices(only);
        if !list.is_empty() {
            return Action::Choices(list);
        }
    }
    Action::Form
}

/// A command as a menu item: its label and shortcut, greyed out when the command says it can't
/// run now. A command that needs values asks for them (dialog, choices or a form).
fn item(app: &mut SlideApp, ui: &mut Ui, id: &'static str) {
    let Some((label, shortcut, params)) = describe(id) else { return };
    let on = ribbon::enabled(app, id);
    if rich_item(app, ui, id, label, on) {
        return;
    }
    match action(id, params) {
        Action::Choices(list) => {
            ui.add_enabled_ui(on, |ui| {
                ui.menu_button(label, |ui| {
                    for (name, p) in list {
                        if ui.button(name).clicked() {
                            let _ = app.run(id, p);
                            ui.close();
                        }
                    }
                });
            });
        }
        a => {
            let text = if matches!(a, Action::Form) && !label.ends_with('…') { format!("{label}…") } else { label.to_string() };
            // An empty shortcut would leave a trailing space in the item's accessible name.
            let button = match shortcut {
                Some(sc) if !sc.is_empty() => egui::Button::new(text).shortcut_text(ribbon::pretty_shortcut(sc)),
                _ => egui::Button::new(text),
            };
            if ui.add_enabled(on, button).clicked() {
                match a {
                    Action::Run(run, p) => {
                        let _ = app.run(run, p);
                    }
                    _ => open_form(ui.ctx(), id),
                }
                ui.close();
            }
        }
    }
}

/// The commands whose values the ribbon picks from a list or gallery get that list as a submenu:
/// font families, sizes, colours, layouts and shapes. Returns whether `id` is one of them.
fn rich_item(app: &mut SlideApp, ui: &mut Ui, id: &str, label: &str, on: bool) -> bool {
    if !matches!(id, "format.font" | "format.size" | "format.color" | "slide.layout" | "shape.insert") {
        return false;
    }
    let label = label.trim_end_matches('…');
    ui.add_enabled_ui(on, |ui| {
        ui.menu_button(label, |ui| match id {
            "format.font" => {
                let fams = ribbon::font_families(app);
                egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    for f in &fams {
                        if ui.button(f.as_str()).clicked() {
                            let _ = app.run("format.font", json!({"family": f}));
                            ui.close();
                        }
                    }
                });
            }
            "format.size" => {
                egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    for s in deckcraft_engine::cmd::format::SIZES.iter().chain([72.0, 80.0, 88.0, 96.0].iter()) {
                        if ui.button(widgets::fmt_num(*s, "")).clicked() {
                            let _ = app.run("format.size", json!({"size": s}));
                            ui.close();
                        }
                    }
                });
            }
            "format.color" => {
                let sc = ribbon::scheme(app);
                if let Some(Some(c)) = widgets::color_grid(ui, &sc, None) {
                    let _ = app.run("format.color", json!({"color": ribbon::cref_param(&c)}));
                }
            }
            "slide.layout" => ribbon::layout_menu(app, ui, "slide.layout"),
            _ => {
                if let Some(p) = ribbon::shapes_gallery(ui) {
                    app.session.set_tool(deckcraft_engine::ToolKind::Shape { preset: p.to_string() });
                    ui.close();
                }
            }
        });
    });
    true
}

/// The parameter form for a command that needs values with no dialog of its own.
#[derive(Clone, Debug, Default, PartialEq)]
struct Form {
    id: String,
    /// `(name, value as typed, what the doc says of it, required)`.
    fields: Vec<(String, String, String, bool)>,
    error: Option<String>,
}

fn form_id() -> egui::Id {
    egui::Id::unique("nexus_command_form")
}

/// Opens the parameter form for `id` (its first set of parameters, required ones first).
pub fn open_form(ctx: &egui::Context, id: &str) {
    let Some((_, _, params)) = describe(id) else { return };
    let mut ps = parse_params(params).unwrap_or_default();
    ps.retain(|p| p.object == 0);
    ps.sort_by_key(|p| p.optional);
    let fields = ps.iter().map(|p| (p.name.to_string(), String::new(), p.about.to_string(), !p.optional)).collect();
    ctx.data_mut(|d| d.insert_temp(form_id(), Form { id: id.to_string(), fields, error: None }));
}

/// A typed value: JSON when it reads as JSON (`12`, `true`, `[1, 2]`, `"text"`), else the text.
fn typed(v: &str) -> Value {
    serde_json::from_str::<Value>(v.trim()).unwrap_or_else(|_| Value::String(v.to_string()))
}

/// The parameter form's window, while one is open.
pub fn args_window(app: &mut SlideApp, ctx: &egui::Context) {
    let Some(mut form) = ctx.data(|d| d.get_temp::<Form>(form_id())) else { return };
    let t = Tokens::get(ctx);
    let title = describe(&form.id).map(|d| d.0).unwrap_or("Command");
    let mut open = true;
    let mut done = false;
    let mut run = false;
    egui::Window::new(title.trim_end_matches('…'))
        .id(egui::Id::unique("nexus_command_form_window"))
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(format!("Values for {}", form.id)).color(t.text_dim));
            ui.add_space(4.0);
            egui::Grid::new("nexus_command_form_grid").num_columns(2).spacing(vec2(8.0, 6.0)).show(ui, |ui| {
                for (name, value, about, required) in form.fields.iter_mut() {
                    ui.label(if *required { format!("{name} *") } else { name.clone() });
                    ui.add(egui::TextEdit::singleline(value).hint_text(about.as_str()).desired_width(260.0));
                    ui.end_row();
                }
            });
            if let Some(e) = &form.error {
                ui.label(egui::RichText::new(e).color(Color32::from_rgb(0xBE, 0x12, 0x3C)));
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(egui::RichText::new("Run").color(t.accent_text)).fill(t.accent).min_size(vec2(72.0, 24.0))).clicked()
                    || ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    run = true;
                }
                if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    done = true;
                }
            });
        });
    if run {
        let mut p = Map::new();
        for (name, value, _, _) in &form.fields {
            if !value.trim().is_empty() {
                p.insert(name.clone(), typed(value));
            }
        }
        let id = form.id.clone();
        match app.run(&id, Value::Object(p)) {
            Ok(_) => done = true,
            Err(e) => form.error = Some(e),
        }
    }
    if done || !open {
        ctx.data_mut(|d| d.remove::<Form>(form_id()));
    } else {
        ctx.data_mut(|d| d.insert_temp(form_id(), form));
    }
}

/// The tool row's clusters, in the order they give way when the window is narrow (the last first).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Cluster {
    Slides,
    Clipboard,
    Font,
    Format,
    Paragraph,
    Insert,
    Arrange,
    Show,
    Find,
}

impl Cluster {
    const ALL: [Cluster; 9] = [
        Cluster::Slides,
        Cluster::Clipboard,
        Cluster::Font,
        Cluster::Format,
        Cluster::Paragraph,
        Cluster::Insert,
        Cluster::Arrange,
        Cluster::Show,
        Cluster::Find,
    ];

    /// Width before the cluster has been drawn once (then its measured width is used).
    fn estimate(self) -> f32 {
        match self {
            Cluster::Slides => 132.0,
            Cluster::Clipboard => 112.0,
            Cluster::Font => 210.0,
            Cluster::Format => 110.0,
            Cluster::Paragraph => 170.0,
            Cluster::Insert => 104.0,
            Cluster::Arrange => 100.0,
            Cluster::Show => 52.0,
            Cluster::Find => 30.0,
        }
    }

    /// The cluster's commands as menu items, for the More tools menu when it doesn't fit.
    fn items(self) -> Vec<(&'static str, &'static str, Value)> {
        match self {
            Cluster::Slides => vec![
                ("New Slide", "slide.new", json!({})),
                ("Duplicate Selected Slides", "slide.duplicate", json!({})),
                ("Reset Slide", "slide.reset", json!({})),
            ],
            Cluster::Clipboard => vec![
                ("Paste", "edit.paste", json!({})),
                ("Cut", "edit.cut", json!({})),
                ("Copy", "edit.copy", json!({})),
                ("Format Painter", "format.painter", json!({})),
            ],
            Cluster::Font => vec![
                ("Font…", "format.fontDialog", json!({})),
                ("Increase Font Size", "format.grow", json!({})),
                ("Decrease Font Size", "format.shrink", json!({})),
            ],
            Cluster::Format => vec![
                ("Bold", "format.bold", json!({})),
                ("Italic", "format.italic", json!({})),
                ("Underline", "format.underline", json!({})),
                ("Font Color", "format.color", json!({"color": "#C00000"})),
            ],
            Cluster::Paragraph => vec![
                ("Align Left", "format.alignLeft", json!({})),
                ("Center", "format.alignCenter", json!({})),
                ("Align Right", "format.alignRight", json!({})),
                ("Justify", "format.justify", json!({})),
                ("Bullets", "format.bullets", json!({})),
                ("Numbering", "format.numbering", json!({})),
            ],
            Cluster::Insert => vec![
                ("Text Box", "insert.textBox", json!({})),
                ("Picture from File…", "app.insertPictureDialog", json!({})),
                ("Insert Table…", "app.dialog", json!({"id": "table"})),
            ],
            Cluster::Arrange => vec![
                ("Bring to Front", "arrange.bringToFront", json!({})),
                ("Send to Back", "arrange.sendToBack", json!({})),
                ("Group", "arrange.group", json!({})),
                ("Ungroup", "arrange.ungroup", json!({})),
            ],
            Cluster::Show => vec![("Play from Start", "show.fromStart", json!({})), ("Play from Current Slide", "show.fromCurrent", json!({}))],
            Cluster::Find => vec![("Find…", "app.find", json!({})), ("Replace…", "app.replace", json!({}))],
        }
    }
}

/// Room kept at the row's end for the More tools button.
const MORE_W: f32 = 34.0;

/// How many clusters of `widths` fit in `avail` (keeping room for More tools when not all do).
fn fitting(widths: &[f32], avail: f32) -> usize {
    if widths.iter().sum::<f32>() <= avail {
        return widths.len();
    }
    let mut used = MORE_W;
    let mut shown = 0;
    for w in widths {
        used += w;
        if used > avail {
            break;
        }
        shown += 1;
    }
    shown
}

/// The tool row: as many clusters as fit, then a More tools menu with the rest.
fn tools(app: &mut SlideApp, ui: &mut Ui) {
    let widths_id = egui::Id::unique("nexus_tool_widths");
    let mut widths: Vec<f32> = ui.data(|d| d.get_temp::<Vec<f32>>(widths_id)).unwrap_or_default();
    widths.resize(Cluster::ALL.len(), 0.0);
    let known: Vec<f32> = Cluster::ALL.iter().enumerate().map(|(i, c)| widths.get(i).copied().filter(|w| *w > 0.0).unwrap_or(c.estimate())).collect();
    let shown = fitting(&known, ui.available_width());
    let mut measured = widths.clone();
    let st = ribbon::fmt_state(app);
    for (i, c) in Cluster::ALL.iter().enumerate().take(shown) {
        let x0 = ui.cursor().min.x;
        if i > 0 {
            divider(ui);
        }
        draw(app, ui, *c, &st);
        if let Some(m) = measured.get_mut(i) {
            *m = ui.cursor().min.x - x0;
        }
    }
    if shown < Cluster::ALL.len() {
        divider(ui);
        let t = Tokens::get(ui.ctx());
        let (rect, more) = ui.allocate_exact_size(vec2(28.0, 26.0), Sense::click());
        if more.hovered() {
            ui.painter().rect_filled(rect, 4.0, t.hover);
        }
        crate::icons::paint(ui.painter(), rect.shrink(4.0), Icon::More, t.text, false);
        more.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "More tools"));
        let more = more.on_hover_text("More tools");
        egui::Popup::menu(&more).show(|ui| {
            ui.set_min_width(200.0);
            for (n, c) in Cluster::ALL.iter().enumerate().skip(shown) {
                if n > shown {
                    ui.separator();
                }
                for (label, id, p) in c.items() {
                    if ui.add_enabled(ribbon::enabled(app, id), egui::Button::new(label)).clicked() {
                        let _ = app.run(id, p);
                        ui.close();
                    }
                }
            }
        });
    }
    if measured != widths {
        ui.data_mut(|d| d.insert_temp(widths_id, measured));
    }
}

/// A thin divider between tool clusters.
fn divider(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.add_space(4.0);
    let (r, _) = ui.allocate_exact_size(vec2(1.0, 20.0), Sense::hover());
    ui.painter().line_segment([r.center_top(), r.center_bottom()], Stroke::new(1.0, t.separator));
    ui.add_space(4.0);
}

/// Gives a tool its plain name for screen readers and tests (the tooltip also has the shortcut).
fn named(resp: egui::Response, name: &str, selected: Option<bool>) -> egui::Response {
    let enabled = resp.enabled();
    resp.widget_info(|| match selected {
        Some(on) => egui::WidgetInfo::selected(egui::Role::CheckBox, enabled, on, name),
        None => egui::WidgetInfo::labeled(egui::Role::Button, enabled, name),
    });
    resp
}

/// An icon tool that runs `id` (toggled on when `on`).
fn tool(app: &mut SlideApp, ui: &mut Ui, icon: Icon, name: &str, id: &str, p: Value, on: Option<bool>) {
    let en = ribbon::enabled(app, id);
    let r = icon_toggle(ui, icon, &ribbon::tip(id, name), en, on.unwrap_or(false));
    if named(r, name, on).clicked() {
        let _ = app.run(id, p);
    }
}

fn draw(app: &mut SlideApp, ui: &mut Ui, c: Cluster, st: &Value) {
    let b = |k: &str| st.get(k).and_then(Value::as_bool).unwrap_or(false);
    match c {
        Cluster::Slides => {
            let en = ribbon::enabled(app, "slide.new");
            let (m, a) = small_button_ex(ui, Icon::NewSlide, "New Slide", en, false, true);
            if m.on_hover_text(ribbon::tip("slide.new", "New Slide")).clicked() {
                let _ = app.run("slide.new", json!({}));
            }
            if let Some(a) = a {
                let a = named(a, "New Slide layouts", None);
                egui::Popup::menu(&a).width(420.0).show(|ui| ribbon::layout_menu(app, ui, "slide.new"));
            }
            let (l, _) = small_button_ex(ui, Icon::Layout, "Layout", ribbon::enabled(app, "slide.layout"), false, true);
            egui::Popup::menu(&l).width(420.0).show(|ui| ribbon::layout_menu(app, ui, "slide.layout"));
        }
        Cluster::Clipboard => {
            let (m, a) = small_button_ex(ui, Icon::Paste, "", ribbon::enabled(app, "edit.paste"), false, true);
            if named(m, "Paste", None).on_hover_text(ribbon::tip("edit.paste", "Paste")).clicked() {
                let _ = app.run("edit.paste", json!({}));
            }
            if let Some(a) = a {
                let a = named(a, "Paste options", None);
                egui::Popup::menu(&a).show(|ui| {
                    for (label, id) in [
                        ("Paste", "edit.paste"),
                        ("Paste and Match Formatting", "edit.pasteText"),
                        ("Paste Special…", "app.pasteSpecial"),
                        ("Paste Formatting", "format.painterApply"),
                    ] {
                        if ui.button(label).clicked() {
                            let _ = app.run(id, json!({}));
                        }
                    }
                });
            }
            tool(app, ui, Icon::Cut, "Cut", "edit.cut", json!({}), None);
            tool(app, ui, Icon::Copy, "Copy", "edit.copy", json!({}), None);
            let on = app.session.painter.is_some();
            let en = ribbon::enabled(app, "format.painter");
            let r = named(icon_toggle(ui, Icon::FormatPainter, "Format Painter (double-click to keep it on)", en, on), "Format Painter", Some(on));
            if r.double_clicked() {
                let _ = app.run("format.painter", json!({"sticky": true}));
            } else if r.clicked() {
                if on {
                    app.session.painter = None;
                } else {
                    let _ = app.run("format.painter", json!({}));
                }
            }
        }
        Cluster::Font => {
            let en = ribbon::enabled(app, "format.bold");
            let font = st.get("font").and_then(Value::as_str).unwrap_or("").to_string();
            let size = st.get("size").and_then(Value::as_f64);
            let fams = ribbon::font_families(app);
            let r = dropdown(ui, "nexus_font", &font, 140.0, en, |ui| {
                for f in &fams {
                    if ui.button(f.as_str()).clicked() {
                        let _ = app.run("format.font", json!({"family": f}));
                        ui.close();
                    }
                }
            });
            named(r, "Font", None);
            ui.add_space(3.0);
            let size_text = size.map(|s| widgets::fmt_num(s, "")).unwrap_or_default();
            let r = dropdown(ui, "nexus_size", &size_text, 48.0, en, |ui| {
                for s in deckcraft_engine::cmd::format::SIZES.iter().chain([72.0, 80.0, 88.0, 96.0].iter()) {
                    if ui.button(widgets::fmt_num(*s, "")).clicked() {
                        let _ = app.run("format.size", json!({"size": s}));
                        ui.close();
                    }
                }
            });
            named(r, "Font Size", None);
        }
        Cluster::Format => {
            tool(app, ui, Icon::Bold, "Bold", "format.bold", json!({}), Some(b("bold")));
            tool(app, ui, Icon::Italic, "Italic", "format.italic", json!({}), Some(b("italic")));
            tool(app, ui, Icon::Underline, "Underline", "format.underline", json!({}), Some(b("underline")));
            let en = ribbon::enabled(app, "format.color");
            let (m, a) = color_split(ui, Icon::FontColor, "Font Color", Color32::from_rgb(0xC0, 0x00, 0x00), en);
            if named(m, "Font Color", None).clicked() {
                let _ = app.run("format.color", json!({"color": "#C00000"}));
            }
            let a = named(a, "Font Color options", None);
            let sc = ribbon::scheme(app);
            widgets::color_popup(&a).show(|ui| {
                if let Some(Some(c)) = widgets::color_grid(ui, &sc, None) {
                    let _ = app.run("format.color", json!({"color": ribbon::cref_param(&c)}));
                }
            });
        }
        Cluster::Paragraph => {
            let al = st.get("align").and_then(Value::as_str).unwrap_or("").to_string();
            tool(app, ui, Icon::AlignLeft, "Align Left", "format.alignLeft", json!({}), Some(al == "l"));
            tool(app, ui, Icon::AlignCenter, "Center", "format.alignCenter", json!({}), Some(al == "ctr"));
            tool(app, ui, Icon::AlignRight, "Align Right", "format.alignRight", json!({}), Some(al == "r"));
            tool(app, ui, Icon::Justify, "Justify", "format.justify", json!({}), Some(al == "just"));
            ui.add_space(2.0);
            let en = ribbon::enabled(app, "format.bullets");
            let (m, a) = small_button_ex(ui, Icon::Bullets, "", en, b("bullets"), true);
            if named(m, "Bullets", Some(b("bullets"))).on_hover_text("Bullets").clicked() {
                let _ = app.run("format.bullets", json!({}));
            }
            if let Some(a) = a {
                let a = named(a, "Bullet styles", None);
                egui::Popup::menu(&a).show(|ui| {
                    if ui.button("None").clicked() {
                        let _ = app.run("format.bullets", json!({"on": false}));
                    }
                    for ch in ["•", "○", "■", "□", "◆", "➢", "✓", "–", "★"] {
                        if ui.button(egui::RichText::new(format!("{ch}  Bullet")).size(14.0)).clicked() {
                            let _ = app.run("format.bullets", json!({"on": true, "char": ch}));
                        }
                    }
                    ui.separator();
                    if ui.button("Bullets and Numbering…").clicked() {
                        let _ = app.run("app.dialog", json!({"id": "bullets"}));
                    }
                });
            }
            let (m, a) = small_button_ex(ui, Icon::Numbering, "", en, b("numbering"), true);
            if named(m, "Numbering", Some(b("numbering"))).on_hover_text("Numbering").clicked() {
                let _ = app.run("format.numbering", json!({}));
            }
            if let Some(a) = a {
                let a = named(a, "Numbering styles", None);
                egui::Popup::menu(&a).show(|ui| {
                    if ui.button("None").clicked() {
                        let _ = app.run("format.numbering", json!({"on": false}));
                    }
                    for (label, sch) in [
                        ("1. 2. 3.", "arabicPeriod"),
                        ("1) 2) 3)", "arabicParenR"),
                        ("I. II. III.", "romanUcPeriod"),
                        ("A. B. C.", "alphaUcPeriod"),
                        ("a) b) c)", "alphaLcParenR"),
                    ] {
                        if ui.button(label).clicked() {
                            let _ = app.run("format.numbering", json!({"on": true, "scheme": sch}));
                        }
                    }
                });
            }
        }
        Cluster::Insert => {
            tool(app, ui, Icon::TextBox, "Text Box", "insert.textBox", json!({}), None);
            let en = ribbon::enabled(app, "shape.insert");
            let r = named(icon_toggle(ui, Icon::Shapes, "Shapes", en, false), "Shapes", None);
            egui::Popup::menu(&r).show(|ui| {
                if let Some(p) = ribbon::shapes_gallery(ui) {
                    app.session.set_tool(deckcraft_engine::ToolKind::Shape { preset: p.to_string() });
                }
            });
            tool(app, ui, Icon::Picture, "Picture", "app.insertPictureDialog", json!({}), None);
            let r = named(icon_toggle(ui, Icon::Table, "Table", ribbon::enabled(app, "insert.table"), false), "Table", None);
            egui::Popup::menu(&r).show(|ui| {
                if let Some((rr, cc)) = ribbon::table_grid_picker(ui) {
                    let _ = app.run("insert.table", json!({"rows": rr, "cols": cc}));
                    ui.close();
                }
                ui.separator();
                if ui.button("Insert Table…").clicked() {
                    let _ = app.run("app.dialog", json!({"id": "table"}));
                    ui.close();
                }
            });
        }
        Cluster::Arrange => {
            let sel = ribbon::enabled(app, "shape.fill");
            let (r, _) = small_button_ex(ui, Icon::Arrange, "Arrange", sel, false, true);
            egui::Popup::menu(&r).show(|ui| ribbon::arrange_menu(app, ui));
        }
        Cluster::Show => {
            let has = app.session.active().is_some_and(|d| !d.doc.slides.is_empty());
            let r =
                named(icon_toggle(ui, Icon::PlayFromStart, &ribbon::tip("show.fromStart", "Play from Start"), has, false), "Play from Start", None);
            if r.clicked() {
                app.start_show(app.show_start_index(), false);
            }
            let r = named(
                icon_toggle(ui, Icon::PlayFromCurrent, &ribbon::tip("show.fromCurrent", "Play from Current Slide"), has, false),
                "Play from Current Slide",
                None,
            );
            if r.clicked() {
                let i = app.session.active().map(|d| d.selection.slide).unwrap_or(0);
                app.start_show(i, false);
            }
        }
        Cluster::Find => tool(app, ui, Icon::Search, "Find", "app.find", json!({}), None),
    }
}

/// Preferences: the accent colour (tabs, buttons, the selected slide): the app's own Amber, one
/// of the palette's presets or any colour from the picker (`ui.accent`, saved with the interface
/// preferences). Text on it stays readable whatever is picked ([`theme::with_accent`]).
pub fn accent_row(app: &mut SlideApp, ui: &mut Ui) {
    let current = app.ui.accent.as_deref().and_then(theme::parse_hex);
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.label("Accent color");
        ui.add_space(4.0);
        if ui.selectable_label(current.is_none(), "App color").clicked() {
            let _ = app.run("ui.accent", json!({"value": null}));
        }
        for (name, c) in theme::ACCENT_PRESETS {
            let (r, resp) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::click());
            ui.painter().circle_filled(r.center(), 8.0, c);
            let on = current == Some(c);
            if on {
                ui.painter().circle_stroke(r.center(), 10.0, Stroke::new(2.0, t.text));
            } else if resp.hovered() {
                ui.painter().circle_stroke(r.center(), 10.0, Stroke::new(1.0, t.text_dim));
            }
            resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::RadioButton, true, on, name));
            if resp.on_hover_text(name).clicked() {
                let _ = app.run("ui.accent", json!({"value": theme::to_hex(c)}));
            }
        }
        ui.add_space(4.0);
        let c = current.unwrap_or(theme::APP_COLOR);
        let mut rgb = [c.r(), c.g(), c.b()];
        let picker = ui.color_edit_button_srgb(&mut rgb);
        if picker.on_hover_text("Any color").changed() {
            let _ = app.run("ui.accent", json!({"value": theme::to_hex(Color32::from_rgb(rgb[0], rgb[1], rgb[2]))}));
        }
    });
}

#[cfg(test)]
#[path = "nexus_toolbar_tests.rs"]
mod tests;

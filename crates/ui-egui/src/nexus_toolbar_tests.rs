//! The NexusOffice menu bar and tool row (ROADMAP F2.2) and the user accent colour (F2.4).
#![allow(clippy::unwrap_used, clippy::expect_used)]

use deckcraft_engine::Session;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::json;

use super::*;
use crate::{Services, SlideApp};

/// Every contextual tab the ribbon can show.
const CONTEXTUAL: [&str; 10] = [
    "Shape Format",
    "Picture Format",
    "Table Design",
    "Layout",
    "SmartArt Design",
    "Chart Design",
    "Playback",
    "Slide Master",
    "Handout Master",
    "Notes Master",
];

fn ids(m: &Menu) -> Vec<&'static str> {
    m.groups
        .iter()
        .flat_map(|(_, es)| {
            es.iter().flat_map(|e| match e {
                Entry::Command(id) => vec![*id],
                Entry::Sub(_, ids) => ids.clone(),
            })
        })
        .collect()
}

/// Every command of a shown tab is in that tab's menu, under its ribbon group, so leaving the
/// ribbon loses no command; locations with no ribbon tab fold into the nearest menu.
#[test]
fn menus_hold_every_command_of_their_tab() {
    let items = placed(deckcraft_engine::command_specs());
    let tabs = ["File", "Home", "Insert", "Design", "Review", "View", "Record", "No Such Tab"];
    let ms = menus(&items, &tabs);
    assert_eq!(ms.iter().map(|m| m.tab).collect::<Vec<_>>(), ["Home", "Insert", "Design", "Review", "View"], "File and empty tabs have no menu");
    for m in &ms {
        let listed = ids(m);
        let expected: Vec<&str> = items.iter().filter(|c| place(c.menu).is_some_and(|p| p.0 == m.tab)).map(|c| c.id).collect();
        assert_eq!(listed.len(), expected.len(), "{}", m.tab);
        for id in expected {
            assert!(listed.contains(&id), "{id} in {}", m.tab);
        }
    }
    let home = &ms[0];
    assert!(home.groups.iter().any(|(g, es)| *g == "Font" && es.contains(&Entry::Command("format.bold"))), "{home:?}");
    // In the ribbon's order: Clipboard, Slides, Font, Paragraph… then the folded groups.
    let order: Vec<&str> = home.groups.iter().map(|(g, _)| *g).collect();
    assert_eq!(order.get(..4), Some(&["Clipboard", "Slides", "Font", "Paragraph"][..]), "{order:?}");
    // Home › Slides › Section is a submenu of the Slides group; Edit and Arrange join Home.
    assert!(
        home.groups
            .iter()
            .any(|(g, es)| *g == "Slides" && es.iter().any(|e| matches!(e, Entry::Sub("Section", ids) if ids.contains(&"section.add"))))
    );
    assert!(ids(home).contains(&"edit.undo"));
    assert!(home.groups.iter().any(|(g, es)| *g == "Arrange" && es.iter().any(|e| matches!(e, Entry::Sub("Align or Distribute", _)))));
    // The views the ribbon's View tab shows (UI commands) are in the View menu too.
    assert!(ids(&ms[4]).contains(&"view.sorter") && ids(&ms[4]).contains(&"view.zoomIn"));
}

/// With every tab the bar can show (the ribbon's and every contextual one), every registry
/// command placed anywhere but File is in some menu.
#[test]
fn every_placed_command_is_in_a_menu() {
    let specs = deckcraft_engine::command_specs();
    let mut tabs: Vec<&str> = crate::ribbon::TABS.to_vec();
    tabs.extend(CONTEXTUAL);
    let all: Vec<&str> = menus(&placed(specs), &tabs).iter().flat_map(ids).collect();
    let placed_count = specs.iter().filter(|c| c.menu.first().is_some_and(|f| *f != "File")).count();
    assert!(placed_count > 200, "{placed_count}");
    for c in specs.iter().filter(|c| c.menu.first().is_some_and(|f| *f != "File")) {
        assert!(all.contains(&c.id), "{} ({:?}) is in no menu", c.id, c.menu);
    }
    // The contextual Layout tab (tables) is the Table Layout menu.
    assert!(menus(&placed(specs), &["Layout"]).first().is_some_and(|m| m.tab == "Table Layout"));
}

/// The menus are built from any registry slice, so an empty one gives no menu (no panic).
#[test]
fn no_commands_no_menus() {
    assert!(menus(&[], &["Home", "Insert"]).is_empty());
    assert!(menus(&placed(deckcraft_engine::command_specs()), &[]).is_empty());
    assert_eq!(place(&[]), None);
}

/// A menu never runs a command that needs values without them: it opens the ribbon's dialog for
/// it, offers its choices, or asks in a form.
#[test]
fn commands_that_need_values_ask_for_them() {
    assert_eq!(action("format.bold", "{}"), Action::Run("format.bold", json!({})));
    let spec = |id: &str| deckcraft_engine::find_command(id).unwrap();
    assert_eq!(action("insert.picture", spec("insert.picture").params), Action::Run("app.insertPictureDialog", json!({})));
    assert_eq!(action("insert.table", spec("insert.table").params), Action::Run("app.dialog", json!({"id": "table"})));
    let Action::Choices(align) = action("format.align", spec("format.align").params) else { panic!("format.align") };
    assert_eq!(align.first(), Some(&("Left".to_string(), json!({"align": "left"}))));
    assert!(align.iter().any(|(l, p)| l == "Justify Low" && *p == json!({"align": "justifyLow"})));
    let Action::Choices(wrap) = action("format.wrap", spec("format.wrap").params) else { panic!("format.wrap") };
    assert_eq!(wrap, vec![("On".to_string(), json!({"on": true})), ("Off".to_string(), json!({"on": false}))]);
    let Action::Choices(variants) = action("design.variant", spec("design.variant").params) else { panic!("design.variant") };
    assert_eq!(variants.len(), 4);
    assert_eq!(action("format.size", spec("format.size").params), Action::Form);
    assert_eq!(action("shape.insert", spec("shape.insert").params), Action::Form);
    for c in deckcraft_engine::command_specs() {
        if deckcraft_engine::cmd::needs_arguments(c.params) {
            assert_ne!(action(c.id, c.params), Action::Run(c.id, json!({})), "{}", c.id);
        }
    }
    assert_eq!(typed("40"), json!(40));
    assert_eq!(typed("[1, 2]"), json!([1, 2]));
    assert_eq!(typed("Inter"), json!("Inter"));
    assert_eq!(typed("true"), json!(true));
}

fn harness_with(width: f32, height: f32, app: SlideApp) -> Harness<'static, SlideApp> {
    let mut h = Harness::builder().with_size(egui::vec2(width, height)).build_ui_state(
        |ui, app: &mut SlideApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(8);
    h
}

/// A toolbar window over a new deck with a selected text box holding text.
fn harness(width: f32, height: f32) -> Harness<'static, SlideApp> {
    let mut s = Session::with_new();
    s.execute("insert.textBox", &json!({"rect": [60, 60, 400, 80], "text": "Hello Nexus"})).unwrap();
    let mut app = SlideApp::new(s, Services::default());
    app.toolbar = true;
    harness_with(width, height, app)
}

/// Opens the menu bar's `name` menu (the title bar's Quick Access Toolbar has a Home button too:
/// the menu bar is the lower one).
fn open_menu(h: &mut Harness<'static, SlideApp>, name: &str) {
    h.query_all_by_label(name).max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y)).unwrap().click();
    h.run_steps(6);
}

fn state(h: &mut Harness<'static, SlideApp>, key: &str) -> serde_json::Value {
    crate::ribbon::fmt_state(h.state_mut()).get(key).cloned().unwrap_or_default()
}

/// The menu bar opens a tab's menu with its groups, a group lists its commands and a click runs
/// the command; the tool row's Bold toggles bold on the selected text box. No ribbon is drawn.
#[test]
fn menus_and_tools_work_in_the_window() {
    let mut h = harness(1280.0, 800.0);
    assert_eq!(state(&mut h, "bold"), json!(false));
    h.get_by_label("Bold").click();
    h.run_steps(6);
    assert_eq!(state(&mut h, "bold"), json!(true), "the tool row's Bold ran format.bold");
    assert_eq!(h.query_all_by_label("Transitions").count(), 1, "one Transitions: the menu, no ribbon tab");
    // Home › Font › Bold, through the menus. Menu items carry their shortcut.
    open_menu(&mut h, "Home");
    h.get_all_by_label("Font").next().unwrap().hover();
    h.run_steps(6);
    h.get_by_label_contains(&crate::ribbon::pretty_shortcut("Cmd+I"));
    h.get_by_label_contains(&crate::ribbon::pretty_shortcut("Cmd+B")).click();
    h.run_steps(6);
    assert_eq!(state(&mut h, "bold"), json!(false), "the menu's Bold ran format.bold again");
    // Home › Font › Font Size › 40: a value the ribbon picks from a list is a submenu of it.
    open_menu(&mut h, "Home");
    h.get_all_by_label("Font").next().unwrap().hover();
    h.run_steps(6);
    h.get_all_by_label("Font Size").next().unwrap().hover();
    h.run_steps(6);
    h.get_by_label("40").scroll_to_me();
    h.run_steps(6);
    h.get_by_label("40").click();
    h.run_steps(6);
    assert_eq!(state(&mut h, "size"), json!(40.0));
    // Home › Paragraph › Align › Right: a command with one value is a submenu of its choices.
    open_menu(&mut h, "Home");
    h.get_all_by_label("Paragraph").next().unwrap().hover();
    h.run_steps(6);
    h.get_all_by_label("Align").next().unwrap().hover();
    h.run_steps(6);
    h.get_by_label("Right").click();
    h.run_steps(6);
    assert_eq!(state(&mut h, "align"), json!("r"));
    // A contextual menu applies to the selected text box.
    assert!(h.query_by_label("Shape Format").is_some());
}

/// A command that needs values with no dialog asks in a small form, and runs with what is typed.
#[test]
fn the_parameter_form_runs_the_command() {
    let mut h = harness(1280.0, 800.0);
    open_form(&h.ctx, "format.size");
    h.run_steps(4);
    h.get_by_role(Role::TextInput).click();
    h.run_steps(2);
    h.get_by_role(Role::TextInput).type_text("40");
    h.run_steps(2);
    h.get_by_label("Run").click();
    h.run_steps(4);
    assert_eq!(state(&mut h, "size"), json!(40.0));
    assert!(h.query_by_label("Run").is_none(), "the form closed");
    // Junk keeps the form open with the error.
    open_form(&h.ctx, "format.size");
    h.run_steps(4);
    h.get_by_label("Run").click();
    h.run_steps(4);
    assert!(h.query_by_label("Run").is_some(), "a missing value keeps the form open");
}

/// A narrow window keeps the row on one line: the tools that don't fit move into More tools, and
/// the ones there still run.
#[test]
fn a_narrow_window_moves_tools_into_more() {
    let h = harness(1280.0, 800.0);
    assert!(h.query_by_label("More tools").is_none(), "everything fits at 1280");
    assert!(h.query_by_label("Find").is_some());
    let mut h = harness(600.0, 700.0);
    assert!(h.query_by_label("Find").is_none(), "Find gave way");
    assert!(h.query_by_label("Justify").is_none(), "the paragraph tools gave way");
    h.get_by_label("More tools").click();
    h.run_steps(6);
    h.get_by_label("Find…");
    h.get_by_label("Justify").click();
    h.run_steps(6);
    assert_eq!(state(&mut h, "align"), json!("just"), "Justify ran from More tools");
    assert_eq!(fitting(&[100.0, 100.0, 100.0], 300.0), 3);
    assert_eq!(fitting(&[100.0, 100.0, 100.0], 299.0), 2);
    assert_eq!(fitting(&[100.0, 100.0], 10.0), 0);
    assert_eq!(fitting(&[], 10.0), 0);
}

/// Off, the window keeps upstream's ribbon.
#[test]
fn the_ribbon_stays_without_the_flag() {
    let h = harness_with(1280.0, 800.0, SlideApp::new(Session::with_new(), Services::default()));
    // (Upstream's own ribbon names its split arrows "Paste options" now, so only the tool row's
    // overflow button tells the two apart.)
    assert!(h.query_by_label("More tools").is_none(), "no tool row");
}

/// `ui.accent` sets, reads and resets the accent; junk is refused; the palette follows.
#[test]
fn the_accent_command_sets_reads_and_resets() {
    let mut h = harness(1280.0, 800.0);
    let coral = theme::SECOND_COLOR;
    assert_eq!(h.state_mut().run("ui.accent", json!({})), Ok(json!({"accent": null})));
    assert_eq!(h.state_mut().run("ui.accent", json!({"value": "#f43f5e"})), Ok(json!({"accent": "#F43F5E"})));
    h.run_steps(2);
    assert_eq!(theme::accent(&h.ctx), Some(coral));
    assert_eq!(Tokens::get(&h.ctx), theme::with_accent(Tokens::light(), coral));
    assert_eq!(h.ctx.global_style().visuals.hyperlink_color, Tokens::get(&h.ctx).accent, "the egui style follows");
    assert_eq!(h.state_mut().run("ui.accent", json!({})), Ok(json!({"accent": "#F43F5E"})), "no value reads it");
    for junk in [json!("red"), json!("#12345"), json!(42), json!([1, 2]), json!({"r": 1})] {
        assert!(h.state_mut().run("ui.accent", json!({"value": junk})).is_err(), "{junk}");
    }
    assert_eq!(h.state().ui.accent.as_deref(), Some("#F43F5E"), "junk leaves it");
    assert_eq!(h.state_mut().run("ui.accent", json!({"value": ""})), Ok(json!({"accent": null})));
    h.run_steps(2);
    assert_eq!(theme::accent(&h.ctx), None);
    assert_eq!(Tokens::get(&h.ctx), Tokens::light());
    h.state_mut().run("ui.accent", json!({"value": "10B981"})).unwrap();
    assert_eq!(h.state_mut().run("ui.accent", json!({"value": null})), Ok(json!({"accent": null})));
    // Dark mode shades the same pick the other way.
    h.state_mut().run("ui.accent", json!({"value": "#F43F5E"})).unwrap();
    h.state_mut().run("view.dark", json!({"on": true})).unwrap();
    h.run_steps(3);
    assert_eq!(Tokens::get(&h.ctx), theme::with_accent(Tokens::dark(), coral));
}

/// The accent is saved with the UI preferences; a saved value that isn't a colour reads as
/// none, and no accent writes no key (an upstream `ui.json` stays the same).
#[test]
fn the_accent_round_trips_through_the_preferences() {
    let mut ui = crate::UiState { accent: Some("#F43F5E".into()), ..Default::default() };
    let text = serde_json::to_string(&ui).unwrap();
    let back: crate::UiState = serde_json::from_str(&text).unwrap();
    assert_eq!(back.accent.as_deref(), Some("#F43F5E"));
    ui.accent = None;
    assert!(!serde_json::to_string(&ui).unwrap().contains("accent"));
    for (saved, read) in [(r##"{"accent":"#abcdef"}"##, Some("#ABCDEF")), (r#"{"accent":"pink"}"#, None), (r#"{"accent":7}"#, None), ("{}", None)] {
        let s: crate::UiState = serde_json::from_str(saved).unwrap();
        assert_eq!(s.accent.as_deref(), read, "{saved}");
    }
    // Through the settings files, as the desktop app saves them.
    let dir = std::env::temp_dir().join(format!("nexus-slide-accent-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let ui = crate::UiState { accent: Some("#6366F1".into()), ..Default::default() };
    crate::settings::save(&dir, &ui, &Default::default());
    let (loaded, _) = crate::settings::load(&dir);
    assert_eq!(loaded.and_then(|u| u.accent).as_deref(), Some("#6366F1"));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Preferences offers the accent presets; picking one applies it at once and Cancel puts the
/// one the dialog opened with back.
#[test]
fn preferences_pick_an_accent_and_cancel_restores_it() {
    let mut h = harness(1280.0, 800.0);
    h.state_mut().run("app.preferences", json!({})).unwrap();
    h.run_steps(4);
    h.get_by_label("Coral").click();
    h.run_steps(4);
    assert_eq!(h.state().ui.accent.as_deref(), Some("#F43F5E"));
    assert_eq!(theme::accent(&h.ctx), Some(theme::SECOND_COLOR));
    h.get_by_label("Cancel").click();
    h.run_steps(4);
    assert_eq!(h.state().ui.accent, None, "Cancel restores the app colour");
    assert_eq!(theme::accent(&h.ctx), None);
}

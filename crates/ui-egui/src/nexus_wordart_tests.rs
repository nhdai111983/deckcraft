//! NexusOffice: the WordArt gallery ([`crate::nexus_wordart`]) in the WPS layout.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use deckcraft_engine::Session;
use deckcraft_engine::nexus_wordart::{DEFAULT_TEXT, STYLES, style_of};
use egui_kittest::kittest::Queryable;
use serde_json::json;

use crate::i18n::Language;
use crate::nexus_wordart::{COLUMNS, Mode, step, tile_id};
use crate::{Services, SlideApp};

type H = egui_kittest::Harness<'static, SlideApp>;

fn harness(lang: Language, setup: impl FnOnce(&mut SlideApp)) -> H {
    let mut session = Session::new();
    session.execute("file.new", &json!({})).unwrap();
    let mut a = SlideApp::new(session, Services::default());
    a.wps = true;
    a.ui.language = lang;
    setup(&mut a);
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1600.0, 900.0)).with_max_steps(64).build_ui_state(
        |ui, app: &mut SlideApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        a,
    );
    h.run_steps(16);
    h
}

/// Opens the ribbon button `label` on `tab` (the lowest one named so).
fn open(h: &mut H, tab: &str, label: &str) {
    h.state_mut().ui.tab = tab.into();
    h.run_steps(20);
    let node = h.query_all_by_label(label).max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y)).unwrap_or_else(|| panic!("no {label:?}"));
    node.click();
    h.run_steps(6);
}

fn wordart(h: &H) -> Vec<deckcraft_model::Shape> {
    h.state().session.active().unwrap().current_slide().unwrap().shapes.iter().filter(|s| s.name.starts_with("WordArt")).cloned().collect()
}

fn drawn(h: &H) -> Vec<String> {
    fn texts(s: &egui::Shape, out: &mut Vec<String>) {
        match s {
            egui::Shape::Text(t) => out.push(t.galley.text().to_string()),
            egui::Shape::Vec(v) => v.iter().for_each(|s| texts(s, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for s in &h.output().shapes {
        texts(&s.shape, &mut out);
    }
    out
}

/// Insert › WordArt opens a gallery of every style (a tile each, named in Vietnamese); clicking
/// a tile inserts a WordArt in that style with Vietnamese text, all of it selected for typing.
#[test]
fn the_gallery_opens_and_a_tile_inserts_that_style() {
    let mut h = harness(Language::Vi, |_| {});
    open(&mut h, "Insert", "WordArt");
    for (i, s) in STYLES.iter().enumerate() {
        assert!(h.query_by_label(s.name.1).is_some(), "tile {i} {:?}", s.name.1);
        assert!(h.query_by_label(s.name.0).is_none(), "English name {:?}", s.name.0);
    }
    // The old text menu is gone.
    assert!(!drawn(&h).iter().any(|t| t.starts_with("WordArt style")), "{:?}", drawn(&h));
    let pick = &STYLES[5];
    h.get_by_label(pick.name.1).click();
    h.run_steps(8);
    let art = wordart(&h);
    assert_eq!(art.len(), 1);
    assert_eq!(style_of(&art[0]).map(|s| s.id), Some(pick.id));
    assert_eq!(art[0].text.as_ref().unwrap().text(), DEFAULT_TEXT.1);
    let sel = h.state().session.active().unwrap().selection.clone();
    let t = sel.text.expect("editing the new WordArt");
    assert_eq!(t.shape, art[0].id);
    assert_eq!(t.ordered(), ((0, 0), (0, DEFAULT_TEXT.1.chars().count())), "all the text selected");
    // The gallery closed; typing replaces the text.
    assert!(h.query_by_label(pick.name.1).is_none(), "the gallery closes on a pick");
    h.event(egui::Event::Text("Xin chào".into()));
    h.run_steps(4);
    assert_eq!(wordart(&h)[0].text.as_ref().unwrap().text(), "Xin chào");
}

/// In Vietnamese the open gallery and a tile's tooltip draw no English.
#[test]
fn the_gallery_has_no_english_in_vietnamese() {
    let mut h = harness(Language::Vi, |_| {});
    open(&mut h, "Insert", "WordArt");
    let before = drawn(&h);
    assert!(before.iter().any(|t| t == "Kiểu WordArt"), "{before:?}");
    h.get_by_label(STYLES[3].name.1).hover();
    h.run_steps(30);
    let texts = drawn(&h);
    assert!(texts.iter().any(|t| t == STYLES[3].name.1), "tooltip in Vietnamese: {texts:?}");
    for s in STYLES {
        assert!(!texts.iter().any(|t| t == s.name.0), "{:?} drawn in English", s.name.0);
        assert_ne!(s.name.0, s.name.1);
    }
    for t in &texts {
        assert!(!["WordArt Styles", "Your text here"].contains(&t.as_str()), "{t:?}");
        assert!(!t.starts_with("WordArt style"), "{t:?}");
    }
    // The English interface keeps English names.
    let mut en = harness(Language::En, |_| {});
    open(&mut en, "Insert", "WordArt");
    assert!(en.query_by_label(STYLES[0].name.0).is_some());
    en.get_by_label(STYLES[0].name.0).click();
    en.run_steps(6);
    assert_eq!(wordart(&en)[0].text.as_ref().unwrap().text(), DEFAULT_TEXT.0);
}

/// Arrow keys move between tiles (rows of four), Enter picks.
#[test]
fn arrow_keys_move_between_tiles_and_enter_picks() {
    assert_eq!(step(0, 12, false, true, false, false), Some(1));
    assert_eq!(step(0, 12, true, false, false, false), Some(11));
    assert_eq!(step(1, 12, false, false, false, true), Some(1 + COLUMNS));
    assert_eq!(step(1, 12, false, false, true, false), None);
    assert_eq!(step(11, 12, false, false, false, true), None);
    assert_eq!(step(11, 12, false, true, false, false), Some(0));
    assert_eq!(step(0, 0, true, false, false, false), None);
    assert_eq!(step(99, 12, true, false, false, false), None);
    let mut h = harness(Language::Vi, |_| {});
    open(&mut h, "Insert", "WordArt");
    h.ctx.memory_mut(|m| m.request_focus(tile_id(Mode::Insert, 0)));
    h.run_steps(2);
    for key in [egui::Key::ArrowRight, egui::Key::ArrowDown] {
        h.key_press(key);
        h.run_steps(2);
    }
    assert!(h.ctx.memory(|m| m.has_focus(tile_id(Mode::Insert, 1 + COLUMNS))), "focus moved right then down");
    h.key_press(egui::Key::Enter);
    h.run_steps(6);
    assert_eq!(wordart(&h).first().and_then(style_of).map(|s| s.id), Some(STYLES[1 + COLUMNS].id));
}

/// Shape Format › WordArt Styles restyles the selected WordArt with the same tiles, marking its
/// current style.
#[test]
fn shape_format_restyles_with_the_same_tiles() {
    let mut h = harness(Language::Vi, |a| {
        a.run("insert.wordArt", json!({"style": "amber"})).unwrap();
        a.run("text.exit", json!({})).unwrap();
        let id = a.session.active().unwrap().doc.slides[0].shapes.iter().find(|s| s.name.starts_with("WordArt")).unwrap().id;
        a.run("edit.select", json!({"ids": [id.0]})).unwrap();
    });
    assert!(crate::ribbon::contextual(h.state()).contains(&"Shape Format"));
    open(&mut h, "Shape Format", "Kiểu WordArt");
    let pick = &STYLES[8];
    h.get_by_label(pick.name.1).click();
    h.run_steps(6);
    assert_eq!(wordart(&h).first().and_then(style_of).map(|s| s.id), Some(pick.id));
}

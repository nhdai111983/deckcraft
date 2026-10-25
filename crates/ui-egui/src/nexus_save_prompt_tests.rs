//! NexusOffice: "Lưu thay đổi?" as a tab of the NexusOffice window ([`crate::nexus_save_prompt`]),
//! and the language of a presentation started in the Vietnamese interface.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use serde_json::json;

use crate::dialogs::Dialog;
use crate::i18n::Language;
use crate::nexus_save_prompt::{BUTTON_SIZE, PADDING, WIDTH};
use crate::{Services, SlideApp};
use deckcraft_engine::Session;

type H = egui_kittest::Harness<'static, SlideApp>;

const NAME: &str = "Báo cáo.pptx";

/// An embedded Vietnamese window with `NAME` (on disk) open and changed, asking "Lưu thay đổi?".
fn asking(tag: &str, embedded: bool) -> (H, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("nexus-slide-save-prompt-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(NAME);
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s.execute("file.saveAs", &json!({"path": path.to_string_lossy(), "format": "pptx"})).unwrap();
    let mut session = Session::new();
    session.execute("file.open", &json!({"path": path.to_string_lossy()})).unwrap();
    session.execute("slide.new", &json!({})).unwrap();
    let mut a = SlideApp::new(session, Services::default());
    a.wps = true;
    a.embedded = embedded;
    a.ui.language = Language::Vi;
    a.dialog = Some(Dialog::new("close"));
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1280.0, 800.0)).with_max_steps(64).build_ui_state(
        |ui, app: &mut SlideApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        a,
    );
    h.run_steps(16);
    (h, path)
}

fn title(h: &H) -> egui::Rect {
    h.get_by_label("Lưu thay đổi?").rect()
}

/// The prompt's button `label` (the tool row above may have a button of the same name).
fn button(h: &H, label: &str) -> egui::Rect {
    let top = title(h).max.y;
    let found: Vec<_> = h.query_all_by_role_and_label(Role::Button, label).filter(|b| b.rect().min.y > top).collect();
    assert_eq!(found.len(), 1, "one {label} in the prompt");
    found[0].rect()
}

fn click(h: &mut H, label: &str) {
    let top = title(h).max.y;
    h.query_all_by_role_and_label(Role::Button, label).find(|b| b.rect().min.y > top).unwrap().click();
    h.run_steps(4);
}

fn open_named(h: &H) -> bool {
    h.state().session.documents().iter().any(|d| d.title() == NAME || d.title() == "Báo cáo")
}

#[test]
fn embedded_prompt_reads_like_every_nexus_app() {
    let (h, path) = asking("look", true);
    let name = h.state().session.active().unwrap().title();
    title(&h);
    h.get_by_label(&format!("Lưu thay đổi của “{name}” trước khi đóng?"));
    let (save, dont, cancel) = (button(&h, "Lưu"), button(&h, "Không lưu"), button(&h, "Hủy"));
    assert!(save.max.x <= dont.min.x && dont.max.x <= cancel.min.x, "Lưu, Không lưu, Hủy: {save:?} {dont:?} {cancel:?}");
    assert!((save.center().y - cancel.center().y).abs() < 0.5, "one row");
    for b in [save, dont, cancel] {
        assert!(b.width() >= BUTTON_SIZE.x - 0.5 && b.height() >= BUTTON_SIZE.y - 0.5, "{b:?}");
    }
    let inner = WIDTH - 2.0 * f32::from(PADDING);
    assert!((cancel.max.x - (title(&h).min.x + inner)).abs() < 1.5, "right-aligned: {cancel:?}");
    let top = title(&h).max.y;
    assert!(h.query_all_by_role_and_label(Role::Button, "Lưu").any(|b| b.rect().min.y > top && b.is_focused()), "the keyboard starts on Lưu");
    assert!(h.query_by_label("Bạn có muốn lưu các thay đổi bạn đã thực hiện với “Báo cáo.pptx” không?").is_none());
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn embedded_prompt_enter_saves_and_escape_cancels() {
    for (key, saves) in [(egui::Key::Enter, true), (egui::Key::Escape, false)] {
        let (mut h, path) = asking(if saves { "enter" } else { "escape" }, true);
        let before = std::fs::read(&path).unwrap();
        h.key_press(key);
        h.run_steps(4);
        assert!(h.state().dialog.is_none(), "{key:?}: answered");
        assert_eq!(std::fs::read(&path).unwrap() != before, saves, "{key:?}: file written");
        assert_eq!(open_named(&h), !saves, "{key:?}: the presentation closes only after Lưu");
        if !saves {
            assert!(h.state().session.active().is_some_and(|d| d.is_dirty()), "Hủy keeps the changes");
        }
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}

#[test]
fn embedded_prompt_dont_save_closes_without_writing() {
    let (mut h, path) = asking("dont", true);
    let before = std::fs::read(&path).unwrap();
    click(&mut h, "Không lưu");
    assert!(h.state().dialog.is_none());
    assert!(!open_named(&h), "closed");
    assert_eq!(std::fs::read(&path).unwrap(), before, "nothing written");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

/// Stand-alone keeps upstream's prompt (its own body text and button row).
#[test]
fn stand_alone_prompt_is_upstreams() {
    let (h, path) = asking("standalone", false);
    assert!(h.query_by_label_contains("trước khi đóng").is_none());
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

/// A presentation started in the Vietnamese interface is in Vietnamese (status bar, Review ›
/// Language, the PPTX it saves); one started in English stays upstream's US English; an opened
/// file keeps its own language.
#[test]
fn new_presentations_in_the_vietnamese_interface_are_vietnamese() {
    let mut a = SlideApp::new(Session::new(), Services::default());
    a.ui.language = Language::Vi;
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1280.0, 800.0)).with_max_steps(64).build_ui_state(
        |ui, app: &mut SlideApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        a,
    );
    h.run_steps(8);
    h.state_mut().run("file.new", json!({})).unwrap();
    h.run_steps(8);
    assert_eq!(crate::dialogs::selected_language(h.state()).0, "vi-VN");
    // The status bar's name for it.
    let shown = h.state().ui.language.tr(&crate::squiggles::lang_name("vi-VN")).into_owned();
    assert_eq!(shown, "Tiếng Việt", "not “Tiếng Anh (Mỹ)”");

    let dir = std::env::temp_dir().join(format!("nexus-slide-lang-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("Mới.pptx");
    h.state_mut().run("file.saveAs", json!({"path": path.to_string_lossy(), "format": "pptx"})).unwrap();
    // An opened file without a language keeps upstream's reading (US English).
    let english = dir.join("English.pptx");
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s.execute("file.saveAs", &json!({"path": english.to_string_lossy(), "format": "pptx"})).unwrap();
    h.state_mut().run("file.open", json!({"path": english.to_string_lossy()})).unwrap();
    h.run_steps(4);
    assert_eq!(crate::dialogs::selected_language(h.state()).0, "en-US", "an opened file keeps its language");
    // The saved Vietnamese deck reads back as Vietnamese.
    h.state_mut().run("file.open", json!({"path": path.to_string_lossy()})).unwrap();
    h.run_steps(4);
    assert_eq!(h.state().session.active().and_then(|d| d.doc.default_lang().map(str::to_string)).as_deref(), Some("vi-VN"));
    let _ = std::fs::remove_dir_all(&dir);
}

/// English interface: new presentations stay upstream's (no language of their own).
#[test]
fn new_presentations_in_english_stay_upstreams() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    assert_eq!(s.active().unwrap().doc.default_lang(), None);
    s.new_language = Some("vi-VN".into());
    s.execute("file.new", &json!({"template": "Briefing"})).unwrap();
    assert_eq!(s.active().unwrap().doc.default_lang(), Some("vi-VN"), "templates too");
}

//! NexusOffice: offscreen screenshots of the WordArt gallery for review (headless, wgpu), in
//! Vietnamese, WPS layout:
//!
//! ```sh
//! cargo run -p deckcraft-ui-egui --example nexus_wordart_shots -- <out-dir>
//! ```
//!
//! Writes `wordart-gallery.png` (Insert › WordArt open), `wordart-hover.png` (a tile hovered,
//! its tooltip), `wordart-inserted.png` (a title slide after three WordArt inserts, the last one
//! being edited), `wordart-title-only.png` (one WordArt on an empty Title Only slide),
//! `wordart-restyle.png` (Shape Format › WordArt Styles on a selected WordArt) and
//! `wordart-dark.png` (the gallery in dark mode).

use deckcraft_engine::Session;
use deckcraft_ui_egui::i18n::Language;
use deckcraft_ui_egui::{Services, SlideApp};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::json;

fn window(setup: impl FnOnce(&mut SlideApp)) -> Harness<'static, SlideApp> {
    let mut session = Session::new();
    // A deck started from the Vietnamese interface is in Vietnamese (as in the app).
    session.new_language = Some("vi-VN".into());
    if let Err(e) = session.execute("file.new", &json!({})) {
        eprintln!("new deck: {e}");
    }
    let mut app = SlideApp::new(session, Services::default());
    app.wps = true;
    app.ui.language = Language::Vi;
    setup(&mut app);
    let mut h = Harness::builder().with_size(egui::vec2(1680.0, 900.0)).with_pixels_per_point(1.0).wgpu().build_ui_state(
        |ui, app: &mut SlideApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(16);
    h
}

fn shot(h: &mut Harness<'_, SlideApp>, dir: &std::path::Path, name: &str) {
    h.run_steps(6);
    match h.render() {
        Ok(img) => {
            let path = dir.join(name);
            match img.save(&path) {
                Ok(()) => println!("{}", path.display()),
                Err(e) => eprintln!("{}: {e}", path.display()),
            }
        }
        Err(e) => eprintln!("{name}: render failed: {e}"),
    }
}

fn open(h: &mut Harness<'_, SlideApp>, tab: &str, label: &str) {
    h.state_mut().ui.tab = tab.into();
    h.run_steps(20);
    match h.query_all_by_label(label).max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y)) {
        Some(n) => n.click(),
        None => eprintln!("no widget named {label:?}"),
    }
    h.run_steps(8);
}

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    let styles = deckcraft_engine::nexus_wordart::STYLES;

    let mut h = window(|_| {});
    open(&mut h, "Insert", "WordArt");
    shot(&mut h, &dir, "wordart-gallery.png");
    if let Some(s) = styles.get(6)
        && let Some(n) = h.query_by_label(s.name.1)
    {
        n.hover();
    }
    h.run_steps(40);
    shot(&mut h, &dir, "wordart-hover.png");

    let mut h = window(|a| {
        for style in ["sunset", "skyNeon", "dusk"] {
            let _ = a.run("insert.wordArt", json!({"style": style, "text": "Nhập chữ của bạn vào đây"}));
        }
    });
    shot(&mut h, &dir, "wordart-inserted.png");

    let mut h = window(|a| {
        let _ = a.run("slide.layout", json!({"layout": "Title Only"}));
        let _ = a.run("insert.wordArt", json!({"style": "brand", "text": "Nhập chữ của bạn vào đây"}));
    });
    shot(&mut h, &dir, "wordart-title-only.png");

    let mut h = window(|a| {
        let _ = a.run("insert.wordArt", json!({"style": "retro", "text": "Chào mừng"}));
        let _ = a.run("text.exit", json!({}));
        let id = a
            .session
            .active()
            .and_then(|d| d.doc.slides.first().and_then(|s| s.shapes.iter().find(|s| s.name.starts_with("WordArt")).map(|s| s.id.0)));
        let _ = a.run("edit.select", json!({"ids": [id]}));
    });
    open(&mut h, "Shape Format", "Kiểu WordArt");
    shot(&mut h, &dir, "wordart-restyle.png");

    let mut h = window(|a| a.ui.brightness = deckcraft_ui_egui::theme::Brightness::Dark);
    open(&mut h, "Insert", "WordArt");
    shot(&mut h, &dir, "wordart-dark.png");
}

//! Offscreen screenshots of the Nexus Slide window for review (headless, wgpu), with the
//! NexusOffice layout (menu bar and one tool row) on:
//!
//! ```sh
//! cargo run -p deckcraft-ui-egui --example shots -- <out-dir> [prefix]
//! ```
//!
//! Writes `<prefix>light.png`, `dark.png`, `coral.png` (user accent #F43F5E), `menu.png` (Home ›
//! Font open), `options.png` (Preferences with the accent row), `1024.png` and `more.png` (the
//! narrow window's More tools menu). The prefix defaults to `dc-`.

use deckcraft_engine::Session;
use deckcraft_ui_egui::{Services, SlideApp};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

fn window(width: f32, height: f32, setup: impl FnOnce(&mut SlideApp)) -> Harness<'static, SlideApp> {
    let mut session = Session::new();
    if let Err(e) = deckcraft_engine::sample::open_sample(&mut session) {
        eprintln!("sample deck: {e}");
    }
    let mut app = SlideApp::new(session, Services::default());
    app.toolbar = true;
    setup(&mut app);
    let mut h = Harness::builder().with_size(egui::vec2(width, height)).with_pixels_per_point(1.0).wgpu().build_ui_state(
        |ui, app: &mut SlideApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(10);
    h
}

fn shot(h: &mut Harness<'_, SlideApp>, dir: &std::path::Path, name: &str) {
    h.run_steps(6);
    match h.render() {
        Ok(img) => {
            let path = dir.join(name);
            match img.save(&path) {
                Ok(()) => println!("{} ({}x{})", path.display(), img.width(), img.height()),
                Err(e) => eprintln!("{}: {e}", path.display()),
            }
        }
        Err(e) => eprintln!("{name}: render failed: {e}"),
    }
}

/// Clicks (or hovers) the widget named `label`, if there is one (the lowest one when several
/// are: the title bar's Quick Access Toolbar also has a Home button above the menu bar).
fn act(h: &mut Harness<'_, SlideApp>, label: &str, click: bool) {
    match h.query_all_by_label(label).max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y)) {
        Some(n) if click => n.click(),
        Some(n) => n.hover(),
        None => eprintln!("no widget named {label:?}"),
    }
    h.run_steps(4);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = std::path::PathBuf::from(args.next().unwrap_or_else(|| ".".into()));
    let prefix = args.next().unwrap_or_else(|| "dc-".into());
    let name = |n: &str| format!("{prefix}{n}.png");

    let mut h = window(1280.0, 800.0, |_| {});
    shot(&mut h, &dir, &name("light"));
    act(&mut h, "Home", true);
    act(&mut h, "Font ⏵", false);
    shot(&mut h, &dir, &name("menu"));

    let mut h = window(1280.0, 800.0, |a| a.ui.brightness = deckcraft_ui_egui::theme::Brightness::Dark);
    shot(&mut h, &dir, &name("dark"));

    let mut h = window(1280.0, 800.0, |a| a.ui.accent = Some("#F43F5E".into()));
    shot(&mut h, &dir, &name("coral"));
    if let Err(e) = h.state_mut().run("app.preferences", serde_json::json!({})) {
        eprintln!("app.preferences: {e}");
    }
    shot(&mut h, &dir, &name("options"));

    let mut h = window(1024.0, 700.0, |_| {});
    shot(&mut h, &dir, &name("1024"));
    act(&mut h, "More tools", true);
    shot(&mut h, &dir, &name("more"));
}

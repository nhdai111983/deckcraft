//! Offscreen screenshots of the Nexus Slide window for review (headless, wgpu): the WPS-style
//! layout (the Nexus apps' default) in Vietnamese, and the menu-bar layout:
//!
//! ```sh
//! cargo run -p deckcraft-ui-egui --example shots -- <out-dir> [prefix]
//! ```
//!
//! Writes `<prefix>wps-home-1440.png`, `wps-insert.png`, `wps-design.png`, `wps-transitions.png`,
//! `wps-animations.png`, `wps-view.png`, `wps-file.png` (File), `wps-print.png`, `wps-1024.png`,
//! `wps-overflow.png` (the "…" panel), `wps-more-tabs.png` (»), `wps-dark.png`, `wps-coral.png`
//! (user accent #F43F5E), `wps-en.png` (English), and the menu-bar layout's `light.png` and
//! `menu.png`. The prefix defaults to `dc-`.

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

/// Clicks the widget with egui id `id` (the WPS layout's "…", », File).
fn click_id(h: &mut Harness<'_, SlideApp>, id: egui::Id) {
    let Some(r) = h.ctx.read_response(id).map(|r| r.rect) else {
        eprintln!("{id:?} is not drawn");
        return;
    };
    h.input_mut().events.push(egui::Event::PointerMoved(r.center()));
    h.run_steps(1);
    for pressed in [true, false] {
        h.input_mut().events.push(egui::Event::PointerButton {
            pos: r.center(),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        h.run_steps(1);
    }
    h.run_steps(4);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = std::path::PathBuf::from(args.next().unwrap_or_else(|| ".".into()));
    let prefix = args.next().unwrap_or_else(|| "dc-".into());
    let name = |n: &str| format!("{prefix}{n}.png");
    use deckcraft_ui_egui::i18n::Language;
    use deckcraft_ui_egui::nexus_wps;

    // The WPS layout (the Nexus apps' default), in Vietnamese.
    let wps = |a: &mut SlideApp| {
        a.wps = true;
        a.ui.language = Language::Vi;
    };
    let mut h = window(1440.0, 900.0, wps);
    shot(&mut h, &dir, &name("wps-home-1440"));
    for (tab, file) in [
        ("Insert", "wps-insert"),
        ("Design", "wps-design"),
        ("Transitions", "wps-transitions"),
        ("Animations", "wps-animations"),
        ("View", "wps-view"),
    ] {
        h.state_mut().ui.tab = tab.into();
        h.run_steps(30);
        shot(&mut h, &dir, &name(file));
    }
    h.state_mut().ui.tab = "Home".into();
    click_id(&mut h, nexus_wps::file_button_id());
    shot(&mut h, &dir, &name("wps-file"));
    if let Some(b) = h.state_mut().backstage.as_mut() {
        b.go(deckcraft_ui_egui::backstage::Page::Print);
    }
    shot(&mut h, &dir, &name("wps-print"));

    let mut h = window(1024.0, 700.0, wps);
    shot(&mut h, &dir, &name("wps-1024"));
    click_id(&mut h, nexus_wps::overflow_button_id());
    shot(&mut h, &dir, &name("wps-overflow"));
    click_id(&mut h, nexus_wps::more_tabs_id());
    shot(&mut h, &dir, &name("wps-more-tabs"));

    let mut h = window(1440.0, 900.0, |a| {
        wps(a);
        a.ui.brightness = deckcraft_ui_egui::theme::Brightness::Dark;
    });
    shot(&mut h, &dir, &name("wps-dark"));
    let mut h = window(1440.0, 900.0, |a| {
        wps(a);
        a.ui.accent = Some("#F43F5E".into());
    });
    shot(&mut h, &dir, &name("wps-coral"));
    let mut h = window(1440.0, 900.0, |a| a.wps = true);
    shot(&mut h, &dir, &name("wps-en"));

    // The menu bar layout (`toolbar`), as before.
    let mut h = window(1280.0, 800.0, |a| a.toolbar = true);
    shot(&mut h, &dir, &name("light"));
    act(&mut h, "Home", true);
    act(&mut h, "Font ⏵", false);
    shot(&mut h, &dir, &name("menu"));
}

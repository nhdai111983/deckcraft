//! The NexusOffice WPS-style layout ([`crate::nexus_wps`]): the top row, the strip, embedding.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use egui::{Color32, Id};
use egui_kittest::kittest::Queryable;
use serde_json::json;

use super::*;
use crate::i18n::Language;
use crate::{Services, SlideApp};
use deckcraft_engine::Session;

type H = egui_kittest::Harness<'static, SlideApp>;

fn png() -> Vec<u8> {
    // 1×1 RGB PNG.
    const HEX: &str =
        "89504e470d0a1a0a0000000d4948445200000001000000010802000000907753de0000000c49444154789c63f8cfc0000003010100c9fe92ef0000000049454e44ae426082";
    (0..HEX.len()).step_by(2).map(|i| u8::from_str_radix(&HEX[i..i + 2], 16).expect("hex")).collect()
}

fn wav() -> Vec<u8> {
    let data = [0u8; 8];
    let mut b = b"RIFF".to_vec();
    b.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&8000u32.to_le_bytes());
    b.extend_from_slice(&16000u32.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(data.len() as u32).to_le_bytes());
    b.extend_from_slice(&data);
    b
}

/// A presentation in the WPS layout, embedded as a tab of the NexusOffice window.
fn harness(size: (f32, f32), lang: Language, sample: bool, setup: impl FnOnce(&mut SlideApp)) -> H {
    let mut session = Session::new();
    session.untitled_prefix = lang.untitled_prefix().to_string();
    if sample {
        deckcraft_engine::sample::open_sample(&mut session).unwrap();
    } else {
        session.execute("file.new", &json!({})).unwrap();
    }
    let mut a = SlideApp::new(session, Services::default());
    a.wps = true;
    a.embedded = true;
    a.ui.language = lang;
    setup(&mut a);
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(size.0, size.1)).with_max_steps(64).build_ui_state(
        |ui, app: &mut SlideApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        a,
    );
    // Fonts install on the first frames; stacks and the "…" fold settle over a few more.
    h.run_steps(16);
    h
}

fn click_id(h: &mut H, id: Id) {
    let r = h.ctx.read_response(id).map(|r| r.rect);
    let r = r.unwrap_or_else(|| panic!("{id:?} is not drawn"));
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
    h.run_steps(6);
}

fn slides(app: &SlideApp) -> usize {
    app.session.active().map_or(0, |d| d.doc.slides.len())
}

/// With the WPS layout there is no title bar (upstream's holds the light/dark switch and
/// Present); File (Tệp) opens the File tab, in Vietnamese and in its light look.
#[test]
fn no_title_bar_and_file_opens_the_file_tab() {
    let up = harness((1440.0, 900.0), Language::En, false, |a| a.wps = false);
    assert!(up.query_by_label("Light/Dark appearance").is_some(), "upstream's title bar switch is found when it is there");

    let mut h = harness((1440.0, 900.0), Language::Vi, false, |_| {});
    assert!(h.query_by_label("Light/Dark appearance").is_none() && h.query_by_label("Giao diện sáng/tối").is_none(), "no title bar");
    h.get_by_label("Tệp").click();
    h.run_steps(4);
    assert!(h.state().backstage.is_some(), "File opens the File tab");
    // Its pages speak Vietnamese and the bordered Back button returns.
    h.get_by_label("Thông tin");
    h.get_by_label("Lưu thành");
    h.get_by_label("Quay lại").click();
    h.run_steps(4);
    assert!(h.state().backstage.is_none(), "Back closes it");
}

/// The Quick Access Toolbar keeps working in the top row: Undo and Redo, and Save writes the
/// presentation; the save state then says so, and its menu holds Save As.
#[test]
fn quick_access_save_undo_and_redo_work() {
    let path = std::env::temp_dir().join(format!("nexus-slide-wps-qat-{}.deckcraft", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let mut h = harness((1440.0, 900.0), Language::En, false, |a| {
        a.run("slide.new", json!({})).unwrap();
    });
    assert_eq!(slides(h.state()), 2);
    h.get_by_label(&crate::ribbon::tip("edit.undo", "Undo")).click();
    h.run_steps(4);
    assert_eq!(slides(h.state()), 1, "Undo undid the new slide");
    h.get_by_label(&crate::ribbon::tip("edit.redo", "Redo")).click();
    h.run_steps(4);
    assert_eq!(slides(h.state()), 2, "Redo redid it");
    assert!(h.query_by_label("Not saved").is_some());
    h.state_mut().session.active_mut().unwrap().path = Some(path.to_string_lossy().into_owned());
    h.run_steps(2);
    h.get_by_label(&crate::ribbon::tip("file.save", "Save")).click();
    h.run_steps(6);
    assert!(path.exists(), "Save wrote the file");
    assert!(!h.state().session.active().unwrap().is_dirty());
    assert!(h.query_by_label("Saved").is_some(), "the save state follows");
    click_id(&mut h, save_state_id());
    assert!(h.query_by_label("Save As…").is_some(), "Save As is in the save state's menu");
    let _ = std::fs::remove_file(&path);
}

/// Each contextual tab's selection, so every tab the layout can show is drawn.
fn scenarios() -> Vec<(&'static str, Box<dyn Fn(&mut SlideApp)>)> {
    let b64 = |b: Vec<u8>| deckcraft_engine::cmd::base64_encode(&b);
    vec![
        ("plain", Box::new(|_: &mut SlideApp| {})),
        (
            "shape",
            Box::new(|a: &mut SlideApp| {
                a.run("shape.insert", json!({"preset": "rect", "rect": [100, 100, 300, 200]})).unwrap();
            }),
        ),
        (
            "picture",
            Box::new(move |a: &mut SlideApp| {
                a.run("insert.picture", json!({"name": "p.png", "data": b64(png())})).unwrap();
            }),
        ),
        (
            "table",
            Box::new(|a: &mut SlideApp| {
                a.run("insert.table", json!({"rows": 2, "cols": 2, "rect": [80, 80, 600, 160]})).unwrap();
            }),
        ),
        (
            "chart",
            Box::new(|a: &mut SlideApp| {
                a.run("insert.chart", json!({})).unwrap();
            }),
        ),
        (
            "smartart",
            Box::new(|a: &mut SlideApp| {
                a.run("insert.smartArt", json!({"kind": "list", "items": ["Một", "Hai"]})).unwrap();
            }),
        ),
        (
            "audio",
            Box::new(move |a: &mut SlideApp| {
                a.run("insert.audio", json!({"name": "a.wav", "data": b64(wav())})).unwrap();
            }),
        ),
        (
            "slide master",
            Box::new(|a: &mut SlideApp| {
                a.run("view.slideMaster", json!({})).unwrap();
            }),
        ),
        (
            "handout master",
            Box::new(|a: &mut SlideApp| {
                a.run("view.handoutMaster", json!({})).unwrap();
            }),
        ),
        (
            "notes master",
            Box::new(|a: &mut SlideApp| {
                a.run("view.notesMaster", json!({})).unwrap();
            }),
        ),
    ]
}

/// Every tab, the contextual ones included, draws its strip at both window sizes without
/// panicking, and no group is taller than the strip's two rows.
#[test]
fn every_tab_fits_the_strip_at_both_sizes() {
    let mut seen = std::collections::BTreeSet::new();
    for size in [(1440.0, 900.0), (1024.0, 700.0)] {
        for (name, setup) in scenarios() {
            let mut h = harness(size, Language::Vi, true, |a| setup(a));
            let tabs = shown_tabs(h.state());
            for tab in tabs {
                seen.insert(tab);
                h.state_mut().ui.tab = tab.to_string();
                // The first look at a tab settles its stacks' rows over a few frames.
                h.run_steps(30);
                TALL_GROUPS.with(|g| g.borrow_mut().clear());
                h.run_steps(1);
                let tall: Vec<String> = TALL_GROUPS.with(|g| g.take());
                assert!(tall.is_empty(), "{tab} ({name}) at {size:?}: {tall:?}");
                assert_eq!(h.state().ui.tab, tab, "{name}");
            }
        }
    }
    for tab in [
        "Home",
        "Shape Format",
        "Picture Format",
        "Table Design",
        "Layout",
        "Chart Design",
        "SmartArt Design",
        "Playback",
        "Slide Master",
        "Handout Master",
        "Notes Master",
    ] {
        assert!(seen.contains(tab), "{tab} was never shown: {seen:?}");
    }
}

/// At 1024 pt Home's last groups fold into "…": its panel shows them and their commands run.
#[test]
fn narrow_windows_fold_groups_into_the_overflow_panel() {
    let wide = harness((2400.0, 900.0), Language::En, false, |_| {});
    assert!(wide.ctx.read_response(overflow_button_id()).is_none(), "everything fits a wide window");

    let mut h = harness((1024.0, 700.0), Language::En, false, |_| {});
    assert!(h.query_by_label("Design Ideas").is_none(), "Designer folded away");
    click_id(&mut h, overflow_button_id());
    assert!(overflow_open(&h.ctx));
    h.get_by_label("Design Ideas").click();
    h.run_steps(4);
    assert_eq!(h.state().ui.pane.as_deref(), Some("designer"), "the folded group's command ran");
    h.key_press(egui::Key::Escape);
    h.run_steps(4);
    assert!(!overflow_open(&h.ctx), "Escape closes the panel");
}

/// Search, Comments and Share stay in the top row; tabs switch, and the » holds the rest.
#[test]
fn search_comments_share_and_tabs_are_reachable() {
    let mut h = harness((1440.0, 900.0), Language::Vi, false, |_| {});
    click_id(&mut h, search_id());
    assert!(h.state().palette.is_some(), "the box opens the command search");
    h.state_mut().palette = None;
    h.run_steps(2);
    h.get_by_label("Chuyển tiếp").click();
    h.run_steps(4);
    assert_eq!(h.state().ui.tab, "Transitions");
    h.get_by_label("Nhận xét").click();
    h.run_steps(4);
    assert_eq!(h.state().ui.pane.as_deref(), Some("comments"));
    click_id(&mut h, share_id());
    assert!(h.query_by_label("Xuất PDF…").is_some(), "Share offers the copies to share");
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    // A narrow window keeps every tab reachable through ».
    let mut n = harness((1024.0, 700.0), Language::Vi, false, |_| {});
    assert!(n.query_by_label("Xem").is_none(), "View doesn't fit");
    click_id(&mut n, more_tabs_id());
    n.get_by_label("Xem").click();
    n.run_steps(4);
    assert_eq!(n.state().ui.tab, "View");
}

/// The viewport commands a frame sent.
fn commands(h: &H) -> Vec<egui::ViewportCommand> {
    h.output().viewport_output.get(&egui::ViewportId::ROOT).map(|v| v.commands.clone()).unwrap_or_default()
}

/// Embedded, the app leaves the host window alone: a drag on its top row doesn't move it, and
/// the window's close button is the host's to answer (no "Save changes?" of its own, no
/// CancelClose); stand-alone it does both.
#[test]
fn embedded_leaves_the_window_alone() {
    for embedded in [false, true] {
        let mut h = harness((1024.0, 700.0), Language::En, false, |a| {
            a.embedded = embedded;
            a.run("slide.new", json!({})).unwrap();
        });
        // A press and a drag on the empty part of the top row.
        let at = egui::pos2(1024.0 * 0.66, 18.0);
        h.input_mut().events.push(egui::Event::PointerMoved(at));
        h.input_mut().events.push(egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        });
        h.step();
        h.input_mut().events.push(egui::Event::PointerMoved(at + egui::vec2(20.0, 0.0)));
        h.step();
        let dragged = commands(&h).iter().any(|c| matches!(c, egui::ViewportCommand::StartDrag));
        h.input_mut().events.push(egui::Event::PointerButton {
            pos: at + egui::vec2(20.0, 0.0),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        });
        h.run_steps(2);
        assert_eq!(dragged, !embedded, "embedded {embedded}: the top row moves only its own window");
        h.input_mut().viewports.entry(egui::ViewportId::ROOT).or_default().events.push(egui::ViewportEvent::Close);
        h.step();
        let cancelled = commands(&h).iter().any(|c| matches!(c, egui::ViewportCommand::CancelClose));
        h.run_steps(2);
        assert_eq!(cancelled, !embedded, "embedded {embedded}: only the stand-alone app answers the window's close");
        assert_eq!(h.state().dialog.as_ref().is_some_and(|d| d.id == "quit"), !embedded);
    }
}

/// File's label keeps 4.5:1 on its faint accent fill, in both themes and with a chosen accent; so
/// do Share's text on the accent and "Saved" on the row.
#[test]
fn file_label_is_readable_on_its_fill() {
    let accents = [
        None,
        Some(Color32::from_rgb(0x4F, 0x46, 0xE5)),
        Some(Color32::from_rgb(0xE1, 0x1D, 0x48)),
        Some(Color32::from_rgb(0xFA, 0xCC, 0x15)),
        Some(Color32::from_rgb(0x10, 0xB9, 0x81)),
    ];
    for base in [Tokens::light(), Tokens::dark()] {
        for accent in accents {
            let t = accent.map_or(base, |c| theme::with_accent(base, c));
            for hovered in [false, true] {
                let fill = file_fill(&t, hovered);
                assert_ne!(fill, t.chrome, "File keeps its tint");
                let ratio = theme::contrast_ratio(file_ink(&t, fill), fill);
                assert!(ratio >= 4.5, "{accent:?} hovered {hovered}: {ratio}");
            }
            assert!(theme::contrast_ratio(t.accent_text, t.accent) >= 4.5, "Share's text on the accent");
        }
    }
    for dark in [false, true] {
        let row = if dark { Tokens::dark().chrome } else { Tokens::light().chrome };
        assert!(theme::contrast_ratio(saved_color(dark), row) >= 4.5, "Saved, dark {dark}");
    }
}

#[test]
fn groups_and_tabs_fit_from_the_first_and_junk_widths_are_harmless() {
    assert_eq!(fit_groups(&[], 100.0), usize::MAX);
    assert_eq!(fit_groups(&[40.0, 40.0], 100.0), usize::MAX);
    assert_eq!(fit_groups(&[40.0, 40.0, 40.0], 100.0), 2);
    assert_eq!(fit_groups(&[120.0, 10.0], 100.0), 0);
    assert_eq!(fit_groups(&[40.0, 0.0, 40.0], 100.0), usize::MAX, "0 = not measured yet");
    assert_eq!(fit_groups(&[f32::NAN, f32::INFINITY, 40.0], 100.0), usize::MAX, "junk widths count as 0");
    assert_eq!(fit_tabs(&[(40.0, false), (40.0, false)], 100.0, 20.0), [true, true]);
    assert_eq!(fit_tabs(&[(40.0, false), (40.0, false), (40.0, false)], 100.0, 20.0), [true, true, false]);
    assert_eq!(fit_tabs(&[(40.0, false), (40.0, false), (40.0, true)], 100.0, 20.0), [true, false, true]);
    assert_eq!(fit_tabs(&[(f32::NAN, false), (f32::INFINITY, true)], 10.0, 20.0), [true, true]);
}

/// The host hooks: File › New, Ctrl+N, a theme and the sample deck go to the host's tabs (this
/// presentation stays as it is); Open asks the host's dialog, and a picked presentation goes to
/// the host unless only the untouched new presentation is here (which it replaces).
#[test]
fn new_and_open_go_to_the_host() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let news: Rc<RefCell<Vec<serde_json::Value>>> = Rc::default();
    let opens: Rc<RefCell<Vec<String>>> = Rc::default();
    let asked: Rc<RefCell<Vec<crate::FileDialogRequest>>> = Rc::default();
    let deck = std::env::temp_dir().join(format!("nexus-slide-host-open-{}.deckcraft", std::process::id()));
    {
        let mut s = Session::with_new();
        s.execute("slide.new", &json!({})).unwrap();
        s.execute("file.saveAs", &json!({"path": deck.to_string_lossy()})).unwrap();
    }
    let answer = deck.to_string_lossy().into_owned();
    let (n, o, q) = (news.clone(), opens.clone(), asked.clone());
    let mut h = harness((1440.0, 900.0), Language::Vi, false, move |a| {
        a.services.new_document = Some(Box::new(move |p| n.borrow_mut().push(p)));
        // The host takes every file except the first (this window holds only its new
        // presentation then, as the NexusOffice tab tells it).
        a.services.open_document = Some(Box::new(move |p: &str| {
            let first = o.borrow().is_empty() && !p.contains("-2-");
            o.borrow_mut().push(p.to_string());
            !first
        }));
        a.services.file_dialog = Some(Box::new(move |req| {
            q.borrow_mut().push(req);
            let (tx, rx) = std::sync::mpsc::channel();
            tx.send(Ok(Some(answer.clone()))).unwrap();
            rx
        }));
    });
    let before = h.state().session.documents().len();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::N);
    h.run_steps(4);
    h.state_mut().run("file.new", json!({"theme": "Nexus"})).unwrap();
    assert_eq!(*news.borrow(), vec![json!({}), json!({"theme": "Nexus"})]);
    assert_eq!(h.state().session.documents().len(), before, "this window keeps its one presentation");
    // Open into the untouched new presentation: it is replaced, nothing goes to the host.
    h.state_mut().run("app.openDialog", json!({})).unwrap();
    for _ in 0..200 {
        h.run_steps(1);
        if h.state().opening.is_none() && h.state().session.active().is_some_and(|d| d.path.is_some()) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(*asked.borrow(), vec![crate::FileDialogRequest::Open { purpose: "open".into() }]);
    assert_eq!(opens.borrow().len(), 1, "the host was asked first, and declined");
    assert_eq!(h.state().session.documents().len(), 1, "the untouched presentation was replaced");
    assert_eq!(slides(h.state()), 2);
    // With work here, Open hands the next file to the host.
    h.state_mut().run("slide.new", json!({})).unwrap();
    let other = std::env::temp_dir().join(format!("nexus-slide-host-open-2-{}.deckcraft", std::process::id()));
    std::fs::copy(&deck, &other).unwrap();
    h.state_mut().open_path_in_background(&other.to_string_lossy());
    assert_eq!(opens.borrow().last(), Some(&other.to_string_lossy().into_owned()));
    assert!(h.state().opening.is_none(), "the host took it: nothing opens here");
    let _ = (std::fs::remove_file(&deck), std::fs::remove_file(&other));
}

/// Save on a new presentation asks the host's dialog for a presentation type only (PowerPoint
/// first), waits for its answer and saves there; "Save changes?" › Save closes it once saved, and
/// Cancel in the dialog keeps it.
#[test]
fn save_asks_the_host_for_a_presentation_path() {
    use std::cell::RefCell;
    use std::rc::Rc;
    type Slot = Rc<RefCell<Option<(crate::FileDialogRequest, std::sync::mpsc::Sender<crate::FileDialogAnswer>)>>>;
    for cancel in [true, false] {
        let slot: Slot = Rc::default();
        let s2 = slot.clone();
        let target = std::env::temp_dir().join(format!("nexus-slide-host-save-{cancel}-{}", std::process::id()));
        let _ = std::fs::remove_file(target.with_extension("pptx"));
        let mut h = harness((1440.0, 900.0), Language::Vi, false, move |a| {
            a.services.file_dialog = Some(Box::new(move |req| {
                let (tx, rx) = std::sync::mpsc::channel();
                *s2.borrow_mut() = Some((req, tx));
                rx
            }));
            a.run("slide.new", json!({})).unwrap();
        });
        h.state_mut().close_presentation();
        h.run_steps(4);
        h.get_by_label("Lưu thay đổi?");
        h.get_by_label("Lưu").click();
        h.run_steps(4);
        assert_eq!(h.state().session.documents().len(), 1, "waits for the dialog");
        let (req, tx) = slot.borrow_mut().take().expect("asked where");
        let crate::FileDialogRequest::Save { name, types } = &req else { panic!("{req:?}") };
        assert_eq!(name, "Trình chiếu 1.pptx");
        assert_eq!(types, &["pptx", "deckcraft"]);
        tx.send(Ok(if cancel { None } else { Some(target.to_string_lossy().into_owned()) })).unwrap();
        h.run_steps(6);
        if cancel {
            assert_eq!(h.state().session.documents().len(), 1, "cancel keeps the presentation");
            assert!(h.state().session.active().unwrap().is_dirty());
        } else {
            assert!(target.with_extension("pptx").is_file(), "saved as a presentation");
            assert!(h.state().session.documents().is_empty(), "closed after the save");
        }
        let _ = std::fs::remove_file(target.with_extension("pptx"));
    }
}

/// An embedded editor adds its fonts to the window's (another editor's families stay), installs
/// them again when another editor replaced them, and never draws a frame without them.
#[test]
fn embedded_fonts_join_the_windows_and_come_back() {
    let mut h = harness((1024.0, 700.0), Language::Vi, false, |_| {});
    // Another editor's fonts, without ours (as Nexus Docs installs its own).
    let mut theirs = egui::FontDefinitions::default();
    theirs.families.insert(egui::FontFamily::Name("semibold".into()), vec!["Ubuntu-Light".into()]);
    h.ctx.set_fonts(theirs);
    h.run_steps(1);
    // This frame `ui` (or `logic`) sees ours are gone: nothing drawn with a missing family.
    h.run_steps(6);
    let families = h.ctx.fonts(|f| f.definitions().families.clone());
    assert!(families.contains_key(&theme::bold_family()), "ours are back");
    assert!(families.contains_key(&egui::FontFamily::Name("semibold".into())), "the other editor's stay");
    h.get_by_label("Tệp");
}

/// Drawn straight after another editor replaced the fonts in the same frame (a host bringing the
/// tab forward mid-frame), `ui` paints nothing instead of panicking on a missing family.
#[test]
fn ui_waits_for_its_fonts() {
    let mut h = harness((1024.0, 700.0), Language::Vi, false, |_| {});
    h.ctx.set_fonts(egui::FontDefinitions::default());
    // Without `logic` first: only `ui`, as a host drawing a tab brought forward mid-frame.
    let mut app = std::mem::replace(h.state_mut(), SlideApp::new(Session::new(), Services::default()));
    let ctx = h.ctx.clone();
    ctx.run_ui(egui::RawInput::default(), |ui| app.ui(ui)).textures_delta.clear();
    for _ in 0..2 {
        ctx.run_ui(egui::RawInput::default(), |ui| {
            app.logic(ui.ctx());
            app.ui(ui);
        })
        .textures_delta
        .clear();
    }
    assert!(app.fonts_installed(&ctx));
}

/// New presentations are "Trình chiếu n" in Vietnamese, upstream's "Presentation n" in English;
/// the slide's placeholders prompt in the interface's language.
#[test]
fn new_presentations_and_prompts_speak_the_interface_language() {
    let h = harness((1024.0, 700.0), Language::Vi, false, |a| {
        a.session.untitled_prefix = "Trình chiếu ".into();
    });
    let mut s = Session::new();
    s.untitled_prefix = "Trình chiếu ".into();
    s.execute("file.new", &json!({})).unwrap();
    assert_eq!(s.active().unwrap().title(), "Trình chiếu 1");
    assert_eq!(h.state().session.untitled_prefix, "Trình chiếu ", "the app sets it from its language");
    let prompt = crate::i18n::prompt_fn(Language::Vi).unwrap();
    assert_eq!(prompt(deckcraft_model::PhType::Title), "Bấm để thêm tiêu đề");
    assert!(crate::i18n::prompt_fn(Language::En).is_none());
}

fn texts(shape: &egui::Shape, out: &mut Vec<String>) {
    match shape {
        egui::Shape::Text(t) => out.push(t.galley.text().to_string()),
        egui::Shape::Vec(v) => v.iter().for_each(|s| texts(s, out)),
        _ => {}
    }
}

/// Every text the last frame drew.
fn drawn(h: &H) -> Vec<String> {
    let mut out = Vec::new();
    for s in &h.output().shapes {
        texts(&s.shape, &mut out);
    }
    out
}

/// Whether `text` matches the catalogue's `template` with each `{name}` standing for anything.
fn fills(template: &str, text: &str) -> bool {
    let parts: Vec<&str> = template.split(['{', '}']).enumerate().filter(|(i, _)| i % 2 == 0).map(|(_, p)| p).collect();
    if parts.len() < 2 {
        return false;
    }
    let mut rest = text;
    for (k, p) in parts.iter().enumerate() {
        if p.is_empty() {
            continue;
        }
        match rest.find(p) {
            Some(i) if k > 0 || i == 0 => rest = rest.get(i + p.len()..).unwrap_or_default(),
            _ => return false,
        }
    }
    parts.last().is_none_or(|p| p.is_empty() || text.ends_with(p))
}

/// Text the interface may draw as it is: names (products, fonts, themes, languages), the
/// presentation's own text, numbers, units and shortcuts.
fn exempt(text: &str, doc_text: &str) -> bool {
    let fonts = deckcraft_fonts::FontDb::global().families();
    // Starter decks' names (shown in Vietnamese, from the engine's table) and paper sizes are names.
    let starters: Vec<&str> = deckcraft_engine::templates::starters().iter().map(|t| deckcraft_engine::nexus_vi::text(Some("vi"), t.name)).collect();
    let papers: Vec<&str> = deckcraft_engine::cmd::file::PAPERS.iter().map(|p| p.1).collect();
    let builtin = deckcraft_model::theme::builtin_themes();
    let themes: Vec<String> = builtin.iter().map(|t| t.name.clone()).collect();
    // The themes' own fonts are names too, installed or not (Inter is missing where craft-fonts isn't).
    let theme_fonts: Vec<String> = builtin.iter().flat_map(|t| [t.fonts.major.latin.clone(), t.fonts.minor.latin.clone()]).collect();
    let plain = text.trim_end_matches('…').trim();
    let letters = plain.as_bytes().windows(2).any(|w| w[0].is_ascii_alphabetic() && w[1].is_ascii_alphabetic());
    // A number with its unit ("10,58 cm", "6 pt") is not a word.
    let unit = plain.trim_start_matches(|c: char| c.is_ascii_digit() || ".,-+ ".contains(c));
    !letters
        || (unit.len() < plain.len() && ["cm", "in", "pt", "px"].contains(&unit))
        || [
            "Nexus Slide",
            "NexusOffice",
            "SmartArt",
            "WordArt",
            "PDF",
            "Aa",
            "Abc",
            "Ctrl",
            "Tiếng Việt",
            "English",
        ]
        .contains(&plain)
        || plain.starts_with("Ctrl+")
        || plain.ends_with('%')
        || fonts.iter().any(|f| f == plain)
        || starters.contains(&plain)
        // Today's date in the Date and Time dialog's Vietnamese formats ("Thứ Hai, 26 tháng 10 năm 2026").
        || deckcraft_text::datetime::DateTime::now_local().is_some_and(|now| {
            (1..=13).any(|k| deckcraft_text::datetime::field_text(&format!("datetime{k}"), Some("vi-VN"), now).as_deref() == Some(plain))
        })
        || papers.contains(&plain)
        // Character Spacing's and Change Case's samples.
        || ["ab", "abc"].contains(&plain)
        || themes.iter().any(|t| t == plain)
        // Languages named in their own language (Language, Review › Language), legal notices kept
        // as written, and the photo album's theme names.
        || crate::squiggles::LANGS.iter().any(|(name, _)| *name == plain)
        || [deckcraft_engine::nexus::COPYRIGHT, deckcraft_engine::nexus::UPSTREAM_COPYRIGHT].contains(&plain)
        // The About page's Vietnamese sentences (from the engine's brand constants).
        || [deckcraft_engine::nexus::ABOUT_SUITE.1, deckcraft_engine::nexus::ATTRIBUTION.1].contains(&plain)
        || theme_fonts.iter().any(|f| f == plain)
        // The presentation's own text: a whole line of it (a title, a note), not any piece.
        || doc_text.lines().any(|l| l.trim() == plain)
}

/// Every label the WPS layout (each tab's strip, the top row, the status bar) and the File tab's
/// pages draw in Vietnamese is translated: a drawn text is a catalogue translation, a filled
/// catalogue template, or a name or the presentation's own text.
#[test]
fn every_visible_label_in_the_wps_layout_and_the_file_tab_is_vietnamese() {
    use crate::backstage::Page;
    let values: Vec<&str> = crate::i18n::vi_entries().iter().map(|(_, v)| *v).collect();
    let ok = |t: &str, doc: &str| {
        // A bullet before, an arrow after (the File tab's lists and links) are not words.
        let t = t.trim().trim_start_matches('•').trim_end_matches('→').trim();
        t.is_empty() || values.contains(&t) || values.iter().any(|v| v.contains('{') && fills(v, t)) || exempt(t, doc)
    };
    let mut missing = std::collections::BTreeSet::new();
    let mut check = |h: &H, place: &str| {
        let doc = h.state().session.active().map(crate::uifonts::ui_text).unwrap_or_default();
        for t in drawn(h) {
            for line in t.lines() {
                if !ok(line, &doc) {
                    missing.insert(format!("{place}: {line:?}"));
                }
            }
        }
    };
    for (name, setup) in scenarios() {
        let mut h = harness((1920.0, 900.0), Language::Vi, false, |a| setup(a));
        for tab in shown_tabs(h.state()) {
            h.state_mut().ui.tab = tab.to_string();
            h.run_steps(30);
            check(&h, &format!("{name} › {tab}"));
        }
    }
    let mut h = harness((1920.0, 1000.0), Language::Vi, false, |a| {
        a.run("slide.new", json!({})).unwrap();
    });
    for page in Page::ALL {
        h.state_mut().open_backstage(page);
        h.run_steps(8);
        check(&h, &format!("Tệp › {}", page.label()));
    }
    let missing: Vec<String> = missing.into_iter().collect();
    assert!(missing.is_empty(), "untranslated ({}):\n{}", missing.len(), missing.join("\n"));
}

/// Merged into a host window's fonts, the bold family is its own face, then the window's text
/// faces (which have Vietnamese), then ours; the window's families stay.
#[test]
fn merged_fonts_fall_back_to_the_windows_text_faces() {
    use egui::FontFamily;
    let mut window = egui::FontDefinitions::empty();
    window.families.insert(FontFamily::Proportional, vec!["Window".into()]);
    window.families.insert(FontFamily::Name("semibold".into()), vec!["WindowSemi".into()]);
    let mut ours = egui::FontDefinitions::empty();
    ours.families.insert(FontFamily::Proportional, vec!["Ours".into()]);
    ours.families.insert(theme::bold_family(), vec!["OursBold".into(), "Ours".into()]);
    let merged = theme::merge_font_definitions(window, ours);
    assert_eq!(merged.families.get(&theme::bold_family()), Some(&vec!["OursBold".to_string(), "Window".into(), "Ours".into()]));
    assert_eq!(merged.families.get(&FontFamily::Proportional), Some(&vec!["Window".to_string(), "Ours".into()]));
    assert!(merged.families.contains_key(&FontFamily::Name("semibold".into())));
}

/// Types `text` into the text field `n` (in drawing order) of the role `role`.
fn type_into(h: &mut H, role: egui::accesskit::Role, n: usize, text: &str) {
    let nodes: Vec<_> = h.query_all_by_role(role).collect();
    let node = nodes.get(n).unwrap_or_else(|| panic!("no {role:?} #{n} (there are {})", nodes.len()));
    node.click();
    h.run_steps(2);
    h.event(egui::Event::Text(text.into()));
    h.run_steps(2);
}

fn notes(app: &SlideApp, slide: usize) -> String {
    app.session.active().and_then(|d| d.doc.slides.get(slide).map(|s| s.notes_text())).unwrap_or_default()
}

/// Every place typing waits in a field until it loses the focus counts as a change, and
/// `commit_drafts` puts it into the presentation: the notes pane, View › Notes Page, the Alt Text
/// field, the SmartArt Text Pane. Notes edited back to how they were are no change.
#[test]
fn drafts_in_every_view_count_and_are_committed() {
    use egui::accesskit::Role;
    // The notes pane under the slide; typed and taken back is no change.
    let mut h = harness((1440.0, 900.0), Language::Vi, false, |_| {});
    type_into(&mut h, Role::MultilineTextInput, 0, "Ghi chú");
    assert!(h.state().has_drafts(), "notes pane");
    for _ in 0.."Ghi chú".chars().count() {
        h.key_press(egui::Key::Backspace);
    }
    h.run_steps(2);
    assert!(!h.state().has_drafts(), "edited back to how it was");
    type_into(&mut h, Role::MultilineTextInput, 0, "Lời chào");
    assert!(h.state_mut().commit_drafts());
    assert!(notes(h.state(), 0).contains("Lời chào"));
    assert!(!h.state().has_drafts());

    // View › Notes Page.
    let mut h = harness((1440.0, 900.0), Language::Vi, false, |a| a.ui.view = crate::ViewMode::NotesPage);
    type_into(&mut h, Role::MultilineTextInput, 0, "Trang ghi chú");
    assert!(h.state().has_drafts(), "Notes Page");
    assert!(h.state_mut().commit_drafts());
    assert!(notes(h.state(), 0).contains("Trang ghi chú"));
    assert!(h.state().session.active().unwrap().is_dirty());

    // Format Shape › Size › Alt Text.
    let mut h = harness((1440.0, 900.0), Language::Vi, false, |a| {
        a.run("shape.insert", json!({"preset": "rect", "rect": [100, 100, 300, 200]})).unwrap();
        a.ui.pane = Some("format".into());
        a.ui.format_tab = "size".into();
    });
    let fields = h.query_all_by_role(Role::MultilineTextInput).count();
    // (The notes pane's field is drawn after the task pane's.)
    type_into(&mut h, Role::MultilineTextInput, fields.saturating_sub(2), "Hình chữ nhật xanh");
    assert!(h.state().has_drafts(), "Alt Text");
    assert!(h.state_mut().commit_drafts());
    let st = h.state().session.active().unwrap();
    let descr = st.selected_shapes().first().map(|s| s.descr.clone()).unwrap_or_default();
    assert_eq!(descr, "Hình chữ nhật xanh");

    // The SmartArt Text Pane.
    let mut h = harness((1440.0, 900.0), Language::Vi, false, |a| {
        a.run("insert.smartArt", json!({"kind": "list", "items": ["Một", "Hai"]})).unwrap();
        a.ui.pane = Some("smartart".into());
    });
    type_into(&mut h, Role::TextInput, 0, " nữa");
    assert!(h.state().has_drafts(), "SmartArt Text Pane");
    assert!(h.state_mut().commit_drafts());
    let st = h.state().session.active().unwrap();
    let (_, d) = deckcraft_engine::cmd::smartart::selected(st).unwrap();
    assert!(d.nodes.iter().any(|n| n.text.contains("nữa")), "{:?}", d.nodes.iter().map(|n| &n.text).collect::<Vec<_>>());
}

/// Typing waiting in a field means the window doesn't hold an untouched presentation: a file the
/// host declines opens beside it instead of replacing it.
#[test]
fn a_draft_keeps_the_new_presentation_from_being_replaced() {
    let deck = std::env::temp_dir().join(format!("nexus-slide-draft-open-{}.deckcraft", std::process::id()));
    Session::with_new().execute("file.saveAs", &json!({"path": deck.to_string_lossy()})).unwrap();
    let mut h = harness((1440.0, 900.0), Language::Vi, false, |a| {
        a.services.open_document = Some(Box::new(|_: &str| false));
    });
    assert!(h.state().only_untouched_blank());
    type_into(&mut h, egui::accesskit::Role::MultilineTextInput, 0, "Đừng mất");
    assert!(!h.state().only_untouched_blank(), "notes are waiting");
    h.state_mut().open_path_in_background(&deck.to_string_lossy());
    for _ in 0..200 {
        h.run_steps(1);
        if h.state().opening.is_none() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(h.state().session.documents().len(), 2, "the new presentation with its notes stays");
    let _ = std::fs::remove_file(&deck);
}

/// Save puts the typing still in a field into the file.
#[test]
fn save_includes_typing_still_in_a_field() {
    let path = std::env::temp_dir().join(format!("nexus-slide-save-draft-{}.deckcraft", std::process::id()));
    let mut h = harness((1440.0, 900.0), Language::Vi, false, |_| {});
    h.state_mut().session.active_mut().unwrap().path = Some(path.to_string_lossy().into_owned());
    type_into(&mut h, egui::accesskit::Role::MultilineTextInput, 0, "Ghi chú đã lưu");
    h.state_mut().save();
    h.run_steps(2);
    assert!(!h.state().session.active().unwrap().is_dirty());
    let mut s = Session::new();
    s.execute("file.open", &json!({"path": path.to_string_lossy()})).unwrap();
    assert!(s.active().unwrap().doc.slides.first().map(|x| x.notes_text()).unwrap_or_default().contains("Ghi chú đã lưu"));
    let _ = std::fs::remove_file(&path);
}

/// The host's Save As answers later: it saves the presentation it was asked for, even when
/// another one is shown by then.
#[test]
fn a_late_save_answer_saves_the_presentation_it_was_asked_for() {
    use std::cell::RefCell;
    use std::rc::Rc;
    type Slot = Rc<RefCell<Option<std::sync::mpsc::Sender<crate::FileDialogAnswer>>>>;
    let slot: Slot = Rc::default();
    let s2 = slot.clone();
    let target = std::env::temp_dir().join(format!("nexus-slide-late-save-{}.pptx", std::process::id()));
    let _ = std::fs::remove_file(&target);
    let mut h = harness((1440.0, 900.0), Language::Vi, false, move |a| {
        a.services.file_dialog = Some(Box::new(move |_req| {
            let (tx, rx) = std::sync::mpsc::channel();
            *s2.borrow_mut() = Some(tx);
            rx
        }));
        a.run("slide.new", json!({})).unwrap();
    });
    let asked_for = h.state().session.active().unwrap().uid;
    h.state_mut().save();
    // Another presentation comes forward before the dialog answers.
    h.state_mut().session.execute("file.new", &json!({})).unwrap();
    let other = h.state().session.active().unwrap().uid;
    slot.borrow_mut().take().unwrap().send(Ok(Some(target.to_string_lossy().into_owned()))).unwrap();
    h.run_steps(4);
    let docs = h.state().session.documents();
    assert!(docs.iter().find(|d| d.uid == asked_for).is_some_and(|d| d.path.is_some() && !d.is_dirty()), "the one asked for is saved");
    assert!(docs.iter().find(|d| d.uid == other).is_some_and(|d| d.path.is_none()), "the other is left alone");
    assert_eq!(h.state().session.active().map(|d| d.uid), Some(other), "and stays shown");
    let _ = std::fs::remove_file(&target);
}

/// Embedded, the stand-alone app's interface size doesn't zoom the host's window.
#[test]
fn embedded_leaves_the_windows_zoom_alone() {
    let h = harness((1024.0, 700.0), Language::Vi, false, |a| a.ui.ui_scale = 1.5);
    assert!((h.ctx.zoom_factor() - 1.0).abs() < 1e-3);
    let alone = harness((1024.0, 700.0), Language::Vi, false, |a| {
        a.embedded = false;
        a.ui.ui_scale = 1.5;
    });
    assert!((alone.ctx.zoom_factor() - 1.5).abs() < 1e-3);
}

/// Tab to a placeholder, F2 into it and Esc out types nothing: the presentation isn't "Chưa lưu".
#[test]
fn entering_a_placeholder_alone_is_not_a_change() {
    let mut h = harness((1440.0, 900.0), Language::Vi, false, |_| {});
    for key in [egui::Key::Tab, egui::Key::F2, egui::Key::Escape, egui::Key::Enter, egui::Key::Escape, egui::Key::Escape] {
        h.key_press(key);
        h.run_steps(2);
    }
    assert!(!h.state().session.active().unwrap().is_dirty(), "nothing typed");
    assert!(!h.state().has_drafts());
}

/// Whether `t`, a line drawn in Vietnamese, is translated: a catalogue translation, a filled
/// template, or text that may stay as it is ([`exempt`], units, numbering samples).
fn vietnamese(t: &str, doc: &str) -> bool {
    let t = t.trim().trim_start_matches('•').trim_end_matches('→').trim();
    let values: Vec<&str> = crate::i18n::vi_entries().iter().map(|(_, v)| *v).collect();
    // A number with its unit ("6 pt", "1.5 in") is not a word either.
    let unit = t.trim_start_matches(|c: char| c.is_ascii_digit() || ".,-+ ".contains(c));
    t.is_empty()
        || values.contains(&t)
        || values.iter().any(|v| v.contains('{') && fills(v, t))
        // Units, the font sample, the numbering styles' samples and the hex hint are not words.
        || ["in", "cm", "pt", "px", "AaBbCcDd", "I. II. III.", "i. ii. iii.", "RRGGBB"].contains(&unit)
        || exempt(t, doc)
}

/// The labels a frame drew, and the ones it asked [`crate::i18n::tl`] for (drop-down items and
/// tooltips included), that are still English.
fn english_in(h: &mut H, place: &str, out: &mut std::collections::BTreeSet<String>) {
    let mut doc = h.state().session.active().map(crate::uifonts::ui_text).unwrap_or_default();
    // The author's name (Options) is the user's.
    doc.push('\n');
    doc.push_str(&h.state().session.prefs.author);
    // The AutoCorrect list is the user's (its entries are what they type), not interface wording.
    if let Ok(v) = h.state_mut().session.execute("text.autocorrect", &json!({})) {
        for pair in v["entries"].as_array().into_iter().flatten() {
            for side in pair.as_array().into_iter().flatten().filter_map(|s| s.as_str()) {
                doc.push('\n');
                doc.push_str(side);
            }
        }
    }
    for t in drawn(h) {
        for line in t.lines() {
            if !vietnamese(line, &doc) {
                out.insert(format!("{place}: {line:?}"));
            }
        }
    }
    for miss in crate::i18n::misses::take() {
        if !vietnamese(&miss, &doc) {
            out.insert(format!("{place} (tl): {miss:?}"));
        }
    }
}

/// NexusOffice: every dialog ([`VI_DIALOGS`]: the Format dialogs, Options, Insert Table, Slide
/// Size, Header and Footer, Hyperlink, Photo Album, Theme Colors/Fonts, …, in the variants that show
/// more), the Format Shape pane on each of its pages and fills, the Format Background pane and the
/// colour menu speak Vietnamese: every label they draw or ask the catalogue for has a translation.
/// A new label without a `vi` entry fails here with its place.
#[test]
fn every_label_in_every_dialog_and_the_format_panes_is_vietnamese() {
    use crate::dialogs::Dialog;
    let b64 = |b: Vec<u8>| deckcraft_engine::cmd::base64_encode(&b);
    let mut missing = std::collections::BTreeSet::new();
    let shape = |a: &mut SlideApp| {
        a.run("shape.insert", json!({"preset": "rect", "rect": [100, 100, 300, 200], "text": "x"})).unwrap();
    };
    let _ = crate::i18n::misses::take();
    // Dialogs.
    let album = format!(r#"[{{"text":"Xin chào"}},{{"name":"Ảnh biển","data":"{}"}}]"#, b64(png()));
    let dialogs: Vec<(&str, Vec<(&str, &str)>)> = vec![
        ("font", vec![]),
        ("paragraph", vec![]),
        ("tabs", vec![("stops", "1:l|2.5:dec")]),
        ("bullets", vec![("tab", "bulleted")]),
        ("bullets", vec![("tab", "numbered")]),
        ("sizePosition", vec![]),
        ("preferences", vec![]),
        ("autocorrect", vec![]),
        ("gridOptions", vec![]),
        ("close", vec![]),
        ("quit", vec![]),
        ("table", vec![]),
        ("slideSize", vec![]),
        ("headerFooter", vec![]),
        ("hyperlink", vec![]),
        ("hyperlink", vec![("mode", "slide")]),
        ("hyperlink", vec![("mode", "email")]),
        ("setupShow", vec![]),
        ("zoom", vec![]),
        ("comment", vec![]),
        ("action", vec![]),
        ("action", vec![("tab", "hover"), ("kind", "customShow")]),
        ("action", vec![("kind", "program")]),
        ("action", vec![("kind", "url")]),
        ("altText", vec![]),
        ("outline", vec![]),
        ("renameSection", vec![]),
        ("renameShape", vec![]),
        ("renameLayout", vec![]),
        ("export", vec![]),
        ("export", vec![("fmt", "gif"), ("gifSize", "custom")]),
        ("export", vec![("fmt", "gif"), ("gifTo", "9")]),
        ("replaceFonts", vec![]),
        ("replaceFonts", vec![("result", "")]),
        ("symbol", vec![]),
        ("equation", vec![]),
        ("pasteSpecial", vec![]),
        ("find", vec![]),
        ("find", vec![("replace", "1"), ("text", "không có")]),
        ("spelling", vec![]),
        ("accessibility", vec![]),
        ("language", vec![]),
        ("customShow", vec![]),
        ("chartData", vec![]),
        ("trim", vec![]),
        ("smartart", vec![]),
        ("about", vec![]),
        ("photoAlbum", vec![]),
        ("photoAlbum", vec![("items", album.as_str()), ("sel", "1")]),
        ("photoAlbum", vec![("items", album.as_str()), ("sel", "0")]),
        ("themeColors", vec![]),
        ("themeFonts", vec![]),
        ("dateTime", vec![]),
    ];
    let mut opened: Vec<&str> = dialogs.iter().map(|(id, _)| *id).collect();
    opened.sort_unstable();
    opened.dedup();
    let mut listed = VI_DIALOGS.to_vec();
    listed.sort_unstable();
    assert_eq!(opened, listed, "VI_DIALOGS names exactly the dialogs this test opens");
    for (id, fields) in dialogs {
        let mut h = harness((1600.0, 1000.0), Language::Vi, false, |a| {
            shape(a);
            // What some dialogs need to show all they can: a chart, a custom show, a misspelt and
            // repeated word, a picture without alt text, a name to rename.
            match id {
                "chartData" => {
                    // The chart's own data is the user's: Vietnamese names, a decimal value.
                    let id = a.run("insert.chart", json!({})).unwrap()["id"].as_u64().unwrap();
                    let series = json!([{"name": "Doanh thu", "values": [1.5, 2.0]}, {"name": "Chi phí", "values": [1.0, 0.5]}]);
                    a.run("chart.data", json!({"id": id, "categories": ["Quý 1", "Quý 2"], "series": series})).unwrap();
                }
                "customShow" | "setupShow" => {
                    a.run("show.customShow", json!({"name": "Tour", "slides": [0]})).unwrap();
                }
                "spelling" => {
                    a.run("shape.insert", json!({"preset": "rect", "rect": [100, 300, 300, 100], "text": "teh teh"})).unwrap();
                }
                "accessibility" => {
                    a.run("insert.picture", json!({"name": "p.png", "data": b64(png())})).unwrap();
                }
                _ => {}
            }
            let mut d = Dialog::new(id);
            if id.starts_with("rename") {
                d.params = json!({"name": "Tên cũ"});
            }
            for (k, v) in &fields {
                d.fields.insert((*k).into(), (*v).into());
            }
            a.dialog = Some(d);
        });
        h.run_steps(8);
        assert!(h.state().dialog.is_some(), "{id} stays open");
        // The spelling suggestions are words of the presentation's language, not interface wording.
        let suggestions: Vec<String> = h
            .state_mut()
            .session
            .execute("review.spelling", &json!({}))
            .ok()
            .and_then(|v| v.as_array().cloned())
            .into_iter()
            .flatten()
            .flat_map(|i| i["suggestions"].as_array().cloned().unwrap_or_default())
            .filter_map(|w| w.as_str().map(str::to_string))
            .collect();
        let mut here = std::collections::BTreeSet::new();
        english_in(&mut h, &format!("dialog {id} {fields:?}"), &mut here);
        // So is what the test typed or named (chart data, the text to find, a picture's name).
        let typed = ["Doanh thu", "Chi phí", "Quý 1", "Quý 2", "không có", "Ảnh biển"].map(String::from);
        missing.extend(here.into_iter().filter(|m| !suggestions.iter().chain(&typed).any(|w| m.ends_with(&format!(": {w:?}")))));
    }
    // The Format Shape pane: each page, and the fills and lines that show more controls.
    let pages: Vec<(&str, &str, Box<dyn Fn(&mut SlideApp)>)> = vec![
        (
            "fill",
            "solid + arrows",
            Box::new(|a: &mut SlideApp| {
                a.run("shape.fill", json!({"color": "accent1"})).unwrap();
                a.run("shape.line", json!({"color": "tx1", "width": 2})).unwrap();
                a.run("shape.line", json!({"head": "triangle"})).unwrap();
                a.run("shape.line", json!({"tail": "oval"})).unwrap();
            }),
        ),
        (
            "fill",
            "gradient",
            Box::new(|a: &mut SlideApp| {
                a.run("shape.fill", json!({"gradient": {"stops": [[0, "accent1"], [1, "accent2"]], "angle": 90}})).unwrap();
            }),
        ),
        (
            "fill",
            "radial gradient",
            Box::new(|a: &mut SlideApp| {
                a.run("shape.fill", json!({"gradient": {"stops": [[0, "accent1"], [1, "accent2"]], "kind": "radial"}})).unwrap();
            }),
        ),
        (
            "fill",
            "picture",
            Box::new(move |a: &mut SlideApp| {
                a.run("shape.fill", json!({"picture": b64(png())})).unwrap();
            }),
        ),
        (
            "fill",
            "tiled picture",
            Box::new(move |a: &mut SlideApp| {
                a.run("shape.fill", json!({"picture": b64(png())})).unwrap();
                a.run("shape.fill", json!({"pictureProps": {"tile": true}})).unwrap();
            }),
        ),
        ("effects", "none", Box::new(|_: &mut SlideApp| {})),
        (
            "effects",
            "all",
            Box::new(|a: &mut SlideApp| {
                a.run("shape.effects", json!({"shadow": "outer"})).unwrap();
                a.run("shape.effects", json!({"reflection": "half"})).unwrap();
                a.run("shape.effects", json!({"glow": {"color": "accent1", "radius": 6}})).unwrap();
                a.run("shape.effects", json!({"threeD": {"bevelTop": "circle", "material": "clear"}})).unwrap();
            }),
        ),
        ("size", "", Box::new(|_: &mut SlideApp| {})),
        ("text", "", Box::new(|_: &mut SlideApp| {})),
    ];
    for (tab, what, setup) in pages {
        let mut h = harness((1600.0, 1400.0), Language::Vi, false, |a| {
            shape(a);
            setup(a);
            a.ui.pane = Some("format".into());
            a.ui.format_tab = tab.into();
        });
        h.run_steps(8);
        english_in(&mut h, &format!("Format Shape › {tab} ({what})"), &mut missing);
    }
    let mut h = harness((1600.0, 1400.0), Language::Vi, false, |a| {
        a.run("insert.picture", json!({"name": "p.png", "data": b64(png())})).unwrap();
        a.ui.pane = Some("format".into());
        a.ui.format_tab = "picture".into();
    });
    h.run_steps(8);
    english_in(&mut h, "Format Shape › picture", &mut missing);
    let mut h = harness((1600.0, 1000.0), Language::Vi, false, |a| {
        a.run("edit.deselect", json!({})).unwrap();
        a.ui.pane = Some("format".into());
    });
    h.run_steps(8);
    english_in(&mut h, "Format Shape (nothing selected)", &mut missing);
    // The Format Background pane, with each kind of fill.
    let backgrounds: Vec<(&str, serde_json::Value)> = vec![
        ("follow", json!({"reset": true})),
        ("solid", json!({"color": "bg2"})),
        ("gradient", json!({"gradient": {"stops": [[0, "accent1"], [1, "accent2"]], "angle": 90}})),
        ("pattern", json!({"pattern": {"preset": "pct20", "fg": "accent1", "bg": "bg1"}})),
        ("picture", json!({"picture": b64(png())})),
    ];
    for (what, p) in backgrounds {
        let mut h = harness((1600.0, 1400.0), Language::Vi, false, |a| {
            a.run("design.background", p.clone()).unwrap();
            if what == "picture" {
                a.run("design.background", json!({"pictureProps": {"tile": true}})).unwrap();
            }
            a.ui.pane = Some("background".into());
        });
        h.run_steps(8);
        english_in(&mut h, &format!("Format Background ({what})"), &mut missing);
    }
    // The colour menu the panes and dialogs open.
    // The fonts it installs (bold among them) take effect from the next frame.
    let frames = std::cell::Cell::new(0);
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(400.0, 600.0)).build_ui(|ui| {
        crate::theme::install_fonts(ui.ctx());
        crate::i18n::set_current(ui.ctx(), Language::Vi);
        frames.set(frames.get() + 1);
        if frames.get() > 2 {
            let _ = crate::widgets::color_grid(ui, &deckcraft_model::Theme::default().colors, Some("No Fill"));
        }
    });
    h.run_steps(4);
    let mut got = Vec::new();
    for s in &h.output().shapes {
        texts(&s.shape, &mut got);
    }
    for t in got {
        if !vietnamese(&t, "") {
            missing.insert(format!("colour menu: {t:?}"));
        }
    }
    for miss in crate::i18n::misses::take() {
        if !vietnamese(&miss, "") {
            missing.insert(format!("colour menu (tl): {miss:?}"));
        }
    }
    let missing: Vec<String> = missing.into_iter().collect();
    assert!(missing.is_empty(), "untranslated ({}):\n{}", missing.len(), missing.join("\n"));
}

/// The File tab's starter gallery shows the starters' Vietnamese names and opens them filled
/// with Vietnamese text; the sample deck opens in Vietnamese too.
#[test]
fn starters_and_the_sample_open_in_the_interface_language() {
    let mut h = harness((1920.0, 1000.0), Language::Vi, false, |_| {});
    h.state_mut().open_backstage(crate::backstage::Page::New);
    h.run_steps(8);
    let shown = drawn(&h);
    assert!(shown.iter().any(|t| t == "Giao ban"), "the Briefing tile is named in Vietnamese: {shown:?}");
    assert!(!shown.iter().any(|t| t == "Briefing"), "{shown:?}");
    h.get_by_label("Giao ban").click();
    h.run_steps(6);
    let title = h.state().session.active().map(|d| d.doc.slides[0].title());
    assert_eq!(title.as_deref(), Some("Giao ban tuần"));
    h.state_mut().open_backstage(crate::backstage::Page::New);
    h.run_steps(8);
    h.get_by_label("Bản mẫu: Bắc cực quang").click();
    h.run_steps(6);
    assert_eq!(h.state().session.active().map(|d| d.title.clone()).as_deref(), Some("Bắc cực quang"));
}

/// The dialogs [`every_label_in_every_dialog_and_the_format_panes_is_vietnamese`]
/// opens in Vietnamese.
const VI_DIALOGS: &[&str] = &[
    "font",
    "paragraph",
    "tabs",
    "bullets",
    "sizePosition",
    "preferences",
    "autocorrect",
    "gridOptions",
    "close",
    "quit",
    "table",
    "slideSize",
    "headerFooter",
    "hyperlink",
    "setupShow",
    "zoom",
    "comment",
    "action",
    "altText",
    "outline",
    "renameSection",
    "renameShape",
    "renameLayout",
    "export",
    "replaceFonts",
    "symbol",
    "equation",
    "pasteSpecial",
    "find",
    "spelling",
    "accessibility",
    "language",
    "customShow",
    "chartData",
    "trim",
    "smartart",
    "about",
    "photoAlbum",
    "themeColors",
    "themeFonts",
    "dateTime",
];

/// Dialogs still drawn (partly) in English: the follow-up list. A dialog leaves it when the
/// coverage test opens it ([`VI_DIALOGS`]).
const NOT_YET_VI: &[&str] = &[];

/// Every dialog id `dialogs.rs`'s `body` handles (its match arms, `"a" | "b" =>` included).
fn dialog_ids() -> Vec<String> {
    let src = include_str!("dialogs.rs");
    let start = src.find("fn body(app: &mut SlideApp").expect("dialogs::body");
    let rest = &src[start..];
    let end = rest[1..].find("\nfn ").map_or(rest.len(), |i| i + 1);
    let mut ids = Vec::new();
    for line in rest[..end].lines() {
        let Some(arm) = line.strip_prefix("        \"") else { continue };
        let Some((pattern, _)) = arm.split_once(" =>") else { continue };
        for id in format!("\"{pattern}").split('|') {
            let id = id.trim().trim_matches('"');
            if !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric()) {
                ids.push(id.to_string());
            }
        }
    }
    ids
}

/// A new dialog has to be opened by the Vietnamese-coverage test or listed in [`NOT_YET_VI`];
/// neither list names a dialog that doesn't exist or names one twice.
#[test]
fn every_dialog_is_covered_in_vietnamese_or_listed_as_not_yet() {
    let ids = dialog_ids();
    assert!(ids.len() >= 40, "the parser found the dialogs ({})", ids.len());
    // NexusOffice: every dialog speaks Vietnamese; none is left for later.
    assert_eq!(NOT_YET_VI, &[] as &[&str], "dialogs still (partly) in English");
    for id in &ids {
        let covered = VI_DIALOGS.contains(&id.as_str());
        let later = NOT_YET_VI.contains(&id.as_str());
        assert!(covered || later, "dialog {id:?} is neither covered in Vietnamese (VI_DIALOGS) nor listed in NOT_YET_VI");
        assert!(!(covered && later), "dialog {id:?} is in both lists");
    }
    for id in VI_DIALOGS.iter().chain(NOT_YET_VI) {
        assert!(ids.iter().any(|d| d == id), "{id:?} is listed but dialogs.rs has no such dialog");
    }
    let mut all: Vec<&str> = VI_DIALOGS.iter().chain(NOT_YET_VI).copied().collect();
    let n = all.len();
    all.sort_unstable();
    all.dedup();
    assert_eq!(n, all.len(), "a dialog is listed twice");
}

/// NexusOffice (vi-VN units): Size and Position shows lengths in centimetres with a decimal comma in
/// Vietnamese ("2,54") and in inches in English ("1"), in the dialog and on the Format Shape pane's
/// Size page; a length typed with a comma or a point reads the same, and the shape keeps points.
#[test]
fn size_and_position_shows_centimetres_with_a_comma_in_vietnamese_and_inches_in_english() {
    use crate::dialogs::Dialog;
    let size_of = |app: &SlideApp, id: u64| {
        let st = app.session.active().unwrap();
        let x = st.shape(deckcraft_model::ShapeId(id as u32)).and_then(|s| s.xfrm).unwrap();
        (x.x, x.y, x.w, x.h)
    };
    // 1 in from the left, 0.5 in down, 1.5 in wide, 1 in high.
    let rect = json!({"preset": "rect", "rect": [72.0, 36.0, 108.0, 72.0]});
    let cases = [
        (Language::Vi, ["2,54", "3,81", "2,54", "1,27"], "Chiều cao (cm):", "2,54 cm"),
        (Language::En, ["1", "1.5", "1", "0.5"], "Height (in):", "1\""),
    ];
    for (lang, [h_text, w_text, x_text, y_text], height_label, pane_text) in cases {
        let id = std::cell::Cell::new(0);
        let mut h = harness((1600.0, 1000.0), lang, false, |a| {
            id.set(a.run("shape.insert", rect.clone()).unwrap()["id"].as_u64().unwrap());
            a.dialog = Some(Dialog::new("sizePosition"));
        });
        h.run_steps(8);
        let d = h.state().dialog.as_ref().expect("the dialog is open");
        let field = |k: &str| d.fields.get(k).cloned().unwrap_or_default();
        assert_eq!([field("h"), field("w"), field("x"), field("y")], [h_text, w_text, x_text, y_text], "{lang:?}");
        let shown = drawn(&h);
        assert!(shown.iter().any(|t| t == height_label), "{lang:?}: {height_label:?} in {shown:?}");
        assert!(shown.iter().any(|t| t == h_text), "{lang:?}: the field shows {h_text:?}");
        // Typed: 5,08 cm (Vietnamese) or 2 in (English) high is 144 pt either way.
        let typed = if lang == Language::Vi { "5,08" } else { "2" };
        h.state_mut().dialog.as_mut().unwrap().fields.insert("h".into(), typed.into());
        h.run_steps(2);
        h.key_press(egui::Key::Enter);
        h.run_steps(4);
        assert!(h.state().dialog.is_none(), "{lang:?}: OK applies and closes");
        let (x, y, w, hh) = size_of(h.state(), id.get());
        assert!(
            (hh - 144.0).abs() < 0.01 && (w - 108.0).abs() < 0.01 && (x - 72.0).abs() < 0.01 && (y - 36.0).abs() < 0.01,
            "{lang:?}: {x} {y} {w} {hh}"
        );
        // The Format Shape pane's Size page shows the same unit.
        h.state_mut().ui.pane = Some("format".into());
        h.state_mut().ui.format_tab = "size".into();
        h.run_steps(8);
        let pane_h = if lang == Language::Vi { "5,08 cm" } else { "2\"" };
        let shown = drawn(&h);
        assert!(shown.iter().any(|t| t == pane_h), "{lang:?}: the pane shows {pane_h:?} ({pane_text:?} before): {shown:?}");
    }
    // A point typed in Vietnamese reads too.
    let id = std::cell::Cell::new(0);
    let mut h = harness((1600.0, 1000.0), Language::Vi, false, |a| {
        id.set(a.run("shape.insert", rect.clone()).unwrap()["id"].as_u64().unwrap());
        a.dialog = Some(Dialog::new("sizePosition"));
    });
    h.run_steps(8);
    h.state_mut().dialog.as_mut().unwrap().fields.insert("w".into(), "7.62".into());
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(4);
    let (_, _, w, _) = size_of(h.state(), id.get());
    assert!((w - 216.0).abs() < 0.01, "7.62 cm is 3 in: {w}");
}

/// NexusOffice (vi-VN units): junk typed into a Size and Position field in Vietnamese ("abc",
/// "1,2,3", a huge number, NaN) keeps the dialog open with a message; nothing panics and the shape
/// is unchanged.
#[test]
fn junk_lengths_in_vietnamese_never_panic() {
    use crate::dialogs::Dialog;
    for junk in ["", "abc", "1,2,3", "NaN", "inf", "1e999", &"9".repeat(400), "-", ",", "2,54 km"] {
        let mut h = harness((1600.0, 1000.0), Language::Vi, false, |a| {
            a.run("shape.insert", json!({"preset": "rect", "rect": [72.0, 36.0, 108.0, 72.0]})).unwrap();
            a.dialog = Some(Dialog::new("sizePosition"));
        });
        h.run_steps(8);
        h.state_mut().dialog.as_mut().unwrap().fields.insert("h".into(), junk.to_string());
        h.run_steps(2);
        h.key_press(egui::Key::Enter);
        h.run_steps(4);
        assert!(h.state().dialog.is_some(), "{junk:?} is not a length: the dialog stays open");
    }
}

/// The chart selected in the active presentation, if any.
fn selected_chart(app: &SlideApp) -> Option<deckcraft_model::chart::Chart> {
    app.session.active()?.selected_shapes().into_iter().find_map(|x| match &x.kind {
        deckcraft_model::ShapeKind::Chart(c) => Some((**c).clone()),
        _ => None,
    })
}

/// NexusOffice (vi-VN numbers): Chart Data is data, so a point groups thousands and a comma is
/// the decimal ("1.500.000", "1.234,5"); the cells show the values that way. A cell that isn't a
/// number is not silently dropped: OK keeps the dialog open with a Vietnamese notice and the chart
/// keeps its data.
#[test]
fn chart_data_reads_vietnamese_thousands_and_keeps_the_data_on_a_bad_cell() {
    use crate::dialogs::Dialog;
    let mut h = harness((1600.0, 1000.0), Language::Vi, false, |a| {
        let id = a.run("insert.chart", json!({})).unwrap()["id"].as_u64().unwrap();
        let series = json!([{"name": "Doanh thu", "values": [1_500_000.0, 2.5]}]);
        a.run("chart.data", json!({"id": id, "categories": ["Quý 1", "Quý 2"], "series": series})).unwrap();
        a.dialog = Some(Dialog::new("chartData"));
    });
    h.run_steps(8);
    let grid = |h: &H| -> Vec<Vec<String>> {
        h.state().dialog.as_ref().and_then(|d| serde_json::from_value(d.params["grid"].clone()).ok()).unwrap_or_default()
    };
    let g = grid(&h);
    assert_eq!(g.get(1).and_then(|r| r.get(1)).map(String::as_str), Some("1.500.000"), "{g:?}");
    assert_eq!(g.get(2).and_then(|r| r.get(1)).map(String::as_str), Some("2,5"), "{g:?}");
    let set = |h: &mut H, a: &str, b: &str| {
        let mut g = grid(h);
        g[1][1] = a.into();
        g[2][1] = b.into();
        h.state_mut().dialog.as_mut().unwrap().params = json!({"grid": g});
        h.run_steps(2);
        h.key_press(egui::Key::Enter);
        h.run_steps(4);
    };
    let values = |h: &H| selected_chart(h.state()).map(|c| c.series[0].values.clone()).unwrap_or_default();
    // A bad cell: the dialog stays, says so in Vietnamese, and nothing changes.
    set(&mut h, "1.234,5", "abc");
    assert!(h.state().dialog.is_some(), "a cell that isn't a number keeps the dialog open");
    assert!(drawn(&h).iter().any(|t| t.contains("abc") && t.contains("không phải là số")), "{:?}", drawn(&h));
    assert_eq!(values(&h), vec![Some(1_500_000.0), Some(2.5)], "the chart keeps its data");
    // Thousands and a decimal comma read as data.
    set(&mut h, "1.234,5", "-1.234");
    assert!(h.state().dialog.is_none(), "valid cells apply");
    assert_eq!(values(&h), vec![Some(1234.5), Some(-1234.0)]);
}

/// NexusOffice (vi-VN units): a length shown in centimetres and left as it is keeps its exact value:
/// OK in Paragraph and Slide Size, and leaving a Format Shape spinner, don't round it through the
/// two decimals shown (27 pt is "0,95" cm, 960 pt is "33,87" cm). Slide Size reads a typed unit.
#[test]
fn unchanged_lengths_keep_their_exact_value_in_vietnamese() {
    use crate::dialogs::Dialog;
    let mut h = harness((1600.0, 1000.0), Language::Vi, false, |a| {
        a.run("shape.insert", json!({"preset": "rect", "rect": [72.0, 36.0, 300.0, 100.0], "text": "x"})).unwrap();
        a.run("format.paragraphDialog", json!({"indentLeft": 27.0, "special": "first", "specialBy": 13.0})).unwrap();
        a.run("format.paragraphDialog", json!({})).unwrap();
    });
    h.run_steps(8);
    assert_eq!(h.state().dialog.as_ref().map(|d| d.id.as_str()), Some("paragraph"));
    assert_eq!(h.state().dialog.as_ref().and_then(|d| d.fields.get("before")).map(String::as_str), Some("0,95"));
    h.key_press(egui::Key::Enter);
    h.run_steps(4);
    assert!(h.state().dialog.is_none());
    let left = h.state().session.active().unwrap().selected_shapes().first().and_then(|s| s.text.as_ref()?.paragraphs.first()?.props.margin_left);
    assert_eq!(left, Some(27.0), "OK with the indent untouched keeps 27 pt");
    // Format Shape › Size: focusing a spinner and leaving it unchanged doesn't move the shape.
    h.state_mut().ui.pane = Some("format".into());
    h.state_mut().ui.format_tab = "size".into();
    h.run_steps(8);
    let height = |h: &H| h.state().session.active().unwrap().selected_shapes().first().and_then(|s| s.xfrm).map(|x| x.h);
    assert_eq!(height(&h), Some(100.0));
    // The pane's box (the ribbon's Shape Format group shows the same height).
    h.get_all_by_value("3,53 cm").max_by(|a, b| a.rect().min.x.total_cmp(&b.rect().min.x)).unwrap().click();
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(4);
    assert_eq!(height(&h), Some(100.0), "an unchanged spinner commits nothing (3,53 cm would be 100,06 pt)");
    h.state_mut().ui.pane = None;
    // Slide Size: OK untouched keeps 960 × 540 pt; "25,4 cm" typed is 720 pt.
    h.state_mut().dialog = Some(Dialog::new("slideSize"));
    h.run_steps(6);
    assert_eq!(h.state().dialog.as_ref().and_then(|d| d.fields.get("w")).map(String::as_str), Some("33,87"));
    h.key_press(egui::Key::Enter);
    h.run_steps(4);
    let size = h.state().session.active().unwrap().doc.slide_size;
    assert_eq!((size.width, size.height), (960.0, 540.0), "OK with the size untouched keeps it");
    h.state_mut().dialog = Some(Dialog::new("slideSize"));
    h.run_steps(6);
    h.state_mut().dialog.as_mut().unwrap().fields.insert("w".into(), "25,4 cm".into());
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(4);
    let size = h.state().session.active().unwrap().doc.slide_size;
    assert!((size.width - 720.0).abs() < 1e-6, "25,4 cm is 720 pt: {}", size.width);
}

/// NexusOffice: a malformed line spacing handed to Paragraph shows "1" and, left as it is, applies
/// single spacing (not 0); Tabs' default stop shown in centimetres and left as it is keeps its exact
/// value (27 pt is "0,95" cm, which would read back as 26,93 pt).
#[test]
fn untouched_defaults_in_paragraph_and_tabs_keep_their_value_in_vietnamese() {
    use crate::dialogs::Dialog;
    let para = |h: &H| h.state().session.active().unwrap().selected_shapes().first().and_then(|s| s.text.as_ref()?.paragraphs.first().cloned());
    let mut h = harness((1600.0, 1000.0), Language::Vi, false, |a| {
        a.run("shape.insert", json!({"preset": "rect", "rect": [72.0, 36.0, 300.0, 100.0], "text": "x"})).unwrap();
        let mut d = Dialog::new("paragraph");
        d.params = json!({"lineKind": "multiple", "lineValue": "abc"});
        a.dialog = Some(d);
    });
    h.run_steps(8);
    assert_eq!(h.state().dialog.as_ref().and_then(|d| d.fields.get("lineValue")).map(String::as_str), Some("1"));
    h.key_press(egui::Key::Enter);
    h.run_steps(4);
    assert!(h.state().dialog.is_none());
    assert_eq!(para(&h).and_then(|p| p.props.line_spacing), Some(deckcraft_model::text::Spacing::Pct(1.0)), "single, not 0");
    let mut d = Dialog::new("tabs");
    d.params = json!({"tabs": [], "defaultTab": 27.0});
    h.state_mut().dialog = Some(d);
    h.run_steps(6);
    assert_eq!(h.state().dialog.as_ref().and_then(|d| d.fields.get("default")).map(String::as_str), Some("0,95"));
    h.key_press(egui::Key::Enter);
    h.run_steps(4);
    assert!(h.state().dialog.is_none());
    assert_eq!(para(&h).and_then(|p| p.props.default_tab), Some(27.0), "the untouched default stop keeps 27 pt");
}

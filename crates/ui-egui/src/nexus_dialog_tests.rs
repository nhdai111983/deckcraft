//! NexusOffice: the dialogs' common look ([`crate::nexus_dialog`]) — visible input frames in the
//! Nexus theme, dialogs as tall as their content — and Insert › Equation driven like a person.

use deckcraft_engine::Session;
use egui::Color32;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable};
use serde_json::json;

use crate::i18n::Language;
use crate::theme::{self, Tokens, contrast_ratio};
use crate::{Services, SlideApp};

type H = Harness<'static, SlideApp>;

/// Nexus Slide as a NexusOffice tab (WPS layout, embedded) in `lang`, with `dialog` open.
fn harness(lang: Language, dialog: &str) -> H {
    let mut session = Session::new();
    session.execute("file.new", &json!({})).unwrap();
    let mut a = SlideApp::new(session, Services::default());
    a.wps = true;
    a.embedded = true;
    a.ui.language = lang;
    let mut h = Harness::builder().with_size(egui::vec2(1440.0, 900.0)).with_max_steps(64).build_ui_state(
        |ui, app: &mut SlideApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        a,
    );
    h.run_steps(12);
    h.state_mut().dialog = Some(crate::dialogs::Dialog::new(dialog));
    h.run_steps(6);
    h
}

fn shape_count(h: &H) -> usize {
    h.state().session.active().and_then(|d| d.current_slide()).map_or(0, |s| s.shapes.len())
}

fn texts(shape: &egui::Shape, out: &mut Vec<String>) {
    match shape {
        egui::Shape::Text(t) => out.push(t.galley.text().to_string()),
        egui::Shape::Vec(v) => v.iter().for_each(|s| texts(s, out)),
        _ => {}
    }
}

fn drawn(h: &H) -> Vec<String> {
    let mut out = Vec::new();
    for s in &h.output().shapes {
        texts(&s.shape, &mut out);
    }
    out
}

/// The lowest widget named `label` (the dialog's button, not the ribbon tab "Chèn" above it).
fn button<'a>(h: &'a H, label: &'a str) -> egui_kittest::Node<'a> {
    h.query_all_by_label(label).max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y)).unwrap_or_else(|| panic!("no {label:?}"))
}

fn type_text(h: &mut H, text: &str) {
    h.input_mut().events.push(egui::Event::Text(text.into()));
    h.run_steps(2);
}

/// The input of Insert › Equation has the keyboard when the dialog opens: typing goes straight in.
#[test]
fn the_equation_input_has_focus_when_the_dialog_opens() {
    let mut h = harness(Language::Vi, "equation");
    let input = h.get_by_role(egui::accesskit::Role::TextInput);
    assert!(input.is_focused(), "the equation input has the keyboard focus");
    type_text(&mut h, "a^2+b^2=c^2");
    let d = h.state().dialog.as_ref().expect("still open");
    assert_eq!(d.fields.get("eq").map(String::as_str), Some("a^2+b^2=c^2"));
}

/// Enter inserts the equation (one text box, pretty-printed); Esc closes without inserting; an
/// empty equation can't be inserted (Chèn is disabled and Enter does nothing).
#[test]
fn enter_inserts_the_equation_esc_cancels_and_empty_inserts_nothing() {
    let mut h = harness(Language::Vi, "equation");
    let before = shape_count(&h);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert!(h.state().dialog.is_some(), "Enter on an empty equation keeps the dialog open");
    assert_eq!(shape_count(&h), before);
    assert!(button(&h, "Chèn").accesskit_node().is_disabled(), "Chèn is disabled while the equation is empty");
    assert!(h.get_by_role(egui::accesskit::Role::TextInput).is_focused(), "the input keeps the keyboard");

    type_text(&mut h, "x^2");
    assert!(!button(&h, "Chèn").accesskit_node().is_disabled(), "Chèn is enabled once there is an equation");
    h.key_press(egui::Key::Enter);
    h.run_steps(4);
    assert!(h.state().dialog.is_none(), "Enter inserts and closes");
    assert_eq!(shape_count(&h), before + 1);
    let st = h.state().session.active().unwrap();
    let text = st.current_slide().unwrap().shapes.last().and_then(|s| s.text.as_ref()).map(|t| t.text()).unwrap_or_default();
    assert_eq!(text, "x²");

    let mut h = harness(Language::Vi, "equation");
    let before = shape_count(&h);
    type_text(&mut h, "y=mx+b");
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert!(h.state().dialog.is_none(), "Esc closes the dialog");
    assert_eq!(shape_count(&h), before, "Esc inserts nothing");
}

/// The dialog shows what will be inserted, live, under the input.
#[test]
fn the_equation_dialog_previews_the_result() {
    let mut h = harness(Language::Vi, "equation");
    type_text(&mut h, "a^2+b^2=c^2");
    h.run_steps(2);
    let shown = drawn(&h);
    assert!(shown.iter().any(|t| t == "Xem trước: a²+b²=c²"), "{shown:?}");
    // Exponents: a sign and digits, or one letter; what follows stays on the line.
    let mut h = harness(Language::Vi, "equation");
    type_text(&mut h, "x^-12+y^n*2");
    let shown = drawn(&h);
    assert!(shown.iter().any(|t| t == "Xem trước: x⁻¹²+yⁿ·2"), "{shown:?}");
}

/// Every text input in the Nexus theme (light, dark, any accent) has a visible 1 px frame: a
/// stroke that isn't transparent, differs from the field and the dialog behind it and reads at
/// 3:1 against both (WCAG non-text contrast); focused, the frame is the accent.
#[test]
fn text_inputs_have_a_visible_frame_in_the_nexus_theme() {
    let mut palettes = vec![Tokens::light(), Tokens::dark()];
    for c in theme::ACCENT_PRESETS.iter().map(|p| p.1).chain([Color32::WHITE, Color32::BLACK, Color32::YELLOW]) {
        palettes.push(theme::with_accent(Tokens::light(), c));
        palettes.push(theme::with_accent(Tokens::dark(), c));
    }
    for t in palettes {
        let ctx = egui::Context::default();
        theme::apply(&ctx, &t);
        let v = ctx.global_style().visuals.clone();
        let field = v.text_edit_bg_color();
        for (state, stroke) in [("inactive", v.widgets.inactive.bg_stroke), ("hovered", v.widgets.hovered.bg_stroke), ("focused", v.selection.stroke)]
        {
            assert!(stroke.width >= 1.0, "dark={} {state}: width {}", t.dark, stroke.width);
            assert_ne!(stroke.color.a(), 0, "dark={} {state}: transparent", t.dark);
            assert_ne!(stroke.color, field, "dark={} {state}: the frame is the field's colour", t.dark);
            assert_ne!(stroke.color, v.window_fill, "dark={} {state}: the frame is the dialog's colour", t.dark);
            for (what, bg) in [("field", field), ("dialog", v.window_fill)] {
                let r = contrast_ratio(stroke.color, bg);
                assert!(r >= 3.0, "dark={} accent={:?} {state} frame on {what}: {r:.2}", t.dark, t.accent);
            }
        }
        // The focused frame is the accent, or the accent shaded towards the text colour to read.
        let f = v.selection.stroke.color;
        let toward = |c: u8, a: u8| if t.dark { c >= a } else { c <= a };
        assert!(
            toward(f.r(), t.accent.r()) && toward(f.g(), t.accent.g()) && toward(f.b(), t.accent.b()),
            "dark={} {f:?} vs accent {:?}",
            t.dark,
            t.accent
        );
        // Text selected in a field stays readable on the selection's fill.
        let r = contrast_ratio(v.selection.stroke.color, v.selection.bg_fill);
        assert!(r >= 4.5, "dark={} accent={:?}: selected text {r:.2}", t.dark, t.accent);
    }
}

/// What the frame actually painted: the Equation dialog's input is drawn with a visible stroke.
#[test]
fn the_equation_input_is_painted_with_a_frame() {
    let h = harness(Language::Vi, "equation");
    let input = h.get_by_role(egui::accesskit::Role::TextInput).rect();
    let field = h.ctx.global_style().visuals.text_edit_bg_color();
    let mut framed = false;
    fn walk(s: &egui::Shape, f: &mut dyn FnMut(&egui::epaint::RectShape)) {
        match s {
            egui::Shape::Rect(r) => f(r),
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, f)),
            _ => {}
        }
    }
    for s in &h.output().shapes {
        walk(&s.shape, &mut |r| {
            if r.rect.expand(2.0).contains_rect(input)
                && r.rect.width() < input.width() + 12.0
                && r.stroke.width >= 1.0
                && r.stroke.color.a() > 0
                && r.stroke.color != field
            {
                framed = true;
            }
        });
    }
    assert!(framed, "no visible frame around the input at {input:?}");
}

/// The dialog is as tall as its content: no empty band, the buttons sit right under the input at
/// the bottom right.
#[test]
fn the_equation_dialog_is_as_tall_as_its_content() {
    let h = harness(Language::Vi, "equation");
    let window = h.ctx.memory(|m| m.area_rect(egui::Id::unique(("dialog", "equation".to_string())))).expect("the dialog is drawn");
    let cancel = button(&h, "Hủy").rect();
    let insert = button(&h, "Chèn").rect();
    let input = h.get_by_role(egui::accesskit::Role::TextInput).rect();
    assert!(window.height() < 260.0, "the dialog is {:.0} px tall", window.height());
    assert!(window.bottom() - insert.bottom() <= 24.0, "{:.0} px under the buttons", window.bottom() - insert.bottom());
    assert!(insert.top() - input.bottom() <= 72.0, "{:.0} px between the input and the buttons", insert.top() - input.bottom());
    assert!(window.right() - insert.right() <= 24.0, "Chèn is at the right edge");
    assert!(cancel.right() <= insert.left() && (cancel.center().y - insert.center().y).abs() < 1.0, "Hủy, then Chèn, on one line");
    assert!(
        (cancel.height() - insert.height()).abs() < 0.5 && cancel.width() >= 72.0 && insert.width() >= 72.0,
        "the same height and a minimum width"
    );
}

/// Insert › Date and Time in the Vietnamese interface: Tiếng Việt is the default language (the
/// choices are Tiếng Việt and English), the formats are Vietnamese, no "listed for English" note,
/// and OK inserts an automatic field in vi-VN. In English the default stays English (US).
#[test]
fn date_and_time_starts_in_the_interface_language() {
    let mut h = harness(Language::Vi, "dateTime");
    let _ = h.get_by_label("Tiếng Việt");
    let now = deckcraft_text::datetime::DateTime::now_local().unwrap();
    let long = deckcraft_text::datetime::field_text("datetime2", Some("vi-VN"), now).unwrap();
    let shown = drawn(&h);
    assert!(shown.contains(&long), "{long:?} in {shown:?}");
    assert!(!shown.iter().any(|t| t.contains("AM") || t.contains("PM") || t.contains("Monday") || t.contains("tiếng Anh")), "{shown:?}");
    h.get_by_label(&long).click();
    h.run_steps(2);
    h.get_by_label("OK").click();
    h.run_steps(4);
    assert!(h.state().dialog.is_none(), "OK inserts and closes");
    let st = h.state().session.active().unwrap();
    let run = st
        .current_slide()
        .unwrap()
        .shapes
        .iter()
        .filter_map(|x| x.text.as_ref())
        .flat_map(|t| t.paragraphs.iter().flat_map(|p| p.runs.iter()))
        .find(|r| matches!(r.kind, deckcraft_model::text::RunKind::Field { ref field } if field == "datetime2"))
        .cloned()
        .expect("a date field");
    assert_eq!(run.props.lang.as_deref(), Some("vi-VN"));
    assert_eq!(run.text, long);

    let h = harness(Language::En, "dateTime");
    let _ = h.get_by_label("English (United States)");
    assert_eq!(crate::dialogs::date_default_lang(Language::En), "en-US");
    assert_eq!(crate::dialogs::date_default_lang(Language::Vi), "vi-VN");
}

/// Every Insert dialog is as tall as its content: its lowest button sits at the bottom edge.
#[test]
fn insert_dialogs_end_right_under_their_buttons() {
    for (id, last) in [
        ("dateTime", "OK"),
        ("equation", "Chèn"),
        ("symbol", "Đóng"),
        ("hyperlink", "OK"),
        ("headerFooter", "Áp dụng cho tất cả"),
        ("table", "Chèn"),
        ("zoom", "OK"),
        ("trim", "OK"),
    ] {
        let h = harness(Language::Vi, id);
        let window = h.ctx.memory(|m| m.area_rect(egui::Id::unique(("dialog", id.to_string())))).expect("the dialog is drawn");
        let last_rect = button(&h, last).rect();
        assert!(window.bottom() - last_rect.bottom() <= 24.0, "{id}: {:.0} px under {last:?}", window.bottom() - last_rect.bottom());
        assert!(window.right() - last_rect.right() <= 24.0, "{id}: {last:?} is at the right edge");
    }
}

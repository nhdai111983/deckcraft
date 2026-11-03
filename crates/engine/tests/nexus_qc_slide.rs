//! NexusOffice QC sweep (2026-10-26), Nexus Slide: engine behaviour.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use deckcraft_engine::Session;
use deckcraft_model::text::Align;
use serde_json::json;

/// QC-SLD-08: text typed into a new shape (a rectangle, an ellipse) is centred, as in PowerPoint
/// and WPS; a text box stays left-aligned.
#[test]
fn text_typed_into_a_new_shape_is_centred() {
    for preset in ["rect", "ellipse", "roundRect"] {
        let mut s = Session::with_new();
        let id = s.execute("shape.insert", &json!({"preset": preset, "rect": [100, 100, 300, 200]})).unwrap()["id"].as_u64().unwrap();
        s.execute("text.edit", &json!({"id": id})).unwrap();
        s.type_text("Xin chào").unwrap();
        let st = s.doc().unwrap();
        let sh = st.shape(deckcraft_model::ShapeId(u32::try_from(id).unwrap())).unwrap();
        let body = sh.text.as_ref().unwrap();
        assert_eq!(body.text(), "Xin chào", "{preset}");
        assert_eq!(body.paragraphs.first().and_then(|p| p.props.align), Some(Align::Center), "{preset}");
    }
    let mut s = Session::with_new();
    let v = s.execute("insert.textBox", &json!({"rect": [100, 100, 300, 60]})).unwrap();
    s.type_text("Hộp văn bản").unwrap();
    let id = deckcraft_model::ShapeId(u32::try_from(v["id"].as_u64().unwrap()).unwrap());
    let body = s.doc().unwrap().shape(id).unwrap().text.clone().unwrap();
    assert_ne!(body.paragraphs.first().and_then(|p| p.props.align), Some(Align::Center), "a text box is not centred");
}

/// A new, untyped shape is still textless to the user (no change to save, nothing to undo) and its
/// empty centred paragraph survives a save and reopen.
#[test]
fn a_new_shape_keeps_its_centring_through_pptx() {
    let mut s = Session::with_new();
    let id = s.execute("shape.insert", &json!({"preset": "rect", "rect": [100, 100, 300, 200]})).unwrap()["id"].as_u64().unwrap();
    let id = deckcraft_model::ShapeId(u32::try_from(id).unwrap());
    assert!(s.doc().unwrap().shape(id).unwrap().text.as_ref().is_some_and(|t| t.is_empty()));
    let path = std::env::temp_dir().join(format!("nexus-qc-sld08-{}.pptx", std::process::id()));
    s.execute("file.saveAs", &json!({"path": path.to_string_lossy()})).unwrap();
    let mut back = Session::new();
    back.execute("file.open", &json!({"path": path.to_string_lossy()})).unwrap();
    let st = back.doc().unwrap();
    let sh = st.current_slide().unwrap().shapes.iter().rfind(|x| x.ph.is_none()).cloned().unwrap();
    assert_eq!(sh.text.as_ref().and_then(|t| t.paragraphs.first()).and_then(|p| p.props.align), Some(Align::Center));
    let _ = std::fs::remove_file(&path);
}

/// The sample deck opened while the interface is Vietnamese (`Session::new_language`, as the
/// NexusOffice window sets it) is in Vietnamese: its text and its language (vi-VN), not English (US).
#[test]
fn the_sample_follows_the_interface_language() {
    let mut s = Session::new();
    s.new_language = Some("vi-VN".into());
    deckcraft_engine::sample::open_sample(&mut s).unwrap();
    let st = s.doc().unwrap();
    assert_eq!(st.doc.default_lang(), Some("vi-VN"));
    let title = st.doc.slides[0].title();
    assert_ne!(title, "Northern Lights", "the sample's text is Vietnamese");
    // And a new presentation in the Vietnamese interface: vi-VN, in a font that has Vietnamese.
    let mut s = Session::new();
    s.new_language = Some("vi-VN".into());
    s.execute("file.new", &json!({})).unwrap();
    let st = s.doc().unwrap();
    assert_eq!(st.doc.default_lang(), Some("vi-VN"));
    let db = deckcraft_fonts::FontDb::global();
    for font in [st.doc.masters[0].theme.fonts.minor.latin.clone(), st.doc.masters[0].theme.fonts.major.latin.clone()] {
        // Where the font is installed (craft-fonts embeds Inter), it has every Vietnamese letter.
        if db.has_family(&font) {
            let face = db.document_face(&font, "Regular");
            for c in "ắằẳẵặấầẩẫậđếềểễệốồổỗộớờởỡợứừửữựỳỷỹỵ".chars() {
                assert!(face.covers(c), "{font:?} lacks {c}");
            }
        }
    }
}

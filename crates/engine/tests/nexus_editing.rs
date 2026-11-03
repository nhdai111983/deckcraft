//! NexusOffice: editing behaviour the NexusOffice window relies on.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use deckcraft_engine::Session;
use deckcraft_model::ShapeId;
use serde_json::json;

/// A session with a rectangle that holds no text body at all (as a shape read from a file without
/// `txBody`).
fn with_textless_shape() -> (Session, ShapeId) {
    let mut s = Session::with_new();
    let id = s.execute("shape.insert", &json!({"preset": "rect", "rect": [100, 100, 300, 200]})).unwrap()["id"].as_u64().unwrap();
    let id = ShapeId(u32::try_from(id).unwrap());
    let st = s.doc_mut().unwrap();
    let mut doc = (*st.doc).clone();
    let slide = Arc::make_mut(doc.slides.first_mut().unwrap());
    deckcraft_model::find_shape_mut(&mut slide.shapes, id).unwrap().text = None;
    st.doc = Arc::new(doc);
    (s, id)
}

/// Entering a shape that has no text body yet (Enter or F2 on it) and leaving it with Esc types
/// nothing: the presentation is not changed ("Chưa lưu" would ask to save nothing, with nothing
/// to undo). Typing then is a change, and undoing it is clean again.
#[test]
fn entering_a_shape_without_text_is_not_a_change() {
    let (mut s, id) = with_textless_shape();
    let path = std::env::temp_dir().join(format!("nexus-editing-{}.deckcraft", std::process::id()));
    s.execute("file.saveAs", &json!({"path": path.to_string_lossy()})).unwrap();
    assert!(!s.doc().unwrap().is_dirty());
    assert!(s.doc().unwrap().shape(id).unwrap().text.is_none());
    s.execute("text.edit", &json!({"id": id.0})).unwrap();
    assert!(!s.doc().unwrap().is_dirty(), "entering the shape changed nothing");
    s.execute("text.exit", &json!({})).unwrap();
    assert!(!s.doc().unwrap().is_dirty(), "nor did leaving it");
    s.execute("text.edit", &json!({"id": id.0})).unwrap();
    s.type_text("Xin chào").unwrap();
    assert!(s.doc().unwrap().is_dirty(), "typing is a change");
    s.execute("edit.undo", &json!({})).unwrap();
    assert!(!s.doc().unwrap().is_dirty(), "undone, it is as saved");
    let _ = std::fs::remove_file(&path);
}

/// A presentation with changes stays changed when a shape is entered: only a saved one is kept
/// as saved.
#[test]
fn entering_a_shape_keeps_a_changed_presentation_changed() {
    let (mut s, id) = with_textless_shape();
    assert!(s.doc().unwrap().is_dirty());
    s.execute("text.edit", &json!({"id": id.0})).unwrap();
    assert!(s.doc().unwrap().is_dirty());
}

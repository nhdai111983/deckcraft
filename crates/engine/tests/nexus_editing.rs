//! NexusOffice: editing behaviour the NexusOffice window relies on.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use deckcraft_engine::Session;
use serde_json::json;

/// Entering a shape that has no text yet (Enter or F2 on a rectangle) and leaving it with Esc
/// types nothing: the presentation is not changed ("Chưa lưu" would ask to save nothing, with
/// nothing to undo). Typing then is a change, and undoing it is clean again.
#[test]
fn entering_an_empty_shape_is_not_a_change() {
    let mut s = Session::with_new();
    let id = s.execute("shape.insert", &json!({"preset": "rect", "rect": [100, 100, 300, 200]})).unwrap()["id"].as_u64().unwrap();
    let path = std::env::temp_dir().join(format!("nexus-editing-{}.deckcraft", std::process::id()));
    s.execute("file.saveAs", &json!({"path": path.to_string_lossy()})).unwrap();
    assert!(!s.doc().unwrap().is_dirty());
    s.execute("text.edit", &json!({"id": id})).unwrap();
    assert!(!s.doc().unwrap().is_dirty(), "entering the shape changed nothing");
    s.execute("text.exit", &json!({})).unwrap();
    assert!(!s.doc().unwrap().is_dirty(), "nor did leaving it");
    s.execute("text.edit", &json!({"id": id})).unwrap();
    s.type_text("Xin chào").unwrap();
    assert!(s.doc().unwrap().is_dirty(), "typing is a change");
    s.execute("edit.undo", &json!({})).unwrap();
    assert!(!s.doc().unwrap().is_dirty(), "undone, it is as saved");
    let _ = std::fs::remove_file(&path);
}

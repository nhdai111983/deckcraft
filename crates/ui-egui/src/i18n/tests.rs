#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;

#[test]
fn languages_parse_from_tags_and_junk_is_english() {
    assert_eq!(Language::parse("vi"), Some(Language::Vi));
    assert_eq!(Language::parse("vi-VN"), Some(Language::Vi));
    assert_eq!(Language::parse(" VI_vn "), Some(Language::Vi));
    assert_eq!(Language::parse("en-US"), Some(Language::En));
    assert_eq!(Language::parse("xx"), None);
    assert_eq!(Language::parse(""), None);
    let ui: crate::UiState = serde_json::from_str(r#"{"language": 42}"#).unwrap();
    assert_eq!(ui.language, Language::En, "a malformed value reads as English");
    let ui: crate::UiState = serde_json::from_str(r#"{"language": "vi"}"#).unwrap();
    assert_eq!(ui.language, Language::Vi);
    // English is written too, so the Nexus apps' Vietnamese default doesn't override the choice.
    let saved = serde_json::to_value(crate::UiState::default()).unwrap();
    assert_eq!(saved_language(&saved), Some(Language::En));
    assert_eq!(saved_language(&serde_json::json!({"language": "vi-VN"})), Some(Language::Vi));
    assert_eq!(saved_language(&serde_json::json!({})), None);
}

#[test]
fn labels_translate_with_line_breaks_and_shortcuts() {
    let vi = Language::Vi;
    assert_eq!(vi.tr("File"), "Tệp");
    assert_eq!(vi.tr("New\nSlide"), "Trang chiếu mới");
    assert_eq!(vi.tr("Bold (Ctrl+B)"), "In đậm (Ctrl+B)");
    assert_eq!(vi.tr("Unknown label"), "Unknown label", "untranslated stays English");
    assert_eq!(Language::En.tr("File"), "File");
    assert_eq!(vi.fill("{n} slides per page", &[("n", "3")]), "3 trang chiếu mỗi trang");
}

/// Every Vietnamese entry is written in NFC (precomposed), as Vietnamese text must be.
#[test]
fn the_catalogue_is_nfc_and_has_no_empty_entries() {
    for (en, vi) in vi::ENTRIES {
        assert!(!vi.trim().is_empty(), "{en:?} has an empty translation");
        assert!(!vi.chars().any(|c| ('\u{0300}'..='\u{0323}').contains(&c)), "{en:?} → {vi:?} has a combining mark (not NFC)");
    }
    let mut keys: Vec<&str> = vi::ENTRIES.iter().map(|(en, _)| *en).collect();
    keys.sort_unstable();
    let before = keys.len();
    keys.dedup();
    assert_eq!(before, keys.len(), "a label is translated twice");
}

/// Values are filled in one pass: a value holding `{name}` stays as it is.
#[test]
fn fill_substitutes_once() {
    assert_eq!(Language::Vi.fill("Saved {path}", &[("path", "/tmp/{path}.pptx")]), "Đã lưu /tmp/{path}.pptx");
    assert_eq!(Language::En.fill("{a}{b}{", &[("a", "{b}"), ("b", "x")]), "{b}x{");
}

/// The reason a file didn't open is told apart without its name: a file called "password.pptx"
/// that isn't there is "not found", not "password-protected".
#[test]
fn open_errors_ignore_the_file_name() {
    let e = open_error(Language::Vi, "/tmp/password.pptx: No such file or directory (os error 2)", "/tmp/password.pptx");
    assert_eq!(e, "password.pptx: không tìm thấy tệp");
    let e = open_error(Language::Vi, "a.pptx is password-protected. Nexus Slide can't open password-protected presentations yet.", "/x/a.pptx");
    assert!(e.contains("mật khẩu"), "{e}");
    assert_eq!(open_error(Language::En, "boom", "x"), "boom");
}

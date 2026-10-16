//! NexusOffices: what the engine writes into the user's files names Nexus Slides, not upstream's
//! product (spec §4.4: default author, `<Application>`, the `.deckcraft` generator, the first
//! built-in theme; the PDF's Creator/Producer are tested in `deckcraft-pdf`). Format identities
//! (the `.deckcraft` extension, the manifest's `"format": "deckcraft"`) are deliberately unchanged.

use std::io::Read;

use deckcraft_engine::Session;
use serde_json::json;

/// A scratch folder of its own, removed when the test ends (pass or fail).
struct Scratch(std::path::PathBuf);

impl Scratch {
    fn new(test: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("deckcraft-nexus-brand-{test}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
    fn path(&self, name: &str) -> std::path::PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn zip_parts(path: &std::path::Path) -> Vec<(String, String)> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut out = vec![];
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let name = f.name().to_string();
        let mut s = String::new();
        if f.read_to_string(&mut s).is_ok() {
            out.push((name, s));
        }
    }
    out
}

fn part(parts: &[(String, String)], name: &str) -> String {
    parts.iter().find(|(n, _)| n == name).map(|(_, s)| s.clone()).unwrap_or_else(|| panic!("no part {name}"))
}

fn no_upstream_name(parts: &[(String, String)]) {
    for (name, text) in parts {
        assert!(!text.contains("DeckCraft") && !text.contains("ArtCraft"), "{name} names upstream: {text}");
    }
}

#[test]
fn default_author_is_nexus() {
    let mut s = Session::with_new();
    assert_eq!(s.prefs.author, "Nexus Slides User");
    s.execute("comment.add", &json!({"text": "Xin chào"})).unwrap();
    let dir = Scratch::new("author");
    let path = dir.path("comment.pptx");
    s.execute("file.saveAs", &json!({"path": path.to_str().unwrap()})).unwrap();
    let parts = zip_parts(&path);
    assert!(parts.iter().any(|(_, t)| t.contains("Nexus Slides User")), "the comment is signed with the Nexus default");
    let app = part(&parts, "docProps/app.xml");
    assert!(app.contains("<Application>Nexus Slides</Application>"), "{app}");
    no_upstream_name(&parts);
}

#[test]
fn new_presentations_use_the_nexus_theme() {
    let mut s = Session::with_new();
    assert_eq!(s.doc().unwrap().doc.masters[0].theme.name, "Nexus");
    let listed = s.execute("design.themes", &json!({})).unwrap();
    let text = listed.to_string();
    assert!(!text.contains("DeckCraft"), "{text}");
    s.execute("slide.new", &json!({"title": "Hello"})).unwrap();
    let dir = Scratch::new("theme");
    let path = dir.path("theme.pptx");
    s.execute("file.saveAs", &json!({"path": path.to_str().unwrap()})).unwrap();
    let parts = zip_parts(&path);
    let theme = part(&parts, "ppt/theme/theme1.xml");
    assert!(theme.contains("name=\"Nexus\""), "{theme}");
    no_upstream_name(&parts);
}

#[test]
fn native_files_name_nexus_as_generator_and_keep_the_format_key() {
    let mut s = Session::with_new();
    let dir = Scratch::new("native");
    let path = dir.path("deck.deckcraft");
    s.execute("file.saveAs", &json!({"path": path.to_str().unwrap()})).unwrap();
    let parts = zip_parts(&path);
    let manifest = part(&parts, "presentation.json");
    let v: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    assert_eq!(v["format"], "deckcraft", "the format identity is unchanged");
    assert!(v["generator"].as_str().unwrap().starts_with("Nexus Slides "), "{}", v["generator"]);
    // And it opens again.
    let mut t = Session::new();
    t.execute("file.open", &json!({"path": path.to_str().unwrap()})).unwrap();
}

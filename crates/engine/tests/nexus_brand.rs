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

fn master_theme(s: &Session) -> String {
    s.doc().unwrap().doc.masters[0].theme.name.clone()
}

/// Scripts and agents written for upstream name the first built-in theme "DeckCraft": every
/// lookup by name accepts it and gives the Nexus theme.
#[test]
fn upstream_theme_name_finds_the_nexus_theme() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"theme": "DeckCraft"})).unwrap();
    assert_eq!(master_theme(&s), "Nexus");

    s.execute("design.theme", &json!({"name": "Harbor"})).unwrap();
    s.execute("design.theme", &json!({"name": "deckcraft"})).unwrap();
    assert_eq!(master_theme(&s), "Nexus");

    s.execute("design.theme", &json!({"name": "Harbor"})).unwrap();
    s.execute("design.colors", &json!({"name": "DeckCraft"})).unwrap();
    assert_eq!(s.doc().unwrap().doc.masters[0].theme.colors.name, "Nexus");

    s.execute("design.effects", &json!({"name": "Soft"})).unwrap();
    let r = s.execute("design.effects", &json!({"name": "DeckCraft"})).unwrap();
    assert_eq!(r["name"], "Nexus");
    assert_eq!(s.doc().unwrap().doc.masters[0].theme.format.name, "Nexus");

    // Other names are untouched by the alias.
    assert!(s.execute("design.theme", &json!({"name": "NotATheme"})).is_err());
}

#[test]
fn photo_album_accepts_the_upstream_theme_name() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"theme": "Harbor"})).unwrap();
    // 1×1 transparent PNG.
    let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";
    s.execute("insert.photoAlbum", &json!({"pictures": [{"name": "a.png", "data": png}], "theme": "DeckCraft"})).unwrap();
    assert_eq!(master_theme(&s), "Nexus");
}

/// The alias is for lookups only: a presentation whose theme is named "DeckCraft" keeps that name.
#[test]
fn an_upstream_deck_keeps_its_theme_name() {
    let s = Session::with_new();
    let mut p: deckcraft_model::Presentation = (*s.doc().unwrap().doc).clone();
    std::sync::Arc::make_mut(&mut p.masters[0]).theme.name = "DeckCraft".into();
    let dir = Scratch::new("upstream");
    let deck = dir.path("upstream.deckcraft");
    std::fs::write(&deck, deckcraft_format::save(&p).unwrap()).unwrap();

    let mut t = Session::new();
    t.execute("file.open", &json!({"path": deck.to_str().unwrap()})).unwrap();
    assert_eq!(master_theme(&t), "DeckCraft");
    let pptx = dir.path("upstream.pptx");
    t.execute("file.saveAs", &json!({"path": pptx.to_str().unwrap()})).unwrap();
    let theme = part(&zip_parts(&pptx), "ppt/theme/theme1.xml");
    assert!(theme.contains("name=\"DeckCraft\""), "{theme}");
}

#[test]
fn pdf_names_nexus_as_creator_and_producer() {
    let mut s = Session::with_new();
    let dir = Scratch::new("pdf");
    let path = dir.path("deck.pdf");
    s.execute("file.export", &json!({"path": path.to_str().unwrap(), "format": "pdf"})).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let has = |needle: &str| {
        let utf16: Vec<u8> = needle.encode_utf16().flat_map(u16::to_be_bytes).collect();
        bytes.windows(needle.len()).any(|w| w == needle.as_bytes()) || bytes.windows(utf16.len()).any(|w| w == utf16.as_slice())
    };
    assert!(has("/Creator") && has("/Producer"), "the PDF has an Info dictionary");
    assert!(has("Nexus Slides"), "Creator/Producer name Nexus Slides");
    assert!(!has("DeckCraft"), "no upstream name in the PDF");
}

#[test]
fn own_table_styles_are_named_nexus() {
    let mut s = Session::with_new();
    s.execute("slide.new", &json!({"layout": "blank"})).unwrap();
    s.execute("insert.table", &json!({"rows": 2, "cols": 2})).unwrap();
    s.execute("table.style", &json!({"style": "light1-accent2"})).unwrap();
    let dir = Scratch::new("table");
    let path = dir.path("table.pptx");
    s.execute("file.saveAs", &json!({"path": path.to_str().unwrap()})).unwrap();
    let parts = zip_parts(&path);
    let styles = part(&parts, "ppt/tableStyles.xml");
    assert!(styles.contains("\"Nexus Light Style 1 - Accent 2\""), "{styles}");
    no_upstream_name(&parts);
}

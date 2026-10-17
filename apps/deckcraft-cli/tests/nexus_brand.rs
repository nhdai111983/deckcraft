//! NexusOffices: the CLI is installed as `nexus-slides-cli` (the app as `nexus-slides`), so what it
//! prints names those, never upstream's product, binaries or community links.

use std::process::{Command, Output};

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_deckcraft-cli")).args(args).output().unwrap()
}

fn all(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn upstream_free(text: &str) {
    // `deck.deckcraft --print` (a file name, then a flag) is fine; the app binary is not.
    for bad in ["DeckCraft", "ArtCraft", "deckcraft-cli", "`deckcraft ", " deckcraft --control", "artcraft", "discord", "storytold"] {
        assert!(!text.contains(bad), "{bad:?} in: {text}");
    }
}

#[test]
fn version_and_usage_name_nexus_slides_cli() {
    let v = all(&cli(&["--version"]));
    assert!(v.starts_with("nexus-slides-cli "), "{v}");
    let usage = all(&cli(&[]));
    assert!(usage.starts_with("nexus-slides-cli works on Nexus Slides presentations"), "{usage}");
    upstream_free(&usage);
}

#[test]
fn every_help_page_and_the_links_are_nexus() {
    let mut pages = vec![all(&cli(&["help"])), all(&cli(&["links"]))];
    for sub in ["info", "render", "render-transition", "convert", "run", "commands", "describe", "app", "mcp", "bench", "links", "version"] {
        pages.push(all(&cli(&["help", sub])));
    }
    for page in pages {
        upstream_free(&page);
    }
    let links = all(&cli(&["links"]));
    assert!(links.contains("https://github.com/nhdai111983/deckcraft"), "{links}");
    let app = all(&cli(&["app", "--port", "1", "slide.new"]));
    assert!(app.contains("no Nexus Slides app on port 1") && app.contains("`nexus-slides --control 1`"), "{app}");
}

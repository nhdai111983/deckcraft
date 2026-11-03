//! NexusOffice branding: Nexus Slide by SonnyTech. Kept in one file so upstream's files change as
//! little as possible; the fork's `nexus/BRANDING_PATCH.md` lists every upstream file touched.
//!
//! Deliberately unchanged (they are identities, not branding): crate names, `DECKCRAFT_*`
//! variables, the `.deckcraft` extension and its `application/x-deckcraft` MIME type, the file
//! format's `"format": "deckcraft"` key and the MCP server's machine name `deckcraft`.

/// The product name: window title, menus, About, messages and what is written into files.
pub const APP_NAME: &str = "Nexus Slide";
/// The suite the app belongs to.
pub const SUITE_NAME: &str = "NexusOffice";
/// The publisher.
pub const PUBLISHER: &str = "SonnyTech";
/// Desktop app id (Wayland app id, X11 WM_CLASS, macOS bundle id, Linux desktop file).
pub const APP_ID: &str = "vn.sonnytech.nexus.slides";
/// The per-user folder on macOS (`~/Library/Application Support/…`) and Windows (`%APPDATA%\…`)
/// for settings, logs, AutoRecover and the user dictionary; its own, so Nexus Slide never reads
/// or overwrites upstream's. Deliberately still "Nexus Slides" after the 2026-10-23 rename to
/// "Nexus Slide": settings, dictionaries and AutoRecover files all live there, and reading the old
/// folder as a fallback would touch several modules, so the folder keeps its name for now.
pub const SETTINGS_DIR: &str = "Nexus Slides";
/// The same folder's name under `$XDG_CONFIG_HOME` / `$XDG_DATA_HOME` (Linux, BSD).
pub const SETTINGS_DIR_XDG: &str = "nexus-slides";
/// The author comments are signed with until the user sets their name (Preferences).
pub const DEFAULT_AUTHOR: &str = "Nexus Slide User";
/// The first built-in theme (and its colour, font and effect schemes): new presentations use it.
pub const THEME_NAME: &str = "Nexus";

/// Upstream's name for the first built-in theme ([`THEME_NAME`] here).
const UPSTREAM_THEME_NAME: &str = "DeckCraft";

/// For lookups of a built-in theme or scheme by name only: upstream's name for the first built-in
/// theme (any case) finds [`THEME_NAME`], so scripts, agents and starters written for upstream keep
/// working. Every other name is returned as given. Never apply it to names that are shown or stored:
/// a presentation whose theme is called "DeckCraft" keeps that name.
pub fn theme_alias(name: &str) -> &str {
    if name.eq_ignore_ascii_case(UPSTREAM_THEME_NAME) { THEME_NAME } else { name }
}

/// The About page's attribution to upstream, which the MIT/Apache licences and ArtCraft's brand
/// licence ask for: text only, no logo, no endorsement implied. (English, Vietnamese.)
pub const ATTRIBUTION: (&str, &str) = (
    "NexusOffice is developed from the open-source apps of ArtCraft (MIT/Apache-2.0 licences). Nexus Slide is based on DeckCraft.",
    "NexusOffice được phát triển dựa trên các ứng dụng mã nguồn mở của ArtCraft (giấy phép MIT/Apache-2.0). Nexus Slide dựa trên DeckCraft.",
);
/// The About page's one-line suite sentence (English, Vietnamese).
pub const ABOUT_SUITE: (&str, &str) = ("Part of the NexusOffice suite by SonnyTech.", "Thuộc bộ ứng dụng văn phòng NexusOffice của SonnyTech.");
/// Upstream's copyright notice (`NOTICE`), kept as upstream wrote it.
pub const UPSTREAM_COPYRIGHT: &str = "MIT OR Apache-2.0 · © 2026 ArtCraft Team and the DeckCraft contributors";
/// This fork's own notice.
pub const COPYRIGHT: &str = "Nexus Slide © 2026 SonnyTech";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_the_branding_table() {
        assert_eq!(APP_NAME, "Nexus Slide");
        assert_eq!(SUITE_NAME, "NexusOffice");
        assert_eq!(PUBLISHER, "SonnyTech");
        assert_eq!(APP_ID, "vn.sonnytech.nexus.slides");
        assert_eq!(ABOUT_SUITE.0, "Part of the NexusOffice suite by SonnyTech.");
        assert_eq!(ABOUT_SUITE.1, "Thuộc bộ ứng dụng văn phòng NexusOffice của SonnyTech.");
        assert!(crate::links::GITHUB.starts_with("https://github.com/sonnytech-vn/deckcraft"));
        assert!(!SETTINGS_DIR.contains("Craft") && !SETTINGS_DIR_XDG.contains("craft"), "never upstream's settings folder");
        for link in [crate::links::APP_PAGE, crate::links::GITHUB, crate::links::ISSUES] {
            assert!(!link.contains("artcraft") && !link.contains("storytold") && !link.contains("discord"), "{link}");
        }
    }

    #[test]
    fn theme_alias_maps_only_upstreams_first_theme() {
        assert_eq!(theme_alias("DeckCraft"), THEME_NAME);
        assert_eq!(theme_alias("deckcraft"), THEME_NAME);
        assert_eq!(theme_alias("Harbor"), "Harbor");
        assert_eq!(theme_alias("DeckCraft 2"), "DeckCraft 2");
        assert_eq!(theme_alias(""), "");
    }

    /// About attributes upstream in plain text (no logo, no link) in both languages.
    #[test]
    fn attribution_names_upstream_in_text_only() {
        for text in [ATTRIBUTION.0, ATTRIBUTION.1] {
            assert!(text.contains("ArtCraft") && text.contains("DeckCraft") && text.contains("MIT/Apache-2.0"), "{text}");
            assert!(!text.contains("http"), "text only: {text}");
        }
        assert!(UPSTREAM_COPYRIGHT.contains("ArtCraft Team"), "upstream's copyright kept");
    }
}

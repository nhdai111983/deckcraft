//! NexusOffices branding: Nexus Slides by Sonnytech. Kept in one file so upstream's files change as
//! little as possible; the fork's `nexus/BRANDING_PATCH.md` lists every upstream file touched.
//!
//! Deliberately unchanged (they are identities, not branding): crate names, `DECKCRAFT_*`
//! variables, the `.deckcraft` extension and its `application/x-deckcraft` MIME type, the file
//! format's `"format": "deckcraft"` key and the MCP server's machine name `deckcraft`.

/// The product name: window title, menus, About, messages and what is written into files.
pub const APP_NAME: &str = "Nexus Slides";
/// The suite the app belongs to.
pub const SUITE_NAME: &str = "NexusOffices";
/// The publisher.
pub const PUBLISHER: &str = "Sonnytech";
/// Desktop app id (Wayland app id, X11 WM_CLASS, macOS bundle id, Linux desktop file).
pub const APP_ID: &str = "vn.sonnytech.nexus.slides";
/// The per-user folder on macOS (`~/Library/Application Support/…`) and Windows (`%APPDATA%\…`)
/// for settings, logs, AutoRecover and the user dictionary; its own, so Nexus Slides never reads
/// or overwrites upstream's.
pub const SETTINGS_DIR: &str = "Nexus Slides";
/// The same folder's name under `$XDG_CONFIG_HOME` / `$XDG_DATA_HOME` (Linux, BSD).
pub const SETTINGS_DIR_XDG: &str = "nexus-slides";
/// The author comments are signed with until the user sets their name (Preferences).
pub const DEFAULT_AUTHOR: &str = "Nexus Slides User";
/// The first built-in theme (and its colour, font and effect schemes): new presentations use it.
pub const THEME_NAME: &str = "Nexus";

/// The About page's attribution to upstream, which the MIT/Apache licences and ArtCraft's brand
/// licence ask for: text only, no logo, no endorsement implied. (English, Vietnamese.)
pub const ATTRIBUTION: (&str, &str) = (
    "NexusOffices is developed from the open-source apps of ArtCraft (MIT/Apache-2.0 licences). Nexus Slides is based on DeckCraft.",
    "NexusOffices được phát triển dựa trên các ứng dụng mã nguồn mở của ArtCraft (giấy phép MIT/Apache-2.0). Nexus Slides dựa trên DeckCraft.",
);
/// Upstream's copyright notice (`NOTICE`), kept as upstream wrote it.
pub const UPSTREAM_COPYRIGHT: &str = "MIT OR Apache-2.0 · © 2026 ArtCraft Team and the DeckCraft contributors";
/// This fork's own notice.
pub const COPYRIGHT: &str = "Nexus Slides © 2026 Sonnytech";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_the_branding_table() {
        assert_eq!(APP_NAME, "Nexus Slides");
        assert_eq!(SUITE_NAME, "NexusOffices");
        assert_eq!(PUBLISHER, "Sonnytech");
        assert_eq!(APP_ID, "vn.sonnytech.nexus.slides");
        assert!(!SETTINGS_DIR.contains("Craft") && !SETTINGS_DIR_XDG.contains("craft"), "never upstream's settings folder");
        for link in [crate::links::APP_PAGE, crate::links::GITHUB, crate::links::ISSUES] {
            assert!(!link.contains("artcraft") && !link.contains("storytold") && !link.contains("discord"), "{link}");
        }
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

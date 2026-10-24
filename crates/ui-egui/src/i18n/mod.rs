//! NexusOffice: the interface language (English, or Vietnamese for the NexusOffice suite).
//!
//! Upstream DeckCraft has no translations: its labels are English literals at each widget. This
//! module is the smallest catalogue mechanism that lets the WPS-style layout ([`crate::nexus_wps`]),
//! the File tab's pages, the status bar and the ribbon's widgets speak Vietnamese without touching
//! every call site: the widgets look their label up with [`tr`] in the language of the frame being
//! drawn ([`current`], set once per frame from [`crate::UiState::language`]). An untranslated label
//! falls back to English, so coverage can grow.
//!
//! Never pass presentation text (slide titles, file names, theme or font names) to [`tr`]: only
//! interface wording is translated.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

mod vi;

/// The interface language.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Upstream's English (the default: tests and the stand-alone DeckCraft read English).
    #[default]
    En,
    /// Tiếng Việt (the NexusOffice suite and the Nexus Slide app).
    Vi,
}

impl Language {
    pub const ALL: [Language; 2] = [Language::En, Language::Vi];

    /// The language's own name, for the language menu.
    pub fn name(self) -> &'static str {
        match self {
            Language::En => "English",
            Language::Vi => "Tiếng Việt",
        }
    }

    /// The BCP 47 tag's language part (`en`, `vi`).
    pub const fn code(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Vi => "vi",
        }
    }

    /// `en`, `vi`, `vi-VN`, `vi_VN` (any case); `None` for a language the interface doesn't speak.
    pub fn parse(code: &str) -> Option<Language> {
        let lang = code.trim().split(['-', '_']).next().unwrap_or("").to_ascii_lowercase();
        match lang.as_str() {
            "en" => Some(Language::En),
            "vi" => Some(Language::Vi),
            _ => None,
        }
    }

    /// Translates an interface label; an untranslated one stays English. A label whose lines are
    /// broken with `\n` (a large ribbon button's) is looked up with the lines joined by a space;
    /// a label followed by its shortcut in brackets (`Bold (Ctrl+B)`) keeps the shortcut.
    pub fn tr(self, text: &str) -> Cow<'_, str> {
        if self == Language::En || text.is_empty() {
            return Cow::Borrowed(text);
        }
        if let Some(t) = self.lookup(text) {
            return Cow::Borrowed(t);
        }
        if text.contains('\n') {
            let joined = text.split('\n').map(str::trim).collect::<Vec<_>>().join(" ");
            if let Some(t) = self.lookup(&joined) {
                return Cow::Borrowed(t);
            }
        }
        // "Label (Shortcut)": the label translated, the shortcut as it is.
        if let Some(open) = text.rfind(" (")
            && text.ends_with(')')
            && let (Some(label), Some(rest)) = (text.get(..open), text.get(open..))
            && let Some(t) = self.lookup(label)
        {
            return Cow::Owned(format!("{t}{rest}"));
        }
        Cow::Borrowed(text)
    }

    /// The translation of exactly `text`, if the catalogue has one.
    pub fn lookup(self, text: &str) -> Option<&'static str> {
        match self {
            Language::En => None,
            Language::Vi => vi::translate(text),
        }
    }

    /// `template` translated, with each `{name}` replaced by its value.
    pub fn fill(self, template: &str, values: &[(&str, &str)]) -> String {
        let mut out = self.tr(template).into_owned();
        for (k, v) in values {
            out = out.replace(&format!("{{{k}}}"), v);
        }
        out
    }

    /// What a new presentation is called before its number: "Presentation" (upstream), "Trình chiếu".
    pub fn untitled_prefix(self) -> &'static str {
        match self {
            Language::En => "Presentation",
            Language::Vi => "Trình chiếu ",
        }
    }
}

/// Every Vietnamese letter beyond ASCII (NFC), lower and upper case, and the đồng sign.
pub const VI_LETTERS: &str = "àáảãạăằắẳẵặâầấẩẫậđèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵ\
ÀÁẢÃẠĂẰẮẲẴẶÂẦẤẨẪẬĐÈÉẺẼẸÊỀẾỂỄỆÌÍỈĨỊÒÓỎÕỌÔỒỐỔỖỘƠỜỚỞỠỢÙÚỦŨỤƯỪỨỬỮỰỲÝỶỸỴ₫";

/// NexusOffice: the placeholder prompts the slide shows in `language` ("Bấm để thêm tiêu đề"), for
/// [`deckcraft_render::RenderOpts::prompt`]; `None` keeps upstream's English.
pub fn prompt_fn(language: Language) -> Option<fn(deckcraft_model::PhType) -> &'static str> {
    fn vi(kind: deckcraft_model::PhType) -> &'static str {
        let en = kind.prompt();
        Language::Vi.lookup(en).unwrap_or(en)
    }
    match language {
        Language::En => None,
        Language::Vi => Some(vi),
    }
}

/// Reads a saved language leniently: an unknown or malformed value (a hand-edited `ui.json`) is
/// English instead of failing every other preference with it.
pub fn lenient<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Language, D::Error> {
    let v = serde_json::Value::deserialize(d)?;
    Ok(v.as_str().and_then(Language::parse).unwrap_or_default())
}

/// The language a saved `ui.json` names, if it names one the interface speaks (so a host can tell
/// "saved English" from "nothing saved" and start in Vietnamese).
pub fn saved_language(ui_json: &serde_json::Value) -> Option<Language> {
    ui_json.get("language").and_then(serde_json::Value::as_str).and_then(Language::parse)
}

fn key() -> egui::Id {
    egui::Id::new("nexus_slide_language")
}

/// The language of the frame being drawn (English when unset).
pub fn current(ctx: &egui::Context) -> Language {
    ctx.data(|d| d.get_temp(key())).unwrap_or_default()
}

/// Sets the language of the frame being drawn; the app calls it before drawing anything.
pub fn set_current(ctx: &egui::Context, language: Language) {
    if current(ctx) != language {
        ctx.data_mut(|d| d.insert_temp(key(), language));
    }
}

/// `text` in the language of the frame being drawn.
pub fn tr<'a>(ctx: &egui::Context, text: &'a str) -> Cow<'a, str> {
    current(ctx).tr(text)
}

/// The Vietnamese catalogue (English, Vietnamese), for audits.
pub fn vi_entries() -> &'static [(&'static str, &'static str)] {
    vi::ENTRIES
}

#[cfg(test)]
mod tests;

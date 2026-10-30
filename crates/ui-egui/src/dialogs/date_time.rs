use super::lang_of;
use super::{DATE_KINDS, Dialog, SlideApp, Ui, check, json};
use crate::i18n::Language;
use crate::i18n::tl;

const DATE_LANGS: &[(&str, &str)] = &[
    ("en-US", "English (United States)"),
    ("en-GB", "English (United Kingdom)"),
    ("fr-FR", "French (France)"),
    ("ja-JP", "Japanese"),
    ("sv-SE", "Swedish"),
];

/// NexusOffice: the Date and Time dialog's languages. In the Vietnamese interface, Tiếng Việt
/// (first, the default) and English, each named in its own language; otherwise upstream's list and
/// Vietnamese.
fn date_langs(ui_lang: Language) -> Vec<(&'static str, &'static str)> {
    if ui_lang == Language::Vi {
        return vec![("vi-VN", "Tiếng Việt"), ("en-US", "English")];
    }
    DATE_LANGS.iter().copied().chain([("vi-VN", "Tiếng Việt")]).collect()
}

/// The language the Date and Time dialog starts in: the interface's.
pub fn date_default_lang(ui_lang: Language) -> &'static str {
    if ui_lang == Language::Vi { "vi-VN" } else { "en-US" }
}

/// Samples `field_text` can compute for `lang`. Named months exist for English and Vietnamese
/// (AM/PM for English); other languages keep the numeric date and the 24-hour times.
pub(super) fn date_rows(lang: &str) -> Vec<(&'static str, String)> {
    let now = deckcraft_text::datetime::DateTime::now_local();
    let mut rows = Vec::new();
    for kind in DATE_KINDS {
        if let Some(now) = now
            && let Some(sample) = deckcraft_text::datetime::field_text(kind, Some(lang), now)
        {
            rows.push((kind, sample));
        }
    }
    if rows.is_empty() {
        rows.push(("datetime1", "datetime1".into()));
    }
    rows
}

/// Render language and format controls; OK applies `insert.dateTime`, and the return value says
/// whether the dialog should close.
pub(super) fn date_time_dialog(app: &mut SlideApp, ui: &mut Ui, d: &mut Dialog) -> bool {
    use crate::nexus_dialog as nd;
    let ui_lang = lang_of(d);
    let langs = date_langs(ui_lang);
    let lang = d.get("language", date_default_lang(ui_lang));
    let rows = date_rows(&lang);
    if !rows.iter().any(|(k, _)| *k == d.get("format", "datetime1"))
        && let Some((k, _)) = rows.first()
    {
        d.fields.insert("format".into(), (*k).to_string());
    }
    let format = d.get("format", "datetime1");
    nd::section(ui, "Available formats:");
    let samples: Vec<String> = rows.iter().map(|(_, s)| s.clone()).collect();
    let (picked, double) = nd::list_box(ui, "date-formats", &samples, rows.iter().position(|(k, _)| *k == format), 260.0);
    if let Some((kind, _)) = picked.and_then(|i| rows.get(i)) {
        d.fields.insert("format".into(), (*kind).to_string());
    }
    ui.add_space(nd::GAP_M);
    let lang_label = langs.iter().find(|(k, _)| *k == lang).map(|(_, l)| *l).unwrap_or("English (United States)");
    let lang_label = tl(ui, lang_label).into_owned();
    let lang_label = lang_label.as_str();
    ui.horizontal(|ui| {
        nd::section(ui, "Language:");
        let opened = craft_ui::ComboBox::from_id_salt("date-lang").selected_text(lang_label).width(220.0).show_ui(ui, |ui| {
            for (k, l) in &langs {
                if ui.selectable_label(lang == *k, tl(ui, l)).clicked() {
                    d.fields.insert("language".into(), (*k).to_string());
                }
            }
        });
        let shown = lang_label.to_string();
        opened.response.widget_info(move || {
            let mut info = egui::WidgetInfo::labeled(egui::Role::ComboBox, true, "Language");
            info.current_text_value = Some(shown.clone());
            info
        });
    });
    // Languages without month and day names list fewer formats; English and Vietnamese list them all.
    let lower = lang.to_ascii_lowercase();
    if !(lower.starts_with("en") || lower.starts_with("vi")) {
        ui.add_space(nd::GAP_S / 2.0);
        nd::hint(ui, "Month and day names are listed for English and Vietnamese. Other languages list the numeric date and 24-hour time.");
    }
    ui.add_space(nd::GAP_S);
    let update = check(ui, d, "Update automatically", "update", true);
    let (ok, cancel) = nd::footer(ui, "OK", true);
    if ok || double {
        let lang = d.get("language", date_default_lang(ui_lang));
        let mut format = d.get("format", "datetime1");
        let probe = deckcraft_text::datetime::DateTime::new(2026, 10, 9, 20, 5, 7);
        if deckcraft_text::datetime::field_text(&format, Some(&lang), probe).is_none() {
            format = "datetime1".into();
        }
        return app.run("insert.dateTime", json!({"format": format, "update": update, "language": lang})).is_ok();
    }
    cancel
}

use super::font::len_kept;
use super::font::pt_field;
use super::font::pt_kept;
use super::lang_of;
use super::{Dialog, SlideApp, Ui, Value, buttons, choice, inches_text, json, json_text, labeled_text, pt_text, seed_once, vec2};
use crate::i18n::units;
use crate::i18n::units::LengthUnit;
use crate::i18n::{tl, tl_in};

fn seed_paragraph_fields(d: &mut Dialog) {
    let lang = lang_of(d);
    let pt_text = |v: Option<&Value>, default: f64| units::localize(lang, &pt_text(v, default)).into_owned();
    seed_once(d, "align", json_text(d.params.get("align"), "left"));
    seed_once(d, "before", inches_text(lang, d.params.get("indentLeft")));
    seed_once(d, "after", inches_text(lang, d.params.get("indentRight")));
    seed_once(d, "special", json_text(d.params.get("special"), "none"));
    seed_once(d, "by", inches_text(lang, d.params.get("specialBy")));
    // NexusOffice: what each length/number field showed, so OK with it untouched keeps the exact value.
    for key in ["before", "after", "by"] {
        let shown = d.fields.get(key).cloned().unwrap_or_default();
        seed_once(d, &format!("{key}Seed"), shown);
    }
    seed_once(d, "spaceBefore", pt_text(d.params.get("spaceBefore"), 0.0));
    seed_once(d, "spaceBeforeSeed", pt_text(d.params.get("spaceBefore"), 0.0));
    seed_once(d, "spaceAfter", pt_text(d.params.get("spaceAfter"), 0.0));
    seed_once(d, "spaceAfterSeed", pt_text(d.params.get("spaceAfter"), 0.0));
    seed_once(d, "lineKind", json_text(d.params.get("lineKind"), "multiple"));
    seed_once(d, "lineValue", pt_text(d.params.get("lineValue"), 1.0));
    seed_once(d, "lineValueSeed", pt_text(d.params.get("lineValue"), 1.0));
}

fn paragraph_return_params(d: &mut Dialog) -> Value {
    let before = len_kept(d, "before", "0").unwrap_or(0.0);
    let after = len_kept(d, "after", "0").unwrap_or(0.0);
    let by = len_kept(d, "by", "0").unwrap_or(0.0);
    let space_before = pt_kept(d, "spaceBefore", "0").unwrap_or(0.0);
    let space_after = pt_kept(d, "spaceAfter", "0").unwrap_or(0.0);
    let line_value = pt_kept(d, "lineValue", "1").unwrap_or(1.0);
    let before_same = d.get("spaceBefore", "") == d.get("spaceBeforeSeed", "");
    let after_same = d.get("spaceAfter", "") == d.get("spaceAfterSeed", "");
    json!({
        "align": d.get("align", "left"),
        "indentLeft": before,
        "indentRight": after,
        "special": d.get("special", "none"),
        "specialBy": by,
        "spaceBefore": space_before,
        "spaceBeforeKind": if before_same { d.params.get("spaceBeforeKind").and_then(Value::as_str).unwrap_or("pts") } else { "pts" },
        "spaceAfter": space_after,
        "spaceAfterKind": if after_same { d.params.get("spaceAfterKind").and_then(Value::as_str).unwrap_or("pts") } else { "pts" },
        "lineKind": d.get("lineKind", "multiple"),
        "lineValue": line_value,
        "tabs": d.params.get("tabs").cloned().unwrap_or_else(|| json!([])),
        "defaultTab": d.params.get("defaultTab").cloned().unwrap_or_else(|| json!(72.0)),
    })
}

/// NexusOffice: the symbol of the unit lengths are shown in ("in", "cm").
fn unit_symbol(ui: &Ui) -> &'static str {
    LengthUnit::of(crate::i18n::current(ui.ctx())).symbol()
}

fn line_preset_label(kind: &str, value: &str) -> &'static str {
    if kind == "exactly" {
        "Exactly"
    } else {
        match value {
            "1" => "Single",
            "1.5" => "1.5 lines",
            "2" => "Double",
            _ => "Multiple",
        }
    }
}

/// Home › Paragraph launcher. One OK applies alignment, indentation and spacing together.
pub(super) fn paragraph_dialog(app: &mut SlideApp, ui: &mut Ui, d: &mut Dialog) -> bool {
    ui.set_min_width(460.0);
    seed_paragraph_fields(d);
    ui.horizontal(|ui| {
        ui.add_sized(vec2(110.0, 20.0), egui::Label::new(tl(ui, "Alignment")));
        choice(
            ui,
            d,
            "paragraph_align",
            "Alignment",
            "align",
            &[
                ("left", "Left"),
                ("center", "Center"),
                ("right", "Right"),
                ("justify", "Justify"),
                ("justLow", "Justify low"),
                ("distributed", "Distributed"),
            ],
        );
    });
    ui.label(egui::RichText::new(tl(ui, "Indentation")).strong());
    ui.horizontal(|ui| {
        let _ = labeled_text(ui, d, "Before text", "before", 64.0);
        ui.label(unit_symbol(ui));
        ui.add_space(12.0);
        let _ = labeled_text(ui, d, "After text", "after", 64.0);
        ui.label(unit_symbol(ui));
    });
    ui.horizontal(|ui| {
        ui.add_sized(vec2(110.0, 20.0), egui::Label::new(tl(ui, "Special")));
        choice(ui, d, "paragraph_special", "Special", "special", &[("none", "None"), ("first", "First line"), ("hanging", "Hanging")]);
        ui.add_space(8.0);
        let _ = labeled_text(ui, d, "By", "by", 64.0);
        ui.label(unit_symbol(ui));
    });
    ui.label(egui::RichText::new(tl(ui, "Spacing")).strong());
    ui.horizontal(|ui| {
        let _ = labeled_text(ui, d, "Spacing before", "spaceBefore", 64.0);
        ui.label("pt");
        ui.add_space(12.0);
        let _ = labeled_text(ui, d, "Spacing after", "spaceAfter", 64.0);
        ui.label("pt");
    });
    let mut picked: Option<(&'static str, Option<&'static str>)> = None;
    ui.horizontal(|ui| {
        let field = ui.add_sized(vec2(110.0, 20.0), egui::Label::new(tl(ui, "Line spacing")));
        let kind = d.get("lineKind", "multiple");
        // NexusOffice: "1,5" in Vietnamese is the 1.5 lines preset too.
        let value = d.get("lineValue", "1").replace(',', ".");
        let presets = [
            ("Single", "multiple", Some("1")),
            ("1.5 lines", "multiple", Some("1.5")),
            ("Double", "multiple", Some("2")),
            ("Multiple", "multiple", None),
            ("Exactly", "exactly", None),
        ]
        .map(|(label, k, v)| (label, tl_in(ui, "lineSpacing", label, label), k, v));
        let current = line_preset_label(&kind, &value);
        let combo = craft_ui::ComboBox::from_id_salt("paragraph_line")
            .selected_text(tl_in(ui, "lineSpacing", current, current))
            .width(140.0)
            .show_ui(ui, |ui| {
                for (label, shown, k, v) in presets {
                    if ui.selectable_label(line_preset_label(&kind, &value) == label, shown).clicked() {
                        picked = Some((k, v));
                    }
                }
            });
        let _ = combo.response.labelled_by(field.id);
        ui.add_space(8.0);
        let _ = labeled_text(ui, d, "At", "lineValue", 64.0);
    });
    if let Some((kind, value)) = picked {
        d.fields.insert("lineKind".into(), kind.into());
        if let Some(v) = value {
            let shown = units::localize(lang_of(d), v).into_owned();
            d.fields.insert("lineValue".into(), shown);
        } else if kind == "exactly" && pt_field(d, "lineValue", "1").is_some_and(|n| n <= 9.99) {
            d.fields.insert("lineValue".into(), "18".into());
        } else if kind == "multiple" && pt_field(d, "lineValue", "1").is_some_and(|n| n > 9.99) {
            d.fields.insert("lineValue".into(), "1".into());
        }
    }
    let mut open_tabs = false;
    ui.horizontal(|ui| {
        if ui.button(tl(ui, "Tabs…")).clicked() {
            open_tabs = true;
        }
    });
    let (ok, cancel) = buttons(ui, "OK");
    if open_tabs {
        let mut tabs = Dialog::new("tabs");
        tabs.params = json!({
            "tabs": d.params.get("tabs").cloned().unwrap_or_else(|| json!([])),
            "defaultTab": d.params.get("defaultTab").cloned().unwrap_or_else(|| json!(72.0)),
            "returnId": "paragraph",
            "returnParams": paragraph_return_params(d),
        });
        app.dialog = Some(tabs);
        return true;
    }
    if ok {
        let Some(before_in) = len_kept(d, "before", "") else {
            app.set_status("Indent before text must be a number of inches");
            return false;
        };
        let Some(after_in) = len_kept(d, "after", "") else {
            app.set_status("Indent after text must be a number of inches");
            return false;
        };
        let Some(by_in) = len_kept(d, "by", "") else {
            app.set_status("Special indent must be a number of inches");
            return false;
        };
        let Some(space_before) = pt_kept(d, "spaceBefore", "") else {
            app.set_status("Spacing before must be a number");
            return false;
        };
        let Some(space_after) = pt_kept(d, "spaceAfter", "") else {
            app.set_status("Spacing after must be a number");
            return false;
        };
        let Some(line_value) = pt_kept(d, "lineValue", "") else {
            app.set_status("Line spacing must be a number");
            return false;
        };
        let mut params = json!({
            "align": d.get("align", "left"),
            "indentLeft": before_in,
            "indentRight": after_in,
            "special": d.get("special", "none"),
            "specialBy": by_in,
            "spaceBefore": space_before,
            "spaceAfter": space_after,
            "spaceBeforeKind": if d.get("spaceBefore", "") == d.get("spaceBeforeSeed", "") { d.params.get("spaceBeforeKind").and_then(Value::as_str).unwrap_or("pts") } else { "pts" },
            "spaceAfterKind": if d.get("spaceAfter", "") == d.get("spaceAfterSeed", "") { d.params.get("spaceAfterKind").and_then(Value::as_str).unwrap_or("pts") } else { "pts" },
        });
        if d.get("lineKind", "multiple") == "exactly" {
            params["lineSpacingPt"] = json!(line_value);
        } else {
            params["lineSpacing"] = json!(line_value);
        }
        if app.run("format.paragraphDialog", params).is_err() {
            return false;
        }
    }
    ok || cancel
}

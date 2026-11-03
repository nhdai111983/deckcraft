use super::find::typed_inches;
use super::lang_of;
use super::{Dialog, SlideApp, Ui, Value, buttons, field, json, vec2};
use crate::i18n::Language;
use crate::i18n::units;
use crate::i18n::{tl, tl_in};

const TAB_KINDS: [(&str, &str); 4] = [("l", "Left"), ("ctr", "Center"), ("r", "Right"), ("dec", "Decimal")];

/// Format › Tabs: the default spacing and the tab stops of the selected paragraphs, edited as a
/// list (inches, as PowerPoint's dialog) and applied together with OK.
pub(super) fn tabs_dialog(app: &mut SlideApp, ui: &mut Ui, d: &mut Dialog) -> bool {
    if !d.fields.contains_key("stops") {
        // Opened from Paragraph (or by `format.tabs` with no fields) the stops come with the
        // dialog; otherwise they are read from the caret's paragraph.
        let r = if d.params.get("tabs").is_some() { d.params.clone() } else { app.session.execute("format.ruler", &json!({})).unwrap_or_default() };
        let stops: Vec<String> = r["tabs"]
            .as_array()
            .map(|a| a.iter().filter_map(|t| Some(format!("{}:{}", t["pos"].as_f64()? / 72.0, t["align"].as_str().unwrap_or("l")))).collect())
            .unwrap_or_default();
        d.fields.insert("stops".into(), stops.join("|"));
        let default_pt = r["defaultTab"].as_f64().unwrap_or(72.0);
        let shown = match lang_of(d) {
            Language::En => format!("{}", default_pt / 72.0),
            lang => units::format_length(lang, default_pt),
        };
        d.fields.insert("defaultSeed".into(), shown.clone());
        d.fields.insert("defaultPt".into(), default_pt.to_string());
        d.fields.insert("default".into(), shown);
    }
    // NexusOffice: positions show in the interface's unit (the stops list keeps inches).
    let lang = lang_of(d);
    let at_text = |inches: f64| match lang {
        Language::En => format!("{inches:.2}\""),
        _ => format!("{} cm", units::format_length(lang, inches * 72.0)),
    };
    let mut stops: Vec<(f64, String)> = d
        .get("stops", "")
        .split('|')
        .filter_map(|x| {
            let (p, a) = x.split_once(':')?;
            Some((p.parse::<f64>().ok().filter(|v| v.is_finite())?, a.to_string()))
        })
        .collect();
    field(ui, d, "Default tab stops (in):", "default", "1");
    ui.add_space(6.0);
    ui.label(tl(ui, "Tab stops:"));
    let mut remove = None;
    egui::Grid::new("tab_stops").num_columns(3).spacing(vec2(12.0, 4.0)).show(ui, |ui| {
        if stops.is_empty() {
            ui.label(tl(ui, "(none)"));
            ui.end_row();
        }
        for (i, (pos, align)) in stops.iter().enumerate() {
            ui.label(at_text(*pos));
            let (k, l) = TAB_KINDS.iter().find(|k| k.0 == align).copied().unwrap_or(("l", "Left"));
            ui.label(tl_in(ui, "tab", k, l));
            let at = at_text(*pos);
            let tip = crate::i18n::current(ui.ctx()).fill("Clear the tab at {pos}", &[("pos", &at)]);
            if ui.small_button(tl(ui, "Clear")).on_hover_text(tip).clicked() {
                remove = Some(i);
            }
            ui.end_row();
        }
    });
    if let Some(i) = remove {
        stops.remove(i);
    }
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        let l = ui.label(tl(ui, "Tab stop position (in):"));
        let mut v = d.get("newpos", "");
        ui.add(egui::TextEdit::singleline(&mut v).desired_width(60.0)).labelled_by(l.id);
        d.fields.insert("newpos".into(), v);
    });
    let cur = d.get("newalign", "l");
    ui.horizontal(|ui| {
        ui.label(tl(ui, "Alignment:"));
        for (k, label) in TAB_KINDS {
            if ui.radio(cur == k, tl_in(ui, "tab", k, label)).clicked() {
                d.fields.insert("newalign".into(), k.into());
            }
        }
    });
    ui.horizontal(|ui| {
        if ui.button(tl(ui, "Set")).clicked()
            && let Some(p) = typed_inches(d, "newpos").filter(|v| v.is_finite() && *v >= 0.0 && *v <= 56.0)
        {
            stops.retain(|(q, _)| (q - p).abs() > 0.005);
            stops.push((p, d.get("newalign", "l")));
            stops.sort_by(|a, b| a.0.total_cmp(&b.0));
            d.fields.insert("newpos".into(), String::new());
        }
        if ui.button(tl(ui, "Clear All")).clicked() {
            stops.clear();
        }
    });
    d.fields.insert("stops".into(), stops.iter().map(|(p, a)| format!("{p}:{a}")).collect::<Vec<_>>().join("|"));
    let (ok, cancel) = buttons(ui, "OK");
    if ok {
        let set: Vec<Value> = stops.iter().map(|(p, a)| json!({"pos": p * 72.0, "align": a})).collect();
        // NexusOffice: in Vietnamese the default stop left as shown keeps its exact value.
        let untouched = lang_of(d) != Language::En && d.fields.get("defaultSeed") == d.fields.get("default");
        let exact = d.fields.get("defaultPt").and_then(|p| p.parse::<f64>().ok()).filter(|v| v.is_finite());
        let dt = match exact.filter(|_| untouched) {
            Some(pt) => pt.clamp(0.02 * 72.0, 56.0 * 72.0),
            None => typed_inches(d, "default").unwrap_or(1.0).clamp(0.02, 56.0) * 72.0,
        };
        if let Err(e) = app.run("format.tabs", json!({"set": set, "defaultTab": dt})) {
            app.set_status(e);
        } else if let Some(ret) = d.params.get_mut("returnParams").and_then(Value::as_object_mut) {
            // Back in Paragraph, its Tabs… shows what was just applied.
            ret.insert("tabs".into(), json!(set));
            ret.insert("defaultTab".into(), json!(dt));
        }
    }
    ok || cancel
}

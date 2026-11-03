use super::lang_of;
use super::{Dialog, SlideApp, Ui, Value, buttons, check, edited, exact_of, field_of, json, num_text, parse_shown, text_row};
use crate::i18n::Language;
use crate::i18n::tl;
use crate::i18n::units;

/// NexusOffice: `pt` as the dialog shows a length: inches in English (upstream's text), centimetres
/// with a decimal comma in Vietnamese.
fn len_text(d: &Dialog, pt: f64) -> String {
    match lang_of(d) {
        Language::En => num_text(pt / 72.0),
        lang => units::format_length(lang, pt),
    }
}

/// NexusOffice: a plain number (degrees, percent) as the dialog shows it.
fn num_shown(d: &Dialog, v: f64) -> String {
    units::localize(lang_of(d), &num_text(v)).into_owned()
}

/// NexusOffice: a typed length in points: inches in English (upstream), centimetres (or the unit
/// typed) in Vietnamese.
fn parse_len(d: &Dialog, s: &str) -> Option<f64> {
    match lang_of(d) {
        Language::En => parse_shown(s).map(|inches| inches * 72.0),
        lang => units::parse_length(lang, s),
    }
}

/// NexusOffice: a typed plain number (degrees, percent) in the dialog's language.
fn parse_num(d: &Dialog, s: &str) -> Option<f64> {
    match lang_of(d) {
        Language::En => parse_shown(s),
        lang => units::parse_measure(lang, s),
    }
}

fn put_dim(d: &mut Dialog, key: &str, exact: f64, shown: String) {
    d.fields.insert(key.into(), shown.clone());
    d.fields.insert(format!("seed{key}"), shown.clone());
    d.fields.insert(format!("prev{key}"), shown);
    d.fields.insert(format!("{key}Pt"), exact.to_string());
}

fn publish_dim(d: &mut Dialog, key: &str, exact: f64, shown: String) {
    d.fields.insert(key.into(), shown.clone());
    d.fields.insert(format!("seed{key}"), shown);
    d.fields.insert(format!("{key}Pt"), exact.to_string());
}

fn selection_xfrm(app: &SlideApp) -> Option<deckcraft_geom::Xfrm> {
    let st = app.session.active()?;
    let id = *st.selection.shapes.first()?;
    let sh = st.shape(id)?;
    Some(deckcraft_engine::cmd::xfrm_of(&st.doc, &st.selection, sh))
}

fn seed_size_position(app: &SlideApp, d: &mut Dialog) {
    if d.fields.contains_key("baseW") {
        return;
    }
    let num = |k: &str| d.params.get(k).and_then(Value::as_f64).filter(|v| v.is_finite());
    let sel = selection_xfrm(app);
    let x = num("x").or_else(|| sel.map(|b| b.x)).unwrap_or(0.0);
    let y = num("y").or_else(|| sel.map(|b| b.y)).unwrap_or(0.0);
    let w = num("w").or_else(|| sel.map(|b| b.w)).unwrap_or(72.0);
    let h = num("h").or_else(|| sel.map(|b| b.h)).unwrap_or(72.0);
    let rot = num("rot").or_else(|| sel.map(|b| b.rot)).unwrap_or(0.0);
    let sw = num("scaleW").unwrap_or(100.0);
    let sh = num("scaleH").unwrap_or(100.0);
    let lock = d.params.get("lockAspect").and_then(Value::as_bool).unwrap_or(false);
    let (tx, ty, tw, th) = (len_text(d, x), len_text(d, y), len_text(d, w), len_text(d, h));
    let (trot, tsw, tsh) = (num_shown(d, rot), num_shown(d, sw), num_shown(d, sh));
    put_dim(d, "x", x, tx);
    put_dim(d, "y", y, ty);
    put_dim(d, "w", w, tw);
    put_dim(d, "h", h, th);
    put_dim(d, "rot", rot, trot);
    put_dim(d, "scaleW", sw, tsw);
    put_dim(d, "scaleH", sh, tsh);
    d.fields.insert("baseW".into(), w.to_string());
    d.fields.insert("baseH".into(), h.to_string());
    let bit = if lock { "1" } else { "0" };
    d.fields.insert("lock".into(), bit.into());
    d.fields.insert("prevlock".into(), bit.into());
}

fn scale_of(pt: f64, base: f64) -> f64 {
    if base > 0.0 { pt / base * 100.0 } else { 100.0 }
}

fn current_points(d: &Dialog, key: &str) -> Option<f64> {
    if edited(d, key) { parse_len(d, &field_of(d, key)) } else { exact_of(d, key) }
}

/// Keep scale tied to size, and the other side tied when aspect is locked.
fn reconcile_size_fields(d: &mut Dialog) {
    let lock = field_of(d, "lock") == "1";
    let was_locked = field_of(d, "prevlock") == "1";
    let base_w = parse_shown(&field_of(d, "baseW")).unwrap_or(1.0);
    let base_h = parse_shown(&field_of(d, "baseH")).unwrap_or(1.0);
    let ar = if base_h > 0.0 { base_w / base_h } else { 1.0 };
    let (w_s, h_s, sw_s, sh_s) = (field_of(d, "w"), field_of(d, "h"), field_of(d, "scaleW"), field_of(d, "scaleH"));
    let (prev_w, prev_h, prev_sw, prev_sh) = (field_of(d, "prevw"), field_of(d, "prevh"), field_of(d, "prevscaleW"), field_of(d, "prevscaleH"));
    if w_s != prev_w {
        if let Some(w_pt) = parse_len(d, &w_s) {
            d.fields.insert("wPt".into(), w_pt.to_string());
            let pct = scale_of(w_pt, base_w);
            let shown = num_shown(d, pct);
            publish_dim(d, "scaleW", pct, shown);
            if lock {
                let h_pt = if ar > 0.0 { w_pt / ar } else { w_pt };
                let shown = len_text(d, h_pt);
                publish_dim(d, "h", h_pt, shown);
                let hp = scale_of(h_pt, base_h);
                let shown = num_shown(d, hp);
                publish_dim(d, "scaleH", hp, shown);
            }
        }
    } else if h_s != prev_h {
        if let Some(h_pt) = parse_len(d, &h_s) {
            d.fields.insert("hPt".into(), h_pt.to_string());
            let pct = scale_of(h_pt, base_h);
            let shown = num_shown(d, pct);
            publish_dim(d, "scaleH", pct, shown);
            if lock {
                let w_pt = h_pt * ar;
                let shown = len_text(d, w_pt);
                publish_dim(d, "w", w_pt, shown);
                let wp = scale_of(w_pt, base_w);
                let shown = num_shown(d, wp);
                publish_dim(d, "scaleW", wp, shown);
            }
        }
    } else if sw_s != prev_sw {
        if let Some(pct) = parse_num(d, &sw_s) {
            let w_pt = base_w * pct / 100.0;
            d.fields.insert("scaleWPt".into(), pct.to_string());
            let shown = len_text(d, w_pt);
            publish_dim(d, "w", w_pt, shown);
            if lock {
                let h_pt = if ar > 0.0 { w_pt / ar } else { w_pt };
                let shown = len_text(d, h_pt);
                publish_dim(d, "h", h_pt, shown);
                let shown = num_shown(d, pct);
                publish_dim(d, "scaleH", pct, shown);
            }
        }
    } else if sh_s != prev_sh {
        if let Some(pct) = parse_num(d, &sh_s) {
            let h_pt = base_h * pct / 100.0;
            d.fields.insert("scaleHPt".into(), pct.to_string());
            let shown = len_text(d, h_pt);
            publish_dim(d, "h", h_pt, shown);
            if lock {
                let w_pt = h_pt * ar;
                let shown = len_text(d, w_pt);
                publish_dim(d, "w", w_pt, shown);
                let shown = num_shown(d, pct);
                publish_dim(d, "scaleW", pct, shown);
            }
        }
    } else if lock
        && !was_locked
        && let Some(w_pt) = current_points(d, "w")
    {
        let h_pt = if ar > 0.0 { w_pt / ar } else { w_pt };
        let shown = len_text(d, h_pt);
        publish_dim(d, "h", h_pt, shown);
        let hp = scale_of(h_pt, base_h);
        let shown = num_shown(d, hp);
        publish_dim(d, "scaleH", hp, shown);
        let wp = scale_of(w_pt, base_w);
        let shown = num_shown(d, wp);
        publish_dim(d, "scaleW", wp, shown);
    }
    for key in ["x", "y", "w", "h", "rot", "scaleW", "scaleH"] {
        let v = field_of(d, key);
        d.fields.insert(format!("prev{key}"), v);
    }
    d.fields.insert("prevlock".into(), if lock { "1".into() } else { "0".into() });
}

fn shown_points(d: &Dialog, key: &str, label: &str) -> Result<f64, String> {
    if edited(d, key) {
        return parse_len(d, &field_of(d, key)).ok_or_else(|| must_be_a_number(d, label));
    }
    exact_of(d, key).ok_or_else(|| must_be_a_number(d, label))
}

fn shown_number(d: &Dialog, key: &str, label: &str) -> Result<f64, String> {
    if edited(d, key) {
        return parse_num(d, &field_of(d, key)).ok_or_else(|| must_be_a_number(d, label));
    }
    exact_of(d, key).ok_or_else(|| must_be_a_number(d, label))
}

/// NexusOffice: an issue the accessibility check reports, in `lang`.
pub(super) fn accessibility_issue(lang: Language, issue: &str) -> String {
    if let Some(ratio) = issue.strip_prefix("Hard-to-read text contrast (").and_then(|r| r.strip_suffix(":1)")) {
        return lang.fill("Hard-to-read text contrast ({ratio}:1)", &[("ratio", &units::localize(lang, ratio))]);
    }
    lang.tr(issue).into_owned()
}

/// "Width must be a number", in the dialog's language.
fn must_be_a_number(d: &Dialog, label: &str) -> String {
    let lang = lang_of(d);
    lang.fill("{label} must be a number", &[("label", &lang.tr(label))])
}

fn size_position_value(d: &Dialog) -> Result<Value, String> {
    let x = shown_points(d, "x", "Horizontal")?;
    let y = shown_points(d, "y", "Vertical")?;
    let rot = shown_number(d, "rot", "Rotation")?;
    let base_w = parse_shown(&field_of(d, "baseW")).unwrap_or(1.0);
    let base_h = parse_shown(&field_of(d, "baseH")).unwrap_or(1.0);
    let mut w = if edited(d, "w") {
        shown_points(d, "w", "Width")?
    } else if edited(d, "scaleW") {
        base_w * shown_number(d, "scaleW", "Scale width")? / 100.0
    } else {
        shown_points(d, "w", "Width")?
    };
    let mut h = if edited(d, "h") {
        shown_points(d, "h", "Height")?
    } else if edited(d, "scaleH") {
        base_h * shown_number(d, "scaleH", "Scale height")? / 100.0
    } else {
        shown_points(d, "h", "Height")?
    };
    if field_of(d, "lock") == "1" {
        let ar = if base_h > 0.0 { base_w / base_h } else { 1.0 };
        let width_drives = edited(d, "w") || edited(d, "scaleW") || (!edited(d, "h") && !edited(d, "scaleH"));
        if width_drives {
            h = if ar > 0.0 { w / ar } else { h };
        } else {
            w = h * ar;
        }
    }
    Ok(json!({"x": x, "y": y, "w": w, "h": h, "rot": rot, "lockAspect": field_of(d, "lock") == "1"}))
}

pub(super) fn size_position_dialog(app: &mut SlideApp, ui: &mut Ui, d: &mut Dialog) -> bool {
    seed_size_position(app, d);
    ui.label(egui::RichText::new(tl(ui, "Size")).strong());
    text_row(ui, d, "Height (in):", "h");
    text_row(ui, d, "Width (in):", "w");
    text_row(ui, d, "Rotation (°):", "rot");
    text_row(ui, d, "Scale height (%):", "scaleH");
    text_row(ui, d, "Scale width (%):", "scaleW");
    check(ui, d, "Lock aspect ratio", "lock", false);
    ui.add_space(8.0);
    ui.label(egui::RichText::new(tl(ui, "Position")).strong());
    ui.label(tl(ui, "From the top-left corner of the slide"));
    text_row(ui, d, "Horizontal (in):", "x");
    text_row(ui, d, "Vertical (in):", "y");
    reconcile_size_fields(d);
    let (ok, cancel) = buttons(ui, "OK");
    if ok {
        match size_position_value(d) {
            Ok(p) => return app.run("shape.sizePosition", p).is_ok(),
            Err(msg) => {
                app.set_status(msg);
                return false;
            }
        }
    }
    cancel
}

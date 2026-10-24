//! A sample deck built entirely in code (our own text, shapes and data), for demos, screenshots
//! and tests: `deckcraft --sample`.

use serde_json::{Value, json};

use crate::{Result, Session};

fn run(s: &mut Session, id: &str, p: Value) -> Result<Value> {
    s.execute(id, &p)
}

fn last_id(v: &Value) -> u64 {
    v.get("id").and_then(Value::as_u64).unwrap_or(0)
}

/// Open the sample presentation in a new document.
pub fn open_sample(s: &mut Session) -> Result<()> {
    open_sample_in(s, None)
}

/// [`open_sample`] in `language` (NexusOffice): `vi` writes the deck's Vietnamese text
/// ([`crate::nexus_vi`]); anything else is upstream's English.
pub fn open_sample_in(s: &mut Session, language: Option<&str>) -> Result<()> {
    let t = |en: &'static str| crate::nexus_vi::text(language, en);
    run(s, "file.new", json!({"theme": crate::nexus::THEME_NAME}))?;
    let st = s.doc()?;
    let title = st.current_slide().and_then(|x| x.shapes.first()).map(|x| x.id.0).unwrap_or(0);
    let sub = st.current_slide().and_then(|x| x.shapes.get(1)).map(|x| x.id.0).unwrap_or(0);
    run(s, "text.set", json!({"id": title, "text": t("Northern Lights")}))?;
    run(s, "text.set", json!({"id": sub, "text": t("A short tour of the aurora — where it comes from, when to look, and how to photograph it")}))?;
    run(s, "slide.notes", json!({"text": t("Welcome everyone. This deck was made in Nexus Slide.")}))?;
    // Decorative bands on the title slide.
    let b = run(s, "shape.insert", json!({"preset": "rect", "rect": [0, 470, 960, 70]}))?;
    run(s, "shape.fill", json!({"id": last_id(&b), "gradient": {"stops": [[0, "accent1"], [0.5, "accent4"], [1, "accent2"]], "angle": 0}}))?;
    run(s, "shape.line", json!({"id": last_id(&b), "none": true}))?;
    run(s, "shape.rename", json!({"id": last_id(&b), "name": t("Aurora Band")}))?;
    run(s, "transition.set", json!({"kind": "fade"}))?;

    // Agenda.
    run(s, "slide.new", json!({"layout": "titleAndContent", "title": t("What we'll cover")}))?;
    let body = s.doc()?.current_slide().and_then(|x| x.shapes.get(1)).map(|x| x.id.0).unwrap_or(0);
    run(
        s,
        "text.set",
        json!({"id": body, "text": t("The science: solar wind meets the magnetosphere\nWhere and when to see it\n\tHigh latitudes, dark skies, long winter nights\n\tKp index and space-weather forecasts\nPhotographing the aurora\nSafety and etiquette in the field")}),
    )?;
    run(s, "transition.set", json!({"kind": "push", "option": "u"}))?;
    run(s, "edit.select", json!({"ids": [body]}))?;
    run(s, "animation.set", json!({"effect": "fade", "class": "entrance"}))?;
    run(s, "animation.options", json!({"textBuild": "byParagraph"}))?;

    // Section header.
    let (science, gases) = (t("The Science"), t("Charged particles, magnetic fields and glowing gases"));
    run(s, "slide.new", json!({"layout": "sectionHeader", "title": science, "body": gases}))?;
    run(s, "design.background", json!({"color": "dk2"}))?;
    let st = s.doc()?;
    let ids: Vec<u64> = st.current_slide().map(|x| x.shapes.iter().map(|s| s.id.0 as u64).collect()).unwrap_or_default();
    run(s, "edit.select", json!({"ids": ids}))?;
    run(s, "format.color", json!({"color": "#FFFFFF"}))?;
    run(s, "transition.set", json!({"kind": "wipe", "option": "r"}))?;

    // Process diagram from shapes.
    run(s, "slide.new", json!({"layout": "titleOnly", "title": t("From the Sun to your sky")}))?;
    let steps = [("Solar wind", "accent2"), ("Magnetosphere", "accent1"), ("Particles collide", "accent4"), ("Light!", "accent3")];
    let mut prev: Option<u64> = None;
    for (i, (label, color)) in steps.iter().enumerate() {
        let x = 60.0 + i as f64 * 220.0;
        let v = run(s, "shape.insert", json!({"preset": if i == 3 { "star5" } else { "roundRect" }, "rect": [x, 210, 190, 120], "text": t(label)}))?;
        let id = last_id(&v);
        run(s, "shape.fill", json!({"id": id, "color": color}))?;
        run(s, "shape.line", json!({"id": id, "none": true}))?;
        run(s, "shape.effects", json!({"id": id, "shadow": "bottom"}))?;
        run(s, "edit.select", json!({"ids": [id]}))?;
        run(s, "format.size", json!({"size": 17}))?;
        run(s, "format.bold", json!({"on": true}))?;
        run(s, "animation.add", json!({"effect": "zoom", "class": "entrance", "start": if i == 0 { "onClick" } else { "afterPrevious" }}))?;
        if let Some(p) = prev {
            let _ = p;
            let a = run(s, "shape.insert", json!({"preset": "rightArrow", "rect": [x - 36.0, 255, 32, 30]}))?;
            run(s, "shape.fill", json!({"id": last_id(&a), "color": "tx2"}))?;
            run(s, "shape.line", json!({"id": last_id(&a), "none": true}))?;
        }
        prev = Some(id);
    }
    let caption = t("Oxygen glows green and red; nitrogen adds blue and purple fringes.");
    let cap = run(s, "insert.textBox", json!({"rect": [60, 380, 840, 60], "text": caption}))?;
    run(s, "edit.select", json!({"ids": [last_id(&cap)]}))?;
    run(s, "format.alignCenter", json!({}))?;
    run(s, "format.size", json!({"size": 20}))?;
    run(s, "format.italic", json!({"on": true}))?;
    run(s, "transition.set", json!({"kind": "morph"}))?;

    // Chart.
    run(s, "slide.new", json!({"layout": "titleOnly", "title": t("Best months to look (clear nights)")}))?;
    run(
        s,
        "insert.chart",
        json!({"type": "column", "rect": [80, 140, 800, 360], "title": t("Average clear, dark nights per month"), "categories": [t("Sep"), t("Oct"), t("Nov"), t("Dec"), t("Jan"), t("Feb"), t("Mar")], "series": [{"name": "Tromsø", "values": [6, 8, 7, 6, 7, 9, 11]}, {"name": "Fairbanks", "values": [9, 10, 8, 9, 11, 12, 13]}, {"name": "Yellowknife", "values": [8, 11, 9, 10, 12, 13, 14]}]}),
    )?;
    run(s, "transition.set", json!({"kind": "fade"}))?;

    // Table.
    run(s, "slide.new", json!({"layout": "titleOnly", "title": t("Camera settings that work")}))?;
    run(
        s,
        "insert.table",
        json!({"rows": 5, "cols": 3, "rect": [100, 150, 760, 260], "data": [[t("Setting"), t("Faint aurora"), t("Bright, active aurora")], [t("Aperture"), t("f/1.4 – f/2.8"), t("f/2.8")], [t("Shutter"), t("8 – 15 s"), t("1 – 4 s")], ["ISO", "1600 – 3200", "800 – 1600"], [t("Focus"), t("Manual, on a bright star"), t("Manual, on a bright star")]]}),
    )?;
    run(s, "transition.set", json!({"kind": "cover", "option": "l"}))?;

    // Two content: comparison with shapes.
    run(s, "slide.new", json!({"layout": "twoContent", "title": t("Field checklist")}))?;
    let st = s.doc()?;
    let (l, r) =
        st.current_slide().map(|x| (x.shapes.get(1).map(|s| s.id.0).unwrap_or(0), x.shapes.get(2).map(|s| s.id.0).unwrap_or(0))).unwrap_or((0, 0));
    run(s, "text.set", json!({"id": l, "text": t("Bring\n\tTripod and spare batteries\n\tHeadlamp with a red mode\n\tWarm layers and a thermos")}))?;
    run(
        s,
        "text.set",
        json!({"id": r, "text": t("Remember\n\tCheck the forecast and cloud cover\n\tLet your eyes adapt for 20 minutes\n\tLeave no trace")}),
    )?;
    run(s, "transition.set", json!({"kind": "split"}))?;

    // Closing.
    run(s, "slide.new", json!({"layout": "title", "title": t("Clear skies!"), "body": t("Made with Nexus Slide — free and open source")}))?;
    let moon = run(s, "shape.insert", json!({"preset": "moon", "rect": [820, 40, 80, 120]}))?;
    run(s, "shape.fill", json!({"id": last_id(&moon), "color": "accent6"}))?;
    run(s, "shape.line", json!({"id": last_id(&moon), "none": true}))?;
    run(s, "shape.effects", json!({"id": last_id(&moon), "glow": {"color": "accent6", "radius": 10}}))?;
    run(s, "transition.set", json!({"kind": "fade"}))?;

    run(s, "section.add", json!({"name": t("Introduction"), "at": 0}))?;
    run(s, "section.add", json!({"name": t("Science"), "at": 2}))?;
    run(s, "section.add", json!({"name": t("In the Field"), "at": 5}))?;
    run(s, "slide.go", json!({"index": 0}))?;
    run(s, "edit.deselect", json!({}))?;
    run(s, "edit.deselect", json!({}))?;
    // The sample opens clean: no undo history, not dirty.
    if let Some(st) = s.active_mut() {
        st.history.undo.clear();
        st.history.redo.clear();
        st.saved_doc = st.doc.clone();
        st.title = t("Northern Lights").into();
    }
    Ok(())
}

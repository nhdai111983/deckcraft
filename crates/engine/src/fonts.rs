//! Fonts a presentation uses that this machine doesn't have, so opening it can say so (its text
//! draws in a stand-in; see `FontDb::document_face`).

use std::collections::HashSet;

use deckcraft_collate::{fold_key, text_eq};
use deckcraft_fonts::{FontDb, sorted_unique_names};
use deckcraft_model::resolve::{self, Ctx};
use deckcraft_model::text::{Bullet, TextBody};
use deckcraft_model::{Presentation, Shape, ShapeKind};

/// Load each embedded face into the font database, so text in that family draws with the file's
/// font on a machine that doesn't have it. A face already loaded (bundled or installed) stays;
/// the part bytes are kept either way. MicroType Express subsets have no TrueType or OpenType
/// font to load and are left for the round trip.
pub fn install_embedded_fonts(p: &Presentation) {
    let db = FontDb::global();
    for font in &p.embedded_fonts {
        for id in font.faces().into_iter().flatten() {
            let Some(item) = p.media(id) else { continue };
            let Some(sfnt) = deckcraft_fonts::embedded_sfnt(&item.data) else { continue };
            db.add_font(sfnt.to_vec());
        }
    }
}

/// The font families the slides' text is drawn in (theme fonts resolved, bullet fonts included)
/// that aren't installed or bundled, in list order (see [`sorted_unique_names`]): names that differ
/// only in case or in NFC/NFD encoding are one font, shown as first met.
pub fn missing_fonts(p: &Presentation) -> Vec<String> {
    let db = FontDb::global();
    sorted_unique_names(used_fonts(p).into_iter().filter(|family| !db.has_family(family)).collect())
}

/// The families the slides' text runs and bullets are drawn in, in the order they are met. Names
/// that are the same text (case and NFC/NFD aside) count once, under the spelling met first.
pub fn used_fonts(p: &Presentation) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut add = |f: String| {
        let f = f.trim().to_string();
        if !f.is_empty() && seen.insert(fold_key(&f)) {
            out.push(f);
        }
    };
    for slide in &p.slides {
        let Some(ctx) = Ctx::for_slide(p, slide) else { continue };
        deckcraft_model::walk(&slide.shapes, &mut |s, _| {
            if let Some(body) = &s.text {
                body_fonts(&ctx, s, body, &mut add);
            }
            if let ShapeKind::Table(t) = &s.kind {
                for cell in t.rows.iter().flat_map(|r| &r.cells) {
                    body_fonts(&ctx, s, &cell.text, &mut add);
                }
            }
        });
    }
    out
}

fn body_fonts(ctx: &Ctx, s: &Shape, body: &TextBody, add: &mut dyn FnMut(String)) {
    for para in &body.paragraphs {
        let has_text = para.runs.iter().any(|r| !r.text.trim().is_empty());
        if !has_text {
            continue;
        }
        for run in para.runs.iter().filter(|r| !r.text.trim().is_empty()) {
            let r = resolve::run(ctx, s, para, &run.props);
            add(resolve::font_family(ctx, &r));
        }
        let pp = resolve::para(ctx, s, para);
        if matches!(pp.bullet, Some(Bullet::Char { .. }))
            && let Some(f) = pp.bullet_font.as_deref()
        {
            add(ctx.master.theme.font(f));
        }
    }
}

/// Replace the font family `from` with `to` everywhere it is named explicitly: text runs (and
/// their east Asian, complex script and symbol faces), bullet fonts, paragraph end marks, list
/// styles of shapes, layouts and masters, the master text styles, the deck default style, speaker
/// notes, tables and the theme fonts. Names compare as text, ignoring case and NFC/NFD encoding;
/// theme references (`+mn-lt`) are left alone. Returns how many names changed.
pub fn replace_font(p: &mut Presentation, from: &str, to: &str) -> usize {
    use std::sync::Arc;
    let from = from.trim();
    let to = to.trim();
    if from.is_empty() || to.is_empty() || from.starts_with('+') || from == to {
        return 0;
    }
    let mut n = 0usize;
    let mut fix = |f: &mut Option<String>| {
        if f.as_deref().is_some_and(|x| text_eq(x.trim(), from)) {
            *f = Some(to.to_string());
            n = n.saturating_add(1);
        }
    };
    replace_list(&mut p.default_text_style, &mut fix);
    for slide in &mut p.slides {
        let sl = Arc::make_mut(slide);
        replace_shapes(&mut sl.shapes, &mut fix);
        replace_body(&mut sl.notes, &mut fix);
    }
    let mut theme_hits = 0usize;
    let masters = p.masters.iter_mut().chain(p.notes_master.iter_mut()).chain(p.handout_master.iter_mut());
    for m in masters {
        let m = Arc::make_mut(m);
        replace_shapes(&mut m.shapes, &mut fix);
        for l in [&mut m.title_style, &mut m.body_style, &mut m.other_style] {
            replace_list(l, &mut fix);
        }
        for lay in &mut m.layouts {
            replace_shapes(&mut lay.shapes, &mut fix);
        }
        for set in [&mut m.theme.fonts.major, &mut m.theme.fonts.minor] {
            for face in [&mut set.latin, &mut set.ea, &mut set.cs].into_iter().chain(set.scripts.iter_mut().map(|s| &mut s.typeface)) {
                if text_eq(face.trim(), from) {
                    *face = to.to_string();
                    theme_hits = theme_hits.saturating_add(1);
                }
            }
        }
    }
    n.saturating_add(theme_hits)
}

fn replace_shapes(shapes: &mut [Shape], fix: &mut dyn FnMut(&mut Option<String>)) {
    for s in shapes {
        if let Some(body) = &mut s.text {
            replace_body(body, fix);
        }
        if let ShapeKind::Table(t) = &mut s.kind {
            for cell in t.rows.iter_mut().flat_map(|r| &mut r.cells) {
                replace_body(&mut cell.text, fix);
            }
        }
        if let Some(children) = s.children_mut() {
            replace_shapes(children, fix);
        }
    }
}

fn replace_body(body: &mut TextBody, fix: &mut dyn FnMut(&mut Option<String>)) {
    replace_list(&mut body.list_style, fix);
    for para in &mut body.paragraphs {
        fix(&mut para.props.bullet_font);
        replace_run(&mut para.end_props, fix);
        for run in &mut para.runs {
            replace_run(&mut run.props, fix);
        }
    }
}

fn replace_list(l: &mut deckcraft_model::text::ListStyle, fix: &mut dyn FnMut(&mut Option<String>)) {
    for lvl in l.levels.iter_mut().flatten() {
        fix(&mut lvl.para.bullet_font);
        replace_run(&mut lvl.run, fix);
    }
}

fn replace_run(r: &mut deckcraft_model::text::RunProps, fix: &mut dyn FnMut(&mut Option<String>)) {
    for f in [&mut r.font, &mut r.font_ea, &mut r.font_cs, &mut r.font_sym] {
        fix(f);
    }
}

#[cfg(test)]
mod tests {
    use deckcraft_model::Presentation;
    use deckcraft_model::text::{Paragraph, Run, TextBody};

    use super::*;

    /// A one-slide deck whose first shape has one run per name, each drawn in that font.
    fn deck_in_fonts(names: &[&str]) -> Presentation {
        let mut p = Presentation::default();
        let slide = std::sync::Arc::make_mut(&mut p.slides[0]);
        let mut body = TextBody::from_text("x");
        body.paragraphs = names
            .iter()
            .map(|n| {
                let mut run = Run::new("Hello");
                run.props.font = Some((*n).into());
                Paragraph { runs: vec![run], ..Default::default() }
            })
            .collect();
        slide.shapes[0].text = Some(body);
        p
    }

    fn strs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn font_lists_merge_case_and_nfd() {
        let names = ["Qqq Zapfino", "Qqq Đồ Hoạ", "Qqq arial", "Qqq Ánh Sáng", "Qqq Arial", "Qqq đồ hoạ", "Qqq A\u{301}nh Sáng"];
        let p = deck_in_fonts(&names);
        let want = strs(&["Qqq Ánh Sáng", "Qqq arial", "Qqq Đồ Hoạ", "Qqq Zapfino"]);
        assert_eq!(missing_fonts(&p), want);
        // In the document's own order the first spelling of each is kept.
        let used: Vec<String> = used_fonts(&p).into_iter().filter(|f| f.starts_with("Qqq ")).collect();
        assert_eq!(used, strs(&["Qqq Zapfino", "Qqq Đồ Hoạ", "Qqq arial", "Qqq Ánh Sáng"]));
        assert_eq!(sorted_unique_names(used), want);
    }

    #[test]
    fn ascii_font_order_unchanged() {
        let p = deck_in_fonts(&["Qqq Times New Roman", "Qqq calibri", "Qqq Arial", "Qqq arial black"]);
        assert_eq!(missing_fonts(&p), strs(&["Qqq Arial", "Qqq arial black", "Qqq calibri", "Qqq Times New Roman"]));
    }

    #[test]
    fn replace_matches_nfd_and_case() {
        let mut p = deck_in_fonts(&["Đô\u{300} Hoa\u{323}", "Other Qqq"]);
        assert_eq!(replace_font(&mut p, " đồ hoạ ", "Lato"), 1);
        assert_eq!(used_fonts(&p).iter().filter(|f| f.as_str() == "Lato").count(), 1);
        assert!(!used_fonts(&p).iter().any(|f| f.contains("Hoa") || f.contains("Hoạ")));
        // A different word with the same letters but another accent is not a match.
        let mut p = deck_in_fonts(&["Đồ Hoả"]);
        assert_eq!(replace_font(&mut p, "đồ hoạ", "Lato"), 0);
    }

    #[test]
    fn font_names_hostile_never_panic() {
        let marks = format!("e{}", "\u{301}".repeat(10_000));
        let big = "Ế".repeat(1 << 19);
        let names = ["\u{10FFFF}", marks.as_str(), "", "   ", big.as_str(), marks.as_str(), big.as_str(), "\u{0}"];
        let mut p = deck_in_fonts(&names);
        // Empty and blank names are not fonts; the rest merge by equality.
        let used = used_fonts(&p);
        for n in ["\u{10FFFF}", marks.as_str(), big.as_str(), "\u{0}"] {
            assert_eq!(used.iter().filter(|f| f.as_str() == n).count(), 1);
        }
        assert!(used.iter().all(|f| !f.trim().is_empty()));
        assert!(missing_fonts(&p).len() >= 4);
        for from in [marks.as_str(), big.as_str(), "\u{10FFFF}", "e\u{301}", " "] {
            let _ = replace_font(&mut p, from, "Lato");
        }
    }
}

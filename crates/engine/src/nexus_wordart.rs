//! NexusOffice: the WordArt styles of Nexus Slide (Insert › WordArt, Shape Format › WordArt
//! Styles), as data, and the commands that use them.
//!
//! Every style is our own, built from the NexusOffice palette (Amber, Rose, Indigo, Violet, Sky,
//! Emerald, Cyan and Obsidian, the colours of the suite's design system); none copies another
//! product's presets, their order, names or looks. A style sets only the look of the text (fill,
//! outline, shadow, glow, reflection, weight), so restyling keeps the words, font and size.
//!
//! `insert.wordArt {text?, style?}` puts a WordArt box on free space on the slide (centred, moved
//! off text, pictures and title placeholders it would cover), sized to its text, with the text
//! selected so typing replaces it. `format.wordArtStyle {style}` restyles the selected text or
//! shapes. `style` is a style id ([`STYLES`]) or, as upstream's command took, a number (any
//! number picks a style).

use deckcraft_color::{ColorTransform, Rgba};
use deckcraft_geom::{Rect, Xfrm};
use deckcraft_model::style::{Glow, Gradient, GradientShape, GradientStop, Reflection, Shadow};
use deckcraft_model::text::{Align, RunProps};
use deckcraft_model::{ColorRef, Fill, Line, PhType, Shape, ShapeKind};
use serde_json::{Value, json};

use crate::cmd::{CommandSpec, bad, cmd, has_text_or_shapes};
use crate::{Result, Session, TextSel};

/// The NexusOffice palette (the suite's design system).
pub mod palette {
    use deckcraft_color::Rgba;
    pub const AMBER: Rgba = Rgba::rgb(0xF5, 0x9E, 0x0B);
    pub const AMBER_INK: Rgba = Rgba::rgb(0xB4, 0x53, 0x09);
    pub const ROSE: Rgba = Rgba::rgb(0xF4, 0x3F, 0x5E);
    pub const ROSE_INK: Rgba = Rgba::rgb(0xBE, 0x12, 0x3C);
    pub const INDIGO: Rgba = Rgba::rgb(0x63, 0x66, 0xF1);
    pub const VIOLET: Rgba = Rgba::rgb(0x8B, 0x5C, 0xF6);
    pub const SKY: Rgba = Rgba::rgb(0x0E, 0xA5, 0xE9);
    pub const EMERALD: Rgba = Rgba::rgb(0x10, 0xB9, 0x81);
    pub const CYAN: Rgba = Rgba::rgb(0x06, 0xB6, 0xD4);
    pub const OBSIDIAN: Rgba = Rgba::rgb(0x0F, 0x17, 0x2A);
    pub const WHITE: Rgba = Rgba::WHITE;
}

/// The text a new WordArt holds (English, Vietnamese).
pub const DEFAULT_TEXT: (&str, &str) = ("Your text here", "Nhập chữ của bạn vào đây");

/// Size of a new WordArt's text, points.
pub const SIZE: f64 = 48.0;

/// How a style fills its letters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paint {
    /// No fill: only the outline draws.
    None,
    Solid(Rgba),
    /// Linear gradient: angle (degrees clockwise from left → right) and up to three stops.
    Gradient(f64, &'static [(f64, Rgba)]),
}

/// Outer shadow: colour, opacity (0–1), blur, distance (points), direction (degrees).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowSpec {
    pub color: Rgba,
    pub alpha: f64,
    pub blur: f64,
    pub dist: f64,
    pub dir: f64,
}

/// A WordArt style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WordArtStyle {
    /// Stable id (`insert.wordArt {style}`).
    pub id: &'static str,
    /// Name (English, Vietnamese) for tooltips.
    pub name: (&'static str, &'static str),
    pub fill: Paint,
    /// Outline colour and width (points).
    pub outline: Option<(Rgba, f64)>,
    pub shadow: Option<ShadowSpec>,
    /// Glow colour, opacity (0–1) and radius (points).
    pub glow: Option<(Rgba, f64, f64)>,
    pub reflection: bool,
}

use palette::*;

const SOFT: ShadowSpec = ShadowSpec { color: OBSIDIAN, alpha: 0.35, blur: 6.0, dist: 3.0, dir: 90.0 };
const LIFT: ShadowSpec = ShadowSpec { color: OBSIDIAN, alpha: 0.4, blur: 4.0, dist: 3.0, dir: 45.0 };

/// The styles, in gallery order (four to a row): solid fills, then gradients, then effects.
pub const STYLES: &[WordArtStyle] = &[
    WordArtStyle {
        id: "amber",
        name: ("Solid amber, soft shadow", "Tô đặc hổ phách, đổ bóng mềm"),
        fill: Paint::Solid(AMBER),
        outline: None,
        shadow: Some(SOFT),
        glow: None,
        reflection: false,
    },
    WordArtStyle {
        id: "indigoEdge",
        name: ("Solid indigo, white outline, shadow", "Tô đặc chàm, viền trắng, đổ bóng"),
        fill: Paint::Solid(INDIGO),
        outline: Some((WHITE, 1.5)),
        shadow: Some(LIFT),
        glow: None,
        reflection: false,
    },
    WordArtStyle {
        id: "roseLine",
        name: ("Rose outline, no fill", "Chỉ viền hồng, không tô"),
        fill: Paint::None,
        outline: Some((ROSE, 1.75)),
        shadow: None,
        glow: None,
        reflection: false,
    },
    WordArtStyle {
        id: "skyNeon",
        name: ("White, sky blue outline and glow", "Chữ trắng, viền và phát sáng xanh trời"),
        fill: Paint::Solid(WHITE),
        outline: Some((SKY, 1.25)),
        shadow: None,
        glow: Some((SKY, 0.6, 8.0)),
        reflection: false,
    },
    WordArtStyle {
        id: "brand",
        name: ("Gradient indigo, cyan and amber", "Chuyển sắc chàm, lam ngọc và hổ phách"),
        fill: Paint::Gradient(0.0, &[(0.0, INDIGO), (0.5, CYAN), (1.0, AMBER)]),
        outline: None,
        shadow: None,
        glow: None,
        reflection: false,
    },
    WordArtStyle {
        id: "sunset",
        name: ("Gradient amber to rose, soft shadow", "Chuyển sắc hổ phách sang hồng, đổ bóng mềm"),
        fill: Paint::Gradient(90.0, &[(0.0, AMBER), (1.0, ROSE)]),
        outline: None,
        shadow: Some(SOFT),
        glow: None,
        reflection: false,
    },
    WordArtStyle {
        id: "lagoon",
        name: ("Gradient emerald to sky blue, white outline, shadow", "Chuyển sắc lục bảo sang xanh trời, viền trắng, đổ bóng"),
        fill: Paint::Gradient(90.0, &[(0.0, EMERALD), (1.0, SKY)]),
        outline: Some((WHITE, 1.0)),
        shadow: Some(LIFT),
        glow: None,
        reflection: false,
    },
    WordArtStyle {
        id: "dusk",
        name: ("Gradient violet to indigo, reflection", "Chuyển sắc tím sang chàm, phản chiếu"),
        fill: Paint::Gradient(90.0, &[(0.0, VIOLET), (1.0, INDIGO)]),
        outline: None,
        shadow: None,
        glow: None,
        reflection: true,
    },
    WordArtStyle {
        id: "obsidian",
        name: ("Charcoal, reflection", "Chữ than chì, phản chiếu"),
        fill: Paint::Solid(OBSIDIAN),
        outline: None,
        shadow: None,
        glow: None,
        reflection: true,
    },
    WordArtStyle {
        id: "retro",
        name: ("Amber, charcoal outline, hard shadow", "Hổ phách, viền than chì, bóng đổ cứng"),
        fill: Paint::Solid(AMBER),
        outline: Some((OBSIDIAN, 1.25)),
        shadow: Some(ShadowSpec { color: OBSIDIAN, alpha: 1.0, blur: 0.0, dist: 4.0, dir: 45.0 }),
        glow: None,
        reflection: false,
    },
    WordArtStyle {
        id: "violetGlow",
        name: ("Solid violet, soft violet glow", "Tô đặc tím, phát sáng tím nhạt"),
        fill: Paint::Solid(VIOLET),
        outline: None,
        shadow: None,
        glow: Some((VIOLET, 0.35, 10.0)),
        reflection: false,
    },
    WordArtStyle {
        id: "sticker",
        name: ("Solid rose, white outline, hard shadow", "Tô đặc hồng, viền trắng, bóng đổ cứng"),
        fill: Paint::Solid(ROSE),
        outline: Some((WHITE, 1.75)),
        shadow: Some(ShadowSpec { color: ROSE_INK, alpha: 1.0, blur: 0.0, dist: 3.0, dir: 45.0 }),
        glow: None,
        reflection: false,
    },
];

/// The style with `id`.
pub fn find(id: &str) -> Option<&'static WordArtStyle> {
    STYLES.iter().find(|s| s.id == id)
}

/// The style a `style` param names: an id, or any whole number (upstream's `0..`, wrapping).
/// Absent is the first style.
pub fn style_param(p: &Value, cmd: &str) -> Result<&'static WordArtStyle> {
    let first = STYLES.first().ok_or_else(|| bad(cmd, "no WordArt styles"))?;
    match p.get("style") {
        None | Some(Value::Null) => Ok(first),
        Some(Value::String(id)) => find(id).ok_or_else(|| bad(cmd, format!("unknown WordArt style `{}`; one of {}", clip(id), ids()))),
        Some(v) => v
            .as_u64()
            .and_then(|n| STYLES.get(usize::try_from(n % STYLES.len() as u64).ok()?))
            .ok_or_else(|| bad(cmd, format!("`style` is a style id ({}) or a whole number", ids()))),
    }
}

fn ids() -> String {
    STYLES.iter().map(|s| s.id).collect::<Vec<_>>().join(", ")
}

/// At most 40 characters of hostile input for an error message.
fn clip(s: &str) -> String {
    s.chars().take(40).collect()
}

fn color(c: Rgba, alpha: f64) -> ColorRef {
    let base = ColorRef::rgb(c);
    if alpha >= 1.0 { base } else { base.with(ColorTransform::Alpha((alpha.clamp(0.0, 1.0) * 100_000.0).round() as i32)) }
}

fn shadow(s: &ShadowSpec) -> Shadow {
    Shadow {
        color: color(s.color, s.alpha),
        blur: s.blur,
        dist: s.dist,
        dir: s.dir,
        inner: false,
        sx: 1.0,
        sy: 1.0,
        kx: 0.0,
        ky: 0.0,
        align: String::new(),
        rotate_with_shape: false,
    }
}

impl WordArtStyle {
    /// The run look this style gives: fill, outline, shadow, glow, reflection and bold, every other
    /// effect cleared.
    pub fn look(&self) -> RunProps {
        RunProps {
            bold: Some(true),
            fill: Some(match self.fill {
                Paint::None => Fill::None,
                Paint::Solid(c) => Fill::solid(color(c, 1.0)),
                Paint::Gradient(angle, stops) => Fill::Gradient(Gradient {
                    stops: stops.iter().map(|(pos, c)| GradientStop { pos: *pos, color: color(*c, 1.0) }).collect(),
                    shape: GradientShape::Linear { angle, scaled: false },
                    rotate_with_shape: true,
                    tile: [0.0; 4],
                }),
            }),
            outline: Some(match self.outline {
                Some((c, w)) => Line::solid(color(c, 1.0), w),
                None => Line::none(),
            }),
            shadow: self.shadow.as_ref().map(shadow),
            glow: self.glow.map(|(c, a, r)| Glow { color: color(c, a), radius: r }),
            reflection: self.reflection.then_some(Reflection { blur: 0.5, start_alpha: 0.5, end_alpha: 0.0, end_pos: 0.6, dist: 2.0 }),
            ..Default::default()
        }
    }

    /// Puts this style's look on `r` (keeping its font, size and the rest).
    pub fn apply(&self, r: &mut RunProps) {
        let l = self.look();
        r.bold = l.bold;
        r.fill = l.fill;
        r.outline = l.outline;
        r.shadow = l.shadow;
        r.glow = l.glow;
        r.reflection = l.reflection;
        r.inner_shadow = None;
        r.soft_edge = None;
        r.highlight = None;
    }

    /// Does `r` have exactly this style's look?
    pub fn matches(&self, r: &RunProps) -> bool {
        let l = self.look();
        r.fill == l.fill && r.outline == l.outline && r.shadow == l.shadow && r.glow == l.glow && r.reflection == l.reflection
    }
}

/// The style whose look the first run of `shape`'s text has, if any.
pub fn style_of(shape: &Shape) -> Option<&'static WordArtStyle> {
    let run = shape.text.as_ref()?.paragraphs.iter().flat_map(|p| p.runs.iter()).next()?;
    STYLES.iter().find(|s| s.matches(&run.props))
}

pub(crate) fn specs() -> Vec<CommandSpec> {
    vec![cmd!(
        "format.wordArtStyle",
        "WordArt Styles",
        ["Shape Format", "WordArt Styles"],
        None,
        "{style: amber|indigoEdge|roseLine|skyNeon|brand|sunset|lagoon|dusk|obsidian|retro|violetGlow|sticker|0..} — the look of the selected text or of all text in the selected shapes",
        has_text_or_shapes,
        restyle
    )]
}

/// Longest text a new WordArt takes (characters).
const MAX_TEXT: usize = 10_000;

/// `insert.wordArt`.
pub(crate) fn insert(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "insert.wordArt";
    let style = style_param(p, CMD)?;
    let vi = crate::nexus_vi::is_vi(s.new_language.as_deref());
    let text: String = match p.get("text") {
        None | Some(Value::Null) => (if vi { DEFAULT_TEXT.1 } else { DEFAULT_TEXT.0 }).to_string(),
        Some(Value::String(t)) if !t.is_empty() => t.chars().take(MAX_TEXT).collect::<String>().replace("\r\n", "\n").replace('\r', "\n"),
        Some(_) => return Err(bad(CMD, "`text` is a non-empty string")),
    };
    let size = s.doc()?.doc.slide_size;
    let mut shape = crate::cmd::insert::new_text_box(Xfrm::new(size.width * 0.2, size.height * 0.4, size.width * 0.6, SIZE * 1.6), &text);
    shape.name = "WordArt".into();
    if let Some(t) = shape.text.as_mut() {
        // Sized to its text: one line per paragraph, the box as wide as the longest.
        t.body.wrap = Some(false);
        for para in &mut t.paragraphs {
            para.props.align = Some(Align::Center);
            for r in &mut para.runs {
                r.props.size = Some(SIZE);
                r.props.font = Some("+mj-lt".into());
                style.apply(&mut r.props);
            }
            para.end_props.size = Some(SIZE);
            para.end_props.font = Some("+mj-lt".into());
            style.apply(&mut para.end_props);
        }
    }
    let id = crate::cmd::insert::add_shape(s, CMD, shape, true)?;
    crate::cmd::text::fit_text_box(s, id)?;
    // Too wide for the slide (long text): smaller type, fitted again.
    let w = s.doc()?.shape(id).and_then(|sh| sh.xfrm).map(|x| x.w).unwrap_or(0.0);
    let room = size.width * 0.9;
    if w.is_finite() && w > room && room > 0.0 {
        let k = (room / w).clamp(0.25, 1.0);
        resize_text(s, id, (SIZE * k).max(12.0))?;
        crate::cmd::text::fit_text_box(s, id)?;
    }
    place(s, id)?;
    // Typing replaces the text: all of it selected, in edit mode.
    let end = s.doc()?.shape(id).and_then(|sh| sh.text.as_ref()).map(deckcraft_model::edit::end).unwrap_or((0, 0));
    s.select(|_, sel| {
        sel.shapes = vec![id];
        sel.text = Some(TextSel { shape: id, anchor: (0, 0), caret: end, ..Default::default() });
    })?;
    Ok(json!({"id": id, "style": style.id}))
}

/// `format.wordArtStyle`.
fn restyle(s: &mut Session, p: &Value) -> Result<Value> {
    let style = style_param(p, "format.wordArtStyle")?;
    crate::cmd::text::format_run(s, &move |r| style.apply(r))?;
    Ok(json!({"style": style.id}))
}

/// Changes shape `id` on the current slide in place, as part of the edit that is running (no
/// extra undo step), like [`crate::cmd::text::fit_text_box`].
fn change_quietly(s: &mut Session, id: deckcraft_model::ShapeId, f: impl FnOnce(&mut Shape)) -> Result<()> {
    let st = s.doc_mut()?;
    let mut d = (*st.doc).clone();
    if let Some(list) = crate::shapes_mut(&mut d, &st.selection)
        && let Some(target) = deckcraft_model::find_shape_mut(list, id)
    {
        f(target);
    }
    st.doc = std::sync::Arc::new(d);
    st.revision += 1;
    Ok(())
}

fn resize_text(s: &mut Session, id: deckcraft_model::ShapeId, size: f64) -> Result<()> {
    change_quietly(s, id, |sh| {
        if let Some(t) = sh.text.as_mut() {
            for para in &mut t.paragraphs {
                for r in &mut para.runs {
                    r.props.size = Some(size);
                }
                para.end_props.size = Some(size);
            }
        }
    })
}

/// Is `sh` a placeholder still showing its prompt (no text, nothing in it)?
fn empty_placeholder(sh: &Shape) -> bool {
    sh.ph.is_some() && matches!(sh.kind, ShapeKind::Shape) && sh.text.as_ref().is_none_or(|t| t.is_empty())
}

fn is_title(sh: &Shape) -> bool {
    sh.ph.as_ref().is_some_and(|p| matches!(p.kind, PhType::Title | PhType::CtrTitle))
}

/// What a new WordArt keeps off on the current slide (every shape but `skip`), and how much it
/// minds covering each: text, pictures and other WordArt fully; a title's prompt nearly so; an
/// empty body placeholder's prompt a little (the slide may have nowhere else).
pub fn obstacles(s: &Session, skip: deckcraft_model::ShapeId) -> Vec<(Rect, f64)> {
    let Some(st) = s.active() else { return Vec::new() };
    let Some(shapes) = crate::cmd::current_shapes(&st.doc, &st.selection) else { return Vec::new() };
    shapes
        .iter()
        .filter(|sh| sh.id != skip && !sh.hidden)
        .take(2000)
        .filter_map(|sh| {
            let b = crate::cmd::xfrm_of(&st.doc, &st.selection, sh).bounds();
            let ok = b.is_finite() && b.width() >= 0.0 && b.height() >= 0.0;
            let weight = match (empty_placeholder(sh), is_title(sh)) {
                (true, true) => 0.8,
                (true, false) => 0.25,
                _ => 1.0,
            };
            ok.then_some((b, weight))
        })
        .collect()
}

/// Where a `w` × `h` WordArt goes on a `slide`-sized slide among `obstacles`: as near the middle
/// as it can be (the middle of the space under a title at the top), off what it would cover.
/// Returns the top-left corner.
pub fn free_spot(slide: (f64, f64), w: f64, h: f64, obstacles: &[(Rect, f64)]) -> (f64, f64) {
    let (sw, sh) = slide;
    let fine = |v: f64| v.is_finite() && v >= 0.0;
    if !(fine(sw) && fine(sh) && fine(w) && fine(h)) || sw <= 0.0 || sh <= 0.0 {
        return (0.0, 0.0);
    }
    let margin = sw.min(sh) * 0.04;
    let gap = 6.0;
    // The middle wanted: the slide's, or the middle of the space under a title at the top.
    let mut want = (sw / 2.0, sh / 2.0);
    if let Some(top) = obstacles.iter().filter(|(r, wt)| *wt >= 0.8 && r.center().y < sh * 0.35).map(|(r, _)| r.y1).reduce(f64::max)
        && top.is_finite()
        && top < sh
    {
        want.1 = (top + sh) / 2.0;
    }
    let span = |len: f64, room: f64| -> (f64, f64) {
        if len + 2.0 * margin >= room { ((room - len) / 2.0, (room - len) / 2.0) } else { (margin, room - len - margin) }
    };
    let (x_lo, x_hi) = span(w, sw);
    let (y_lo, y_hi) = span(h, sh);
    let clamp = |v: f64, lo: f64, hi: f64| if lo <= hi { v.clamp(lo, hi) } else { lo };
    let cx = clamp(want.0 - w / 2.0, x_lo, x_hi);
    let cy = clamp(want.1 - h / 2.0, y_lo, y_hi);
    let mut xs = vec![cx];
    for k in [-2.0, -1.0, 1.0, 2.0] {
        xs.push(clamp(cx + k * sw / 8.0, x_lo, x_hi));
    }
    let mut ys = vec![cy];
    let step = (sh / 48.0).max(1.0);
    let mut y = y_lo;
    while y <= y_hi && ys.len() < 200 {
        ys.push(y);
        y += step;
    }
    ys.push(y_hi);
    let area = (w * h).max(1.0);
    let score = |x: f64, y: f64| {
        let r = Rect::new(x, y, x + w, y + h);
        let covered: f64 = obstacles
            .iter()
            .map(|(o, wt)| {
                let i = r.intersect(o.inflate(gap, gap));
                if i.width() > 0.0 && i.height() > 0.0 { wt * i.area() / area } else { 0.0 }
            })
            .sum();
        let off = ((x + w / 2.0 - want.0) / sw).abs() + ((y + h / 2.0 - want.1) / sh).abs();
        covered + 0.35 * off
    };
    let mut best = (cx, cy);
    let mut best_score = score(cx, cy);
    for &x in &xs {
        for &y in &ys {
            let sc = score(x, y);
            if sc < best_score - 1e-9 {
                best = (x, y);
                best_score = sc;
            }
        }
    }
    best
}

/// Moves new WordArt `id` to free space ([`free_spot`]).
fn place(s: &mut Session, id: deckcraft_model::ShapeId) -> Result<()> {
    let st = s.doc()?;
    let size = st.doc.slide_size;
    let Some(x) = st.shape(id).and_then(|sh| sh.xfrm) else { return Ok(()) };
    let (left, top) = free_spot((size.width, size.height), x.w, x.h, &obstacles(s, id));
    change_quietly(s, id, |sh| {
        if let Some(x) = sh.xfrm.as_mut() {
            x.x = left;
            x.y = top;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;

    fn deck(layout: Option<&str>) -> Session {
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        if let Some(l) = layout {
            s.execute("slide.layout", &json!({"layout": l})).unwrap();
        }
        s
    }

    fn wordart(s: &Session) -> Vec<Shape> {
        s.doc().unwrap().current_slide().unwrap().shapes.iter().filter(|sh| sh.ph.is_none() && sh.text_box).cloned().collect()
    }

    fn rect(s: &Session, sh: &Shape) -> Rect {
        let st = s.doc().unwrap();
        crate::cmd::xfrm_of(&st.doc, &st.selection, sh).bounds()
    }

    fn overlap(a: Rect, b: Rect) -> f64 {
        let i = a.intersect(b);
        if i.width() > 0.0 && i.height() > 0.0 { i.area() } else { 0.0 }
    }

    /// Every style, by id and by number, gives its own look: fill, outline, shadow, glow and
    /// reflection as the table says, bold, in the heading font, at the WordArt size.
    #[test]
    fn every_style_inserts_its_effects() {
        assert!(STYLES.len() >= 12, "a gallery of 4 × 3 at least");
        let mut seen = std::collections::HashSet::new();
        for (i, st) in STYLES.iter().enumerate() {
            assert!(seen.insert(st.id), "{} twice", st.id);
            for style in [json!(st.id), json!(i)] {
                let mut s = deck(None);
                let r = s.execute("insert.wordArt", &json!({"style": style})).unwrap();
                assert_eq!(r["style"], json!(st.id));
                let sh = wordart(&s).pop().unwrap();
                let run = &sh.text.as_ref().unwrap().paragraphs[0].runs[0].props;
                assert!(st.matches(run), "{}: {run:?}", st.id);
                assert_eq!(style_of(&sh).map(|x| x.id), Some(st.id));
                assert_eq!(run.bold, Some(true));
                assert_eq!(run.size, Some(SIZE));
                assert_eq!(run.font.as_deref(), Some("+mj-lt"));
                assert_eq!(run.shadow.is_some(), st.shadow.is_some(), "{}", st.id);
                assert_eq!(run.glow.is_some(), st.glow.is_some(), "{}", st.id);
                assert_eq!(run.reflection.is_some(), st.reflection, "{}", st.id);
                match st.fill {
                    Paint::None => assert_eq!(run.fill, Some(Fill::None)),
                    Paint::Solid(_) => assert!(matches!(run.fill, Some(Fill::Solid { .. }))),
                    Paint::Gradient(..) => assert!(matches!(run.fill, Some(Fill::Gradient(_)))),
                }
            }
        }
        // Each look is told apart from the others.
        for a in STYLES {
            assert_eq!(STYLES.iter().filter(|b| b.matches(&a.look())).count(), 1, "{}", a.id);
        }
        // Upstream's numbers wrap.
        let mut s = deck(None);
        let r = s.execute("insert.wordArt", &json!({"style": STYLES.len() * 3 + 1})).unwrap();
        assert_eq!(r["style"], json!(STYLES[1].id));
    }

    /// Junk params are errors or defaults, never a panic, and leave nothing half made.
    #[test]
    fn junk_params_never_panic() {
        let mut s = deck(None);
        for p in [
            json!(null),
            json!([1, 2]),
            json!({"style": "nope"}),
            json!({"style": -1}),
            json!({"style": 1.5}),
            json!({"style": 1e308}),
            json!({"style": u64::MAX}),
            json!({"style": {"a": 1}}),
            json!({"style": "x".repeat(100_000)}),
            json!({"text": 5}),
            json!({"text": ""}),
            json!({"text": "\r\n\r\n\n"}),
            json!({"text": "Ặ".repeat(20_000)}),
            json!({"text": "e\u{0302}\u{0323} NFD"}),
        ] {
            let _ = s.execute("insert.wordArt", &p);
            let _ = s.execute("format.wordArtStyle", &p);
        }
        assert!(s.execute("insert.wordArt", &json!({"style": "nope"})).is_err());
        assert!(s.execute("format.wordArtStyle", &json!({"style": "nope"})).is_err());
        // The very long one is shrunk to fit and capped.
        for sh in wordart(&s) {
            let r = rect(&s, &sh);
            assert!(r.is_finite(), "{r:?}");
        }
        // Placement survives hostile geometry.
        for (slide, w, h) in [((f64::NAN, 1.0), 1.0, 1.0), ((10.0, 10.0), f64::INFINITY, 1.0), ((0.0, 0.0), 0.0, 0.0), ((10.0, 10.0), 500.0, 500.0)] {
            let (x, y) = free_spot(slide, w, h, &[(Rect::new(f64::NAN, 0.0, 1.0, 1.0), 1.0)]);
            assert!(x.is_finite() && y.is_finite());
        }
    }

    /// The new WordArt never covers a placeholder that has text, nor another WordArt; on a slide
    /// whose title is empty it stays off the title's prompt, in the middle of the space under it.
    #[test]
    fn placement_keeps_off_text_and_titles() {
        // A title slide whose title has text: the box keeps off it.
        let mut s = deck(None);
        let title = s.doc().unwrap().current_slide().unwrap().shapes.iter().find(|sh| is_title(sh)).map(|sh| sh.id).expect("a title");
        s.execute("text.edit", &json!({"id": title.0})).unwrap();
        s.execute("text.insert", &json!({"text": "Kế hoạch quý IV"})).unwrap();
        s.execute("text.exit", &json!({})).unwrap();
        s.execute("insert.wordArt", &json!({})).unwrap();
        let slide = s.doc().unwrap().current_slide().unwrap().clone();
        let t = slide.shapes.iter().find(|sh| sh.id == title).unwrap();
        assert!(!t.text.as_ref().unwrap().is_empty());
        let first = wordart(&s).pop().unwrap();
        assert_eq!(overlap(rect(&s, &first), rect(&s, t)), 0.0, "covers the title");
        // A second WordArt keeps off the first and off the title.
        s.execute("insert.wordArt", &json!({"style": "sunset"})).unwrap();
        let both = wordart(&s);
        assert_eq!(both.len(), 2);
        assert_eq!(overlap(rect(&s, &both[0]), rect(&s, &both[1])), 0.0, "WordArt over WordArt");
        assert_eq!(overlap(rect(&s, &both[1]), rect(&s, t)), 0.0, "covers the title");
        // Inside the slide.
        let size = s.doc().unwrap().doc.slide_size;
        for sh in &both {
            let r = rect(&s, sh);
            assert!(r.x0 >= 0.0 && r.y0 >= 0.0 && r.x1 <= size.width && r.y1 <= size.height, "{r:?}");
        }

        // An empty Title Only slide: under the title's prompt, centred in the space below it.
        let mut s = deck(Some("Title Only"));
        s.execute("insert.wordArt", &json!({})).unwrap();
        let slide = s.doc().unwrap().current_slide().unwrap().clone();
        let t = slide.shapes.iter().find(|sh| is_title(sh)).expect("a title");
        let tr = rect(&s, t);
        let r = rect(&s, &wordart(&s)[0]);
        assert_eq!(overlap(r, tr), 0.0, "on the title's prompt");
        let size = s.doc().unwrap().doc.slide_size;
        assert!((r.center().x - size.width / 2.0).abs() < 1.0, "centred across: {r:?}");
        let body_mid = (tr.y1 + size.height) / 2.0;
        assert!((r.center().y - body_mid).abs() < size.height / 24.0, "centred in the space under the title: {r:?} vs {body_mid}");

        // An empty title slide (the first slide of a new deck): off the title's prompt.
        let mut s = deck(None);
        s.execute("insert.wordArt", &json!({})).unwrap();
        let slide = s.doc().unwrap().current_slide().unwrap().clone();
        let t = slide.shapes.iter().find(|sh| is_title(sh)).expect("a title");
        assert_eq!(overlap(rect(&s, &wordart(&s)[0]), rect(&s, t)), 0.0, "on the title's prompt");
    }

    /// The box is as big as its text (not a fixed width) and the text is all selected, being
    /// edited, so typing replaces it; one undo takes the whole WordArt away.
    #[test]
    fn inserted_text_is_selected_for_typing() {
        let mut s = deck(None);
        s.new_language = Some("vi-VN".into());
        s.execute("insert.wordArt", &json!({"style": "brand"})).unwrap();
        let sh = wordart(&s).pop().unwrap();
        let body = sh.text.as_ref().unwrap();
        assert_eq!(body.text(), DEFAULT_TEXT.1);
        assert_eq!(body.body.wrap, Some(false));
        let sel = s.doc().unwrap().selection.clone();
        assert_eq!(sel.shapes, vec![sh.id]);
        let t = sel.text.expect("editing");
        assert_eq!(t.shape, sh.id);
        assert_eq!(t.ordered(), ((0, 0), (0, DEFAULT_TEXT.1.chars().count())));
        let w = sh.xfrm.unwrap().w;
        // Short text, narrow box.
        let mut short = deck(None);
        short.execute("insert.wordArt", &json!({"text": "Hi"})).unwrap();
        let narrow = wordart(&short).pop().unwrap().xfrm.unwrap().w;
        assert!(narrow < w / 3.0, "{narrow} vs {w}");
        // Typing replaces the selected text and keeps the look.
        s.execute("text.insert", &json!({"text": "Chào mừng"})).unwrap();
        let sh = wordart(&s).pop().unwrap();
        assert_eq!(sh.text.as_ref().unwrap().text(), "Chào mừng");
        assert_eq!(style_of(&sh).map(|x| x.id), Some("brand"));
        // English interface: English text.
        let mut en = deck(None);
        en.execute("insert.wordArt", &json!({})).unwrap();
        assert_eq!(wordart(&en).pop().unwrap().text.unwrap().text(), DEFAULT_TEXT.0);
        // One undo removes it.
        en.execute("edit.undo", &json!({})).unwrap();
        assert!(wordart(&en).is_empty());
    }

    /// Restyling keeps the words, font and size and swaps the look (selected shape or text).
    #[test]
    fn restyle_swaps_the_look() {
        let mut s = deck(None);
        s.execute("insert.wordArt", &json!({"text": "Nexus", "style": "amber"})).unwrap();
        s.execute("format.wordArtStyle", &json!({"style": "dusk"})).unwrap();
        let sh = wordart(&s).pop().unwrap();
        assert_eq!(style_of(&sh).map(|x| x.id), Some("dusk"));
        let run = &sh.text.as_ref().unwrap().paragraphs[0].runs[0].props;
        assert_eq!(run.size, Some(SIZE));
        assert!(run.shadow.is_none(), "amber's shadow is gone");
        // With the shape selected (not editing).
        s.execute("edit.select", &json!({"ids": [sh.id.0]})).unwrap();
        s.execute("format.wordArtStyle", &json!({"style": 2})).unwrap();
        assert_eq!(style_of(&wordart(&s).pop().unwrap()).map(|x| x.id), Some(STYLES[2].id));
        assert_eq!(wordart(&s).pop().unwrap().text.unwrap().text(), "Nexus");
    }

    /// Text reflections are drawn: ink under the text that a plain style doesn't have.
    #[test]
    fn reflection_is_drawn_under_the_text() {
        let ink_below = |style: &str| {
            let mut s = deck(Some("Blank"));
            s.execute("insert.wordArt", &json!({"text": "HHHH", "style": style})).unwrap();
            let sh = wordart(&s).pop().unwrap();
            let r = rect(&s, &sh);
            let doc = s.doc().unwrap().doc.clone();
            let img = deckcraft_render::render_slide(&doc, 0, &deckcraft_render::RenderOpts { scale: 1.0, ..Default::default() });
            let mut n = 0;
            for y in (r.y1 as u32 + 2)..(r.y1 as u32 + 30).min(img.height) {
                for x in (r.x0 as u32)..(r.x1 as u32).min(img.width) {
                    let p = img.pixel(x, y);
                    if p[0] < 200 || p[1] < 200 || p[2] < 200 {
                        n += 1;
                    }
                }
            }
            n
        };
        let plain = ink_below("brand");
        let mirrored = ink_below("obsidian");
        assert!(mirrored > 200 && mirrored > plain * 2 + 100, "reflection {mirrored} vs plain {plain}");
    }
}

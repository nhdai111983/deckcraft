//! Rendered slides as GPU textures: the canvas slide and thumbnails, cached by the slide's and its
//! master's `Arc` identity (an unchanged slide keeps its `Arc`, so it never re-renders).
//!
//! The canvas texture covers the slide plus whatever of the slide's drawing reaches off it and is
//! in view (PowerPoint's Normal view shows off-slide objects; thumbnails clip to the slide).

use std::collections::HashMap;
use std::sync::Arc;

use deckcraft_geom::Rect;
use deckcraft_model::{Master, Presentation, ShapeId, Slide};
use deckcraft_render::{Image, Region, RenderOpts};
use egui::{ColorImage, TextureFilter, TextureHandle, TextureOptions, TextureWrapMode};

#[derive(Clone, PartialEq)]
struct Key {
    slide: usize,
    master: usize,
    size: (u32, u32),
    edit: bool,
    extra: u64,
    /// The pasteboard region the slide's drawing calls for (canvas only).
    want: Region,
    editing_text: Option<ShapeId>,
}

struct Entry {
    key: Key,
    tex: TextureHandle,
    /// The region the texture covers (at least `key.want`).
    region: Region,
    // Keep the Arcs alive so their addresses can't be reused while cached.
    _slide: Arc<Slide>,
    _master: Option<Arc<Master>>,
    stamp: u64,
}

#[derive(Default)]
pub struct Textures {
    canvas: Option<Entry>,
    /// How far the canvas slide's shapes reach (from their boxes), for the slide and master `Arc`s
    /// held (kept alive so their addresses can't be reused).
    estimate: Option<(Arc<Slide>, Option<Arc<Master>>, Rect)>,
    /// How far the last canvas render of a slide (by index) actually drew: text spilling out of
    /// its box shows there.
    drawn: Option<(usize, Rect)>,
    /// Canvas renders so far (tests, performance checks).
    pub canvas_renders: u64,
    /// NexusOffice: those of them that drew only the part of the canvas an edit changed.
    pub region_renders: u64,
    thumbs: HashMap<usize, Entry>,
    /// Thumbnails actually rendered, over the life of this cache. The number *held*
    /// ([`Textures::thumb_count`]) cannot see a pane that renders the same rows over and over --
    /// the cache thrash behind #969 -- because the count settles while the work does not.
    thumb_renders: u64,
    /// SmartArt gallery pictures by layout, style and colours (and, for hover previews, the
    /// outline's hash).
    smartart: HashMap<String, TextureHandle>,
    clock: u64,
    pub last_render_ms: f64,
}

impl Textures {
    /// SmartArt gallery pictures cached so far (tests).
    pub fn smartart_cached(&self) -> usize {
        self.smartart.len()
    }

    /// A SmartArt layout drawn small for a gallery (`nodes` for a preview of the selected
    /// graphic's own outline), cached; `look` is the layout kind, style and colours; drawn with the active presentation's theme.
    pub fn smartart(
        &mut self,
        ctx: &egui::Context,
        p: &deckcraft_model::Presentation,
        slide: usize,
        (kind, style, colors): (&str, &str, &str),
        nodes: Option<&[deckcraft_model::DiagramNode]>,
        width_px: u32,
    ) -> Option<TextureHandle> {
        let mut key = format!("{kind}|{style}|{colors}|{width_px}");
        if let Some(n) = nodes {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            for x in n {
                (x.text.as_str(), x.level).hash(&mut h);
            }
            key.push_str(&format!("|{:x}", h.finish()));
        }
        if let Some(t) = self.smartart.get(&key) {
            return Some(t.clone());
        }
        let mut s = p.slides.get(slide).or_else(|| p.slides.first())?.as_ref().clone();
        let g = deckcraft_engine::cmd::smartart::preview(kind, style, colors, nodes);
        let id = g.id;
        s.shapes = vec![g];
        let img = deckcraft_render::render_shape(p, &s, id, width_px as f64 / 336.0)?;
        let tex = upload(ctx, format!("smartart-{key}"), to_color_image(&img, false));
        if self.smartart.len() > 300 {
            self.smartart.clear();
        }
        self.smartart.insert(key, tex.clone());
        Some(tex)
    }
}

/// Snap `rect` onto the physical pixel grid.
///
/// The editor draws the slide as a texture of device pixels. A rect that starts or ends
/// between pixels is sampled with a linear filter, so every glyph edge is a blend of two
/// texels and small text looks soft. On a Retina display that offset is half a point, which
/// is the whole edge of an editing-size glyph. The slide show draws much larger glyphs, so
/// the same blend is not visible there (#184).
pub fn device_pixel_rect(rect: egui::Rect, ppp: f32) -> egui::Rect {
    if !ppp.is_finite() || ppp <= 0.0 || !rect.is_finite() {
        return rect;
    }
    let snap = |v: f32| {
        let p = v * ppp;
        if p.is_finite() { p.round() / ppp } else { v }
    };
    let min = egui::pos2(snap(rect.min.x), snap(rect.min.y));
    let step = 1.0 / ppp;
    let max = egui::pos2(snap(rect.max.x).max(min.x + step), snap(rect.max.y).max(min.y + step));
    egui::Rect::from_min_max(min, max)
}

/// Editor slide textures. Nearest magnification keeps a texel that is stretched by a
/// fraction of a pixel from blurring the glyph; minification still blends when many texels
/// share a screen pixel.
pub fn editor_texture_options() -> TextureOptions {
    TextureOptions {
        magnification: TextureFilter::Nearest,
        minification: TextureFilter::Linear,
        wrap_mode: TextureWrapMode::ClampToEdge,
        mipmap_mode: None,
    }
}

pub fn to_color_image(img: &Image, grayscale: bool) -> ColorImage {
    let w = img.width as usize;
    let h = img.height as usize;
    let mut px = Vec::with_capacity(w * h);
    for c in img.pixels.as_chunks::<4>().0 {
        let (r, g, b, a) = (c[0], c[1], c[2], c[3]);
        if grayscale {
            let y = ((r as u32 * 77 + g as u32 * 150 + b as u32 * 29) >> 8) as u8;
            px.push(egui::Color32::from_rgba_premultiplied(y, y, y, a));
        } else {
            px.push(egui::Color32::from_rgba_premultiplied(r, g, b, a));
        }
    }
    px.resize(w * h, egui::Color32::TRANSPARENT);
    ColorImage::new([w, h], px)
}

/// The largest texture side the renderer (GPU or software fallback) takes; egui panics on
/// a bigger texture.
pub fn max_side(ctx: &egui::Context) -> u32 {
    ctx.input(|i| i.max_texture_side).clamp(1, u32::MAX as usize) as u32
}

/// `(w, h)` scaled down, keeping its shape, so neither side exceeds `max` (and none is 0).
/// Render sizes go through this so a huge display, DPI or slide shape doesn't render a
/// picture the GPU can't hold.
pub fn fit_size((w, h): (u32, u32), max: u32) -> (u32, u32) {
    let (w, h, max) = (w.max(1), h.max(1), max.max(1));
    let big = w.max(h);
    if big <= max {
        return (w, h);
    }
    let k = max as f64 / big as f64;
    (((w as f64 * k).floor() as u32).clamp(1, max), ((h as f64 * k).floor() as u32).clamp(1, max))
}

/// `img` shrunk by whole-pixel box averaging until both sides fit `max`.
pub fn shrink_to(img: ColorImage, max: u32) -> ColorImage {
    let [w, h] = img.size;
    let max = max.max(1) as usize;
    if w <= max && h <= max {
        return img;
    }
    let f = w.max(h).div_ceil(max);
    let (nw, nh) = (w.div_ceil(f).max(1), h.div_ceil(f).max(1));
    let mut px = Vec::with_capacity(nw * nh);
    for y in 0..nh {
        for x in 0..nw {
            let mut acc = [0u32; 4];
            let mut n = 0u32;
            for sy in y * f..((y + 1) * f).min(h) {
                for sx in x * f..((x + 1) * f).min(w) {
                    if let Some(c) = img.pixels.get(sy * w + sx) {
                        for (a, v) in acc.iter_mut().zip(c.to_array()) {
                            *a += v as u32;
                        }
                        n += 1;
                    }
                }
            }
            let n = n.max(1);
            px.push(egui::Color32::from_rgba_premultiplied((acc[0] / n) as u8, (acc[1] / n) as u8, (acc[2] / n) as u8, (acc[3] / n) as u8));
        }
    }
    ColorImage::new([nw, nh], px)
}

/// NexusOffice: the part of canvas region `canvas` (pixels from the slide's corner) that slide
/// area `changed` (points) covers at `scale` pixels a point, a pixel more all round; `None` when
/// it is most of the canvas (drawing it whole costs about the same) or not a number.
pub(crate) fn patch_region(canvas: Region, changed: Rect, scale: f64) -> Option<Region> {
    if !(changed.is_finite() && scale.is_finite() && scale > 0.0) {
        return None;
    }
    let (cx0, cy0) = (canvas.x as i64, canvas.y as i64);
    let (cx1, cy1) = (cx0 + canvas.w as i64, cy0 + canvas.h as i64);
    let px = |v: f64, up: bool| {
        let v = v * scale;
        (if up { v.ceil() + 1.0 } else { v.floor() - 1.0 }).clamp(i32::MIN as f64, i32::MAX as f64) as i64
    };
    let (x0, y0) = (px(changed.x0, false).max(cx0), px(changed.y0, false).max(cy0));
    let (x1, y1) = (px(changed.x1, true).min(cx1), px(changed.y1, true).min(cy1));
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let (w, h) = ((x1 - x0) as u64, (y1 - y0) as u64);
    if w.saturating_mul(h).saturating_mul(2) > (canvas.w as u64).saturating_mul(canvas.h as u64) {
        return None;
    }
    Some(Region { x: x0 as i32, y: y0 as i32, w: w as u32, h: h as u32 })
}

/// Upload a texture, shrunk to the renderer's limit if need be.
pub fn upload(ctx: &egui::Context, name: impl Into<String>, img: ColorImage) -> TextureHandle {
    ctx.load_texture(name, shrink_to(img, max_side(ctx)), TextureOptions::LINEAR)
}

/// Replace a texture's picture, shrunk to the renderer's limit if need be.
pub fn replace(ctx: &egui::Context, tex: &mut TextureHandle, img: ColorImage) {
    tex.set(shrink_to(img, max_side(ctx)), TextureOptions::LINEAR);
}

/// [`upload`] for editor slide pictures, drawn on the device pixel grid with
/// [`editor_texture_options`] (#184).
pub fn upload_editor(ctx: &egui::Context, name: impl Into<String>, img: ColorImage) -> TextureHandle {
    ctx.load_texture(name, shrink_to(img, max_side(ctx)), editor_texture_options())
}

/// [`replace`] for editor slide pictures (#184).
pub fn replace_editor(ctx: &egui::Context, tex: &mut TextureHandle, img: ColorImage) {
    tex.set(shrink_to(img, max_side(ctx)), editor_texture_options());
}

fn master_of(p: &Presentation, slide: &Slide) -> Option<Arc<Master>> {
    p.masters.iter().find(|m| m.layout(slide.layout).is_some()).cloned()
}

fn same(a: &Option<Arc<Master>>, b: &Option<Arc<Master>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

/// Mix the preview playhead into the canvas cache key. Zero means "no preview", so a playhead of
/// 0s still misses the at-rest texture (entrances start hidden).
fn anim_key(anim: Option<(&deckcraft_anim::Timeline, f64)>) -> u64 {
    let Some((_, t)) = anim else { return 0 };
    let ms = if t.is_finite() { (t * 1000.0).clamp(0.0, 1.0e12) as u64 } else { 0 };
    ms.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

fn threads() -> u16 {
    if cfg!(target_arch = "wasm32") { 0 } else { std::thread::available_parallelism().map(|n| n.get().min(8) as u16).unwrap_or(0) }
}

impl Textures {
    /// The canvas texture for slide `index`, the slide being `size` pixels, and the region it
    /// covers: the slide, grown over the pasteboard to show what reaches off the slide within
    /// `visible` (the canvas, in pixels from the slide's top-left corner). `extra` changes with the
    /// GIF frames. `editing_text` replaces its placeholder prompt with gray empty list markers.
    #[allow(clippy::too_many_arguments)]
    pub fn slide(
        &mut self,
        ctx: &egui::Context,
        p: &Presentation,
        index: usize,
        size: (u32, u32),
        visible: Region,
        edit: bool,
        extra: u64,
        grayscale: bool,
        gif_times: &[(ShapeId, f64)],
        editing_text: Option<ShapeId>,
        anim: Option<(&deckcraft_anim::Timeline, f64)>,
    ) -> Option<(TextureHandle, Region)> {
        let slide = p.slides.get(index)?.clone();
        let master = master_of(p, &slide);
        let scale = size.0 as f64 / p.slide_size.width.max(1.0);
        let estimate = match &self.estimate {
            Some((s, m, r)) if Arc::ptr_eq(s, &slide) && same(m, &master) => *r,
            _ => {
                let r = deckcraft_render::slide_extent(p, index);
                self.estimate = Some((slide.clone(), master.clone(), r));
                r
            }
        };
        let reach = match self.drawn {
            Some((i, d)) if i == index => estimate.union(d),
            _ => estimate,
        };
        // The GPU's (or software fallback's) texture limit: a larger canvas texture panics in egui.
        let max_side = max_side(ctx);
        let want = fit_texture(deckcraft_render::pasteboard_region(size, scale, reach, visible), visible, max_side);
        let key = Key {
            slide: Arc::as_ptr(&slide) as usize,
            master: master.as_ref().map(|m| Arc::as_ptr(m) as usize).unwrap_or(0),
            size,
            edit,
            // NexusOffice: the placeholder prompts follow the interface's language.
            extra: extra ^ grayscale as u64 ^ anim_key(anim) ^ (u64::from(crate::i18n::current(ctx) != crate::i18n::Language::En) << 61),
            want,
            editing_text,
        };
        let prompt = crate::i18n::prompt_fn(crate::i18n::current(ctx));
        if let Some(e) = &self.canvas
            && e.key == key
        {
            return Some((e.tex.clone(), e.region));
        }
        let t0 = crate::now_ms();
        let render = |region: Region| {
            let state_of = |id: ShapeId| anim.map(|(tl, time)| crate::preview::shape_state_at(tl, id, time)).unwrap_or_default();
            let state: Option<&dyn Fn(ShapeId) -> deckcraft_render::ShapeState> = if anim.is_some() { Some(&state_of) } else { None };
            let opts = RenderOpts {
                scale,
                edit,
                gif_times,
                threads: threads(),
                offset: region.offset(),
                size: Some(region.size()),
                state,
                editing_text,
                prompt,
                ..Default::default()
            };
            deckcraft_render::render_slide_drawn(p, index, &opts)
        };
        // NexusOffice: an edit that changed only some shapes of the slide shown (typing, a nudge)
        // draws again only where they drew and draw, over the canvas texture as it is; an edit
        // nothing on the slide shows (the notes) draws nothing.
        if let Some(e) = self.canvas.as_mut()
            && e.key == (Key { slide: e.key.slide, ..key.clone() })
            && e.tex.size() == [e.region.w as usize, e.region.h as usize]
            && anim.is_none()
        {
            let patch = match deckcraft_render::changed_area(p, &e._slide, &slide, editing_text) {
                deckcraft_render::Changed::Nothing => Some(None),
                deckcraft_render::Changed::Area(r) => patch_region(e.region, r, scale).map(Some),
                deckcraft_render::Changed::Unknown => None,
            };
            let drawn = match patch {
                Some(Some(sub)) => {
                    let (img, drawn) = render(sub);
                    (img.width == sub.w && img.height == sub.h).then(|| {
                        let at = [sub.x.saturating_sub(e.region.x).max(0) as usize, sub.y.saturating_sub(e.region.y).max(0) as usize];
                        e.tex.set_partial(at, to_color_image(&img, grayscale), editor_texture_options());
                        Some(drawn)
                    })
                }
                Some(None) => Some(None),
                None => None,
            };
            if let Some(drawn) = drawn {
                if let Some(drawn) = drawn {
                    self.canvas_renders += 1;
                    self.region_renders += 1;
                    self.drawn = Some(match self.drawn {
                        Some((i, d)) if i == index => (index, d.union(drawn)),
                        _ => (index, drawn),
                    });
                    self.last_render_ms = crate::now_ms() - t0;
                }
                e.key = key;
                e._slide = slide;
                e._master = master;
                return Some((e.tex.clone(), e.region));
            }
        }
        let (mut img, drawn) = render(want);
        self.canvas_renders += 1;
        let mut region = want;
        // Text spilling out of its box can reach further than the boxes said. Once drawn, the
        // region it calls for is known (and is next frame's key): render again only if it shows
        // more than this render covered.
        let key =
            Key { want: fit_texture(deckcraft_render::pasteboard_region(size, scale, estimate.union(drawn), visible), visible, max_side), ..key };
        if !region.contains(&key.want) {
            region = key.want;
            img = render(region).0;
            self.canvas_renders += 1;
        }
        self.drawn = Some((index, drawn));
        self.last_render_ms = crate::now_ms() - t0;
        let ci = to_color_image(&img, grayscale);
        let tex = match self.canvas.take() {
            Some(mut e) => {
                replace_editor(ctx, &mut e.tex, ci);
                e.tex
            }
            None => upload_editor(ctx, "slide-canvas", ci),
        };
        self.canvas = Some(Entry { key, tex: tex.clone(), region, _slide: slide, _master: master, stamp: 0 });
        Some((tex, region))
    }

    /// Cached thumbnails (bounded; soak tests check it).
    pub fn thumb_count(&self) -> usize {
        self.thumbs.len()
    }

    /// How many thumbnails this cache has rendered, ever. A pane that is done asks for no more:
    /// the number stops climbing, which is what tells a thrashing pane from a settled one.
    pub fn thumb_renders(&self) -> u64 {
        self.thumb_renders
    }

    /// Force the canvas to re-render next time (fonts loaded, theme changed in place…).
    pub fn invalidate(&mut self) {
        self.canvas = None;
        self.estimate = None;
        self.drawn = None;
        self.thumbs.clear();
    }

    /// A thumbnail texture; renders at most `budget` new thumbnails per frame (returns None when
    /// it is over budget, so the caller draws a placeholder and asks for another frame).
    pub fn thumb(&mut self, ctx: &egui::Context, p: &Presentation, index: usize, width_px: u32, budget: &mut u32) -> Option<TextureHandle> {
        let slide = p.slides.get(index)?.clone();
        let master = master_of(p, &slide);
        let h = (width_px as f64 * p.slide_size.height / p.slide_size.width.max(1.0)).round().max(1.0) as u32;
        let key = Key {
            slide: Arc::as_ptr(&slide) as usize,
            master: master.as_ref().map(|m| Arc::as_ptr(m) as usize).unwrap_or(0),
            size: (width_px, h),
            edit: false,
            extra: 0,
            want: Region::slide(width_px, h),
            editing_text: None,
        };
        self.clock += 1;
        let clock = self.clock;
        if let Some(e) = self.thumbs.get_mut(&key.slide)
            && e.key == key
        {
            e.stamp = clock;
            return Some(e.tex.clone());
        }
        if *budget == 0 {
            // Keep showing the stale thumbnail while the new one waits.
            return self.thumbs.get(&key.slide).map(|e| e.tex.clone());
        }
        *budget -= 1;
        let (tw, th) = fit_size((width_px, h), max_side(ctx));
        let scale = tw as f64 / p.slide_size.width.max(1.0);
        let img = deckcraft_render::render_slide(p, index, &RenderOpts { scale, size: Some((tw, th)), threads: 0, ..Default::default() });
        self.thumb_renders += 1;
        let tex = upload_editor(ctx, format!("thumb-{index}"), to_color_image(&img, false));
        // Drop the old entry for this slide address; entries for slides that no longer exist age out.
        let region = key.want;
        self.thumbs.insert(key.slide, Entry { key, tex: tex.clone(), region, _slide: slide, _master: master, stamp: clock });
        if self.thumbs.len() > 600 {
            let mut v: Vec<(usize, u64)> = self.thumbs.iter().map(|(k, e)| (*k, e.stamp)).collect();
            v.sort_by_key(|x| x.1);
            for (k, _) in v.into_iter().take(200) {
                self.thumbs.remove(&k);
            }
        }
        Some(tex)
    }
}

#[cfg(test)]
mod tests {
    use super::device_pixel_rect;
    use egui::{Rect, pos2, vec2};

    fn on_grid(v: f32, ppp: f32) -> bool {
        let px = v * ppp;
        (px - px.round()).abs() < 1e-3
    }

    #[test]
    fn a_retina_slide_rect_lands_on_device_pixels() {
        let ppp = 2.0;
        let r = Rect::from_min_size(pos2(10.3, 20.25), vec2(100.2, 50.4));
        let s = device_pixel_rect(r, ppp);
        assert!(on_grid(s.min.x, ppp) && on_grid(s.min.y, ppp) && on_grid(s.max.x, ppp) && on_grid(s.max.y, ppp));
        assert!(s.width() > 0.0 && s.height() > 0.0);
        // One texture pixel per device pixel: the snapped size is an integer pixel count.
        let px = ((s.width() * ppp).round() as u32, (s.height() * ppp).round() as u32);
        assert!((s.width() / px.0 as f32 * ppp - 1.0).abs() < 1e-3);
        assert!((s.height() / px.1 as f32 * ppp - 1.0).abs() < 1e-3);
        let aligned = Rect::from_min_max(pos2(1.0, 2.5), pos2(80.0, 40.0));
        assert_eq!(device_pixel_rect(aligned, ppp), aligned);
    }

    #[test]
    fn a_bad_scale_does_not_move_the_rect() {
        let r = Rect::from_min_size(pos2(3.0, 4.0), vec2(10.0, 12.0));
        assert_eq!(device_pixel_rect(r, 0.0), r);
        assert_eq!(device_pixel_rect(r, f32::NAN), r);
    }
}

/// `r` cut to at most `max` pixels a side, keeping as much of `visible` as fits.
pub(crate) fn fit_texture(mut r: Region, visible: Region, max: u32) -> Region {
    fn axis(start: &mut i32, len: &mut u32, vis: i32, max: u32) {
        if *len > max {
            let hi = *start as i64 + (*len - max) as i64;
            *start = (vis as i64).clamp(*start as i64, hi) as i32;
            *len = max;
        }
    }
    axis(&mut r.x, &mut r.w, visible.x, max);
    axis(&mut r.y, &mut r.h, visible.y, max);
    r
}

#[cfg(test)]
mod fit_tests {
    use super::*;

    #[test]
    fn canvas_textures_stay_within_the_texture_limit() {
        let r = Region { x: -100, y: -50, w: 5000, h: 1000 };
        let f = fit_texture(r, Region { x: 300, y: 0, w: 800, h: 600 }, 2048);
        assert_eq!((f.x, f.y, f.w, f.h), (300, -50, 2048, 1000));
        // Visible past the far end: the cut is pinned there.
        let f = fit_texture(r, Region { x: 4800, y: 0, w: 800, h: 600 }, 2048);
        assert_eq!((f.x, f.w), (-100 + 5000 - 2048, 2048));
        assert_eq!(fit_texture(r, r, 8192), r);
    }

    #[test]
    fn render_sizes_and_pictures_shrink_to_the_limit_keeping_their_shape() {
        assert_eq!(fit_size((195, 10920), 2048), (36, 2048));
        assert_eq!(fit_size((3000, 1500), 2048), (2048, 1024));
        assert_eq!(fit_size((800, 600), 2048), (800, 600));
        assert_eq!(fit_size((0, 0), 0), (1, 1));
        // A 5000 × 3 picture: half red, half blue, averaged down to 1667 × 1.
        let img = ColorImage::new(
            [5000, 3],
            (0..3)
                .flat_map(|y| (0..5000).map(move |x| (x, y)))
                .map(|(x, _)| if x < 2500 { egui::Color32::RED } else { egui::Color32::BLUE })
                .collect(),
        );
        let small = shrink_to(img, 2048);
        assert_eq!(small.size, [1667, 1]);
        assert_eq!(small.pixels[0], egui::Color32::RED);
        assert_eq!(small.pixels[1666], egui::Color32::BLUE);
        let ok = ColorImage::filled([10, 10], egui::Color32::WHITE);
        assert_eq!(shrink_to(ok.clone(), 2048).size, ok.size);
    }
}

/// NexusOffice: an edit draws again only the part of the canvas it changed.
#[cfg(test)]
mod patch_tests {
    use super::*;
    use deckcraft_render::{Changed, RenderOpts, changed_area, render_slide_drawn};
    use serde_json::json;

    fn render(p: &Presentation, index: usize, region: Region, scale: f64, editing: Option<ShapeId>) -> Image {
        let opts = RenderOpts { scale, edit: true, offset: region.offset(), size: Some(region.size()), editing_text: editing, ..Default::default() };
        render_slide_drawn(p, index, &opts).0
    }

    /// The old canvas with the changed part drawn again over it is the new canvas, pixel for
    /// pixel (within rounding), for typed Vietnamese text and a new paragraph, at two scales.
    #[test]
    fn a_canvas_with_its_changed_part_drawn_again_is_the_canvas_drawn_again() {
        let mut s = deckcraft_engine::Session::with_new();
        s.execute("slide.new", &json!({"layout": "titleAndContent", "title": "Báo cáo", "body": "Doanh thu"})).unwrap();
        let id = s.execute("insert.textBox", &json!({"rect": [60, 300, 300, 60], "text": "Ghi chú"})).unwrap()["id"].as_u64().unwrap();
        // Upstream (2026-10): a text box made with its text is selected, not in text editing.
        s.execute("text.edit", &json!({"id": id, "end": true})).unwrap();
        let mut patched = 0;
        for text in ["ữ", " tiếng Việt có dấu", "\nDòng mới"] {
            let before = s.doc().unwrap().doc.clone();
            s.execute("text.insert", &json!({ "text": text })).unwrap();
            let st = s.doc().unwrap();
            let after = st.doc.clone();
            let editing = st.selection.text.as_ref().map(|t| t.shape);
            let index = st.selection.slide;
            let (old_slide, new_slide) = (before.slides[index].clone(), after.slides[index].clone());
            let Changed::Area(area) = changed_area(&after, &old_slide, &new_slide, editing) else { panic!("{text:?}: a text edit changes an area") };
            for scale in [1.0, 1.5] {
                let w = (after.slide_size.width * scale).round() as u32;
                let h = (after.slide_size.height * scale).round() as u32;
                let canvas = Region::slide(w, h);
                let mut old = render(&before, index, canvas, scale, editing);
                let new = render(&after, index, canvas, scale, editing);
                let Some(sub) = patch_region(canvas, area, scale) else { continue };
                let part = render(&after, index, sub, scale, editing);
                assert_eq!((part.width, part.height), (sub.w, sub.h));
                let row = w as usize * 4;
                for y in 0..sub.h as usize {
                    let from = y * sub.w as usize * 4;
                    let to = (sub.y as usize + y) * row + sub.x as usize * 4;
                    old.pixels[to..to + sub.w as usize * 4].copy_from_slice(&part.pixels[from..from + sub.w as usize * 4]);
                }
                let off = old.pixels.iter().zip(&new.pixels).filter(|(x, y)| x.abs_diff(**y) > 2).count();
                assert_eq!(off, 0, "{text:?} at {scale}: {off} channels differ from the canvas drawn whole");
                patched += 1;
            }
        }
        assert!(patched >= 4, "edits were drawn as parts: {patched}");
    }

    /// Typing into a text box draws part of the canvas per key, never the whole slide again.
    #[test]
    fn typing_draws_only_part_of_the_canvas() {
        let ctx = egui::Context::default();
        let mut t = Textures::default();
        let mut s = deckcraft_engine::Session::with_new();
        s.execute("slide.new", &json!({"layout": "titleAndContent", "title": "Báo cáo", "body": "Doanh thu"})).unwrap();
        let id = s.execute("insert.textBox", &json!({"rect": [60, 300, 300, 60], "text": "Ghi"})).unwrap()["id"].as_u64().unwrap();
        // Upstream (2026-10): a text box made with its text is selected, not in text editing.
        s.execute("text.edit", &json!({"id": id, "end": true})).unwrap();
        let draw = |t: &mut Textures, s: &deckcraft_engine::Session| {
            let st = s.doc().unwrap();
            let editing = st.selection.text.as_ref().map(|x| x.shape);
            t.slide(&ctx, &st.doc, st.selection.slide, (960, 540), Region::slide(960, 540), true, 0, false, &[], editing, None).is_some()
        };
        assert!(draw(&mut t, &s));
        let whole = t.canvas_renders;
        for c in " chú ý".chars() {
            s.execute("text.insert", &json!({ "text": c.to_string() })).unwrap();
            assert!(draw(&mut t, &s));
        }
        assert_eq!(t.region_renders, 6, "each key drew part of the canvas");
        assert_eq!(t.canvas_renders - whole, 6, "and nothing whole");
    }

    #[test]
    fn what_changed_is_found() {
        let mut s = deckcraft_engine::Session::with_new();
        s.execute("slide.new", &json!({"layout": "titleAndContent", "title": "T", "body": "B"})).unwrap();
        let p0 = s.doc().unwrap().doc.clone();
        let i = s.doc().unwrap().selection.slide;
        // The notes: nothing on the slide changes.
        let mut notes = (*p0.slides[i]).clone();
        notes.notes = deckcraft_model::text::TextBody::from_text("ghi chú");
        assert_eq!(changed_area(&p0, &p0.slides[i], &notes, None), Changed::Nothing);
        // Another layout: the whole slide.
        let mut other = (*p0.slides[i]).clone();
        other.layout = deckcraft_model::LayoutId(9999);
        assert_eq!(changed_area(&p0, &p0.slides[i], &other, None), Changed::Unknown);
        assert_eq!(patch_region(Region::slide(1000, 500), Rect::new(10.0, 10.0, 20.0, 20.0), 2.0), Some(Region { x: 19, y: 19, w: 22, h: 22 }));
        assert_eq!(patch_region(Region::slide(100, 100), Rect::new(0.0, 0.0, 90.0, 90.0), 1.0), None, "most of the canvas");
        assert_eq!(patch_region(Region::slide(100, 100), Rect::new(f64::NAN, 0.0, 1.0, 1.0), 1.0), None);
    }
}

//! Programmatic art: shrine, character rig, fireworks, cards. Everything is drawn in a virtual
//! 1080-wide coordinate space and scaled to the window. A simple camera (zoom/focus/shake) can be
//! applied to the world layer. PNG sprites in assets/sprites override parts.
use egui::{epaint::Mesh, Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke, TextureHandle, Vec2};
use std::f32::consts::{PI, TAU};

pub const VW: f32 = 1080.0;

pub fn ground_for(vh: f32) -> f32 {
    vh - 110.0_f32.min(vh * 0.12)
}

pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, (a.clamp(0.0, 1.0) * 255.0) as u8)
}
pub fn with_a(c: Color32, a: f32) -> Color32 {
    let [r, g, b, _] = c.to_srgba_unmultiplied();
    rgba(r, g, b, a)
}
pub fn shade(c: Color32, f: f32) -> Color32 {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied((r as f32 * f).min(255.0) as u8, (g as f32 * f).min(255.0) as u8, (b as f32 * f).min(255.0) as u8, a)
}
pub const GOLD: Color32 = Color32::from_rgb(255, 200, 60);
pub const GOLD_D: Color32 = Color32::from_rgb(196, 138, 20);
pub const RED: Color32 = Color32::from_rgb(178, 24, 36);
pub const SKIN: Color32 = Color32::from_rgb(246, 204, 160);
pub const HAIR: Color32 = Color32::from_rgb(40, 28, 24);

// ---------------------------------------------------------------- easing
pub fn clamp01(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}
pub fn ease_out_back(t: f32) -> f32 {
    let t = clamp01(t);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}
pub fn ease_out_elastic(t: f32) -> f32 {
    let t = clamp01(t);
    if t == 0.0 || t == 1.0 {
        return t;
    }
    2f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * (TAU / 3.0)).sin() + 1.0
}
pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - clamp01(t)).powi(3)
}
pub fn ease_in_out(t: f32) -> f32 {
    let t = clamp01(t);
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}
pub fn smooth(t: f32) -> f32 {
    let t = clamp01(t);
    t * t * (3.0 - 2.0 * t)
}
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
fn lerp2(a: (f32, f32), b: (f32, f32), t: f32) -> (f32, f32) {
    (lerp(a.0, b.0, t), lerp(a.1, b.1, t))
}
/// cheap smooth noise in -1..1
pub fn noise1(t: f32) -> f32 {
    ((t * 1.7).sin() * 0.5 + (t * 3.1 + 1.3).sin() * 0.3 + (t * 7.3 + 0.7).sin() * 0.2).clamp(-1.0, 1.0)
}

// ---------------------------------------------------------------- painter
/// Virtual-canvas painter with an optional camera (zoom about a focus point + offset).
pub struct Cx<'a> {
    pub p: &'a Painter,
    pub s: f32,
    pub o: Pos2,
    pub vh: f32,
    pub time: f32,
    /// false in green-screen mode: soft alpha glows would leave green fringes after chroma key
    pub soft: bool,
    pub z: f32,
    pub fx: f32,
    pub fy: f32,
    pub ox: f32,
    pub oy: f32,
    /// ground line override (krathong theme uses a higher river bank)
    pub gy: Option<f32>,
    /// transparent mode: soft dark halo behind characters so they stay readable over any camera
    pub halo: bool,
}

impl<'a> Cx<'a> {
    pub fn new(p: &'a Painter, rect: Rect, time: f32) -> Self {
        let s = rect.width() / VW;
        Self { p, s, o: rect.min, vh: rect.height() / s, time, soft: true, z: 1.0, fx: 0.0, fy: 0.0, ox: 0.0, oy: 0.0, gy: None, halo: false }
    }
    pub fn with_painter<'b>(&self, p: &'b Painter) -> Cx<'b> {
        Cx { p, s: self.s, o: self.o, vh: self.vh, time: self.time, soft: self.soft, z: self.z, fx: self.fx, fy: self.fy, ox: self.ox, oy: self.oy, gy: self.gy, halo: self.halo }
    }
    #[inline]
    pub fn k(&self) -> f32 {
        self.s * self.z
    }
    pub fn pt(&self, x: f32, y: f32) -> Pos2 {
        let x2 = (x - self.fx) * self.z + self.fx + self.ox;
        let y2 = (y - self.fy) * self.z + self.fy + self.oy;
        Pos2::new(self.o.x + x2 * self.s, self.o.y + y2 * self.s)
    }
    pub fn ground(&self) -> f32 {
        self.gy.unwrap_or_else(|| ground_for(self.vh))
    }
    pub fn circle(&self, x: f32, y: f32, r: f32, c: Color32) {
        self.p.circle_filled(self.pt(x, y), r * self.k(), c);
    }
    pub fn ring(&self, x: f32, y: f32, r: f32, w: f32, c: Color32) {
        self.p.circle_stroke(self.pt(x, y), r * self.k(), Stroke::new(w * self.k(), c));
    }
    pub fn rect(&self, x0: f32, y0: f32, x1: f32, y1: f32, r: f32, c: Color32) {
        let rr = (r * self.k()).clamp(0.0, 255.0) as u8;
        self.p.rect_filled(Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1)), rr, c);
    }
    pub fn rect_line(&self, x0: f32, y0: f32, x1: f32, y1: f32, r: f32, w: f32, c: Color32) {
        let rr = (r * self.k()).clamp(0.0, 255.0) as u8;
        self.p.rect_stroke(Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1)), rr, Stroke::new(w * self.k(), c), egui::StrokeKind::Middle);
    }
    pub fn polyline(&self, pts: &[(f32, f32)], w: f32, c: Color32) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        self.p.add(Shape::line(v, Stroke::new(w * self.k(), c)));
    }
    pub fn poly(&self, pts: &[(f32, f32)], c: Color32) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        self.p.add(Shape::convex_polygon(v, c, Stroke::NONE));
    }
    /// arbitrary (possibly concave) polygon via triangle fan from a given interior point
    pub fn fan(&self, center: (f32, f32), pts: &[(f32, f32)], c: Color32) {
        let mut mesh = Mesh::default();
        mesh.colored_vertex(self.pt(center.0, center.1), c);
        for &(x, y) in pts {
            mesh.colored_vertex(self.pt(x, y), c);
        }
        let n = pts.len() as u32;
        for i in 1..n {
            mesh.add_triangle(0, i, i + 1);
        }
        self.p.add(Shape::mesh(mesh));
    }
    pub fn line(&self, a: (f32, f32), b: (f32, f32), w: f32, c: Color32) {
        self.p.line_segment([self.pt(a.0, a.1), self.pt(b.0, b.1)], Stroke::new(w * self.k(), c));
        self.p.circle_filled(self.pt(a.0, a.1), w * self.k() * 0.5, c);
        self.p.circle_filled(self.pt(b.0, b.1), w * self.k() * 0.5, c);
    }
    pub fn seg(&self, a: (f32, f32), b: (f32, f32), w: f32, c: Color32) {
        self.p.line_segment([self.pt(a.0, a.1), self.pt(b.0, b.1)], Stroke::new(w * self.k(), c));
    }
    pub fn ellipse(&self, cx: f32, cy: f32, rx: f32, ry: f32, c: Color32) {
        self.ellipse_rot(cx, cy, rx, ry, 0.0, c);
    }
    pub fn ellipse_rot(&self, cx: f32, cy: f32, rx: f32, ry: f32, ang: f32, c: Color32) {
        let n = 32;
        let (sn, cs) = ang.sin_cos();
        let pts: Vec<(f32, f32)> = (0..n)
            .map(|i| {
                let a = i as f32 / n as f32 * TAU;
                let (x, y) = (rx * a.cos(), ry * a.sin());
                (cx + x * cs - y * sn, cy + x * sn + y * cs)
            })
            .collect();
        self.poly(&pts, c);
    }
    /// soft radial glow (normal alpha blend)
    /// soft dark backing behind a character (transparent mode only) - keeps light outfits readable on bright cameras
    pub fn halo(&self, x: f32, y: f32, rx: f32, ry: f32) {
        if !self.halo || !self.soft {
            return;
        }
        let n = 5;
        for i in 0..n {
            let u = i as f32 / n as f32;
            self.ellipse(x, y, rx * (1.0 - 0.45 * u), ry * (1.0 - 0.45 * u), rgba(10, 6, 24, 0.07));
        }
    }
    pub fn glow(&self, x: f32, y: f32, r: f32, c: Color32, a: f32) {
        if !self.soft || a <= 0.003 {
            return;
        }
        self.radial(x, y, r, with_a(c, a), with_a(c, 0.0));
    }
    /// additive-ish glow: premultiplied colour with reduced alpha so it brightens what's below
    /// but still shows on a transparent window.
    pub fn glow_add(&self, x: f32, y: f32, r: f32, c: Color32, intensity: f32) {
        if !self.soft || intensity <= 0.003 {
            return;
        }
        let i = intensity.clamp(0.0, 1.0);
        let [cr, cg, cb, _] = c.to_srgba_unmultiplied();
        let center = Color32::from_rgba_premultiplied((cr as f32 * i) as u8, (cg as f32 * i) as u8, (cb as f32 * i) as u8, (i * 0.45 * 255.0) as u8);
        self.radial(x, y, r, center, Color32::TRANSPARENT);
    }
    fn radial(&self, x: f32, y: f32, r: f32, inner: Color32, outer: Color32) {
        let mut mesh = Mesh::default();
        let n = 40;
        let center = self.pt(x, y);
        mesh.colored_vertex(center, inner);
        for i in 0..=n {
            let ang = i as f32 / n as f32 * TAU;
            mesh.colored_vertex(center + Vec2::new(ang.cos(), ang.sin()) * r * self.k(), outer);
        }
        for i in 1..=n as u32 {
            mesh.add_triangle(0, i, i + 1);
        }
        self.p.add(Shape::mesh(mesh));
    }
    pub fn vgradient(&self, x0: f32, y0: f32, x1: f32, y1: f32, top: Color32, bot: Color32) {
        let mut mesh = Mesh::default();
        mesh.colored_vertex(self.pt(x0, y0), top);
        mesh.colored_vertex(self.pt(x1, y0), top);
        mesh.colored_vertex(self.pt(x1, y1), bot);
        mesh.colored_vertex(self.pt(x0, y1), bot);
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        self.p.add(Shape::mesh(mesh));
    }
    pub fn font(&self, size: f32, bold: bool) -> FontId {
        // quantise: animated sizes (pop-ins, camera zoom) would otherwise rasterise a new glyph set every frame
        // and keep growing egui's font atlas (seen in the soak test as RSS steps 36 -> 325 MB)
        let px = (size * self.k()).max(4.0);
        let px = if px < 48.0 { px.round() } else { (px / 3.0).round() * 3.0 };
        FontId::new(px, if bold { FontFamily::Name("bold".into()) } else { FontFamily::Proportional })
    }
    pub fn text(&self, x: f32, y: f32, anchor: Align2, t: &str, size: f32, c: Color32, bold: bool) -> Rect {
        self.p.text(self.pt(x, y), anchor, t, self.font(size, bold), c)
    }
    /// text with outline (readable over any background / chroma)
    pub fn text_o(&self, x: f32, y: f32, anchor: Align2, t: &str, size: f32, c: Color32, outline: Color32, bold: bool) -> Rect {
        let d = (size * 0.07).max(1.5);
        for i in 0..8 {
            let a = i as f32 / 8.0 * TAU;
            self.text(x + a.cos() * d, y + a.sin() * d, anchor, t, size, outline, bold);
        }
        self.text(x, y, anchor, t, size, c, bold)
    }
    pub fn measure(&self, t: &str, size: f32, bold: bool) -> Vec2 {
        let g = self.p.layout_no_wrap(t.to_string(), self.font(size, bold), Color32::WHITE);
        g.size() / self.k()
    }
    /// centered, wrapped paragraph whose top-center is (x, y); returns height
    pub fn text_wrapped(&self, x: f32, y: f32, t: &str, size: f32, width: f32, c: Color32, bold: bool) -> f32 {
        let mut job = egui::text::LayoutJob::simple(t.to_string(), self.font(size, bold), c, width * self.k());
        job.halign = egui::Align::Center;
        let g = self.p.layout_job(job);
        let h = g.size().y / self.k();
        self.p.galley(self.pt(x, y), g, c);
        h
    }
    pub fn image(&self, tex: &TextureHandle, x0: f32, y0: f32, x1: f32, y1: f32, tint: Color32) {
        self.p.image(tex.id(), Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1)), Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), tint);
    }
    pub fn clip_rect_v(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
        Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1))
    }
}

// ---------------------------------------------------------------------------
pub struct Sprites {
    pub background: Option<TextureHandle>,
    pub buddha: Option<TextureHandle>,
    pub character: Option<TextureHandle>,
}

pub const SHRINE_X: f32 = 860.0;
pub const PRAY_X: f32 = 610.0;

/// Painted temple backdrop (background = "scene").
pub fn draw_backdrop(cx: &Cx, sprites: &Sprites) {
    let h = cx.vh;
    if let Some(t) = &sprites.background {
        cx.image(t, -40.0, -40.0, VW + 40.0, h + 40.0, Color32::WHITE);
        return;
    }
    cx.vgradient(-60.0, -60.0, VW + 60.0, h * 0.75, rgba(40, 20, 70, 1.0), rgba(250, 140, 70, 1.0));
    for i in 0..40 {
        let x = (i as f32 * 263.7) % VW;
        let y = (i as f32 * 97.3) % (h * 0.4);
        let tw = 0.5 + 0.5 * (cx.time * 2.0 + i as f32).sin();
        cx.circle(x, y, 2.0, rgba(255, 255, 230, 0.3 + 0.5 * tw));
    }
    cx.glow(300.0, h * 0.62, 260.0, rgba(255, 220, 140, 1.0), 0.45);
    let g = cx.ground();
    // soft clouds
    for (x, y, w) in [(160.0, h * 0.18, 120.0), (520.0, h * 0.1, 90.0), (760.0, h * 0.24, 140.0)] {
        let dx = (cx.time * 0.05).sin() * 10.0;
        for (ox, oy, r) in [(-0.5, 0.1, 0.4), (0.0, -0.1, 0.55), (0.5, 0.1, 0.42), (0.2, 0.15, 0.45)] {
            cx.circle(x + dx + ox * w, y + oy * w, r * w * 0.6, rgba(255, 196, 160, 0.10));
        }
    }
    // distant chedi + ubosot silhouettes (tiered roofs with chofa)
    let far = rgba(120, 52, 74, 0.78);
    draw_chedi(cx, 90.0, g - 40.0, 1.0, far);
    draw_ubosot(cx, 330.0, g - 40.0, 0.95, far, rgba(255, 190, 110, 0.55));
    draw_chedi(cx, 560.0, g - 40.0, 0.7, far);
    // tree line
    for i in 0..14 {
        let x = i as f32 * 82.0 - 20.0;
        let r = 34.0 + ((i * 29) % 17) as f32;
        cx.circle(x, g - 44.0 - ((i * 13) % 9) as f32, r, rgba(70, 40, 60, 0.85));
    }
    cx.vgradient(-60.0, g - 40.0, VW + 60.0, h + 60.0, rgba(120, 60, 40, 1.0), rgba(60, 25, 20, 1.0));
    for i in 0..12 {
        let x = i as f32 * 100.0;
        cx.seg((x, g - 40.0), (x - 80.0, h), 2.0, rgba(0, 0, 0, 0.15));
    }
}

/// Floor / platform used in transparent & green modes.
fn draw_chedi(cx: &Cx, x: f32, base: f32, sc: f32, c: Color32) {
    cx.rect(x - 70.0 * sc, base - 30.0 * sc, x + 70.0 * sc, base, 2.0, c);
    cx.rect(x - 54.0 * sc, base - 52.0 * sc, x + 54.0 * sc, base - 28.0 * sc, 2.0, c);
    cx.ellipse(x, base - 90.0 * sc, 50.0 * sc, 46.0 * sc, c);
    cx.rect(x - 16.0 * sc, base - 150.0 * sc, x + 16.0 * sc, base - 120.0 * sc, 1.0, c);
    cx.poly(&[(x - 14.0 * sc, base - 148.0 * sc), (x + 14.0 * sc, base - 148.0 * sc), (x, base - 280.0 * sc)], c);
    for i in 0..5 {
        let y = base - 160.0 * sc - i as f32 * 18.0 * sc;
        let w = (12.0 - i as f32 * 2.0) * sc;
        cx.ellipse(x, y, w, 3.0 * sc, c);
    }
}

/// Thai ordination hall silhouette: stacked gable roofs with chofa horns + glowing windows.
fn draw_ubosot(cx: &Cx, x: f32, base: f32, sc: f32, c: Color32, win: Color32) {
    let w = 130.0 * sc;
    cx.rect(x - w, base - 80.0 * sc, x + w, base, 2.0, c);
    for i in 0..4 {
        let wx = x - w + 34.0 * sc + i as f32 * 58.0 * sc;
        cx.rect(wx, base - 62.0 * sc, wx + 18.0 * sc, base - 22.0 * sc, 9.0 * sc, win);
    }
    for (k, (rw, ry, rh)) in [(1.15, 80.0, 70.0), (0.85, 128.0, 70.0), (0.55, 176.0, 66.0)].iter().enumerate() {
        let bw = w * rw;
        let y0 = base - ry * sc;
        let top = y0 - rh * sc;
        cx.poly(&[(x - bw, y0), (x + bw, y0), (x + bw * 0.25, top), (x - bw * 0.25, top)], c);
        for d in [-1.0f32, 1.0] {
            let ex = x + d * bw;
            cx.poly(&[(ex, y0), (ex + d * 10.0 * sc, y0 - 18.0 * sc), (ex + d * 18.0 * sc, y0 - 30.0 * sc), (ex + d * 4.0 * sc, y0 - 12.0 * sc)], c);
        }
        if k == 2 {
            cx.poly(&[(x - 6.0 * sc, top), (x + 6.0 * sc, top), (x + 14.0 * sc, top - 30.0 * sc), (x, top - 14.0 * sc), (x - 14.0 * sc, top - 30.0 * sc)], c);
        }
    }
}

pub fn draw_platform(cx: &Cx) {
    let g = cx.ground();
    cx.ellipse(540.0, g + 10.0, 560.0, 34.0, rgba(0, 0, 0, 0.28));
    cx.rect(20.0, g - 6.0, VW - 20.0, g + 18.0, 10.0, rgba(122, 62, 38, 0.95));
    cx.rect(20.0, g - 6.0, VW - 20.0, g + 2.0, 6.0, rgba(196, 120, 70, 1.0));
}

/// carpet unrolls from the left: `reach` 0..1
pub fn draw_carpet(cx: &Cx, reach: f32, alpha: f32, royal: bool) {
    if alpha <= 0.0 || reach <= 0.0 {
        return;
    }
    let g = cx.ground();
    let end = (PRAY_X + 80.0) * ease_out_cubic(reach);
    let c = if royal { rgba(170, 20, 40, alpha) } else { rgba(220, 170, 50, alpha) };
    cx.poly(&[(0.0, g - 4.0), (end - 10.0, g - 4.0), (end, g + 14.0), (0.0, g + 14.0)], c);
    cx.rect(0.0, g - 6.0, end, g - 2.0, 0.0, with_a(GOLD, alpha));
    // rolled end
    if reach < 1.0 {
        cx.circle(end, g + 2.0, 12.0, shade(c, 0.8));
    }
}

fn flame(cx: &Cx, x: f32, y: f32, h: f32, seed: f32) {
    let t = cx.time;
    let fl = noise1(t * 6.0 + seed);
    let tip_dx = fl * 4.0;
    let hh = h * (1.0 + 0.18 * noise1(t * 9.0 + seed * 3.0));
    cx.glow_add(x, y - hh * 0.5, 46.0 + 8.0 * fl, rgba(255, 170, 60, 1.0), 0.75);
    let outer = [(x - 8.0, y), (x - 9.0, y - hh * 0.35), (x - 4.0, y - hh * 0.75), (x + tip_dx, y - hh), (x + 4.0, y - hh * 0.75), (x + 9.0, y - hh * 0.35), (x + 8.0, y)];
    cx.poly(&outer, rgba(255, 140, 30, 0.95));
    let inner = [(x - 5.0, y), (x - 5.0, y - hh * 0.3), (x + tip_dx * 0.6, y - hh * 0.72), (x + 5.0, y - hh * 0.3), (x + 5.0, y)];
    cx.poly(&inner, rgba(255, 220, 90, 1.0));
    cx.ellipse(x, y - 4.0, 3.0, 5.0, rgba(255, 255, 230, 1.0));
}

fn smoke(cx: &Cx, x: f32, y: f32, seed: f32) {
    let t = cx.time;
    let n = 18;
    let mut prev: Option<(f32, f32)> = None;
    for i in 0..n {
        let u = i as f32 / n as f32;
        let yy = y - u * 170.0;
        let amp = 3.0 + u * 22.0;
        let xx = x + (yy * 0.045 + t * 1.6 + seed).sin() * amp + (t * 0.7 + seed).sin() * u * 10.0;
        if let Some(p) = prev {
            cx.seg(p, (xx, yy), 2.0 + u * 9.0, rgba(225, 225, 235, 0.32 * (1.0 - u).powf(1.3)));
        }
        prev = Some((xx, yy));
    }
}

/// The Buddha shrine on the right. `glow` 0..1 intensifies the halo (during prayer).
pub fn draw_shrine(cx: &Cx, sprites: &Sprites, glow: f32, show_cylinder: bool) {
    let g = cx.ground();
    let x = SHRINE_X;
    let t = cx.time;
    let pulse = 0.5 + 0.5 * (t * 1.6).sin();
    cx.glow(x, g - 250.0, 230.0 + glow * 90.0 + pulse * 12.0, GOLD, 0.22 + glow * 0.45 + pulse * 0.05);
    // altar
    cx.rect(x - 170.0, g - 70.0, x + 170.0, g, 8.0, rgba(140, 20, 30, 1.0));
    cx.rect(x - 170.0, g - 70.0, x + 170.0, g - 60.0, 4.0, GOLD_D);
    cx.rect(x - 130.0, g - 120.0, x + 130.0, g - 70.0, 8.0, rgba(170, 30, 40, 1.0));
    cx.rect(x - 130.0, g - 120.0, x + 130.0, g - 110.0, 4.0, GOLD);
    for i in 0..5 {
        cx.circle(x - 100.0 + i as f32 * 50.0, g - 35.0, 9.0, GOLD_D);
        let sh = ((t * 0.8 - i as f32 * 0.25).sin() * 0.5 + 0.5).powi(8);
        cx.circle(x - 103.0 + i as f32 * 50.0, g - 38.0, 3.0, rgba(255, 255, 220, sh));
    }
    let base = g - 120.0;
    let hy = base - 225.0;
    // halo rays (rotate slowly, stronger while praying)
    if cx.soft {
        let rays = 16;
        let ra = 0.08 + 0.25 * glow + 0.04 * pulse;
        for i in 0..rays {
            let a = i as f32 / rays as f32 * TAU + t * 0.25;
            let len = 150.0 + 40.0 * (t * 2.0 + i as f32).sin() + glow * 60.0;
            let (s, c) = a.sin_cos();
            cx.poly(&[(x + c * 70.0 - s * 5.0, hy + s * 70.0 + c * 5.0), (x + c * 70.0 + s * 5.0, hy + s * 70.0 - c * 5.0), (x + c * len, hy + s * len)], rgba(255, 225, 130, ra));
        }
    }
    if let Some(tex) = &sprites.buddha {
        let sz = tex.size_vec2();
        let hgt = 330.0;
        let w = hgt * sz.x / sz.y.max(1.0);
        cx.image(tex, x - w / 2.0, base - hgt, x + w / 2.0, base, Color32::WHITE);
    } else {
        cx.circle(x, hy, 72.0 + glow * 6.0 + pulse * 2.0, with_a(GOLD, 0.32 + glow * 0.4));
        cx.ring(x, hy, 76.0 + glow * 6.0 + pulse * 2.0, 5.0, with_a(GOLD, 0.85));
        for i in 0..7 {
            let px = x - 105.0 + i as f32 * 35.0;
            cx.ellipse(px, base - 18.0, 24.0, 22.0, if i % 2 == 0 { rgba(240, 150, 170, 1.0) } else { rgba(250, 180, 195, 1.0) });
        }
        cx.rect(x - 115.0, base - 14.0, x + 115.0, base, 6.0, rgba(210, 120, 140, 1.0));
        cx.ellipse(x, base - 52.0, 112.0, 34.0, GOLD_D);
        cx.ellipse(x, base - 58.0, 104.0, 26.0, GOLD);
        cx.poly(&[(x - 72.0, base - 70.0), (x + 72.0, base - 70.0), (x + 50.0, base - 175.0), (x - 50.0, base - 175.0)], GOLD);
        cx.poly(&[(x - 50.0, base - 175.0), (x + 10.0, base - 175.0), (x + 72.0, base - 70.0), (x + 30.0, base - 70.0)], GOLD_D);
        cx.ellipse(x, base - 80.0, 38.0, 14.0, rgba(255, 215, 110, 1.0));
        cx.rect(x - 16.0, base - 195.0, x + 16.0, base - 170.0, 6.0, GOLD);
        cx.ellipse(x - 44.0, base - 228.0, 9.0, 24.0, GOLD_D);
        cx.ellipse(x + 44.0, base - 228.0, 9.0, 24.0, GOLD_D);
        cx.ellipse(x, base - 230.0, 42.0, 48.0, rgba(255, 210, 80, 1.0));
        cx.ellipse(x, base - 262.0, 40.0, 20.0, GOLD_D);
        cx.ellipse(x, base - 285.0, 20.0, 18.0, GOLD_D);
        cx.poly(&[(x - 8.0, base - 298.0), (x + 8.0, base - 298.0), (x, base - 335.0)], GOLD);
        cx.seg((x - 22.0, base - 232.0), (x - 8.0, base - 230.0), 3.0, rgba(140, 90, 10, 1.0));
        cx.seg((x + 8.0, base - 230.0), (x + 22.0, base - 232.0), 3.0, rgba(140, 90, 10, 1.0));
        cx.seg((x - 9.0, base - 207.0), (x + 9.0, base - 207.0), 3.0, rgba(170, 100, 20, 1.0));
        // moving highlight sweep on the statue
        let sw = ((t * 0.35).fract() * 2.0 - 0.5) * 260.0;
        cx.glow_add(x - 120.0 + sw, base - 150.0, 40.0, rgba(255, 250, 220, 1.0), 0.25);
    }
    // candles
    for (i, side) in [-1.0f32, 1.0].iter().enumerate() {
        let cxp = x + side * 150.0;
        cx.rect(cxp - 9.0, g - 160.0, cxp + 9.0, g - 70.0, 3.0, rgba(255, 245, 220, 1.0));
        cx.rect(cxp - 9.0, g - 160.0, cxp - 4.0, g - 70.0, 2.0, rgba(235, 220, 190, 1.0));
        cx.rect(cxp - 14.0, g - 74.0, cxp + 14.0, g - 68.0, 2.0, GOLD_D);
        cx.seg((cxp, g - 160.0), (cxp, g - 166.0), 2.0, HAIR);
        flame(cx, cxp, g - 164.0, 30.0, i as f32 * 5.3);
    }
    // incense bowl + smoke wisps
    let ix = x - 70.0;
    for k in 0..3 {
        let sx = ix - 8.0 + k as f32 * 8.0;
        smoke(cx, sx, g - 172.0, k as f32 * 2.1);
        cx.seg((sx, g - 136.0), (sx + (k as f32 - 1.0) * 2.0, g - 170.0), 2.0, rgba(150, 60, 30, 1.0));
        let ember = 0.7 + 0.3 * (t * 5.0 + k as f32).sin();
        cx.circle(sx + (k as f32 - 1.0) * 2.0, g - 170.0, 2.5, rgba(255, (90.0 * ember) as u8, 40, 1.0));
    }
    cx.ellipse(ix, g - 132.0, 26.0, 12.0, GOLD_D);
    cx.ellipse(ix, g - 136.0, 22.0, 6.0, rgba(120, 90, 60, 1.0));
    if show_cylinder {
        draw_cylinder(cx, x + 100.0, g - 70.0, 0.0, 0.55, 0.0, -1.0);
    }
}

/// เซียมซี cylinder. (x,y) = bottom center, angle radians, scale.
/// `jostle` = time used to bounce the sticks, `lift` 0..1 = one stick rising out (-1 = none).
pub fn draw_cylinder(cx: &Cx, x: f32, y: f32, ang: f32, sc: f32, jostle: f32, lift: f32) {
    let rot = |px: f32, py: f32| -> (f32, f32) {
        let (s, c) = ang.sin_cos();
        (x + (px * c - py * s) * sc, y + (px * s + py * c) * sc)
    };
    for i in 0..9 {
        let off = -24.0 + i as f32 * 6.0;
        let bounce = if jostle > 0.0 { ((jostle * 19.0 + i as f32 * 1.7).sin()).abs() * 10.0 } else { 0.0 };
        let mut top = -150.0 - ((i * 37) % 5) as f32 * 7.0 - bounce;
        if i == 4 && lift <= -1.5 {
            continue;
        }
        if i == 4 && lift >= 0.0 {
            top -= ease_in_out(lift) * 70.0;
        }
        let b = rot(off, -90.0);
        let e = rot(off * 1.15, top);
        cx.seg(b, e, 4.0 * sc, rgba(215, 170, 90, 1.0));
        cx.circle(e.0, e.1, 2.5 * sc, RED);
        if i == 4 && lift > 0.0 {
            cx.glow_add(e.0, e.1, 26.0 * sc, GOLD, 0.6 * lift);
        }
    }
    let body = [rot(-34.0, 0.0), rot(34.0, 0.0), rot(36.0, -100.0), rot(-36.0, -100.0)];
    cx.poly(&body, RED);
    cx.poly(&[rot(-34.0, 0.0), rot(-20.0, 0.0), rot(-21.0, -100.0), rot(-36.0, -100.0)], shade(RED, 0.75));
    cx.poly(&[rot(-34.0, -10.0), rot(34.0, -10.0), rot(34.0, -18.0), rot(-34.0, -18.0)], GOLD);
    cx.poly(&[rot(-35.0, -82.0), rot(35.0, -82.0), rot(36.0, -92.0), rot(-36.0, -92.0)], GOLD);
    let c = rot(0.0, -50.0);
    cx.circle(c.0, c.1, 15.0 * sc, GOLD);
    cx.text(c.0, c.1, Align2::CENTER_CENTER, "ซี", 16.0 * sc, RED, true);
}

/// a single fortune stick lying/flying at (x,y) with angle
pub fn draw_stick(cx: &Cx, x: f32, y: f32, ang: f32, glow: f32) {
    let (s, c) = ang.sin_cos();
    let len = 58.0;
    if glow > 0.0 {
        cx.glow_add(x, y, 90.0, GOLD, glow);
    }
    cx.line((x - c * len, y - s * len), (x + c * len, y + s * len), 7.0, rgba(225, 180, 100, 1.0));
    cx.seg((x - c * len, y - s * len + 2.0), (x + c * len, y + s * len + 2.0), 2.0, rgba(180, 130, 60, 1.0));
    cx.circle(x + c * len, y + s * len, 5.0, RED);
}

// ---------------------------------------------------------------------------
/// Animation inputs for the character rig.
#[derive(Clone)]
pub struct Pose {
    pub phase: f32,     // walk cycle phase (radians), advances with distance
    pub walk: f32,      // 0..1 blend of the walk cycle
    pub kneel: f32,     // 0..1
    pub bow: f32,       // 0..1 deep bow
    pub hands: f32,     // 0..1 blend from swinging arms to joined/holding hands
    pub raise: f32,     // 0..1 joined hands: chest -> forehead
    pub hold_cyl: Option<(f32, f32, f32)>, // (tilt, jostle time, lift; -2 = stick gone)
    pub facing: f32,    // 1 = right, -1 = left
    pub blink: f32,     // 0..1 eyes closed
    pub breath: f32,    // -1..1
    pub cape_lag: f32,  // trailing offset of cloth (px)
    pub squash: f32,    // >0 squash (landing), <0 stretch (hop)
    pub carry: f32,     // 0..1 hands low in front (carrying a krathong)
    pub reach: Option<((f32, f32), f32)>, // absolute hand target + blend
    pub expr: u8,       // 0 normal, 1 happy closed eyes ^^, 2 sparkle eyes
}

impl Default for Pose {
    fn default() -> Self {
        Self { phase: 0.0, walk: 0.0, kneel: 0.0, bow: 0.0, hands: 0.0, raise: 0.0, hold_cyl: None, facing: 1.0, blink: 0.0, breath: 0.0, cape_lag: 0.0, squash: 0.0, carry: 0.0, reach: None, expr: 0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub tier: usize,
    pub shirt: Color32,
    pub pants: Color32,
    pub scale: f32,
    pub hair: Color32,
    pub style: u8,
}

pub struct Rig {
    pub hip: (f32, f32),
    pub shoulder: (f32, f32),
    pub head: (f32, f32),
    pub legs: [((f32, f32), (f32, f32)); 2], // (knee, foot)  [back, front]
    pub arms: [((f32, f32), (f32, f32)); 2], // (elbow, hand) [back, front]
    pub up: (f32, f32),
    pub fwd: (f32, f32),
}

/// chibi head radius (before scale)
pub const HEAD_R: f32 = 46.0;

fn ik(a: (f32, f32), t: (f32, f32), l1: f32, l2: f32, bend: f32) -> (f32, f32) {
    let dx = t.0 - a.0;
    let dy = t.1 - a.1;
    let d = (dx * dx + dy * dy).sqrt().clamp(0.001, l1 + l2 - 0.01);
    let cos_a = ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d)).clamp(-1.0, 1.0);
    let base = dy.atan2(dx);
    let ang = base + bend * cos_a.acos();
    (a.0 + ang.cos() * l1, a.1 + ang.sin() * l1)
}

/// Solve the chibi skeleton (feet planted at ground `g`).
pub fn solve_rig(x: f32, g: f32, pose: &Pose, k: f32) -> Rig {
    let f = pose.facing;
    let thigh = 24.0 * k;
    let shin = 22.0 * k;
    let mut legs = [((0.0, 0.0), (0.0, 0.0)); 2];
    let mut lowest: f32 = 0.0;
    for i in 0..2 {
        let ph = pose.phase + i as f32 * PI;
        let wa = 0.5 * ph.sin();
        let wb = 0.1 + 0.9 * ph.cos().max(0.0).powf(1.5);
        let (ka, kb) = (1.5 - i as f32 * 0.1, 2.9);
        let a = lerp(lerp(0.0, wa, pose.walk), ka, pose.kneel);
        let b = lerp(lerp(0.04, wb, pose.walk), kb, pose.kneel);
        let knee = (a.sin() * f * thigh, a.cos() * thigh);
        let foot = (knee.0 + (a - b).sin() * f * shin, knee.1 + (a - b).cos() * shin);
        lowest = lowest.max(knee.1).max(foot.1);
        legs[i] = (knee, foot);
    }
    let squash = pose.squash * 7.0 * k;
    let hip = (x, g - lowest - 7.0 * k + squash);
    for l in legs.iter_mut() {
        l.0 = (hip.0 + l.0 .0, hip.1 + l.0 .1);
        l.1 = (hip.0 + l.1 .0, hip.1 + l.1 .1);
    }
    let lean = 0.06 * pose.walk + pose.bow * 0.5 + pose.kneel * 0.03;
    let up = (lean.sin() * f, -lean.cos());
    let fwd = (lean.cos() * f, lean.sin());
    let torso = 50.0 * k * (1.0 + 0.02 * pose.breath) - squash * 0.4;
    let shoulder = (hip.0 + up.0 * torso, hip.1 + up.1 * torso);
    let nod = pose.bow * 0.3;
    let hup = ((lean + nod).sin() * f, -(lean + nod).cos());
    let hr = HEAD_R * k;
    let head = (shoulder.0 + hup.0 * (hr * 0.82), shoulder.1 + hup.1 * (hr * 0.82));
    let ua = 22.0 * k;
    let fa = 21.0 * k;
    let sh = (shoulder.0 - up.0 * 7.0 * k, shoulder.1 - up.1 * 7.0 * k);
    let chest = (shoulder.0 - up.0 * 16.0 * k + fwd.0 * 22.0 * k, shoulder.1 - up.1 * 16.0 * k + fwd.1 * 22.0 * k);
    let forehead = (head.0 + fwd.0 * (hr + 10.0 * k) + up.0 * 2.0 * k, head.1 + fwd.1 * (hr + 10.0 * k) + up.1 * 2.0 * k);
    let low = (hip.0 + fwd.0 * 34.0 * k + up.0 * 12.0 * k, hip.1 + fwd.1 * 34.0 * k + up.1 * 12.0 * k);
    let mut arms = [((0.0, 0.0), (0.0, 0.0)); 2];
    for i in 0..2 {
        let ph = pose.phase + i as f32 * PI + PI;
        let sw = -0.6 * ph.sin() * pose.walk + 0.08;
        let bend = 0.3 + 0.3 * (ph.sin().max(0.0)) * pose.walk;
        let el = (sh.0 + (sw + lean).sin() * f * ua, sh.1 + (sw + lean).cos() * ua);
        let hd = (el.0 + (sw + lean + bend).sin() * f * fa, el.1 + (sw + lean + bend).cos() * fa);
        let mut target = if pose.hold_cyl.is_some() {
            (shoulder.0 + fwd.0 * 34.0 * k - up.0 * 14.0 * k, shoulder.1 + fwd.1 * 34.0 * k - up.1 * 14.0 * k)
        } else {
            lerp2(lerp2(chest, forehead, ease_in_out(pose.raise)), low, pose.carry)
        };
        if let Some((p, w)) = pose.reach {
            // keep the hand within reach of the shoulder
            let dx = p.0 - sh.0;
            let dy = p.1 - sh.1;
            let d = (dx * dx + dy * dy).sqrt().max(0.01);
            let m = (ua + fa) * 0.98;
            let pp = if d > m { (sh.0 + dx / d * m, sh.1 + dy / d * m) } else { p };
            target = lerp2(target, pp, clamp01(w));
        }
        let hand = lerp2(hd, target, ease_in_out(pose.hands));
        let elbow = if pose.hands > 0.01 { ik(sh, hand, ua, fa, -f) } else { el };
        arms[i] = (elbow, hand);
    }
    Rig { hip, shoulder, head, legs, arms, up, fwd }
}

/// an arc / polyline helper
pub fn arc_pts(cx0: f32, cy0: f32, rx: f32, ry: f32, a0: f32, a1: f32, n: usize) -> Vec<(f32, f32)> {
    (0..=n).map(|i| {
        let a = lerp(a0, a1, i as f32 / n as f32);
        (cx0 + a.cos() * rx, cy0 + a.sin() * ry)
    }).collect()
}

fn draw_eye(cx: &Cx, x: f32, y: f32, k: f32, closed: f32, expr: u8, f: f32, big: f32) {
    let ink = rgba(58, 34, 30, 1.0);
    if expr == 1 {
        // happy ^^ arcs
        let pts = arc_pts(x, y + 3.0 * k, 6.5 * k * big, 6.0 * k, PI * 1.1, PI * 1.9, 8);
        cx.polyline(&pts, 3.2 * k, ink);
        return;
    }
    if closed > 0.6 {
        let pts = arc_pts(x, y - 1.0 * k, 6.5 * k * big, 3.0 * k, PI * 0.1, PI * 0.9, 8);
        cx.polyline(&pts, 3.0 * k, ink);
        return;
    }
    let ry = 9.0 * k * big * (1.0 - closed);
    let rx = 6.8 * k * big;
    cx.ellipse(x, y, rx, ry, ink);
    // iris gradient (lighter bottom)
    cx.ellipse(x, y + ry * 0.28, rx * 0.78, ry * 0.6, rgba(120, 70, 50, 1.0));
    cx.ellipse(x, y + ry * 0.45, rx * 0.55, ry * 0.32, rgba(190, 120, 80, 1.0));
    if expr == 2 {
        // sparkle eyes: star highlight + gold glint
        sparkle(cx, x + 1.5 * k * f, y - ry * 0.25, 6.5 * k * big, Color32::WHITE);
        cx.circle(x - 2.0 * k * f, y + ry * 0.45, 1.6 * k, rgba(255, 230, 140, 1.0));
    } else {
        cx.circle(x + 2.0 * k * f, y - ry * 0.38, 2.9 * k * big, Color32::WHITE);
        cx.circle(x - 2.2 * k * f, y + ry * 0.42, 1.4 * k * big, rgba(255, 255, 255, 0.9));
    }
    // upper lash line
    let pts = arc_pts(x, y, rx * 1.05, ry * 1.02, PI * 1.12, PI * 1.88, 8);
    cx.polyline(&pts, 2.4 * k, ink);
}

/// Draw a chibi character with feet at (x, ground).
pub fn draw_character(cx: &Cx, sprites: &Sprites, x: f32, pose: &Pose, look: &Look) -> Rig {
    let g = cx.ground();
    let k = look.scale;
    let f = pose.facing;
    let tier = look.tier;
    let t = cx.time;
    let rig = solve_rig(x, g, pose, k);
    let hr = HEAD_R * k;
    {
        // readability halo spans head..feet (transparent mode only)
        let top = rig.head.1 - hr * 1.3;
        let cy = (top + g) / 2.0;
        cx.halo((rig.head.0 + x) / 2.0, cy, hr * 1.9, (g - top) / 2.0 + 14.0 * k);
    }
    // palette per tier (tier 0/1 keep the viewer's own pastel colours)
    let (shirt, skirt) = match tier {
        0 | 1 => (look.shirt, look.pants),
        2 => (rgba(246, 196, 72, 1.0), rgba(196, 40, 70, 1.0)),
        3 => (rgba(252, 210, 84, 1.0), rgba(150, 30, 90, 1.0)),
        _ => (rgba(255, 228, 130, 1.0), rgba(120, 18, 60, 1.0)),
    };
    let body_c = lerp2(rig.hip, rig.shoulder, 0.5);
    // soft contact shadow
    cx.ellipse(x, g + 2.0, 44.0 * k * (1.0 + 0.1 * pose.kneel), 9.0 * k, rgba(0, 0, 0, 0.22));
    if tier >= 3 {
        let pulse = 0.5 + 0.5 * (t * 3.0).sin();
        let r = if tier >= 4 { 190.0 } else { 130.0 };
        cx.glow(body_c.0, body_c.1 - 20.0 * k, r * k, GOLD, 0.28 + 0.18 * pulse);
        if tier >= 4 && cx.soft {
            let o = (body_c.0, body_c.1 - 20.0 * k);
            let hdx = rig.head.0 - o.0;
            let hdy = rig.head.1 - o.1;
            let hdist = (hdx * hdx + hdy * hdy).sqrt().max(1.0);
            for i in 0..12 {
                let ang = i as f32 / 12.0 * TAU + t * 0.6;
                let (s, c) = ang.sin_cos();
                // skip rays that would run through the face
                if (hdx * c + hdy * s) / hdist > 0.55 {
                    continue;
                }
                let inner = 58.0 * k;
                let len = (180.0 + 26.0 * (t * 3.0 + i as f32).sin()) * k;
                cx.poly(&[(o.0 + c * inner, o.1 + s * inner), (o.0 + c * len - s * 12.0, o.1 + s * len + c * 12.0), (o.0 + c * len + s * 12.0, o.1 + s * len - c * 12.0)], rgba(255, 232, 150, 0.12));
            }
        }
    }
    // royal cape behind the body
    if tier >= 4 {
        let n = 6;
        let top_l = (rig.shoulder.0 - 16.0 * k * f, rig.shoulder.1 + 4.0 * k);
        let top_r = (rig.shoulder.0 + 6.0 * k * f, rig.shoulder.1);
        let mut hem = vec![];
        for i in 0..=n {
            let u = i as f32 / n as f32;
            let wave = (t * 5.0 + u * 4.0).sin() * (3.0 + pose.walk * 5.0) * k;
            let hx = rig.hip.0 - f * (22.0 + u * 56.0) * k - f * pose.cape_lag * u + wave * 0.5;
            let hy = (g - 3.0).min(rig.hip.1 + 52.0 * k) + wave * 0.6 - pose.kneel * u * 8.0;
            hem.push((hx, hy));
        }
        for i in 0..n {
            let u0 = i as f32 / n as f32;
            let u1 = (i + 1) as f32 / n as f32;
            let col = if i % 2 == 0 { rgba(176, 24, 60, 1.0) } else { rgba(158, 18, 52, 1.0) };
            cx.poly(&[lerp2(top_r, top_l, u0), lerp2(top_r, top_l, u1), hem[i + 1], hem[i]], col);
        }
        for i in 0..n {
            cx.seg(hem[i], hem[i + 1], 5.0 * k, GOLD);
        }
    }
    // optional sprite override
    if let Some(tex) = &sprites.character {
        let sz = tex.size_vec2();
        let hgt = 220.0 * k * (1.0 - pose.kneel * 0.25);
        let w = hgt * sz.x / sz.y.max(1.0);
        let bob = pose.walk * pose.phase.sin().abs() * 6.0;
        let (x0, x1) = if f > 0.0 { (x - w / 2.0, x + w / 2.0) } else { (x + w / 2.0, x - w / 2.0) };
        cx.image(tex, x0, g - hgt - bob, x1, g - bob, Color32::WHITE);
        return rig;
    }
    let skin = SKIN;
    let skin_d = shade(SKIN, 0.86);
    let shoe = if tier >= 2 { GOLD_D } else { rgba(120, 72, 60, 1.0) };
    let sleeve = if tier >= 2 { shade(shirt, 0.95) } else { shirt };
    let draw_leg = |i: usize, dark: f32| {
        let (knee, foot) = rig.legs[i];
        cx.line(rig.hip, knee, 15.0 * k, shade(skin, dark));
        cx.line(knee, foot, 13.0 * k, shade(skin, dark));
        cx.ellipse_rot(foot.0 + 5.0 * k * f, foot.1 + 1.0 * k, 11.0 * k, 6.5 * k, 0.0, shade(shoe, dark));
    };
    let draw_arm = |i: usize, dark: f32| {
        let (el, hd) = rig.arms[i];
        cx.line(rig.shoulder, el, 12.0 * k, shade(sleeve, dark));
        cx.line(el, hd, 10.0 * k, shade(skin, dark));
        cx.circle(hd.0, hd.1, 7.0 * k, shade(skin, dark));
        if tier >= 2 {
            let m = lerp2(el, hd, 0.25);
            cx.circle(m.0, m.1, 5.0 * k, shade(GOLD, dark));
        }
    };
    draw_leg(0, 0.8);
    if pose.hold_cyl.is_none() {
        draw_arm(0, 0.8);
    }
    draw_leg(1, 1.0);
    // skirt / pha nung (flared, follows hip)
    let wid = 1.0 + 0.12 * pose.squash.max(0.0);
    let side = (-rig.up.1, rig.up.0);
    let waist = (rig.hip.0 + rig.up.0 * 10.0 * k, rig.hip.1 + rig.up.1 * 10.0 * k);
    let hem_y = (rig.hip.1 + 26.0 * k).min(g - 4.0);
    let flare = (26.0 + tier.min(4) as f32 * 2.0 + pose.kneel * 10.0) * k * wid;
    let sw = (t * 4.0).sin() * 2.0 * k * pose.walk;
    cx.poly(&[(waist.0 - side.0 * 19.0 * k, waist.1 - side.1 * 19.0 * k), (waist.0 + side.0 * 19.0 * k, waist.1 + side.1 * 19.0 * k), (rig.hip.0 + flare + sw, hem_y), (rig.hip.0 - flare + sw, hem_y)], skirt);
    cx.seg((rig.hip.0 - flare + sw, hem_y - 2.0 * k), (rig.hip.0 + flare + sw, hem_y - 2.0 * k), 5.0 * k, if tier >= 2 { GOLD } else { shade(skirt, 1.25) });
    // torso: rounded bean
    cx.line(rig.hip, rig.shoulder, 40.0 * k * wid, shirt);
    cx.circle(rig.shoulder.0, rig.shoulder.1 + 4.0 * k, 21.0 * k * wid, shirt);
    // white collar / neck ribbon
    let nk = (rig.shoulder.0 - rig.up.0 * 4.0 * k, rig.shoulder.1 - rig.up.1 * 4.0 * k);
    cx.ellipse(nk.0, nk.1, 14.0 * k, 5.0 * k, if tier >= 2 { GOLD } else { rgba(255, 252, 245, 1.0) });
    if tier >= 1 {
        // pha sabai sash
        let a = (rig.shoulder.0 - 14.0 * k * f, rig.shoulder.1 - 2.0 * k);
        let b = (rig.hip.0 + 16.0 * k * f, rig.hip.1 - 4.0 * k);
        cx.seg(a, b, 8.0 * k, if tier >= 2 { rgba(200, 30, 70, 1.0) } else { GOLD });
        cx.seg(a, b, 2.0 * k, rgba(255, 245, 200, 0.8));
    }
    if tier >= 2 {
        cx.seg((waist.0 - side.0 * 20.0 * k, waist.1 - side.1 * 20.0 * k), (waist.0 + side.0 * 20.0 * k, waist.1 + side.1 * 20.0 * k), 6.0 * k, GOLD_D);
        let pend = (nk.0 + rig.fwd.0 * 4.0 * k - rig.up.0 * 10.0 * k, nk.1 - rig.up.1 * 10.0 * k);
        cx.circle(pend.0, pend.1, 5.0 * k, rgba(90, 210, 150, 1.0));
        cx.ring(pend.0, pend.1, 6.0 * k, 2.0 * k, GOLD);
    }
    // ---- head
    let (hx, hy) = rig.head;
    let hair = look.hair;
    let hair_hi = shade(hair, 1.6);
    let back = (hx - 8.0 * k * f, hy - 4.0 * k);
    // style-specific back hair
    match look.style {
        1 => {
            // ponytail
            let sway = (t * 3.0).sin() * 4.0 * k + pose.cape_lag * 0.3;
            cx.ellipse_rot(back.0 - (hr * 0.95 + sway) * f, back.1 + 8.0 * k, 16.0 * k, 26.0 * k, 0.5 * f, hair);
        }
        2 => {
            // twin buns
            cx.circle(hx - hr * 0.55 * f, hy - hr * 0.85, 16.0 * k, hair);
            cx.circle(hx + hr * 0.35 * f, hy - hr * 0.98, 15.0 * k, hair);
        }
        3 => {
            // Thai topknot (jook)
            cx.circle(hx - 2.0 * k * f, hy - hr * 1.02, 15.0 * k, hair);
            cx.ring(hx - 2.0 * k * f, hy - hr * 0.88, 9.0 * k, 3.0 * k, rgba(255, 120, 160, 1.0));
        }
        _ => {
            // bob
            cx.ellipse(back.0, back.1 + 14.0 * k, hr * 1.04, hr * 0.92, hair);
        }
    }
    cx.circle(back.0, back.1, hr * 1.06, hair);
    // face
    cx.circle(hx + 4.0 * k * f, hy + 4.0 * k, hr * 0.92, skin);
    cx.ellipse(hx + 4.0 * k * f, hy + hr * 0.62, hr * 0.55, hr * 0.22, rgba(255, 255, 255, 0.0));
    // ear
    cx.circle(hx - 12.0 * k * f, hy + 10.0 * k, 8.0 * k, skin_d);
    // fringe (bangs) – soft scallops across the forehead
    let fr_y = hy - hr * 0.42;
    cx.ellipse_rot(hx + 2.0 * k * f, fr_y - 10.0 * k, hr * 0.98, hr * 0.48, -0.1 * f, hair);
    for i in 0..4 {
        let u = i as f32 / 3.0;
        let bx = hx + lerp(-10.0, 34.0, u) * k * f;
        cx.ellipse_rot(bx, fr_y + 6.0 * k, 12.0 * k, 15.0 * k, 0.35 * f, hair);
    }
    cx.ellipse_rot(hx - 4.0 * k * f, hy - hr * 0.72, 18.0 * k, 5.0 * k, -0.4 * f, with_a(hair_hi, 0.55));
    // face features on the near side (3/4 view)
    let ey = hy + 8.0 * k;
    let e1 = hx + 12.0 * k * f;
    let e2 = hx + 38.0 * k * f;
    let closed = pose.blink;
    cx.ellipse(e1 - 2.0 * k * f, ey + 15.0 * k, 9.0 * k, 5.0 * k, rgba(255, 130, 150, 0.45));
    cx.ellipse(e2 + 1.0 * k * f, ey + 15.0 * k, 6.0 * k, 4.5 * k, rgba(255, 130, 150, 0.45));
    draw_eye(cx, e1, ey, k, closed, pose.expr, f, 1.0);
    draw_eye(cx, e2, ey, k, closed, pose.expr, f, 0.82);
    let mx = hx + 26.0 * k * f;
    let my = hy + 26.0 * k;
    if pose.expr == 2 {
        cx.poly(&[(mx - 6.0 * k, my - 1.0 * k), (mx + 6.0 * k, my - 1.0 * k), (mx + 3.0 * k, my + 5.0 * k), (mx - 3.0 * k, my + 5.0 * k)], rgba(200, 70, 80, 1.0));
        cx.ellipse(mx, my + 3.0 * k, 2.5 * k, 1.5 * k, rgba(255, 150, 160, 1.0));
    } else {
        let pts = arc_pts(mx, my - 3.0 * k, 5.0 * k, 4.0 * k, PI * 0.15, PI * 0.85, 6);
        cx.polyline(&pts, 2.4 * k, rgba(160, 70, 60, 1.0));
    }
    // flower hair clip for tiers 0-2
    if tier <= 2 {
        let fx = hx - hr * 0.5 * f;
        let fy = hy - hr * 0.62;
        for i in 0..5 {
            let a = i as f32 / 5.0 * TAU + 0.3;
            cx.circle(fx + a.cos() * 5.0 * k, fy + a.sin() * 5.0 * k, 4.2 * k, rgba(255, 170, 200, 1.0));
        }
        cx.circle(fx, fy, 3.0 * k, rgba(255, 230, 120, 1.0));
    }
    // front arm / joined hands / cylinder
    if let Some((tilt, jostle, lift)) = pose.hold_cyl {
        let hd = rig.arms[1].1;
        draw_arm(0, 0.8);
        draw_cylinder(cx, hd.0 + 6.0 * k * f, hd.1 + 26.0 * k, (0.35 + tilt) * f, 0.6 * k, jostle, lift);
        draw_arm(1, 1.0);
    } else if pose.hands > 0.5 && pose.carry < 0.5 && pose.reach.map(|r| r.1 < 0.5).unwrap_or(true) {
        draw_arm(1, 1.0);
        // joined palms (wai): a small rounded leaf shape pointing along `up`
        let hd = rig.arms[1].1;
        let u = rig.up;
        let s2 = (-u.1, u.0);
        let pt = |a: f32, b: f32| (hd.0 + u.0 * a * k + s2.0 * b * k, hd.1 + u.1 * a * k + s2.1 * b * k);
        cx.poly(&[pt(-6.0, -6.0), pt(-6.0, 6.0), pt(10.0, 5.0), pt(18.0, 0.0), pt(10.0, -5.0)], skin);
    } else {
        draw_arm(1, 1.0);
    }
    // crown (Thai chada)
    if tier >= 3 {
        let up = (rig.head.0 - rig.shoulder.0, rig.head.1 - rig.shoulder.1);
        let ul = (up.0 * up.0 + up.1 * up.1).sqrt().max(0.01);
        let u = (up.0 / ul, up.1 / ul);
        let sd = (-u.1, u.0);
        let cb = (hx + u.0 * hr * 0.86, hy + u.1 * hr * 0.86);
        let p = |a: f32, b: f32| (cb.0 + sd.0 * a * k + u.0 * b * k, cb.1 + sd.1 * a * k + u.1 * b * k);
        cx.poly(&[p(-30.0, -4.0), p(30.0, -4.0), p(26.0, 10.0), p(-26.0, 10.0)], GOLD);
        let tiers = if tier >= 4 { 4 } else { 3 };
        for i in 0..tiers {
            let w0 = 22.0 - i as f32 * 5.0;
            let y0 = 8.0 + i as f32 * 11.0;
            cx.poly(&[p(-w0, y0), p(w0, y0), p(w0 - 4.0, y0 + 11.0), p(-w0 + 4.0, y0 + 11.0)], if i % 2 == 0 { GOLD } else { GOLD_D });
        }
        let top = 8.0 + tiers as f32 * 11.0;
        cx.poly(&[p(-6.0, top), p(6.0, top), p(0.0, top + 24.0)], GOLD);
        let gem = p(0.0, 3.0);
        cx.circle(gem.0, gem.1, 5.0 * k, rgba(230, 40, 80, 1.0));
        cx.circle(gem.0 - 1.5 * k, gem.1 - 1.5 * k, 1.6 * k, Color32::WHITE);
        if tier >= 4 {
            for (a, c) in [(-17.0, rgba(60, 150, 255, 1.0)), (17.0, rgba(60, 210, 140, 1.0))] {
                let g1 = p(a, 3.0);
                cx.circle(g1.0, g1.1, 3.8 * k, c);
            }
            let tw = (t * 6.0).sin().abs();
            let tip = p(0.0, top + 24.0);
            sparkle(cx, tip.0, tip.1, 12.0 * k * tw, rgba(255, 255, 255, 0.95));
        }
    }
    rig
}

pub fn sparkle(cx: &Cx, x: f32, y: f32, r: f32, c: Color32) {
    if r <= 0.1 {
        return;
    }
    cx.poly(&[(x, y - r), (x + r * 0.22, y), (x, y + r), (x - r * 0.22, y)], c);
    cx.poly(&[(x - r, y), (x, y - r * 0.22), (x + r, y), (x, y + r * 0.22)], c);
}

/// Cute attendant carrying a golden tiered parasol (entourage of the top tier).
pub fn draw_attendant(cx: &Cx, sprites: &Sprites, x: f32, pose: &Pose, idx: usize) {
    let hairs = [rgba(50, 32, 28, 1.0), rgba(30, 30, 40, 1.0)];
    let shirts = [rgba(255, 190, 205, 1.0), rgba(190, 225, 255, 1.0)];
    let look = Look { tier: 1, shirt: shirts[idx % 2], pants: rgba(150, 40, 80, 1.0), scale: 0.72, hair: hairs[idx % 2], style: if idx % 2 == 0 { 2 } else { 3 } };
    let rig = solve_rig(x, cx.ground(), pose, look.scale);
    let px = rig.shoulder.0 + 20.0 * pose.facing;
    let top = rig.shoulder.1 - 150.0;
    let sway = (cx.time * 2.0 + x * 0.01).sin() * 3.0;
    cx.seg((px, rig.shoulder.1 + 22.0), (px + sway, top), 4.0, GOLD_D);
    for (i, w) in [(0, 62.0), (1, 48.0), (2, 34.0)] {
        let y = top + i as f32 * 24.0;
        let xx = px + sway * (1.0 - i as f32 * 0.2);
        cx.poly(&[(xx - w, y + 16.0), (xx + w, y + 16.0), (xx + w * 0.7, y), (xx - w * 0.7, y)], if i % 2 == 0 { GOLD } else { rgba(236, 168, 40, 1.0) });
        for j in 0..5 {
            let fx = xx - w + j as f32 * w * 0.5;
            cx.circle(fx, y + 18.0 + (cx.time * 4.0 + j as f32).sin() * 1.5, 3.0, GOLD_D);
        }
    }
    cx.poly(&[(px + sway - 7.0, top), (px + sway + 7.0, top), (px + sway, top - 26.0)], GOLD);
    let mut p2 = pose.clone();
    p2.hands = 0.0;
    draw_character(cx, sprites, x, &p2, &look);
}

/// Foreground flowers / lotus pots framing the shrine scene (depth layer).
pub fn draw_foreground_flowers(cx: &Cx) {
    let g = cx.ground();
    let t = cx.time;
    for (i, x) in [30.0f32, 1050.0].iter().enumerate() {
        // pot
        cx.poly(&[(x - 34.0, g + 4.0), (x + 34.0, g + 4.0), (x + 26.0, g + 46.0), (x - 26.0, g + 46.0)], rgba(150, 70, 40, 1.0));
        cx.rect(x - 38.0, g - 2.0, x + 38.0, g + 8.0, 4.0, rgba(190, 100, 55, 1.0));
        for j in 0..3 {
            let a = -0.5 + j as f32 * 0.5 + (t * 0.8 + i as f32).sin() * 0.03;
            let tip = (x + a.sin() * 70.0, g - a.cos() * 70.0 - 10.0);
            cx.seg((*x, g), tip, 4.0, rgba(70, 140, 70, 1.0));
            draw_lotus(cx, tip.0, tip.1, 0.55 + 0.1 * j as f32);
        }
        cx.ellipse_rot(x - 20.0, g - 10.0, 22.0, 8.0, -0.4, rgba(60, 150, 80, 1.0));
        cx.ellipse_rot(x + 22.0, g - 12.0, 22.0, 8.0, 0.4, rgba(80, 170, 90, 1.0));
    }
}

/// small pink lotus bloom
pub fn draw_lotus(cx: &Cx, x: f32, y: f32, sc: f32) {
    let pet = |a: f32, len: f32, c: Color32| {
        let (s, co) = (a.sin(), a.cos());
        cx.ellipse_rot(x + s * len * 0.5 * sc, y - co * len * 0.5 * sc, 7.0 * sc, len * 0.55 * sc, a, c);
    };
    for a in [-1.0f32, 1.0] {
        pet(a * 0.9, 30.0, rgba(255, 170, 200, 1.0));
    }
    for a in [-0.45f32, 0.45] {
        pet(a, 34.0, rgba(255, 195, 215, 1.0));
    }
    pet(0.0, 36.0, rgba(255, 220, 232, 1.0));
    cx.ellipse(x, y, 10.0 * sc, 4.0 * sc, rgba(110, 170, 90, 1.0));
}

/// Name banner above the character; `appear` drives an elastic bounce-in (0..1).
type DeferredDraw = Box<dyn Fn(&Cx)>;
/// a deferred tag: optional collision rect (world coords) + draw fn taking a (dx, dy) nudge
struct TagEntry {
    rect: Option<[f32; 4]>,
    prio: f32,
    draw: Box<dyn Fn(&Cx, f32, f32)>,
}
thread_local! {
    /// While set, name tags are collected instead of drawn, so they can be painted on top of result cards.
    static TAG_LAYER: std::cell::RefCell<Option<Vec<TagEntry>>> = const { std::cell::RefCell::new(None) };
}
/// Start collecting name tags (call before drawing the world).
pub fn defer_tags_begin() {
    TAG_LAYER.with(|l| *l.borrow_mut() = Some(vec![]));
}
fn overlaps(a: &[f32; 4], b: &[f32; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}
/// Find the smallest nudge that moves `r` clear of every rect in `placed` (prefers up, then sideways, small down).
pub fn resolve_tag(r: [f32; 4], placed: &[[f32; 4]]) -> (f32, f32) {
    let gap = 6.0;
    let mut best = (0.0f32, 0.0f32);
    for _ in 0..6 {
        let cur = [r[0] + best.0, r[1] + best.1, r[2] + best.0, r[3] + best.1];
        let Some(o) = placed.iter().find(|p| overlaps(&cur, p)) else {
            return best;
        };
        let h = r[3] - r[1];
        let mut cands = vec![
            (0.0, o[1] - gap - cur[3]),                       // above it
            (0.0, o[3] + gap - cur[1]),                       // below it
            (o[0] - gap - cur[2], 0.0),                       // left of it
            (o[2] + gap - cur[0], 0.0),                       // right of it
        ];
        // stay in frame; don't slide far down over the face
        cands.retain(|(dx, dy)| {
            let (x0, x1) = (cur[0] + dx, cur[2] + dx);
            x0 >= 4.0 && x1 <= VW - 4.0 && cur[1] + dy >= 0.0 && *dy <= h * 0.7
        });
        let Some(c) = cands.into_iter().min_by(|a, b| {
            let ka = a.0.abs() * 1.2 + a.1.abs() * if a.1 > 0.0 { 1.6 } else { 1.0 };
            let kb = b.0.abs() * 1.2 + b.1.abs() * if b.1 > 0.0 { 1.6 } else { 1.0 };
            ka.partial_cmp(&kb).unwrap_or(std::cmp::Ordering::Equal)
        }) else {
            return best;
        };
        best = (best.0 + c.0, best.1 + c.1);
    }
    best
}
/// Draw all collected name tags now (after cards), with the same camera as the world.
/// Prominent tags (the active giver's banner) are placed first; smaller tags that would overlap get nudged.
pub fn defer_tags_flush(cx: &Cx) {
    let v = TAG_LAYER.with(|l| l.borrow_mut().take()).unwrap_or_default();
    let mut order: Vec<usize> = (0..v.len()).filter(|&i| v[i].rect.is_some()).collect();
    order.sort_by(|&a, &b| v[b].prio.partial_cmp(&v[a].prio).unwrap_or(std::cmp::Ordering::Equal));
    let mut offs = vec![(0.0f32, 0.0f32); v.len()];
    let mut placed: Vec<[f32; 4]> = vec![];
    for i in order {
        let r = v[i].rect.unwrap();
        let d = resolve_tag(r, &placed);
        offs[i] = d;
        placed.push([r[0] + d.0, r[1] + d.1, r[2] + d.0, r[3] + d.1]);
    }
    for (e, d) in v.iter().zip(offs) {
        (e.draw)(cx, d.0, d.1);
    }
}
fn push_tag(cx: &Cx, rect: Option<[f32; 4]>, prio: f32, f: Box<dyn Fn(&Cx, f32, f32)>) {
    let e = TagEntry { rect, prio, draw: f };
    let e = TAG_LAYER.with(|l| match l.borrow_mut().as_mut() {
        Some(v) => {
            v.push(e);
            None
        }
        None => Some(e),
    });
    if let Some(e) = e {
        (e.draw)(cx, 0.0, 0.0);
    }
}
/// Queue `f` on the tag layer if it is active, else draw immediately.
pub fn on_tag_layer(cx: &Cx, f: DeferredDraw) {
    push_tag(cx, None, 0.0, Box::new(move |cx, _, _| f(cx)));
}

/// geometry shared by layout + drawing: (clamped centre x, half width incl. ribbons, height)
fn banner_geom(cx: &Cx, x: f32, name: &str, tier: usize, tier_name: &str, sc: f32) -> (f32, f32, f32) {
    let size = [26.0, 28.0, 32.0, 36.0, 42.0][tier.min(4)] * sc;
    let m = cx.measure(name, size, true);
    let w = m.x + (36.0 + tier as f32 * 10.0) * sc;
    let tails = if tier >= 3 { (40.0 + tier as f32 * 8.0) * sc } else { 0.0 };
    let half = (w / 2.0 + tails).max(cx.measure(tier_name, 20.0, true).x / 2.0 + 16.0);
    let pad = 12.0;
    let x = if VW - 2.0 * pad > 2.0 * half { x.clamp(pad + half, VW - pad - half) } else { VW / 2.0 };
    (x, half, size * 1.55)
}

pub fn draw_name_banner(cx: &Cx, x: f32, y: f32, name: &str, tier: usize, tier_name: &str, appear: f32, alpha: f32) {
    let (name, tier_name) = (name.to_string(), tier_name.to_string());
    let sc = ease_out_elastic(appear).max(0.0);
    let rect = if sc > 0.02 && alpha > 0.05 {
        let (cxm, half, h) = banner_geom(cx, x, &name, tier, &tier_name, sc);
        let top = y - h - if tier_name.is_empty() { 0.0 } else { 26.0 };
        Some([cxm - half, top, cxm + half, y + 4.0])
    } else {
        None
    };
    let prio = tier as f32 + if tier_name.is_empty() { 0.0 } else { 10.0 } + (y / 10_000.0);
    push_tag(
        cx,
        rect,
        prio,
        Box::new(move |cx, dx, dy| {
            if dx.abs() + dy.abs() > 0.5 && alpha > 0.02 {
                // leader line from the nudged tag back to the character
                let (cxm, _, _) = banner_geom(cx, x, &name, tier, &tier_name, sc.max(0.03));
                cx.seg((cxm + dx, y + dy), (x, y), 2.0, rgba(255, 255, 255, 0.55 * alpha));
            }
            draw_name_banner_now(cx, x + dx, y + dy, &name, tier, &tier_name, appear, alpha)
        }),
    );
}

fn draw_name_banner_now(cx: &Cx, x: f32, y: f32, name: &str, tier: usize, tier_name: &str, appear: f32, alpha: f32) {
    let t = cx.time;
    let sc = ease_out_elastic(appear).max(0.0);
    if sc <= 0.02 || alpha <= 0.0 {
        return;
    }
    let size = [26.0, 28.0, 32.0, 36.0, 42.0][tier.min(4)] * sc;
    let m = cx.measure(name, size, true);
    let w = m.x + (36.0 + tier as f32 * 10.0) * sc;
    // keep the whole tag (incl. tier-3+ ribbon tails and the tier-name ribbon) inside the frame
    let (x, _, h) = banner_geom(cx, x, name, tier, tier_name, sc);
    let (x0, y0) = (x - w / 2.0, y - h);
    let a = alpha;
    match tier {
        0 => {
            cx.rect(x0, y0, x0 + w, y, h / 2.0, rgba(0, 0, 0, 0.6 * a));
            cx.text(x, y - h / 2.0, Align2::CENTER_CENTER, name, size, with_a(Color32::WHITE, a), true);
        }
        1 => {
            cx.rect(x0, y0, x0 + w, y, h / 2.0, rgba(20, 20, 40, 0.8 * a));
            cx.rect_line(x0, y0, x0 + w, y, h / 2.0, 3.0 * sc, with_a(GOLD, a));
            cx.text(x, y - h / 2.0, Align2::CENTER_CENTER, name, size, with_a(GOLD, a), true);
        }
        _ => {
            if tier >= 3 {
                let tl = (40.0 + tier as f32 * 8.0) * sc;
                let wv = (t * 4.0).sin() * 4.0 * sc;
                let rc = rgba(170, 20, 40, a);
                cx.poly(&[(x0 + 10.0, y0 + 8.0), (x0 - tl, y0 + h * 0.3 + wv), (x0 - tl * 0.6, y0 + h * 0.6 + wv), (x0 + 10.0, y - 2.0)], rc);
                cx.poly(&[(x0 - tl * 0.6, y0 + h * 0.6 + wv), (x0 - tl, y0 + h + 6.0 + wv), (x0 + 10.0, y - 2.0)], rc);
                cx.poly(&[(x0 + w - 10.0, y0 + 8.0), (x0 + w - 10.0, y - 2.0), (x0 + w + tl * 0.6, y0 + h * 0.6 - wv), (x0 + w + tl, y0 + h * 0.3 - wv)], rc);
                cx.poly(&[(x0 + w - 10.0, y - 2.0), (x0 + w + tl, y0 + h + 6.0 - wv), (x0 + w + tl * 0.6, y0 + h * 0.6 - wv)], rc);
                cx.glow(x, y - h / 2.0, w * 0.75, GOLD, 0.35 * a);
            }
            cx.vgradient(x0, y0, x0 + w, y, rgba(255, 236, 150, a), rgba(214, 150, 30, a));
            // shine sweep
            let sw = ((t * 0.5).fract() * 1.6 - 0.3) * w;
            if sw > 0.0 && sw < w {
                cx.poly(&[(x0 + sw, y0), (x0 + sw + 14.0, y0), (x0 + sw - 6.0, y), (x0 + sw - 20.0, y)], rgba(255, 255, 255, 0.35 * a));
            }
            cx.rect_line(x0, y0, x0 + w, y, 6.0, (3.0 + tier as f32) * sc, rgba(130, 70, 0, a));
            cx.text_o(x, y - h / 2.0, Align2::CENTER_CENTER, name, size, rgba(120, 10, 20, a), rgba(255, 245, 210, 0.9 * a), true);
            if !tier_name.is_empty() {
                let ts = size * 0.5;
                let tm = cx.measure(tier_name, ts, true);
                cx.rect(x - tm.x / 2.0 - 12.0, y0 - ts * 1.45, x + tm.x / 2.0 + 12.0, y0 - 2.0, 8.0, rgba(120, 10, 20, 0.95 * a));
                cx.text(x, y0 - ts * 0.75, Align2::CENTER_CENTER, tier_name, ts, with_a(GOLD, a), true);
            }
            if tier >= 4 {
                for i in 0..8 {
                    let ph = (t * 1.3 + i as f32 * 0.37).fract();
                    let sx = x0 + (i as f32 * 0.137 * 7.0).fract() * w;
                    sparkle(cx, sx, y0 + h * (1.0 - ph) - 10.0, 12.0 * (1.0 - (ph * 2.0 - 1.0).abs()), rgba(255, 255, 255, 0.95 * a));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
#[derive(Clone)]
pub struct Particle {
    pub pos: Pos2,
    pub vel: Vec2,
    pub life: f32,
    pub max: f32,
    pub color: Color32,
    pub size: f32,
    /// 0 spark, 1 glitter, 2 rocket, 3 petal, 4 dust mote, 5 ember/trail, 6 willow, 7 flash
    pub kind: u8,
    pub tier: usize,
    pub drag: f32,
    pub grav: f32,
    pub rot: f32,
    pub vrot: f32,
    pub crackle: bool,
    pub shape: u8,
}

impl Particle {
    pub fn new(kind: u8, pos: Pos2, vel: Vec2, life: f32, color: Color32, size: f32) -> Self {
        Self { pos, vel, life, max: life, color, size, kind, tier: 0, drag: 0.0, grav: 0.0, rot: 0.0, vrot: 0.0, crackle: false, shape: 0 }
    }
}

pub fn draw_particles(cx: &Cx, ps: &[Particle], ambient: bool) {
    for p in ps {
        let is_amb = p.kind == 3 || p.kind == 4 || p.kind == 8;
        if is_amb != ambient {
            continue;
        }
        let a = (p.life / p.max).clamp(0.0, 1.0);
        match p.kind {
            2 => {
                cx.glow_add(p.pos.x, p.pos.y, 26.0, rgba(255, 220, 140, 1.0), 0.9);
                cx.line((p.pos.x, p.pos.y), (p.pos.x - p.vel.x * 0.025, p.pos.y - p.vel.y * 0.025), 4.0, rgba(255, 240, 200, 1.0));
            }
            1 => {
                let tw = 0.5 + 0.5 * (cx.time * 18.0 + p.pos.x * 0.37 + p.rot).sin();
                sparkle(cx, p.pos.x, p.pos.y, p.size * 2.2 * tw * a.sqrt(), with_a(p.color, a));
            }
            3 => {
                let (s, c) = (p.rot).sin_cos();
                cx.ellipse_rot(p.pos.x, p.pos.y, p.size, p.size * 0.55 * (0.4 + 0.6 * c.abs()), s, with_a(p.color, a.min(1.0) * 0.9));
            }
            4 => {
                let tw = 0.6 + 0.4 * (cx.time * 2.0 + p.rot).sin();
                let fade = (a * 3.0).min(1.0) * ((1.0 - a) * 6.0).min(1.0);
                cx.glow_add(p.pos.x, p.pos.y, p.size * 4.0, p.color, 0.5 * tw * fade);
                cx.circle(p.pos.x, p.pos.y, p.size * 0.5, with_a(p.color, 0.6 * tw * fade));
            }
            5 => {
                cx.circle(p.pos.x, p.pos.y, p.size * a, with_a(p.color, a * 0.8));
            }
            7 => {
                cx.glow_add(p.pos.x, p.pos.y, p.size * (1.0 + (1.0 - a) * 0.6), p.color, a);
            }
            8 => {
                // floating sky lantern (โคมลอย)
                let fade = (a * 4.0).min(1.0) * (((p.max - p.life) * 2.0).min(1.0));
                let sz = p.size;
                let sw = (cx.time * 1.3 + p.rot).sin() * 0.08;
                let fl = 0.8 + 0.2 * noise1(cx.time * 5.0 + p.rot);
                cx.glow_add(p.pos.x, p.pos.y, 40.0 * sz, rgba(255, 150, 50, 1.0), 0.55 * fade * fl);
                let (s2, c2) = sw.sin_cos();
                let r = |x: f32, y: f32| (p.pos.x + (x * c2 - y * s2) * sz, p.pos.y + (x * s2 + y * c2) * sz);
                cx.poly(&[r(-11.0, -16.0), r(11.0, -16.0), r(14.0, 14.0), r(-14.0, 14.0)], rgba(255, 170, 70, 0.95 * fade));
                cx.poly(&[r(-6.0, -12.0), r(6.0, -12.0), r(8.0, 12.0), r(-8.0, 12.0)], rgba(255, 230, 150, 0.9 * fade * fl));
                cx.seg(r(-14.0, 14.0), r(14.0, 14.0), 2.0 * sz, rgba(150, 70, 20, fade));
            }
            _ => {
                // spark / willow: streak + additive head, twinkle near end of life
                let tail = p.vel * if p.kind == 6 { 0.09 } else { 0.04 };
                let tw = if a < 0.35 { (0.5 + 0.5 * (cx.time * 30.0 + p.rot * 10.0).sin()).powi(2) } else { 1.0 };
                let col = with_a(p.color, a * tw);
                cx.seg((p.pos.x, p.pos.y), (p.pos.x - tail.x, p.pos.y - tail.y), p.size * (0.5 + 0.5 * a), col);
                cx.circle(p.pos.x, p.pos.y, p.size * 0.55, with_a(Color32::WHITE, a * tw * 0.9));
                if p.size > 3.5 {
                    cx.glow_add(p.pos.x, p.pos.y, p.size * 3.5, p.color, 0.35 * a * tw);
                }
            }
        }
    }
}

pub fn shimmer(cx: &Cx, strength: f32) {
    if !cx.soft || strength <= 0.0 {
        return;
    }
    let t = cx.time;
    let h = cx.vh;
    for i in 0..3 {
        let ph = ((t * 0.35) + i as f32 / 3.0).fract();
        let x = -400.0 + ph * (VW + 800.0);
        cx.poly(&[(x, 0.0), (x + 90.0, 0.0), (x - 160.0, h), (x - 250.0, h)], rgba(255, 230, 150, 0.12 * strength));
    }
}

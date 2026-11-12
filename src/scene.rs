//! Programmatic art: shrine, character, fireworks, cards. Everything is drawn in a virtual
//! 1080-wide coordinate space and scaled to the window. PNG sprites in assets/sprites override parts.
use egui::{epaint::Mesh, Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke, TextureHandle, Vec2};
use std::f32::consts::{PI, TAU};

pub const VW: f32 = 1080.0;

pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, (a.clamp(0.0, 1.0) * 255.0) as u8)
}
pub fn with_a(c: Color32, a: f32) -> Color32 {
    let [r, g, b, _] = c.to_srgba_unmultiplied();
    rgba(r, g, b, a)
}
pub const GOLD: Color32 = Color32::from_rgb(255, 200, 60);
pub const GOLD_D: Color32 = Color32::from_rgb(196, 138, 20);
pub const RED: Color32 = Color32::from_rgb(178, 24, 36);
pub const SKIN: Color32 = Color32::from_rgb(246, 204, 160);
pub const HAIR: Color32 = Color32::from_rgb(40, 28, 24);

pub fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}
pub fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Virtual-canvas painter.
pub struct Cx<'a> {
    pub p: &'a Painter,
    pub s: f32,
    pub o: Pos2,
    pub vh: f32,
    pub time: f32,
    /// false in green-screen mode: soft alpha glows would leave green fringes after chroma key
    pub soft: bool,
}

impl<'a> Cx<'a> {
    pub fn new(p: &'a Painter, rect: Rect, time: f32) -> Self {
        let s = rect.width() / VW;
        Self { p, s, o: rect.min, vh: rect.height() / s, time, soft: true }
    }
    pub fn pt(&self, x: f32, y: f32) -> Pos2 {
        Pos2::new(self.o.x + x * self.s, self.o.y + y * self.s)
    }
    pub fn ground(&self) -> f32 {
        self.vh - 110.0_f32.min(self.vh * 0.12)
    }
    pub fn circle(&self, x: f32, y: f32, r: f32, c: Color32) {
        self.p.circle_filled(self.pt(x, y), r * self.s, c);
    }
    pub fn ring(&self, x: f32, y: f32, r: f32, w: f32, c: Color32) {
        self.p.circle_stroke(self.pt(x, y), r * self.s, Stroke::new(w * self.s, c));
    }
    pub fn rect(&self, x0: f32, y0: f32, x1: f32, y1: f32, r: f32, c: Color32) {
        let rr = (r * self.s).clamp(0.0, 255.0) as u8;
        self.p.rect_filled(Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1)), rr, c);
    }
    pub fn rect_line(&self, x0: f32, y0: f32, x1: f32, y1: f32, r: f32, w: f32, c: Color32) {
        let rr = (r * self.s).clamp(0.0, 255.0) as u8;
        self.p.rect_stroke(Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1)), rr, Stroke::new(w * self.s, c), egui::StrokeKind::Middle);
    }
    pub fn poly(&self, pts: &[(f32, f32)], c: Color32) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        self.p.add(Shape::convex_polygon(v, c, Stroke::NONE));
    }
    pub fn line(&self, a: (f32, f32), b: (f32, f32), w: f32, c: Color32) {
        self.p.line_segment([self.pt(a.0, a.1), self.pt(b.0, b.1)], Stroke::new(w * self.s, c));
        // round caps
        self.p.circle_filled(self.pt(a.0, a.1), w * self.s * 0.5, c);
        self.p.circle_filled(self.pt(b.0, b.1), w * self.s * 0.5, c);
    }
    pub fn ellipse(&self, cx: f32, cy: f32, rx: f32, ry: f32, c: Color32) {
        let n = 40;
        let pts: Vec<(f32, f32)> = (0..n).map(|i| {
            let a = i as f32 / n as f32 * TAU;
            (cx + rx * a.cos(), cy + ry * a.sin())
        }).collect();
        self.poly(&pts, c);
    }
    /// soft radial glow
    pub fn glow(&self, x: f32, y: f32, r: f32, c: Color32, a: f32) {
        if !self.soft {
            return;
        }
        let mut mesh = Mesh::default();
        let n = 48;
        let center = self.pt(x, y);
        mesh.colored_vertex(center, with_a(c, a));
        for i in 0..=n {
            let ang = i as f32 / n as f32 * TAU;
            mesh.colored_vertex(center + Vec2::new(ang.cos(), ang.sin()) * r * self.s, with_a(c, 0.0));
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
        FontId::new((size * self.s).max(6.0), if bold { FontFamily::Name("bold".into()) } else { FontFamily::Proportional })
    }
    pub fn text(&self, x: f32, y: f32, anchor: Align2, t: &str, size: f32, c: Color32, bold: bool) -> Rect {
        self.p.text(self.pt(x, y), anchor, t, self.font(size, bold), c)
    }
    /// text with dark outline (readable over any background / chroma)
    pub fn text_o(&self, x: f32, y: f32, anchor: Align2, t: &str, size: f32, c: Color32, outline: Color32, bold: bool) -> Rect {
        let d = (size * 0.07).max(1.5);
        for (dx, dy) in [(-d, 0.0), (d, 0.0), (0.0, -d), (0.0, d), (-d * 0.7, -d * 0.7), (d * 0.7, d * 0.7), (-d * 0.7, d * 0.7), (d * 0.7, -d * 0.7)] {
            self.text(x + dx, y + dy, anchor, t, size, outline, bold);
        }
        self.text(x, y, anchor, t, size, c, bold)
    }
    pub fn measure(&self, t: &str, size: f32, bold: bool) -> Vec2 {
        let g = self.p.layout_no_wrap(t.to_string(), self.font(size, bold), Color32::WHITE);
        g.size() / self.s
    }
    pub fn text_wrapped(&self, x: f32, y: f32, t: &str, size: f32, width: f32, c: Color32, bold: bool) -> f32 {
        let mut job = egui::text::LayoutJob::simple(t.to_string(), self.font(size, bold), c, width * self.s);
        job.halign = egui::Align::Center;
        let g = self.p.layout_job(job);
        let h = g.size().y / self.s;
        self.p.galley(self.pt(x, y), g, c);
        h
    }
    pub fn image(&self, tex: &TextureHandle, x0: f32, y0: f32, x1: f32, y1: f32, tint: Color32) {
        self.p.image(tex.id(), Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1)), Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), tint);
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
        cx.image(t, 0.0, 0.0, VW, h, Color32::WHITE);
        return;
    }
    cx.vgradient(0.0, 0.0, VW, h * 0.75, rgba(40, 20, 70, 1.0), rgba(250, 140, 70, 1.0));
    // stars
    for i in 0..40 {
        let x = (i as f32 * 263.7) % VW;
        let y = (i as f32 * 97.3) % (h * 0.4);
        let tw = 0.5 + 0.5 * (cx.time * 2.0 + i as f32).sin();
        cx.circle(x, y, 2.0, rgba(255, 255, 230, 0.3 + 0.5 * tw));
    }
    cx.glow(300.0, h * 0.62, 260.0, rgba(255, 220, 140, 1.0), 0.45);
    // distant chedi silhouettes
    let g = cx.ground();
    for (x, sc) in [(120.0, 1.0), (330.0, 0.7), (520.0, 0.85)] {
        let base = g - 40.0;
        cx.poly(&[(x - 60.0 * sc, base), (x + 60.0 * sc, base), (x + 25.0 * sc, base - 90.0 * sc), (x - 25.0 * sc, base - 90.0 * sc)], rgba(90, 40, 60, 0.75));
        cx.poly(&[(x - 25.0 * sc, base - 90.0 * sc), (x + 25.0 * sc, base - 90.0 * sc), (x, base - 260.0 * sc)], rgba(90, 40, 60, 0.75));
    }
    // floor
    cx.vgradient(0.0, g - 40.0, VW, h, rgba(120, 60, 40, 1.0), rgba(60, 25, 20, 1.0));
    for i in 0..12 {
        let x = i as f32 * 100.0;
        cx.line((x, g - 40.0), (x - 80.0, h), 2.0, rgba(0, 0, 0, 0.15));
    }
}

/// Floor shadow / platform used in transparent & green modes.
pub fn draw_platform(cx: &Cx) {
    let g = cx.ground();
    cx.ellipse(540.0, g + 10.0, 560.0, 34.0, rgba(0, 0, 0, 0.28));
    cx.rect(20.0, g - 6.0, VW - 20.0, g + 18.0, 10.0, rgba(122, 62, 38, 0.95));
    cx.rect(20.0, g - 6.0, VW - 20.0, g + 2.0, 6.0, rgba(196, 120, 70, 1.0));
}

pub fn draw_carpet(cx: &Cx, alpha: f32, royal: bool) {
    let g = cx.ground();
    let c = if royal { rgba(170, 20, 40, alpha) } else { rgba(220, 170, 50, alpha) };
    cx.poly(&[(0.0, g - 4.0), (PRAY_X + 60.0, g - 4.0), (PRAY_X + 80.0, g + 14.0), (0.0, g + 14.0)], c);
    cx.rect(0.0, g - 6.0, PRAY_X + 70.0, g - 2.0, 0.0, with_a(GOLD, alpha));
}

/// The Buddha shrine on the right. `glow` 0..1 intensifies the halo (during prayer).
pub fn draw_shrine(cx: &Cx, sprites: &Sprites, glow: f32, show_cylinder: bool) {
    let g = cx.ground();
    let x = SHRINE_X;
    let t = cx.time;
    // aura
    cx.glow(x, g - 250.0, 230.0 + glow * 80.0, GOLD, 0.25 + glow * 0.5);
    // altar steps
    cx.rect(x - 170.0, g - 70.0, x + 170.0, g, 8.0, rgba(140, 20, 30, 1.0));
    cx.rect(x - 170.0, g - 70.0, x + 170.0, g - 60.0, 4.0, GOLD_D);
    cx.rect(x - 130.0, g - 120.0, x + 130.0, g - 70.0, 8.0, rgba(170, 30, 40, 1.0));
    cx.rect(x - 130.0, g - 120.0, x + 130.0, g - 110.0, 4.0, GOLD);
    for i in 0..5 {
        cx.circle(x - 100.0 + i as f32 * 50.0, g - 35.0, 9.0, GOLD_D);
    }
    let base = g - 120.0;
    if let Some(tex) = &sprites.buddha {
        let sz = tex.size_vec2();
        let hgt = 330.0;
        let w = hgt * sz.x / sz.y.max(1.0);
        cx.image(tex, x - w / 2.0, base - hgt, x + w / 2.0, base, Color32::WHITE);
    } else {
        // halo
        cx.circle(x, base - 225.0, 72.0 + glow * 6.0, with_a(GOLD, 0.35 + glow * 0.4));
        cx.ring(x, base - 225.0, 76.0 + glow * 6.0, 5.0, with_a(GOLD, 0.8));
        // lotus base
        for i in 0..7 {
            let px = x - 105.0 + i as f32 * 35.0;
            cx.ellipse(px, base - 18.0, 24.0, 22.0, if i % 2 == 0 { rgba(240, 150, 170, 1.0) } else { rgba(250, 180, 195, 1.0) });
        }
        cx.rect(x - 115.0, base - 14.0, x + 115.0, base, 6.0, rgba(210, 120, 140, 1.0));
        // crossed legs
        cx.ellipse(x, base - 52.0, 112.0, 34.0, GOLD_D);
        cx.ellipse(x, base - 58.0, 104.0, 26.0, GOLD);
        // body + robe
        cx.poly(&[(x - 72.0, base - 70.0), (x + 72.0, base - 70.0), (x + 50.0, base - 175.0), (x - 50.0, base - 175.0)], GOLD);
        cx.poly(&[(x - 50.0, base - 175.0), (x + 10.0, base - 175.0), (x + 72.0, base - 70.0), (x + 30.0, base - 70.0)], GOLD_D);
        // hands in lap
        cx.ellipse(x, base - 80.0, 38.0, 14.0, rgba(255, 215, 110, 1.0));
        // neck & head
        cx.rect(x - 16.0, base - 195.0, x + 16.0, base - 170.0, 6.0, GOLD);
        cx.ellipse(x - 44.0, base - 228.0, 9.0, 24.0, GOLD_D);
        cx.ellipse(x + 44.0, base - 228.0, 9.0, 24.0, GOLD_D);
        cx.ellipse(x, base - 230.0, 42.0, 48.0, rgba(255, 210, 80, 1.0));
        // hair curls + ushnisha
        cx.ellipse(x, base - 262.0, 40.0, 20.0, GOLD_D);
        cx.ellipse(x, base - 285.0, 20.0, 18.0, GOLD_D);
        cx.poly(&[(x - 8.0, base - 298.0), (x + 8.0, base - 298.0), (x, base - 335.0)], GOLD);
        // serene face
        cx.line((x - 22.0, base - 232.0), (x - 8.0, base - 230.0), 3.0, rgba(140, 90, 10, 1.0));
        cx.line((x + 8.0, base - 230.0), (x + 22.0, base - 232.0), 3.0, rgba(140, 90, 10, 1.0));
        cx.line((x - 9.0, base - 207.0), (x + 9.0, base - 207.0), 3.0, rgba(170, 100, 20, 1.0));
    }
    // candles
    for side in [-1.0, 1.0] {
        let cxp = x + side * 150.0;
        cx.rect(cxp - 9.0, g - 160.0, cxp + 9.0, g - 70.0, 3.0, rgba(255, 245, 220, 1.0));
        cx.rect(cxp - 14.0, g - 74.0, cxp + 14.0, g - 68.0, 2.0, GOLD_D);
        let fl = (t * 13.0 + side * 3.0).sin() * 2.0;
        cx.glow(cxp, g - 175.0, 40.0, rgba(255, 190, 80, 1.0), 0.6);
        cx.poly(&[(cxp - 7.0, g - 164.0), (cxp + 7.0, g - 164.0), (cxp + fl, g - 192.0)], rgba(255, 170, 40, 1.0));
        cx.circle(cxp, g - 168.0, 4.0, rgba(255, 250, 200, 1.0));
    }
    // incense bowl + smoke
    let ix = x - 70.0;
    cx.ellipse(ix, g - 132.0, 26.0, 12.0, GOLD_D);
    for k in 0..3 {
        let sx = ix - 8.0 + k as f32 * 8.0;
        cx.line((sx, g - 136.0), (sx, g - 170.0), 2.0, rgba(150, 60, 30, 1.0));
        cx.circle(sx, g - 170.0, 2.5, rgba(255, 90, 40, 1.0));
        for j in 0..6 {
            let ph = (t * 0.6 + j as f32 / 6.0 + k as f32 * 0.33).fract();
            let sy = g - 172.0 - ph * 120.0;
            let sxx = sx + (ph * 9.0 + t + k as f32).sin() * 12.0 * ph;
            cx.circle(sxx, sy, 3.0 + ph * 9.0, rgba(230, 230, 240, 0.25 * (1.0 - ph)));
        }
    }
    if show_cylinder {
        draw_cylinder(cx, x + 100.0, g - 70.0, 0.0, 0.55);
    }
}

/// เซียมซี cylinder, (x,y) = bottom center, angle radians, scale.
pub fn draw_cylinder(cx: &Cx, x: f32, y: f32, ang: f32, sc: f32) {
    let rot = |px: f32, py: f32| -> (f32, f32) {
        let (s, c) = ang.sin_cos();
        (x + (px * c - py * s) * sc, y + (px * s + py * c) * sc)
    };
    // sticks
    for i in 0..9 {
        let off = -24.0 + i as f32 * 6.0;
        let top = -150.0 - ((i * 37) % 5) as f32 * 7.0;
        cx.line(rot(off, -90.0), rot(off * 1.15, top), 4.0 * sc, rgba(215, 170, 90, 1.0));
        cx.circle(rot(off * 1.15, top).0, rot(off * 1.15, top).1, 2.5 * sc, RED);
    }
    let body = [rot(-34.0, 0.0), rot(34.0, 0.0), rot(36.0, -100.0), rot(-36.0, -100.0)];
    cx.poly(&body, RED);
    cx.poly(&[rot(-34.0, -10.0), rot(34.0, -10.0), rot(34.0, -18.0), rot(-34.0, -18.0)], GOLD);
    cx.poly(&[rot(-35.0, -82.0), rot(35.0, -82.0), rot(36.0, -92.0), rot(-36.0, -92.0)], GOLD);
    let c = rot(0.0, -50.0);
    cx.circle(c.0, c.1, 15.0 * sc, GOLD);
    cx.text(c.0, c.1, Align2::CENTER_CENTER, "ซี", 16.0 * sc, RED, true);
}

// ---------------------------------------------------------------------------
pub struct Pose {
    pub walk: f32,  // walk phase (radians), 0 = standing
    pub moving: bool,
    pub wai: f32,   // 0..1 hands joined, raise to forehead
    pub bow: f32,   // 0..1 bow forward
    pub kneel: f32, // 0..1
    pub hold_cyl: Option<f32>, // shake angle
    pub facing: f32, // 1 = right, -1 = left
}

pub struct Look {
    pub tier: usize,
    pub shirt: Color32,
    pub pants: Color32,
    pub scale: f32,
}

/// Draw a character with feet at (x, g).
pub fn draw_character(cx: &Cx, sprites: &Sprites, x: f32, pose: &Pose, look: &Look) {
    let g = cx.ground();
    let k = look.scale;
    let f = pose.facing;
    let tier = look.tier;
    let t = cx.time;
    let (shirt, pants) = match tier {
        0 => (look.shirt, look.pants),
        1 => (look.shirt, rgba(40, 40, 70, 1.0)),
        2 => (rgba(232, 180, 50, 1.0), rgba(120, 20, 40, 1.0)),
        3 => (rgba(250, 205, 70, 1.0), rgba(110, 10, 30, 1.0)),
        _ => (rgba(255, 230, 120, 1.0), rgba(90, 0, 25, 1.0)),
    };
    let leg = 72.0 * k;
    let torso = 92.0 * k;
    let hip_y = g - leg * (1.0 - pose.kneel * 0.6);
    let hip = (x, hip_y);
    let a = pose.bow * 0.55 * f;
    let shoulder = (hip.0 + a.sin() * torso, hip.1 - a.cos() * torso);
    let head = (shoulder.0 + a.sin() * 40.0 * k, shoulder.1 - a.cos() * 40.0 * k);

    // aura (tier >= 4th)
    if tier >= 3 {
        let pulse = 0.5 + 0.5 * (t * 3.0).sin();
        let r = if tier >= 4 { 210.0 } else { 150.0 };
        cx.glow(x, hip_y - 60.0 * k, r * k, GOLD, 0.35 + 0.2 * pulse);
        if tier >= 4 {
            for i in 0..12 {
                let ang = i as f32 / 12.0 * TAU + t * 0.6;
                let (s, c) = ang.sin_cos();
                let cxp = x;
                let cyp = hip_y - 60.0 * k;
                cx.poly(&[(cxp, cyp), (cxp + c * 230.0 * k - s * 14.0, cyp + s * 230.0 * k + c * 14.0), (cxp + c * 230.0 * k + s * 14.0, cyp + s * 230.0 * k - c * 14.0)], rgba(255, 230, 140, 0.18));
            }
        }
    }
    // royal cape (tier 5)
    if tier >= 4 {
        let sway = (t * 2.0).sin() * 8.0;
        cx.poly(&[(shoulder.0 - 26.0 * k * f, shoulder.1), (shoulder.0 + 10.0 * k * f, shoulder.1), (x - 40.0 * k * f + sway, g - 6.0), (x - 95.0 * k * f + sway, g - 2.0)], rgba(160, 10, 35, 1.0));
        cx.line((x - 40.0 * k * f + sway, g - 6.0), (x - 95.0 * k * f + sway, g - 2.0), 6.0 * k, GOLD);
    }

    if let Some(tex) = &sprites.character {
        let sz = tex.size_vec2();
        let hgt = 260.0 * k * (1.0 - pose.kneel * 0.3);
        let w = hgt * sz.x / sz.y.max(1.0);
        let bob = if pose.moving { pose.walk.sin().abs() * 6.0 } else { 0.0 };
        let (x0, x1) = if f > 0.0 { (x - w / 2.0, x + w / 2.0) } else { (x + w / 2.0, x - w / 2.0) };
        cx.image(tex, x0, g - hgt - bob, x1, g - bob, Color32::WHITE);
    } else {
        // legs
        if pose.kneel > 0.5 {
            cx.line(hip, (x + 30.0 * k * f, g - 12.0 * k), 22.0 * k, pants);
            cx.line((x + 30.0 * k * f, g - 12.0 * k), (x - 25.0 * k * f, g - 8.0 * k), 20.0 * k, pants);
            cx.ellipse(x - 34.0 * k * f, g - 6.0 * k, 12.0 * k, 7.0 * k, rgba(60, 40, 30, 1.0));
        } else {
            let sw = if pose.moving { pose.walk.sin() * 24.0 * k } else { 0.0 };
            for (dx, lift) in [(sw, (pose.walk.cos()).max(0.0)), (-sw, (-pose.walk.cos()).max(0.0))] {
                let foot = (x + dx, g - lift * 8.0 * k * pose.moving as i32 as f32);
                cx.line(hip, foot, 20.0 * k, pants);
                cx.ellipse(foot.0 + 8.0 * k * f, foot.1, 15.0 * k, 7.0 * k, rgba(60, 40, 30, 1.0));
            }
        }
        // torso
        cx.line(hip, shoulder, 50.0 * k, shirt);
        if tier >= 1 {
            // sash
            cx.line((shoulder.0 - 18.0 * k, shoulder.1), (hip.0 + 18.0 * k, hip.1), 9.0 * k, if tier >= 2 { RED } else { GOLD });
        }
        if tier >= 2 {
            cx.line((hip.0 - 24.0 * k, hip.1 - 2.0), (hip.0 + 24.0 * k, hip.1 - 2.0), 8.0 * k, GOLD_D);
            cx.circle(shoulder.0, shoulder.1 + 14.0 * k, 9.0 * k, rgba(80, 200, 120, 1.0));
            cx.ring(shoulder.0, shoulder.1 + 14.0 * k, 10.0 * k, 3.0 * k, GOLD);
        }
        // arms
        let arm_c = if tier >= 2 { rgba(240, 195, 70, 1.0) } else { shirt };
        let front = (shoulder.0 + 34.0 * k * f, shoulder.1 + 30.0 * k);
        if let Some(sh) = pose.hold_cyl {
            let hands = (shoulder.0 + 48.0 * k * f, shoulder.1 + 22.0 * k);
            cx.line(shoulder, hands, 15.0 * k, arm_c);
            draw_cylinder(cx, hands.0 + 10.0 * k * f, hands.1 + 25.0 * k, (0.35 + sh) * f, 0.75 * k);
            cx.circle(hands.0, hands.1, 9.0 * k, SKIN);
        } else if pose.wai > 0.01 {
            let raise = smooth((pose.wai - 0.4) / 0.6);
            let hand = (
                lerp(front.0, head.0 + 40.0 * k * f, raise),
                lerp(front.1 - 6.0 * k, head.1 + 4.0 * k, raise),
            );
            let elbow = (shoulder.0 + 14.0 * k * f, shoulder.1 + 48.0 * k - raise * 18.0 * k);
            cx.line(shoulder, elbow, 15.0 * k, arm_c);
            cx.line(elbow, hand, 13.0 * k, arm_c);
            // joined palms (pointed up)
            cx.poly(&[(hand.0 - 8.0 * k, hand.1 + 10.0 * k), (hand.0 + 8.0 * k, hand.1 + 10.0 * k), (hand.0 + 5.0 * k, hand.1 - 10.0 * k), (hand.0, hand.1 - 20.0 * k), (hand.0 - 5.0 * k, hand.1 - 10.0 * k)], SKIN);
        } else {
            let sw = if pose.moving { pose.walk.sin() * 26.0 * k } else { 0.0 };
            for d in [sw, -sw] {
                let hand = (shoulder.0 + d, shoulder.1 + 78.0 * k);
                cx.line(shoulder, hand, 14.0 * k, arm_c);
                cx.circle(hand.0, hand.1, 9.0 * k, SKIN);
            }
        }
        // head
        cx.circle(head.0 - 4.0 * k * f, head.1 - 4.0 * k, 33.0 * k, HAIR);
        cx.circle(head.0 + 3.0 * k * f, head.1 + 2.0 * k, 29.0 * k, SKIN);
        cx.ellipse(head.0 - 10.0 * k * f, head.1 - 22.0 * k, 26.0 * k, 12.0 * k, HAIR);
        let eye_closed = pose.wai > 0.3 || pose.kneel > 0.5;
        let ex = head.0 + 16.0 * k * f;
        if eye_closed {
            cx.line((ex - 5.0 * k, head.1), (ex + 5.0 * k, head.1), 2.5 * k, HAIR);
        } else {
            cx.circle(ex, head.1, 4.0 * k, HAIR);
        }
        cx.circle(ex + 2.0 * f * k, head.1 + 11.0 * k, 5.0 * k, rgba(255, 150, 150, 0.6));
        cx.line((head.0 + 12.0 * k * f, head.1 + 16.0 * k), (head.0 + 22.0 * k * f, head.1 + 14.0 * k), 2.5 * k, rgba(150, 60, 50, 1.0));
    }
    // crown (tier 4+)
    if tier >= 3 {
        let cy = head.1 - 30.0 * k;
        let w = 30.0 * k;
        cx.poly(&[(head.0 - w, cy + 12.0 * k), (head.0 + w, cy + 12.0 * k), (head.0 + w, cy - 6.0 * k), (head.0 - w, cy - 6.0 * k)], GOLD);
        for i in 0..5 {
            let px = head.0 - w + i as f32 * w * 0.5;
            cx.poly(&[(px - 7.0 * k, cy - 4.0 * k), (px + 7.0 * k, cy - 4.0 * k), (px, cy - (24.0 + if i == 2 { 16.0 } else { 0.0 }) * k)], GOLD);
        }
        cx.circle(head.0, cy + 3.0 * k, 5.0 * k, rgba(220, 30, 60, 1.0));
        if tier >= 4 {
            cx.circle(head.0 - 16.0 * k, cy + 3.0 * k, 4.0 * k, rgba(40, 140, 255, 1.0));
            cx.circle(head.0 + 16.0 * k, cy + 3.0 * k, 4.0 * k, rgba(40, 200, 120, 1.0));
            let tw = (t * 6.0).sin().abs();
            sparkle(cx, head.0 + 6.0 * k, cy - 36.0 * k, 10.0 * k * tw, rgba(255, 255, 255, 0.9));
        }
    }
}

pub fn sparkle(cx: &Cx, x: f32, y: f32, r: f32, c: Color32) {
    if r <= 0.1 {
        return;
    }
    cx.poly(&[(x, y - r), (x + r * 0.25, y), (x, y + r), (x - r * 0.25, y)], c);
    cx.poly(&[(x - r, y), (x, y - r * 0.25), (x + r, y), (x, y + r * 0.25)], c);
}

/// Attendant with golden parasol (entourage of top tier).
pub fn draw_attendant(cx: &Cx, sprites: &Sprites, x: f32, pose: &Pose) {
    let look = Look { tier: 1, shirt: rgba(150, 20, 40, 1.0), pants: rgba(40, 20, 30, 1.0), scale: 0.72 };
    let g = cx.ground();
    let top = g - 290.0;
    cx.line((x + 30.0, g - 110.0), (x + 30.0, top), 5.0, GOLD_D);
    for (i, w) in [(0, 70.0), (1, 55.0), (2, 40.0)] {
        let y = top + i as f32 * 26.0;
        cx.poly(&[(x + 30.0 - w, y + 18.0), (x + 30.0 + w, y + 18.0), (x + 30.0 + w * 0.7, y), (x + 30.0 - w * 0.7, y)], if i % 2 == 0 { GOLD } else { rgba(230, 160, 30, 1.0) });
    }
    cx.poly(&[(x + 22.0, top), (x + 38.0, top), (x + 30.0, top - 30.0)], GOLD);
    draw_character(cx, sprites, x, pose, &look);
}

/// Name banner above the character. Grows fancier with tier.
pub fn draw_name_banner(cx: &Cx, x: f32, y: f32, name: &str, tier: usize, tier_name: &str) {
    let t = cx.time;
    let size = [26.0, 28.0, 32.0, 36.0, 42.0][tier.min(4)];
    let m = cx.measure(name, size, true);
    let w = m.x + 36.0 + tier as f32 * 10.0;
    let h = size * 1.55;
    let (x0, y0) = (x - w / 2.0, y - h);
    match tier {
        0 => {
            cx.rect(x0, y0, x0 + w, y, h / 2.0, rgba(0, 0, 0, 0.6));
            cx.text(x, y - h / 2.0, Align2::CENTER_CENTER, name, size, Color32::WHITE, true);
        }
        1 => {
            cx.rect(x0, y0, x0 + w, y, h / 2.0, rgba(20, 20, 40, 0.8));
            cx.rect_line(x0, y0, x0 + w, y, h / 2.0, 3.0, GOLD);
            cx.text(x, y - h / 2.0, Align2::CENTER_CENTER, name, size, GOLD, true);
        }
        _ => {
            if tier >= 3 {
                // ribbon tails
                let tl = 40.0 + tier as f32 * 8.0;
                cx.poly(&[(x0 + 10.0, y0 + 8.0), (x0 - tl, y0 + h * 0.3), (x0 - tl * 0.6, y0 + h * 0.6), (x0 - tl, y0 + h + 6.0), (x0 + 10.0, y - 2.0)], rgba(170, 20, 40, 1.0));
                cx.poly(&[(x0 + w - 10.0, y0 + 8.0), (x0 + w - 10.0, y - 2.0), (x0 + w + tl, y0 + h + 6.0), (x0 + w + tl * 0.6, y0 + h * 0.6), (x0 + w + tl, y0 + h * 0.3)], rgba(170, 20, 40, 1.0));
                cx.glow(x, y - h / 2.0, w * 0.75, GOLD, 0.35);
            }
            cx.vgradient(x0, y0, x0 + w, y, rgba(255, 236, 150, 1.0), rgba(214, 150, 30, 1.0));
            cx.rect_line(x0, y0, x0 + w, y, 6.0, 3.0 + tier as f32, rgba(130, 70, 0, 1.0));
            cx.text_o(x, y - h / 2.0, Align2::CENTER_CENTER, name, size, rgba(120, 10, 20, 1.0), rgba(255, 245, 210, 0.9), true);
            if !tier_name.is_empty() {
                let ts = size * 0.5;
                let tm = cx.measure(tier_name, ts, true);
                cx.rect(x - tm.x / 2.0 - 12.0, y0 - ts * 1.45, x + tm.x / 2.0 + 12.0, y0 - 2.0, 8.0, rgba(120, 10, 20, 0.95));
                cx.text(x, y0 - ts * 0.75, Align2::CENTER_CENTER, tier_name, ts, GOLD, true);
            }
            if tier >= 4 {
                for i in 0..8 {
                    let ph = (t * 1.3 + i as f32 * 0.37).fract();
                    let sx = x0 + (i as f32 * 0.137 * 7.0).fract() * w;
                    sparkle(cx, sx, y0 + h * (1.0 - ph) - 10.0, 12.0 * (1.0 - (ph * 2.0 - 1.0).abs()), rgba(255, 255, 255, 0.95));
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
    pub kind: u8, // 0 = spark, 1 = glitter (twinkles), 2 = rocket
    pub tier: usize,
}

pub fn draw_particles(cx: &Cx, ps: &[Particle]) {
    for p in ps {
        let a = (p.life / p.max).clamp(0.0, 1.0);
        match p.kind {
            2 => {
                cx.line((p.pos.x, p.pos.y), (p.pos.x - p.vel.x * 0.03, p.pos.y - p.vel.y * 0.03), 4.0, rgba(255, 230, 160, 0.9));
                cx.glow(p.pos.x, p.pos.y, 18.0, rgba(255, 220, 120, 1.0), 0.8);
            }
            1 => {
                let tw = 0.5 + 0.5 * (cx.time * 15.0 + p.pos.x).sin();
                sparkle(cx, p.pos.x, p.pos.y, p.size * 2.2 * tw, with_a(p.color, a));
            }
            _ => {
                let tail = p.vel * 0.035;
                cx.line((p.pos.x, p.pos.y), (p.pos.x - tail.x, p.pos.y - tail.y), p.size, with_a(p.color, a));
            }
        }
    }
}

pub fn shimmer(cx: &Cx, strength: f32) {
    if !cx.soft {
        return;
    }
    // diagonal golden light sweeps + soft vignette
    let t = cx.time;
    for i in 0..3 {
        let ph = ((t * 0.35) + i as f32 / 3.0).fract();
        let x = -400.0 + ph * (VW + 800.0);
        let h = cx.vh;
        cx.poly(&[(x, 0.0), (x + 90.0, 0.0), (x - 160.0, h), (x - 250.0, h)], rgba(255, 230, 150, 0.12 * strength));
    }
    let _ = PI;
}

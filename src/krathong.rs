//! ลอยกระทง theme art: night river, moon, city lights, krathong, blessing visuals.
use crate::scene::*;
use egui::{Align2, Color32};

/// bank top (where the character kneels) in the krathong theme
pub fn bank_y(vh: f32) -> f32 {
    (vh * 0.60).min(vh - 230.0).max(vh * 0.45)
}
pub fn water_top(vh: f32) -> f32 {
    bank_y(vh) + 14.0
}
pub const KNEEL_X: f32 = 330.0;
pub const BANK_EDGE: f32 = 450.0;

/// Night sky backdrop (background = "scene").
pub fn draw_sky(cx: &Cx, sprites: &Sprites) {
    let h = cx.vh;
    if let Some(t) = &sprites.background {
        cx.image(t, -40.0, -40.0, VW + 40.0, h + 40.0, Color32::WHITE);
        return;
    }
    let wt = water_top(h);
    cx.vgradient(-60.0, -60.0, VW + 60.0, wt, rgba(8, 12, 40, 1.0), rgba(48, 30, 90, 1.0));
    for i in 0..70 {
        let x = (i as f32 * 151.3) % VW;
        let y = (i as f32 * 71.9) % (wt * 0.8);
        let tw = 0.5 + 0.5 * (cx.time * (1.0 + (i % 5) as f32 * 0.4) + i as f32).sin();
        cx.circle(x, y, 1.0 + (i % 3) as f32 * 0.6, rgba(255, 255, 240, 0.25 + 0.6 * tw));
    }
    draw_moon(cx, 900.0, 130.0, 62.0);
    // distant city skyline with twinkling windows
    let base = wt;
    let mut x = -20.0;
    let mut i = 0;
    while x < VW + 20.0 {
        let w = 40.0 + ((i * 37) % 50) as f32;
        let hgt = 30.0 + ((i * 53) % 90) as f32;
        cx.rect(x, base - hgt, x + w, base, 0.0, rgba(20, 16, 45, 1.0));
        for wy in 0..((hgt / 14.0) as i32) {
            for wx in 0..((w / 12.0) as i32) {
                let on = ((i * 7 + wy * 3 + wx * 5) % 7) < 2;
                if on {
                    let fl = 0.6 + 0.4 * (cx.time * 0.7 + (wx + wy + i) as f32).sin();
                    cx.rect(x + 4.0 + wx as f32 * 12.0, base - hgt + 6.0 + wy as f32 * 14.0, x + 9.0 + wx as f32 * 12.0, base - hgt + 11.0 + wy as f32 * 14.0, 0.0, rgba(255, 210, 120, 0.7 * fl));
                }
            }
        }
        x += w + 6.0;
        i += 1;
    }
}

pub fn draw_moon(cx: &Cx, x: f32, y: f32, r: f32) {
    let pulse = 0.5 + 0.5 * (cx.time * 0.8).sin();
    cx.glow(x, y, r * 3.2, rgba(255, 240, 200, 1.0), 0.28 + 0.05 * pulse);
    cx.circle(x, y, r, rgba(255, 246, 220, 1.0));
    cx.circle(x - r * 0.3, y - r * 0.2, r * 0.18, rgba(235, 225, 195, 1.0));
    cx.circle(x + r * 0.25, y + r * 0.3, r * 0.12, rgba(235, 225, 195, 1.0));
    cx.circle(x + r * 0.35, y - r * 0.35, r * 0.08, rgba(235, 225, 195, 1.0));
}

/// River surface with animated waves (+ moon reflection when `moon`).
pub fn draw_river(cx: &Cx, moon: bool, opaque: bool) {
    let h = cx.vh;
    let wt = water_top(h);
    let a = if opaque { 1.0 } else { 0.9 };
    cx.vgradient(-60.0, wt, VW + 60.0, h + 60.0, rgba(20, 40, 90, a), rgba(6, 14, 40, a));
    let t = cx.time;
    if moon {
        for j in 0..18 {
            let y = wt + 10.0 + j as f32 * (h - wt) / 20.0;
            let w = 30.0 + 40.0 * (1.0 - j as f32 / 18.0) * (0.6 + 0.4 * (t * 2.0 + j as f32 * 1.7).sin());
            let dx = (t * 1.5 + j as f32).sin() * 6.0;
            cx.seg((900.0 - w + dx, y), (900.0 + w + dx, y), 3.0, rgba(255, 240, 200, 0.35 * (1.0 - j as f32 / 20.0)));
        }
    }
    // wave lines (perspective spacing)
    let rows = 14;
    for j in 0..rows {
        let u = j as f32 / rows as f32;
        let y = wt + 6.0 + (h - wt) * u * u * 1.05;
        let amp = 1.5 + u * 4.0;
        let seg = 40.0;
        let mut x = -20.0;
        while x < VW + 20.0 {
            let y1 = y + (x * 0.02 + t * 1.4 + j as f32).sin() * amp;
            let y2 = y + ((x + seg) * 0.02 + t * 1.4 + j as f32).sin() * amp;
            let vis = ((x * 0.013 + j as f32 * 2.1 + t * 0.4).sin() * 0.5 + 0.5).powi(2);
            cx.seg((x, y1), (x + seg, y2), 1.5 + u * 1.5, rgba(150, 190, 255, 0.10 + 0.25 * vis));
            x += seg;
        }
    }
}

/// Riverside terrace (ท่าน้ำ) bottom-left: little Thai sala, string lights, grass, stone wall + steps.
pub fn draw_bank(cx: &Cx) {
    let h = cx.vh;
    let g = bank_y(h);
    let t = cx.time;
    let e = BANK_EDGE;
    // little sala (pavilion) silhouette on the far left, behind the walkway
    let sx = 120.0;
    let roof_y = g - 190.0;
    for px in [sx - 60.0, sx + 60.0] {
        cx.rect(px - 6.0, roof_y + 40.0, px + 6.0, g, 2.0, rgba(120, 40, 40, 1.0));
    }
    cx.poly(&[(sx - 95.0, roof_y + 46.0), (sx + 95.0, roof_y + 46.0), (sx + 60.0, roof_y + 10.0), (sx - 60.0, roof_y + 10.0)], rgba(150, 40, 45, 1.0));
    cx.poly(&[(sx - 62.0, roof_y + 12.0), (sx + 62.0, roof_y + 12.0), (sx + 30.0, roof_y - 30.0), (sx - 30.0, roof_y - 30.0)], rgba(175, 50, 52, 1.0));
    cx.seg((sx - 95.0, roof_y + 46.0), (sx + 95.0, roof_y + 46.0), 4.0, GOLD);
    cx.seg((sx - 62.0, roof_y + 12.0), (sx + 62.0, roof_y + 12.0), 3.0, GOLD);
    // chofa finials
    for (x0, d) in [(sx + 95.0, 1.0f32), (sx - 95.0, -1.0)] {
        cx.poly(&[(x0, roof_y + 46.0), (x0 + d * 8.0, roof_y + 26.0), (x0 + d * 16.0, roof_y + 18.0), (x0 + d * 6.0, roof_y + 36.0)], GOLD);
    }
    cx.poly(&[(sx - 6.0, roof_y - 30.0), (sx + 6.0, roof_y - 30.0), (sx, roof_y - 60.0)], GOLD);
    cx.glow(sx, roof_y + 90.0, 90.0, rgba(255, 190, 110, 1.0), 0.35);
    // string of lanterns from the sala towards the river
    let (ax, ay, bx, by) = (sx + 95.0, roof_y + 50.0, e - 10.0, g - 120.0);
    cx.seg((e - 10.0, g), (e - 10.0, g - 126.0), 4.0, rgba(90, 60, 40, 1.0));
    let n = 7;
    let mut prev = (ax, ay);
    for i in 1..=n {
        let u = i as f32 / n as f32;
        let p = (lerp(ax, bx, u), lerp(ay, by, u) + (u * std::f32::consts::PI).sin() * 26.0);
        cx.seg(prev, p, 1.5, rgba(60, 40, 40, 1.0));
        prev = p;
        if i < n {
            let fl = 0.85 + 0.15 * (t * 2.0 + i as f32).sin();
            let c = [rgba(255, 120, 120, 1.0), rgba(255, 200, 90, 1.0), rgba(255, 150, 200, 1.0)][i % 3];
            cx.glow(p.0, p.1 + 12.0, 26.0, c, 0.45 * fl);
            cx.ellipse(p.0, p.1 + 12.0, 8.0, 10.0, c);
            cx.rect(p.0 - 4.0, p.1 + 1.0, p.0 + 4.0, p.1 + 4.0, 1.0, GOLD_D);
        }
    }
    // stone retaining wall (front face of the terrace)
    let wall_bot = h + 80.0;
    cx.poly(&[(-60.0, g), (e, g), (e + 10.0, wall_bot), (-60.0, wall_bot)], rgba(56, 48, 76, 1.0));
    cx.vgradient(-60.0, g + 8.0, e + 4.0, h + 40.0, rgba(78, 66, 96, 1.0), rgba(26, 22, 44, 1.0));
    let mut row = 0;
    let mut y = g + 20.0;
    while y < h + 20.0 {
        cx.seg((-60.0, y), (e + 4.0, y), 2.0, rgba(30, 26, 40, 0.7));
        let off = if row % 2 == 0 { 0.0 } else { 30.0 };
        let mut x = -60.0 + off;
        while x < e {
            cx.seg((x, y), (x, y + 26.0), 2.0, rgba(30, 26, 40, 0.6));
            x += 60.0;
        }
        y += 26.0;
        row += 1;
    }
    // walkway deck + grass edge
    cx.rect(-60.0, g - 4.0, e + 6.0, g + 10.0, 3.0, rgba(150, 100, 70, 1.0));
    cx.rect(-60.0, g - 4.0, e + 6.0, g + 1.0, 3.0, rgba(190, 140, 95, 1.0));
    // steps down to the water on the right edge
    for i in 0..4 {
        let y = g + 10.0 + i as f32 * 18.0;
        let x0 = e - 10.0 + i as f32 * 20.0;
        cx.rect(x0, y, x0 + 70.0, y + 14.0, 3.0, rgba(160, 150, 160, 1.0));
        cx.rect(x0, y, x0 + 70.0, y + 4.0, 3.0, rgba(200, 190, 200, 1.0));
    }
    // flowers + bushes along the walkway
    for i in 0..7 {
        let x = 10.0 + i as f32 * 62.0;
        if (x - sx).abs() < 40.0 {
            continue;
        }
        let sw = (t * 1.3 + i as f32).sin() * 2.0;
        cx.ellipse(x, g - 6.0, 26.0, 12.0, rgba(46, 100, 60, 1.0));
        cx.ellipse(x - 6.0, g - 10.0, 16.0, 9.0, rgba(66, 130, 74, 1.0));
        let c = [rgba(255, 150, 190, 1.0), rgba(255, 220, 120, 1.0), rgba(255, 250, 245, 1.0)][i % 3];
        for j in 0..3 {
            cx.circle(x - 12.0 + j as f32 * 12.0 + sw, g - 16.0 - (j % 2) as f32 * 5.0, 4.5, c);
        }
    }
}

/// A krathong at (x, y = waterline). `sc` scale, `tier` luxury, `seed` animation offset.
pub fn draw_krathong(cx: &Cx, x: f32, y: f32, sc: f32, tier: usize, seed: f32, name: Option<&str>, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let t = cx.time;
    let bob = (t * 1.8 + seed).sin() * 2.5 * sc;
    let tilt = (t * 1.3 + seed * 1.7).sin() * 0.05;
    let y = y + bob;
    let s = sc * [1.0, 1.05, 1.15, 1.3, 1.5][tier.min(4)];
    let leaf = if tier >= 2 { rgba(225, 175, 50, alpha) } else { rgba(60, 140, 60, alpha) };
    let leaf2 = if tier >= 2 { rgba(250, 210, 90, alpha) } else { rgba(90, 170, 80, alpha) };
    // reflection
    if cx.soft {
        for i in 0..5 {
            let yy = y + 10.0 * s + i as f32 * 9.0 * s;
            let wob = (t * 3.0 + i as f32 + seed).sin() * 4.0 * s;
            cx.glow_add(x + wob, yy, (16.0 - i as f32 * 2.0) * s, rgba(255, 180, 80, 1.0), 0.35 * alpha * (1.0 - i as f32 / 5.0));
        }
    }
    // base (banana trunk slice)
    cx.ellipse(x, y + 4.0 * s, 46.0 * s, 11.0 * s, rgba(210, 225, 170, alpha));
    cx.ellipse(x, y, 44.0 * s, 9.0 * s, rgba(170, 200, 120, alpha));
    // folded banana-leaf petals
    let petals = 9;
    for i in 0..petals {
        let u = i as f32 / (petals - 1) as f32;
        let px = x + (u - 0.5) * 80.0 * s;
        let depth = 1.0 - ((u - 0.5) * 2.0).abs();
        let hgt = (18.0 + depth * 10.0) * s;
        let w = 10.0 * s;
        let r = |dx: f32, dy: f32| {
            let (sn, cs) = tilt.sin_cos();
            (x + ((px - x) + dx) * cs - dy * sn, y + ((px - x) + dx) * sn + dy * cs)
        };
        cx.poly(&[r(-w, 0.0), r(w, 0.0), r(0.0, -hgt)], if i % 2 == 0 { leaf } else { leaf2 });
    }
    // flower mound
    let fy = y - 18.0 * s;
    cx.ellipse(x, fy + 4.0 * s, 30.0 * s, 12.0 * s, rgba(240, 150, 30, alpha));
    let flowers = [(-18.0, 0.0, 0), (0.0, -4.0, 1), (18.0, 0.0, 0), (-9.0, 5.0, 2), (9.0, 5.0, 2)];
    for (dx, dy, kind) in flowers {
        let c = match kind {
            0 => rgba(255, 160, 20, alpha),
            1 => rgba(255, 120, 160, alpha),
            _ => rgba(255, 250, 240, alpha),
        };
        cx.circle(x + dx * s, fy + dy * s, 7.0 * s, c);
        cx.circle(x + dx * s, fy + dy * s, 2.5 * s, rgba(255, 230, 120, alpha));
    }
    if tier >= 3 {
        for i in 0..6 {
            let a = i as f32 / 6.0 * std::f32::consts::TAU;
            cx.circle(x + a.cos() * 36.0 * s, y - 4.0 * s + a.sin() * 6.0 * s, 3.0 * s, rgba(255, 255, 255, alpha));
        }
    }
    // incense
    for i in 0..3 {
        let ix = x + (i as f32 - 1.0) * 5.0 * s + 14.0 * s;
        cx.seg((ix, fy), (ix + (i as f32 - 1.0) * 3.0 * s, fy - 34.0 * s), 1.6 * s, rgba(150, 60, 30, alpha));
        cx.circle(ix + (i as f32 - 1.0) * 3.0 * s, fy - 34.0 * s, 1.6 * s, rgba(255, 90, 40, alpha));
    }
    // candles (more for higher tiers)
    let n = [1, 1, 2, 3, 5][tier.min(4)];
    for i in 0..n {
        let off = if n == 1 { -8.0 } else { (i as f32 / (n - 1) as f32 - 0.5) * 34.0 };
        let cxp = x + off * s;
        let top = fy - 26.0 * s - if i == n / 2 { 6.0 * s } else { 0.0 };
        cx.rect(cxp - 3.0 * s, top, cxp + 3.0 * s, fy, 1.0, rgba(255, 245, 220, alpha));
        let fl = noise1(t * 7.0 + seed + i as f32);
        cx.glow_add(cxp, top - 6.0 * s, 22.0 * s, rgba(255, 170, 60, 1.0), 0.8 * alpha);
        cx.poly(&[(cxp - 3.5 * s, top), (cxp + 3.5 * s, top), (cxp + fl * 2.0 * s, top - 14.0 * s)], rgba(255, 190, 60, alpha));
        cx.circle(cxp, top - 3.0 * s, 1.8 * s, rgba(255, 255, 220, alpha));
    }
    if tier >= 4 {
        cx.glow(x, y - 10.0 * s, 80.0 * s, GOLD, 0.35 * alpha);
        let tw = (t * 5.0 + seed).sin().abs();
        sparkle(cx, x + 30.0 * s, y - 40.0 * s, 9.0 * s * tw, rgba(255, 255, 255, alpha));
    }
    if let Some(nm) = name {
        let size = 18.0 * sc.max(0.6);
        cx.text_o(x, y + 26.0 * s, Align2::CENTER_CENTER, nm, size, rgba(255, 240, 200, alpha), rgba(10, 20, 50, alpha * 0.9), true);
    }
}

/// Foreground depth layer: lotus pads + blooms on the near water and reeds at the corners.
pub fn draw_foreground(cx: &Cx) {
    let vh = cx.vh;
    let t = cx.time;
    let wt = water_top(vh);
    let base = vh - 18.0;
    if base < wt + 40.0 {
        return;
    }
    for (i, x) in [600.0f32, 690.0, 1010.0].iter().enumerate() {
        let bob = (t * 1.2 + i as f32).sin() * 2.0;
        cx.ellipse(*x, base + bob, 44.0, 12.0, rgba(40, 110, 70, 1.0));
        cx.ellipse(*x - 4.0, base - 2.0 + bob, 40.0, 9.0, rgba(60, 140, 85, 1.0));
        if i != 1 {
            draw_lotus(cx, *x + 8.0, base - 12.0 + bob, 0.8);
            cx.glow(*x + 8.0, base - 18.0 + bob, 40.0, rgba(255, 190, 220, 1.0), 0.25);
        }
    }
    for j in 0..6 {
        let x = 1060.0 - j as f32 * 9.0;
        let sw = (t * 1.4 + j as f32).sin() * 4.0;
        cx.seg((x, vh), (x - 10.0 + sw, vh - 120.0 - (j % 3) as f32 * 25.0), 3.5, rgba(30, 70, 50, 1.0));
    }
    // fireflies
    for i in 0..8 {
        let ph = i as f32 * 1.37;
        let x = 520.0 + (ph * 97.0) % 520.0 + (t * 0.5 + ph).sin() * 20.0;
        let y = wt + 30.0 + (ph * 53.0) % (vh - wt - 60.0).max(10.0) + (t * 0.7 + ph).cos() * 10.0;
        let a = 0.5 + 0.5 * (t * 2.5 + ph * 3.0).sin();
        cx.glow(x, y, 12.0, rgba(220, 255, 150, 1.0), 0.5 * a);
        cx.circle(x, y, 1.8, rgba(240, 255, 200, a));
    }
}

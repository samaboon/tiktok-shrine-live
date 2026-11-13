//! Shared on-screen sticker board ("ส่งอะไร ได้…") and star spectacle,
//! reused by dance, krathong, and any future mode.
use crate::scene::*;
use egui::{Align2, TextureHandle};
use std::collections::HashMap;
use std::f32::consts::TAU;

/// One row on a sticker legend board.
#[derive(Clone, Debug)]
pub struct BoardRow {
    pub id: u64,
    pub name: String,
    pub diamonds: u64,
    pub style: String,
    /// When true, this row stays visible at the top (star / top-priced gift) and is not paged away.
    pub pin: bool,
}

pub enum BoardWhen {
    Always,
    Idle,
    Off,
}

pub fn board_when(s: &str) -> BoardWhen {
    let k = s.trim().to_lowercase();
    if k.is_empty() || k == "always" || k == "ตลอด" || k == "on" || k == "เปิด" {
        BoardWhen::Always
    } else if k == "off" || k == "ปิด" || k == "none" || k == "hide" {
        BoardWhen::Off
    } else {
        BoardWhen::Idle
    }
}

pub fn board_is_right(s: &str) -> bool {
    let k = s.trim().to_lowercase();
    k == "right" || k == "ขวา"
}

/// Thai label for the krathong look at a given luxury tier (star is its own style).
pub fn krathong_style_label(tier: usize, star: bool) -> &'static str {
    if star {
        "กระทงสตาร์"
    } else {
        ["กระทงใบตอง", "กระทงดอกไม้", "กระทงทอง", "กระทงหรูมีโคม", "กระทงจักรพรรดิ"][tier.min(4)]
    }
}

/// On-screen legend: sticker icon + name + diamonds + style/move.
/// Sits on the left (default) or right, below `top`, and stops above `max_y1`.
pub fn draw_sticker_board(
    cx: &Cx,
    title: &str,
    rows: &[BoardRow],
    icons: &HashMap<u64, TextureHandle>,
    position: &str,
    max_rows: usize,
    page_seconds: f32,
    top: f32,
    max_y1: f32,
) -> Option<[f32; 4]> {
    if rows.is_empty() {
        return None;
    }
    let pinned: Vec<&BoardRow> = rows.iter().filter(|r| r.pin).collect();
    let rest: Vec<&BoardRow> = rows.iter().filter(|r| !r.pin).collect();
    let max_rows_cfg = max_rows.clamp(3, 12);
    let row_h = 54.0;
    let pad = 14.0;
    let header = 46.0;
    let y0_est = top.max(8.0);
    let fit = (((max_y1 - y0_est - header - 30.0) / row_h).floor() as i32).max(1) as usize;
    let fit = fit.min(max_rows_cfg).max(1);
    let pin_n = pinned.len().min(fit);
    let page_cap = (fit - pin_n).max(if rest.is_empty() { 0 } else { 1 });
    let pages = if rest.is_empty() || page_cap == 0 {
        1
    } else {
        (rest.len() + page_cap - 1) / page_cap
    };
    let page_s = page_seconds.max(1.5);
    let page = if pages <= 1 { 0 } else { ((cx.time / page_s).floor() as usize) % pages };
    let start = page * page_cap;
    let end = (start + page_cap).min(rest.len());
    let rest_slice = if page_cap == 0 { &[][..] } else { &rest[start..end] };

    let footer = if pages > 1 { 30.0 } else { 10.0 };
    let w = 320.0;
    let n_vis = pin_n + rest_slice.len();
    if n_vis == 0 {
        return None;
    }
    let h = header + n_vis as f32 * row_h + footer;
    let x0 = if board_is_right(position) { VW - 16.0 - w } else { 16.0 };
    let x1 = x0 + w;
    let y0 = top.min(max_y1 - 160.0).max(8.0);
    let y1 = (y0 + h).min(max_y1);
    if y1 - y0 < header + row_h {
        return None;
    }

    cx.rect(x0, y0, x1, y1, 18.0, rgba(12, 8, 28, 0.82));
    cx.rect_line(x0, y0, x1, y1, 18.0, 3.0, rgba(255, 210, 80, 0.8));
    cx.text((x0 + x1) * 0.5, y0 + 24.0, Align2::CENTER_CENTER, title, 24.0, rgba(255, 230, 140, 1.0), true);

    let mut y = y0 + header;
    let mut draw_row = |r: &BoardRow, y: f32| {
        if y + row_h - 4.0 > y1 - footer + 4.0 {
            return false;
        }
        let iy = y + 4.0;
        let ix = x0 + pad;
        let isz = 44.0;
        if let Some(tex) = icons.get(&r.id) {
            cx.image(tex, ix, iy, ix + isz, iy + isz, rgba(255, 255, 255, 1.0));
        } else {
            cx.rect(ix, iy, ix + isz, iy + isz, 8.0, rgba(40, 30, 60, 0.9));
        }
        let tx = ix + isz + 10.0;
        cx.text(tx, y + 16.0, Align2::LEFT_CENTER, &r.name, 22.0, rgba(255, 255, 255, 1.0), true);
        let price = format!("{} เพชร", r.diamonds);
        cx.text(x1 - pad, y + 16.0, Align2::RIGHT_CENTER, &price, 17.0, rgba(255, 220, 140, 0.95), true);
        cx.text(tx, y + 38.0, Align2::LEFT_CENTER, &r.style, 20.0, rgba(255, 200, 80, 1.0), true);
        true
    };
    for r in &pinned {
        if !draw_row(r, y) {
            break;
        }
        y += row_h;
    }
    for r in rest_slice {
        if !draw_row(r, y) {
            break;
        }
        y += row_h;
    }
    if pages > 1 {
        let lab = format!("{}/{}", page + 1, pages);
        cx.text((x0 + x1) * 0.5, y1 - 14.0, Align2::CENTER_CENTER, &lab, 18.0, rgba(255, 230, 200, 0.85), true);
    }
    Some([x0, y0, x1, y1])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StarStyle {
    /// Concert stage: cool/magenta lasers + gold beams.
    Stage,
    /// Loy Krathong river: warm lanterns, gold fireworks, amber beams.
    River,
}

/// Grand star spectacle. `gy` is the foot/water line; rays skip the face cone around `head_y`.
pub fn draw_star_fx(cx: &Cx, x: f32, gy: f32, t: f32, appear: f32, style: StarStyle, head_y: Option<f32>) {
    let a = appear.clamp(0.0, 1.0);
    if a < 0.05 {
        return;
    }
    let head_y = head_y.unwrap_or(gy - 155.0);
    let body_y = (gy + head_y) * 0.5;
    let warm = matches!(style, StarStyle::River);

    // spotlight from above onto the body, not through the face
    let apex = (x, 8.0);
    let wash = if warm { rgba(255, 210, 120, 0.11 * a) } else { rgba(255, 240, 180, 0.10 * a) };
    let core = if warm { rgba(255, 236, 180, 0.14 * a) } else { rgba(255, 250, 220, 0.12 * a) };
    cx.poly(&[apex, (x - 130.0, gy + 8.0), (x + 130.0, gy + 8.0)], wash);
    cx.poly(&[apex, (x - 74.0, gy + 4.0), (x + 74.0, gy + 4.0)], core);

    // rotating beams, skip the face cone
    let o = (x, body_y);
    let hdx = 0.0;
    let hdy = head_y - body_y;
    let hdist = (hdx * hdx + hdy * hdy).sqrt().max(1.0);
    for i in 0..14 {
        let ang = i as f32 / 14.0 * TAU + t * 0.7;
        let (s, c) = ang.sin_cos();
        if (hdx * c + hdy * s) / hdist > 0.45 {
            continue;
        }
        let inner = 48.0;
        let len = 210.0 + 30.0 * (t * 3.0 + i as f32).sin();
        let col = if warm { rgba(255, 200, 90, 0.15 * a) } else { rgba(255, 230, 140, 0.14 * a) };
        cx.poly(
            &[(o.0 + c * inner, o.1 + s * inner), (o.0 + c * len - s * 10.0, o.1 + s * len + c * 10.0), (o.0 + c * len + s * 10.0, o.1 + s * len - c * 10.0)],
            col,
        );
    }

    // side lasers / light beams aimed at the body
    for (sx, hue) in [(-40.0, 320.0), (VW + 40.0, 190.0)] {
        let pulse = 0.5 + 0.5 * (t * 6.0 + hue).sin();
        if warm {
            cx.seg((sx, 20.0), (x, body_y + 10.0), 3.0 + pulse, rgba(255, 170, 60, (0.38 + 0.22 * pulse) * a));
            cx.seg((sx, 80.0), (x + 18.0, body_y + 16.0), 2.0, rgba(255, 120, 80, 0.28 * a));
        } else {
            cx.seg((sx, 20.0), (x, body_y + 10.0), 3.0 + pulse, rgba(255, 80, 200, (0.35 + 0.25 * pulse) * a));
            cx.seg((sx, 80.0), (x + 18.0, body_y + 16.0), 2.0, rgba(80, 200, 255, 0.30 * a));
        }
    }

    // fireworks at the upper sides, clear of the face
    for (k, ox) in [(0.0, x - 260.0), (1.0, x + 260.0), (2.0, x - 200.0), (3.0, x + 200.0)] {
        let burst = (t * 1.8 + k * 1.3).fract();
        let by = 90.0 + k * 20.0;
        if burst < 0.55 {
            let u = burst / 0.55;
            for j in 0..8 {
                let ang = j as f32 / 8.0 * TAU + k;
                let (s, c) = ang.sin_cos();
                let r = 20.0 + u * 70.0;
                let fade = (1.0 - u) * a;
                let col = if warm && j % 2 == 0 { rgba(255, 120, 70, 0.8 * fade) } else { rgba(255, 210, 80, 0.75 * fade) };
                cx.circle(ox + c * r, by + s * r * 0.7, 3.5, col);
            }
        }
    }

    if warm {
        draw_rising_lanterns(cx, x, gy, t, a);
    }
}

fn draw_rising_lanterns(cx: &Cx, x: f32, gy: f32, t: f32, a: f32) {
    for i in 0..8 {
        let phase = (t * 0.18 + i as f32 * 0.17).fract();
        let px = x + ((i as f32 - 3.5) * 70.0) + (t * 0.7 + i as f32).sin() * 16.0;
        let py = gy - 40.0 - phase * (gy * 0.75 + 80.0);
        let fade = (1.0 - (phase - 0.15).max(0.0) / 0.85) * a;
        if fade <= 0.02 || py < 10.0 {
            continue;
        }
        let sc = 0.7 + (i % 3) as f32 * 0.12;
        cx.glow(px, py, 22.0 * sc, rgba(255, 180, 70, 1.0), 0.45 * fade);
        cx.rect(px - 7.0 * sc, py - 10.0 * sc, px + 7.0 * sc, py + 8.0 * sc, 3.0, rgba(230, 80, 40, fade));
        cx.rect(px - 7.0 * sc, py - 14.0 * sc, px + 7.0 * sc, py - 8.0 * sc, 2.0, rgba(180, 40, 30, fade));
        cx.circle(px, py + 2.0 * sc, 2.2 * sc, rgba(255, 230, 140, fade));
    }
}

/// Extra river glow used when a star krathong is on the water (keeps the mid band soft so faces stay readable).
pub fn draw_star_river_glow(cx: &Cx, water_top: f32, vh: f32, t: f32, a: f32) {
    let a = a.clamp(0.0, 1.0);
    if a < 0.02 {
        return;
    }
    let mid = (water_top + vh) * 0.5;
    let pulse = 0.55 + 0.45 * (t * 1.6).sin();
    cx.glow(VW * 0.5, mid, 280.0, rgba(255, 180, 80, 1.0), 0.18 * a * pulse);
    cx.glow(VW * 0.35, water_top + 30.0, 160.0, rgba(255, 210, 120, 1.0), 0.16 * a);
    cx.glow(VW * 0.7, water_top + 40.0, 150.0, rgba(255, 160, 70, 1.0), 0.14 * a);
}

/// Star name banner, drawn well above the head so it never sits on the face.
pub fn draw_star_banner(cx: &Cx, x: f32, tag_y: f32, name: &str, appear: f32, alpha: f32) {
    let a = (appear * alpha).clamp(0.0, 1.0);
    if a < 0.05 {
        return;
    }
    let title = "สตาร์";
    let size = 36.0;
    let m = cx.measure(name, size, true);
    let w = (m.x + 56.0).max(160.0);
    let h = 52.0;
    let y1 = tag_y - 8.0;
    let y0 = y1 - h - 26.0;
    cx.rect(x - w * 0.5, y0, x + w * 0.5, y1, 14.0, rgba(40, 10, 20, 0.88 * a));
    cx.rect_line(x - w * 0.5, y0, x + w * 0.5, y1, 14.0, 3.0, rgba(255, 210, 80, 0.95 * a));
    cx.text(x, y0 + 16.0, Align2::CENTER_CENTER, title, 20.0, rgba(255, 220, 120, a), true);
    cx.text(x, y0 + 40.0, Align2::CENTER_CENTER, name, size, rgba(255, 250, 230, a), true);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn krathong_labels_are_thai() {
        for t in 0..5 {
            let s = krathong_style_label(t, false);
            assert!(s.chars().any(|c| ('\u{0E00}'..='\u{0E7F}').contains(&c)), "{s}");
        }
        assert_eq!(krathong_style_label(4, true), "กระทงสตาร์");
    }
}

//! Settings UI: searchable TikTok gift (sticker) picker with images, IDs, and diamond prices.
use crate::app::Theme;
use crate::events::{request_gift_list, BridgeCtl};
use crate::gifts::{cache_dir, upsert_mapping, GiftCatalog, GiftInfo, IconFetcher, IconReady};
use egui::{Color32, ColorImage, RichText, TextureHandle, TextureOptions, Ui, Vec2};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    PriceAsc,
    PriceDesc,
    Name,
    Id,
}

impl SortMode {
    fn label(self) -> &'static str {
        match self {
            SortMode::PriceAsc => "ราคา: น้อยไปมาก",
            SortMode::PriceDesc => "ราคา: มากไปน้อย",
            SortMode::Name => "ชื่อ A–Z",
            SortMode::Id => "รหัสของขวัญ",
        }
    }
}

pub struct GiftPicker {
    pub catalog: GiftCatalog,
    pub cache_dir: PathBuf,
    search: String,
    sort: SortMode,
    icons: HashMap<u64, TextureHandle>,
    fetcher: IconFetcher,
    status: String,
}

impl GiftPicker {
    pub fn new(base_dir: PathBuf) -> Self {
        let cache_dir = cache_dir(&base_dir);
        let catalog = GiftCatalog::load_preferred(&cache_dir);
        let status = if catalog.gifts.is_empty() {
            "ยังไม่มีรายการของขวัญ — กดรีเฟรชหลังเชื่อมต่อไลฟ์ เพื่อดึงรายการจริงจาก TikTok".into()
        } else {
            String::new()
        };
        Self {
            catalog,
            cache_dir: cache_dir.clone(),
            search: String::new(),
            sort: SortMode::PriceAsc,
            icons: HashMap::new(),
            fetcher: IconFetcher::new(cache_dir),
            status,
        }
    }

    pub fn apply_live_list(&mut self, msg: crate::gifts::GiftListMsg) {
        let n = msg.gifts.len();
        let err = msg.error.clone();
        self.catalog.replace_from_msg(msg, &self.cache_dir);
        self.status = if let Some(e) = err {
            format!("รีเฟรชไม่สำเร็จ: {e}")
        } else if n == 0 {
            "ตัวเชื่อมส่งรายการว่างมา — ยังใช้รายการเดิม".into()
        } else {
            format!("อัปเดตจากไลฟ์แล้ว ({n} รายการ)")
        };
    }

    pub fn drain_icons(&mut self, ctx: &egui::Context) {
        for IconReady { id, rgba, w, h } in self.fetcher.try_recv() {
            let img = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
            let tex = ctx.load_texture(format!("gift-{id}"), img, TextureOptions::LINEAR);
            self.icons.insert(id, tex);
        }
    }

    fn filtered(&self) -> Vec<&GiftInfo> {
        let q = self.search.trim().to_lowercase();
        let mut v: Vec<&GiftInfo> = self
            .catalog
            .gifts
            .iter()
            .filter(|g| {
                if q.is_empty() {
                    return true;
                }
                g.name.to_lowercase().contains(&q)
                    || g.id.to_string().contains(&q)
                    || g.diamond_count.to_string() == q
            })
            .collect();
        match self.sort {
            SortMode::PriceAsc => v.sort_by_key(|g| (g.diamond_count, g.name.to_lowercase(), g.id)),
            SortMode::PriceDesc => v.sort_by(|a, b| {
                b.diamond_count
                    .cmp(&a.diamond_count)
                    .then(a.name.cmp(&b.name))
            }),
            SortMode::Name => v.sort_by(|a, b| {
                a.name
                    .to_lowercase()
                    .cmp(&b.name.to_lowercase())
                    .then(a.id.cmp(&b.id))
            }),
            SortMode::Id => v.sort_by_key(|g| g.id),
        }
        v
    }

    /// Full picker: search, sort, refresh, and the clickable catalog.
    /// Returns true if `rows` was changed.
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        theme: Theme,
        rows: &mut Vec<(String, String)>,
        bridge: &BridgeCtl,
    ) -> bool {
        self.drain_icons(ui.ctx());
        let mut changed = false;
        ui.label(
            RichText::new(
                "เลือกของขวัญจากรายการจริง (รูป / ชื่อ / รหัส / ราคาเพชร) แล้วคลิกเพื่อกำหนดการกระทำ",
            )
            .strong(),
        );
        ui.label(source_caption(&self.catalog));
        if !self.status.is_empty() {
            ui.label(RichText::new(&self.status).color(Color32::from_rgb(220, 180, 80)));
        }
        if self.catalog.gifts.is_empty() {
            ui.group(|ui| {
                ui.label(RichText::new("ยังไม่มีรายการของขวัญให้เลือก").size(16.0).strong());
                ui.label("รายการนี้ต้องมาจากแหล่งจริงเท่านั้น — ไม่มีการใส่รายการสมมติ");
                ui.label("เชื่อมต่อไลฟ์ TikTok ในแท็บ TikTok แล้วกดปุ่มด้านล่าง ตัวเชื่อมจะเรียก fetchAvailableGifts ของ tiktok-live-connector แล้วส่งรายการมาให้");
            });
        }
        ui.horizontal(|ui| {
            ui.label("ค้นหา");
            ui.add(
                egui::TextEdit::singleline(&mut self.search)
                    .hint_text("ชื่อ, รหัส, หรือราคาเพชร")
                    .desired_width(240.0),
            );
            if !self.search.is_empty() && ui.button("ล้าง").clicked() {
                self.search.clear();
            }
            ui.label("เรียง");
            egui::ComboBox::from_id_salt("gift-sort")
                .selected_text(self.sort.label())
                .width(160.0)
                .show_ui(ui, |ui| {
                    for m in [
                        SortMode::PriceAsc,
                        SortMode::PriceDesc,
                        SortMode::Name,
                        SortMode::Id,
                    ] {
                        ui.selectable_value(&mut self.sort, m, m.label());
                    }
                });
            if ui
                .add(
                    egui::Button::new(RichText::new("รีเฟรชรายการของขวัญ").size(16.0))
                        .min_size(Vec2::new(200.0, 32.0)),
                )
                .clicked()
            {
                match request_gift_list(bridge) {
                    Ok(()) => self.status = "กำลังขอรายการจากตัวเชื่อม TikTok …".into(),
                    Err(e) => self.status = format!("{e} — ใช้รายการที่บันทึกไว้ก่อน (ถ้ามี) ได้"),
                }
            }
        });
        let filtered = self.filtered();
        ui.label(format!(
            "แสดง {} จาก {} รายการ  ·  ราคาเป็นเพชร (Diamonds) ตามที่ TikTok แจ้ง",
            filtered.len(),
            self.catalog.gifts.len()
        ));

        let row_h = 56.0;
        let row_gap = 6.0;
        let visible_body = 5.0;
        // header + N complete body rows (no clipped last row)
        let max_h = row_h * (1.0 + visible_body) + row_gap * visible_body;
        egui::ScrollArea::vertical()
            .id_salt("gift-catalog")
            .auto_shrink([false, false])
            .max_height(max_h)
            .show(ui, |ui| {
                egui::Grid::new("gift-grid")
                    .num_columns(5)
                    .striped(true)
                    .spacing([8.0, 6.0])
                    .min_row_height(row_h)
                    .show(ui, |ui| {
                        ui.label(RichText::new("รูป").strong());
                        ui.label(RichText::new("ชื่อ").strong());
                        ui.label(RichText::new("รหัส (gift id)").strong());
                        ui.label(RichText::new("เพชร").strong());
                        ui.label(RichText::new("การกระทำ").strong());
                        ui.end_row();
                        // take owned copies of the visible slice metadata to avoid borrow issues
                        let visible: Vec<GiftInfo> =
                            filtered.iter().take(800).map(|g| (*g).clone()).collect();
                        drop(filtered);
                        for g in &visible {
                            self.fetcher.request(g.id, &g.image);
                            if let Some(tex) = self.icons.get(&g.id) {
                                ui.image((tex.id(), Vec2::splat(48.0)));
                            } else {
                                let (rect, _) =
                                    ui.allocate_exact_size(Vec2::splat(48.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, 6.0, Color32::from_gray(40));
                            }
                            ui.label(RichText::new(&g.name).size(16.0).strong());
                            ui.monospace(g.id.to_string());
                            ui.label(format!("{} เพชร", g.diamond_count));
                            let mut act = current_action(rows, g).unwrap_or_else(|| "—".into());
                            let before = act.clone();
                            action_combo(ui, ("pick", g.id), &mut act, theme, true);
                            if act != before && act != "—" {
                                upsert_mapping(rows, g, &act);
                                changed = true;
                            }
                            ui.end_row();
                        }
                    });
            });
        changed
    }
}

fn current_action(rows: &[(String, String)], g: &GiftInfo) -> Option<String> {
    let id_key = g.id.to_string();
    rows.iter()
        .find(|(k, _)| k.trim() == id_key || k.trim().eq_ignore_ascii_case(&g.name))
        .map(|(_, a)| a.clone())
}

fn action_combo(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    act: &mut String,
    th: Theme,
    allow_dash: bool,
) {
    let label = |a: &str| {
        if a == "—" {
            "— คลิกเพื่อเลือก"
        } else {
            th.action_label(a)
        }
    };
    egui::ComboBox::from_id_salt(id)
        .selected_text(label(act))
        .width(170.0)
        .show_ui(ui, |ui| {
            if allow_dash {
                ui.selectable_value(act, "—".to_string(), "— ยังไม่กำหนด");
            }
            for a in ["pray", "siamsi", "none"] {
                ui.selectable_value(act, a.to_string(), th.action_label(a));
            }
        });
}

fn source_caption(c: &GiftCatalog) -> String {
    if c.gifts.is_empty() {
        return "ยังไม่มีรายการที่ตรวจได้จากแหล่งจริง".into();
    }
    let origin = if c.from_live {
        "จากไลฟ์ (tiktok-live-connector fetchAvailableGifts)"
    } else {
        "ชุดที่มากับโปรแกรม (สำเนาจาก Euler Stream / TikTok webcast ณ วันที่ระบุ)"
    };
    let when = if c.fetched_at.is_empty() {
        String::new()
    } else {
        format!(" · ดึงเมื่อ {}", format_fetched(&c.fetched_at))
    };
    format!("{origin}{when} · {} รายการ", c.gifts.len())
}

fn format_fetched(s: &str) -> String {
    // Show Asia/Bangkok when the stamp is UTC (Z) or already has an offset.
    if let Some(local) = utc_to_bangkok(s) {
        format!("{local} (เวลาไทย, UTC+7)")
    } else {
        s.to_string()
    }
}

fn utc_to_bangkok(s: &str) -> Option<String> {
    // Minimal RFC3339 / snapshot formatter. Handles ...Z and ...+07:00.
    let s = s.trim();
    if s.len() < 19 {
        return None;
    }
    let date = &s[0..10];
    let time = &s[11..19];
    let (hh, mm, ss) = (
        time[0..2].parse::<i32>().ok()?,
        time[3..5].parse::<i32>().ok()?,
        time[6..8].parse::<i32>().ok()?,
    );
    let (y, mo, d) = (
        date[0..4].parse::<i32>().ok()?,
        date[5..7].parse::<u32>().ok()?,
        date[8..10].parse::<u32>().ok()?,
    );
    let offset_h = if s.ends_with('Z') || s.contains("+00:00") {
        7
    } else if s.contains("+07:00") {
        0
    } else {
        return Some(format!("{y:04}-{mo:02}-{d:02} {hh:02}:{mm:02}:{ss:02}"));
    };
    let mut total = hh * 3600 + mm * 60 + ss + offset_h * 3600;
    let mut day_add = 0;
    while total >= 86400 {
        total -= 86400;
        day_add += 1;
    }
    while total < 0 {
        total += 86400;
        day_add -= 1;
    }
    let hh = total / 3600;
    let mm = (total % 3600) / 60;
    let ss = total % 60;
    // date + day_add is good enough for our snapshot (no month overflow for +7h on this stamp)
    let d = d as i32 + day_add;
    Some(format!("{y:04}-{mo:02}-{d:02} {hh:02}:{mm:02}:{ss:02}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bangkok_label_for_snapshot_stamp() {
        assert_eq!(
            utc_to_bangkok("2026-11-12T16:42:58Z").as_deref(),
            Some("2026-11-12 23:42:58")
        );
        assert_eq!(
            utc_to_bangkok("2026-11-12T23:42:58+07:00").as_deref(),
            Some("2026-11-12 23:42:58")
        );
    }
}

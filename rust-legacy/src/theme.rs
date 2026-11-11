//! 视觉主题：配色、字体、圆角卡片与文本绘制辅助

use egui::{Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Stroke, StrokeKind};

pub const PAD: f32 = 20.0; // 给阴影预留的边距

pub const BG: Color32 = Color32::from_rgb(0x15, 0x17, 0x1C);
pub const BG_RAISE: Color32 = Color32::from_rgb(0x1B, 0x1E, 0x25);
pub const BG_SOFT: Color32 = Color32::from_rgb(0x20, 0x24, 0x2C);
pub const BG_HOVER: Color32 = Color32::from_rgb(0x26, 0x2B, 0x35);
pub const BORDER: Color32 = Color32::from_rgb(0x2C, 0x31, 0x3C);
pub const BORDER_SOFT: Color32 = Color32::from_rgb(0x23, 0x27, 0x30);

pub const TEXT: Color32 = Color32::from_rgb(0xE9, 0xEB, 0xF1);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x9E, 0xA7, 0xBB);
pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x71, 0x7C, 0x92);

pub const ACCENT: Color32 = Color32::from_rgb(0x6E, 0xA8, 0xFE);
pub const OK: Color32 = Color32::from_rgb(0x3D, 0xDC, 0x97);
pub const BAD: Color32 = Color32::from_rgb(0xFF, 0x7B, 0x7B);

pub fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// 加载系统中文字体，注入 egui
pub fn setup_fonts(ctx: &egui::Context) {
    let mut defs = egui::FontDefinitions::default();

    let candidates = [
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/msyh.ttf",
        "C:/Windows/Fonts/simhei.ttf",
        "C:/Windows/Fonts/simsun.ttc",
        "C:/Windows/Fonts/Deng.ttf",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut data = egui::FontData::from_owned(bytes);
            data.index = 0;
            defs.font_data.insert("cjk".to_owned(), std::sync::Arc::new(data));
            for fam in [FontFamily::Proportional, FontFamily::Monospace] {
                defs.families.entry(fam).or_default().insert(0, "cjk".to_owned());
            }
            break;
        }
    }

    ctx.set_fonts(defs);

    ctx.all_styles_mut(|style| {
        use egui::TextStyle::*;
        style.text_styles = [
            (Heading, FontId::new(19.0, FontFamily::Proportional)),
            (Body, FontId::new(14.0, FontFamily::Proportional)),
            (Monospace, FontId::new(13.0, FontFamily::Monospace)),
            (Button, FontId::new(14.0, FontFamily::Proportional)),
            (Small, FontId::new(12.0, FontFamily::Proportional)),
        ]
        .into_iter()
        .collect();
        style.visuals.window_fill = Color32::TRANSPARENT;
        style.visuals.panel_fill = Color32::TRANSPARENT;
        style.visuals.extreme_bg_color = BG_RAISE;
        style.visuals.override_text_color = Some(TEXT);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    });
}

pub fn font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn font_mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

pub fn text_mono(p: &Painter, pos: Pos2, anchor: Align2, s: &str, size: f32, color: Color32) -> Rect {
    p.text(pos, anchor, s, font_mono(size), color)
}

/// 卡片底 + 柔和外阴影
pub fn card(p: &Painter, rect: Rect, radius: u8, fill: Color32, border: Color32) {
    // 外阴影（多层叠加，模拟高斯模糊）
    let steps = 10;
    for i in (1..=steps).rev() {
        let t = i as f32 / steps as f32;
        let expand = 1.0 + t * 13.0;
        let a = (16.0 * (1.0 - t).powi(2)) as u8;
        if a == 0 {
            continue;
        }
        p.rect_filled(
            rect.expand(expand).translate(egui::vec2(0.0, expand * 0.35)),
            radius.saturating_add(expand as u8 + 2),
            Color32::from_black_alpha(a),
        );
    }
    p.rect(
        rect,
        radius,
        fill,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );
}

pub fn pill(p: &Painter, rect: Rect, color: Color32, label: &str, text_color: Color32) {
    p.rect_filled(rect, rect.height() as u8 / 2, alpha(color, 30));
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        font(11.0),
        text_color,
    );
}

pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

pub fn truncate(p: &Painter, s: &str, size: f32, max_w: f32) -> String {
    if max_w <= 6.0 {
        return String::new();
    }
    let f = font(size);
    let w = p
        .layout_no_wrap(s.to_owned(), f.clone(), Color32::WHITE)
        .rect
        .width();
    if w <= max_w {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut lo = 0usize;
    let mut hi = chars.len();
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        let mut probe: String = chars[..mid].iter().collect();
        probe.push('…');
        let pw = p
            .layout_no_wrap(probe, f.clone(), Color32::WHITE)
            .rect
            .width();
        if pw <= max_w {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let mut out: String = chars[..lo].iter().collect();
    out.push('…');
    out
}

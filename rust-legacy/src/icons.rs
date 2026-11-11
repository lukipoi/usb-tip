//! 类型图标绘制 + 托盘/窗口图标位图生成

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke};
use std::f32::consts::PI;

use crate::model::DeviceKind;

/// 在 rect 区域居中绘制某类型的线性图标（0..1 归一化设计空间）
pub fn kind_icon(p: &Painter, rect: Rect, kind: DeviceKind, color: Color32) {
    let s = rect.width().min(rect.height());
    let c = rect.center();
    let pt = |x: f32, y: f32| Pos2::new(c.x + (x - 0.5) * s, c.y + (y - 0.5) * s);
    let lw = (s * 0.085).max(1.3);
    let st = Stroke::new(lw, color);

    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], st);
    };
    let dot = |x: f32, y: f32, r: f32| {
        p.circle_filled(pt(x, y), r * s, color);
    };
    let ring = |x: f32, y: f32, r: f32, w: f32| {
        p.circle_stroke(pt(x, y), r * s, Stroke::new(w, color));
    };
    let boxr = |x0: f32, y0: f32, x1: f32, y1: f32, r: f32, w: f32| {
        p.rect_stroke(
            Rect::from_min_max(pt(x0, y0), pt(x1, y1)),
            (r * s) as u8,
            Stroke::new(w, color),
            egui::StrokeKind::Middle,
        );
    };
    let boxf = |x0: f32, y0: f32, x1: f32, y1: f32, r: f32| {
        p.rect_filled(
            Rect::from_min_max(pt(x0, y0), pt(x1, y1)),
            (r * s) as u8,
            color,
        );
    };
    let arc = |cx: f32, cy: f32, r: f32, a0: f32, a1: f32, w: f32| {
        let steps = 22;
        let mut pts = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let a = a0 + (a1 - a0) * t;
            pts.push(Pos2::new(
                c.x + (cx - 0.5 + r * a.cos()) * s,
                c.y + (cy - 0.5 + r * a.sin()) * s,
            ));
        }
        p.add(Shape::line(pts, Stroke::new(w, color)));
    };

    match kind {
        DeviceKind::Serial => {
            // 双向收发箭头
            line((0.12, 0.36), (0.80, 0.36));
            p.add(Shape::convex_polygon(
                vec![pt(0.90, 0.36), pt(0.74, 0.26), pt(0.74, 0.46)],
                color,
                Stroke::NONE,
            ));
            line((0.88, 0.64), (0.20, 0.64));
            p.add(Shape::convex_polygon(
                vec![pt(0.10, 0.64), pt(0.26, 0.54), pt(0.26, 0.74)],
                color,
                Stroke::NONE,
            ));
        }
        DeviceKind::Storage => {
            // U 盘
            boxr(0.33, 0.32, 0.67, 0.86, 0.10, lw);
            boxf(0.42, 0.18, 0.58, 0.32, 0.03);
            line((0.40, 0.50), (0.60, 0.50));
            line((0.40, 0.64), (0.55, 0.64));
        }
        DeviceKind::Hid => {
            // 键盘
            boxr(0.13, 0.31, 0.87, 0.69, 0.07, lw);
            let kw = 0.055;
            for i in 0..4 {
                let x = 0.20 + i as f32 * 0.155;
                boxf(x, 0.39, x + kw, 0.45, 0.01);
            }
            boxf(0.30, 0.53, 0.70, 0.60, 0.01);
        }
        DeviceKind::Network => {
            arc(0.5, 0.82, 0.40, PI * 1.22, PI * 1.78, lw);
            arc(0.5, 0.82, 0.27, PI * 1.25, PI * 1.75, lw);
            arc(0.5, 0.82, 0.145, PI * 1.30, PI * 1.70, lw);
            dot(0.5, 0.82, 0.055);
        }
        DeviceKind::Audio => {
            // 喇叭
            p.add(Shape::convex_polygon(
                vec![pt(0.18, 0.40), pt(0.36, 0.40), pt(0.54, 0.20), pt(0.54, 0.80), pt(0.36, 0.60), pt(0.18, 0.60)],
                color,
                Stroke::NONE,
            ));
            arc(0.54, 0.50, 0.20, -PI * 0.32, PI * 0.32, lw * 0.85);
            arc(0.54, 0.50, 0.33, -PI * 0.30, PI * 0.30, lw * 0.85);
        }
        DeviceKind::Camera => {
            boxr(0.14, 0.30, 0.86, 0.80, 0.10, lw);
            ring(0.50, 0.55, 0.16, lw * 0.9);
            boxf(0.36, 0.22, 0.52, 0.30, 0.02);
        }
        DeviceKind::Printer => {
            boxr(0.20, 0.44, 0.80, 0.74, 0.08, lw);
            boxr(0.32, 0.18, 0.68, 0.44, 0.03, lw);
            boxf(0.30, 0.56, 0.70, 0.60, 0.0);
        }
        DeviceKind::Bluetooth => {
            line((0.50, 0.14), (0.76, 0.34));
            line((0.76, 0.34), (0.30, 0.70));
            line((0.50, 0.86), (0.76, 0.66));
            line((0.76, 0.66), (0.30, 0.30));
            line((0.50, 0.14), (0.50, 0.86));
        }
        DeviceKind::SmartCard => {
            boxr(0.18, 0.28, 0.82, 0.72, 0.07, lw);
            boxf(0.30, 0.42, 0.52, 0.58, 0.03);
            line((0.60, 0.46), (0.70, 0.46));
            line((0.60, 0.55), (0.70, 0.55));
        }
        DeviceKind::Android => {
            boxr(0.26, 0.34, 0.74, 0.82, 0.10, lw);
            line((0.36, 0.24), (0.44, 0.33));
            line((0.64, 0.24), (0.56, 0.33));
            dot(0.38, 0.48, 0.038);
            dot(0.62, 0.48, 0.038);
        }
        DeviceKind::Hub => {
            // 分线器：主干 + 三路分支
            boxr(0.34, 0.50, 0.66, 0.78, 0.05, lw);
            line((0.50, 0.50), (0.50, 0.34));
            line((0.22, 0.34), (0.78, 0.34));
            line((0.22, 0.34), (0.22, 0.24));
            line((0.78, 0.34), (0.78, 0.24));
            dot(0.22, 0.21, 0.048);
            dot(0.78, 0.21, 0.048);
            dot(0.50, 0.30, 0.048);
        }
        DeviceKind::Modem => {
            let steps = 26;
            let mut pts = Vec::with_capacity(steps + 1);
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let x = 0.10 + t * 0.80;
                let y = 0.50 + (t * PI * 2.4).sin() * 0.20;
                pts.push(pt(x, y));
            }
            p.add(Shape::line(pts, st));
        }
        DeviceKind::Composite => {
            boxf(0.18, 0.18, 0.45, 0.45, 0.05);
            boxf(0.55, 0.18, 0.82, 0.45, 0.05);
            boxf(0.18, 0.55, 0.45, 0.82, 0.05);
            boxf(0.55, 0.55, 0.82, 0.82, 0.05);
        }
        DeviceKind::Generic => {
            ring(0.5, 0.5, 0.32, lw);
            dot(0.5, 0.5, 0.08);
        }
    }
}

// ---------------- 位图图标（托盘 / 窗口） ----------------

fn sd_round_rect(px: f32, py: f32, cx: f32, cy: f32, hw: f32, hh: f32, r: f32) -> f32 {
    let qx = (px - cx).abs() - (hw - r);
    let qy = (py - cy).abs() - (hh - r);
    let ax = qx.max(0.0);
    let ay = qy.max(0.0);
    (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - r
}

fn sd_circle(px: f32, py: f32, cx: f32, cy: f32, r: f32) -> f32 {
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt() - r
}

fn sd_segment(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let pax = px - ax;
    let pay = py - ay;
    let bax = bx - ax;
    let bay = by - ay;
    let h = ((pax * bax + pay * bay) / (bax * bax + bay * bay).max(1e-6)).clamp(0.0, 1.0);
    ((pax - bax * h).powi(2) + (pay - bay * h).powi(2)).sqrt()
}

/// 生成圆形渐变底 + 白色 USB 符号的 RGBA 位图（直线 sRGB，premultiplied 不需要）
pub fn icon_rgba(size: u32) -> Vec<u8> {
    let s = size as f32;
    let mut out = vec![0u8; (size * size * 4) as usize];
    let samples = 4;
    let mut seed = 0x9E37_79B9u32;
    let mut rand = || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed as f32) / (u32::MAX as f32)
    };

    for y in 0..size {
        for x in 0..size {
            let mut cover_bg = 0.0f32;
            let mut cover_fg = 0.0f32;
            for _ in 0..samples {
                let px = x as f32 + rand();
                let py = y as f32 + rand();

                // 底：圆角方块
                let d_bg = sd_round_rect(px, py, s * 0.5, s * 0.5, s * 0.5, s * 0.5, s * 0.29);
                if d_bg < 0.0 {
                    cover_bg += 1.0;
                }

                // USB 三叉戟（白色）
                let mut inside = false;
                // 竖杆
                inside |= sd_segment(px, py, s * 0.5, s * 0.34, s * 0.5, s * 0.74) < s * 0.045;
                // 顶球
                inside |= sd_circle(px, py, s * 0.5, s * 0.29, s * 0.075) < 0.0;
                // 左臂 + 方块
                inside |= sd_segment(px, py, s * 0.5, s * 0.56, s * 0.31, s * 0.45) < s * 0.042;
                inside |= sd_round_rect(px, py, s * 0.30, s * 0.44, s * 0.075, s * 0.075, s * 0.02) < 0.0;
                // 右臂 + 圆点
                inside |= sd_segment(px, py, s * 0.5, s * 0.65, s * 0.70, s * 0.54) < s * 0.042;
                inside |= sd_circle(px, py, s * 0.71, s * 0.53, s * 0.072) < 0.0;

                if inside && d_bg < 0.0 {
                    cover_fg += 1.0;
                }
            }
            let bg = cover_bg / samples as f32;
            let fg = cover_fg / samples as f32;

            // 渐变：左上 #6EA8FE -> 右下 #A78BFA
            let t = (x as f32 / s + y as f32 / s) * 0.5;
            let r = 0x6Eu8 as f32 * (1.0 - t) + 0xA7 as f32 * t;
            let g = 0xA8 as f32 * (1.0 - t) + 0x8B as f32 * t;
            let b = 0xFE as f32 * (1.0 - t) + 0xFA as f32 * t;

            let (mut cr, mut cg, mut cb) = (r, g, b);
            if fg > 0.0 {
                let k = fg / bg.max(0.001);
                let k = k.min(1.0);
                cr = cr * (1.0 - k) + 255.0 * k;
                cg = cg * (1.0 - k) + 255.0 * k;
                cb = cb * (1.0 - k) + 255.0 * k;
            }

            let i = ((y * size + x) * 4) as usize;
            out[i] = cr as u8;
            out[i + 1] = cg as u8;
            out[i + 2] = cb as u8;
            out[i + 3] = (bg * 255.0) as u8;
        }
    }
    out
}

/// 画一个"标志"字样（标题栏用）：小圆角块 + 白点
pub fn logo(p: &Painter, rect: Rect) {
    let r = rect.width().min(rect.height());
    p.rect_filled(rect, (r * 0.30) as u8, Color32::from_rgb(0x6E, 0xA8, 0xFE));
    let c = rect.center();
    p.circle_filled(Pos2::new(c.x, c.y - r * 0.10), r * 0.13, Color32::WHITE);
    p.line_segment(
        [
            Pos2::new(c.x, c.y - r * 0.10),
            Pos2::new(c.x, c.y + r * 0.22),
        ],
        Stroke::new(r * 0.075, Color32::WHITE),
    );
    p.line_segment(
        [
            Pos2::new(c.x, c.y + r * 0.06),
            Pos2::new(c.x - r * 0.16, c.y - r * 0.02),
        ],
        Stroke::new(r * 0.06, Color32::WHITE),
    );
    p.line_segment(
        [
            Pos2::new(c.x, c.y + r * 0.14),
            Pos2::new(c.x + r * 0.16, c.y + r * 0.06),
        ],
        Stroke::new(r * 0.06, Color32::WHITE),
    );
}


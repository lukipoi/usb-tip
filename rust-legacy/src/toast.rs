//! 右下角弹窗（Toast）视图

use egui::{Align2, Color32, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui};

use crate::icons;
use crate::model::DeviceInfo;
use crate::theme::*;

pub struct Toast {
    pub key: String,
    pub dev: DeviceInfo,
    pub arrival: bool,
    pub born: f64,
    pub ttl: f64,
    pub id: u64,
    pub hover_start: Option<f64>,
}

impl Toast {
    pub fn progress(&self, now: f64) -> f32 {
        let e = (now - self.born) / self.ttl;
        (1.0 - e as f32).clamp(0.0, 1.0)
    }
}

pub fn draw(ui: &mut Ui, rect: Rect, t: &Toast, progress: f32, hovered: bool) -> Response {
    let resp = ui.allocate_rect(rect, Sense::click());
    let p = ui.painter();

    let fill = if hovered { BG_HOVER } else { BG_RAISE };
    card(p, rect, 16u8, fill, if hovered { Color32::from_rgb(0x3B, 0x42, 0x50) } else { BORDER });

    // 左侧类型图标块
    let side = 46.0;
    let icon_rect = Rect::from_min_size(
        Pos2::new(rect.left() + 16.0, rect.center().y - side * 0.5),
        egui::vec2(side, side),
    );
    let kc = t.dev.kind.color();
    p.rect_filled(icon_rect, 14u8, alpha(kc, 30));
    p.rect_stroke(
        icon_rect,
        14u8,
        Stroke::new(1.0, alpha(kc, 55)),
        StrokeKind::Inside,
    );
    icons::kind_icon(p, icon_rect.shrink(12.0), t.dev.kind, kc);

    // 状态标签（右上）
    let (st_text, st_color) = if t.arrival {
        ("已插入", OK)
    } else {
        ("已拔出", BAD)
    };
    let st_pos = Pos2::new(rect.right() - 16.0, rect.top() + 22.0);
    p.circle_filled(
        Pos2::new(st_pos.x - measure(p, st_text, 11.0) - 9.0, st_pos.y),
        3.0,
        st_color,
    );
    p.text(st_pos, Align2::RIGHT_CENTER, st_text, font(11.0), st_color);

    // 设备名
    let text_left = icon_rect.right() + 14.0;
    let name_w = rect.right() - 16.0 - text_left - 56.0;
    let name = truncate(p, &t.dev.name, 15.5, name_w);
    p.text(
        Pos2::new(text_left, rect.top() + 26.0),
        Align2::LEFT_CENTER,
        &name,
        font(15.5),
        TEXT,
    );

    // 第二行：类型徽章 + COM + VID/PID
    let mut x = text_left;
    let by = rect.top() + 52.0;
    let mut label = t.dev.kind.label().to_string();
    if !t.dev.com_port.is_empty() {
        label.push_str(" · ");
        label.push_str(&t.dev.com_port);
    }
    let bw = measure(p, &label, 11.0) + 16.0;
    let badge = Rect::from_min_size(Pos2::new(x, by - 9.0), egui::vec2(bw, 18.0));
    pill(p, badge, kc, &label, kc);
    x = badge.right() + 10.0;

    let vp = if t.dev.vid == "----" {
        "无 VID/PID".to_string()
    } else {
        format!("VID_{}  PID_{}", t.dev.vid, t.dev.pid)
    };
    p.text(
        Pos2::new(x, by),
        Align2::LEFT_CENTER,
        &vp,
        font_mono(11.5),
        TEXT_DIM,
    );

    // 底部倒计时进度条
    let bar_x0 = rect.left() + 14.0;
    let bar_x1 = rect.right() - 14.0;
    let bar_w = (bar_x1 - bar_x0) * progress;
    if bar_w > 1.0 {
        p.rect_filled(
            Rect::from_min_size(
                Pos2::new(bar_x0, rect.bottom() - 6.0),
                egui::vec2(bar_w, 2.0),
            ),
            1u8,
            alpha(kc, 150),
        );
    }

    resp
}

fn measure(p: &Painter, s: &str, size: f32) -> f32 {
    p.layout_no_wrap(s.to_owned(), font(size), Color32::WHITE)
        .rect
        .width()
}

//! 主面板：设备列表 + 详情

use egui::{Align2, Color32, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, Ui};

use crate::icons;
use crate::model::{DeviceInfo, DeviceKind};
use crate::theme::*;
use crate::App;

pub const PANEL_W: f32 = 940.0;
pub const PANEL_H: f32 = 580.0;

const TITLE_H: f32 = 46.0;
const LEFT_W: f32 = 316.0;
const ROW_H: f32 = 66.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Online,
    Offline,
}

impl Filter {
    fn label(self) -> &'static str {
        match self {
            Self::All => "全部",
            Self::Online => "在线",
            Self::Offline => "已断开",
        }
    }
}

pub fn draw(app: &mut App, ui: &mut Ui) {
    let rect = ui.max_rect().shrink(crate::theme::PAD);
    let p = ui.painter().clone();
    card(&p, rect, 18, BG, BORDER);

    title_bar(app, ui, Rect::from_min_size(rect.min, egui::vec2(rect.width(), TITLE_H)));

    let body = Rect::from_min_max(
        Pos2::new(rect.left(), rect.top() + TITLE_H),
        Pos2::new(rect.right(), rect.bottom()),
    );
    let left = Rect::from_min_max(
        body.min,
        Pos2::new(body.left() + LEFT_W, body.bottom()),
    );
    let right = Rect::from_min_max(
        Pos2::new(body.left() + LEFT_W, body.top()),
        body.max,
    );

    p.line_segment(
        [
            Pos2::new(right.left(), body.top() + 6.0),
            Pos2::new(right.left(), body.bottom() - 6.0),
        ],
        Stroke::new(1.0, BORDER_SOFT),
    );

    left_pane(app, ui, left);
    right_pane(app, ui, right);
}

fn title_bar(app: &mut App, ui: &mut Ui, rect: Rect) {
    let p = ui.painter().clone();
    let resp = ui.interact(rect, ui.scope_id().with("titlebar"), Sense::click_and_drag());
    if resp.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if resp.double_clicked() {
        app.want_minimize = true;
    }

    // logo
    let lr = Rect::from_min_size(
        Pos2::new(rect.left() + 18.0, rect.center().y - 10.0),
        egui::vec2(20.0, 20.0),
    );
    icons::logo(&p, lr);

    p.text(
        Pos2::new(lr.right() + 10.0, rect.center().y),
        Align2::LEFT_CENTER,
        "USB TIP",
        font(12.5),
        TEXT_DIM,
    );
    let sep_x = lr.right() + 66.0;
    p.line_segment(
        [
            Pos2::new(sep_x, rect.center().y - 7.0),
            Pos2::new(sep_x, rect.center().y + 7.0),
        ],
        Stroke::new(1.0, BORDER),
    );
    p.text(
        Pos2::new(sep_x + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        "USB 插拔监视",
        font(13.0),
        TEXT,
    );

    // 右上统计
    let online = app.devices.iter().filter(|d| d.present).count();
    let txt = format!("在线 {} · 记录 {}", online, app.devices.len());
    p.text(
        Pos2::new(rect.right() - 92.0, rect.center().y),
        Align2::RIGHT_CENTER,
        &txt,
        font(11.5),
        TEXT_FAINT,
    );

    // 关闭按钮
    let cb = Rect::from_center_size(
        Pos2::new(rect.right() - 28.0, rect.center().y),
        egui::vec2(28.0, 28.0),
    );
    let cresp = ui.interact(cb, ui.scope_id().with("btn_close"), Sense::click());
    if cresp.hovered() {
        ui.painter().rect_filled(cb, 8u8, Color32::from_rgb(0x5A, 0x25, 0x2A));
    }
    let c = cb.center();
    let cc = if cresp.hovered() { Color32::WHITE } else { TEXT_DIM };
    p.line_segment(
        [Pos2::new(c.x - 5.0, c.y - 5.0), Pos2::new(c.x + 5.0, c.y + 5.0)],
        Stroke::new(1.4, cc),
    );
    p.line_segment(
        [Pos2::new(c.x + 5.0, c.y - 5.0), Pos2::new(c.x - 5.0, c.y + 5.0)],
        Stroke::new(1.4, cc),
    );
    if cresp.clicked() {
        app.want_close = true;
    }

    // 隐藏按钮
    let hb = Rect::from_center_size(
        Pos2::new(rect.right() - 60.0, rect.center().y),
        egui::vec2(28.0, 28.0),
    );
    let hresp = ui.interact(hb, ui.scope_id().with("btn_min"), Sense::click());
    if hresp.hovered() {
        ui.painter().rect_filled(hb, 8u8, BG_HOVER);
    }
    let hc = hb.center();
    p.line_segment(
        [
            Pos2::new(hc.x - 5.0, hc.y + 1.0),
            Pos2::new(hc.x + 5.0, hc.y + 1.0),
        ],
        Stroke::new(1.4, if hresp.hovered() { Color32::WHITE } else { TEXT_DIM }),
    );
    if hresp.clicked() {
        app.want_minimize = true;
    }

    p.line_segment(
        [
            Pos2::new(rect.left() + 1.0, rect.bottom()),
            Pos2::new(rect.right() - 1.0, rect.bottom()),
        ],
        Stroke::new(1.0, BORDER_SOFT),
    );
}

fn left_pane(app: &mut App, ui: &mut Ui, rect: Rect) {
    let p = ui.painter().clone();

    // ---- 搜索框 ----
    let sb = Rect::from_min_size(
        Pos2::new(rect.left() + 16.0, rect.top() + 14.0),
        egui::vec2(rect.width() - 32.0, 36.0),
    );
    p.rect_filled(sb, 10u8, BG_RAISE);
    p.rect_stroke(sb, 10u8, Stroke::new(1.0, BORDER_SOFT), StrokeKind::Inside);
    // 放大镜
    let mc = Pos2::new(sb.left() + 18.0, sb.center().y);
    p.circle_stroke(mc, 5.2, Stroke::new(1.4, TEXT_FAINT));
    p.line_segment(
        [
            Pos2::new(mc.x + 3.8, mc.y + 3.8),
            Pos2::new(mc.x + 7.0, mc.y + 7.0),
        ],
        Stroke::new(1.4, TEXT_FAINT),
    );

    let mut search = std::mem::take(&mut app.search);
    let te_rect = Rect::from_min_max(
        Pos2::new(sb.left() + 32.0, sb.top() + 1.0),
        Pos2::new(sb.right() - 8.0, sb.bottom() - 1.0),
    );
    ui.put(
        te_rect,
        egui::TextEdit::singleline(&mut search)
            .frame(egui::Frame::NONE)
            .hint_text(egui::RichText::new("搜索名称 / VID / PID").size(12.5).color(TEXT_FAINT))
            .font(font(12.5)),
    );
    app.search = search;

    // ---- 过滤 chips ----
    let mut x = rect.left() + 16.0;
    let cy = sb.bottom() + 22.0;
    for f in [Filter::All, Filter::Online, Filter::Offline] {
        let w = measure(&p, f.label(), 11.5) + 22.0;
        let cr = Rect::from_min_size(Pos2::new(x, cy - 12.0), egui::vec2(w, 24.0));
        let resp = ui.interact(cr, ui.scope_id().with(("filter", f.label())), Sense::click());
        let on = app.filter == f;
        let bg = if on {
            alpha(ACCENT, 34)
        } else if resp.hovered() {
            BG_HOVER
        } else {
            BG_RAISE
        };
        p.rect(cr, 8u8, bg, Stroke::new(1.0, if on { alpha(ACCENT, 90) } else { BORDER_SOFT }), StrokeKind::Inside);
        p.text(
            cr.center(),
            Align2::CENTER_CENTER,
            f.label(),
            font(11.5),
            if on { ACCENT } else { TEXT_DIM },
        );
        if resp.clicked() {
            app.filter = f;
        }
        x = cr.right() + 8.0;
    }

    // ---- 列表 ----
    let list = Rect::from_min_max(
        Pos2::new(rect.left(), cy + 22.0),
        Pos2::new(rect.right(), rect.bottom() - 8.0),
    );

    // 过滤后的索引
    let q = app.search.trim().to_lowercase();
    let mut visible: Vec<usize> = Vec::new();
    let mut order: Vec<usize> = (0..app.devices.len()).collect();
    order.sort_by(|&a, &b| {
        let da = &app.devices[a];
        let db = &app.devices[b];
        db.present
            .cmp(&da.present)
            .then((db.vid != "----").cmp(&(da.vid != "----")))
            .then(db.last_change_ms.cmp(&da.last_change_ms))
    });
    for i in order {
        let d = &app.devices[i];
        if app.filter == Filter::Online && !d.present {
            continue;
        }
        if app.filter == Filter::Offline && d.present {
            continue;
        }
        if !q.is_empty() {
            let hay = format!(
                "{} {} {} {} {}",
                d.name.to_lowercase(),
                d.vid.to_lowercase(),
                d.pid.to_lowercase(),
                d.kind.label().to_lowercase(),
                d.com_port.to_lowercase()
            );
            if !hay.contains(&q) {
                continue;
            }
        }
        visible.push(i);
    }

    let content_h = visible.len() as f32 * ROW_H;
    if list.contains(ui.ctx().pointer_latest_pos().unwrap_or(Pos2::ZERO)) {
        let dy = ui.input(|i| i.smooth_scroll_delta.y);
        if dy.abs() > 0.1 {
            app.scroll = (app.scroll - dy).clamp(0.0, (content_h - list.height()).max(0.0));
        }
    }
    let max_scroll = (content_h - list.height()).max(0.0);
    app.scroll = app.scroll.clamp(0.0, max_scroll);

    if visible.is_empty() {
        p.text(
            list.center(),
            Align2::CENTER_CENTER,
            if app.devices.is_empty() { "还没有记录到 USB 设备" } else { "没有匹配的设备" },
            font(12.0),
            TEXT_FAINT,
        );
        return;
    }

    let clip = ui.painter().with_clip_rect(list);
    for (n, &idx) in visible.iter().enumerate() {
        let y = list.top() + n as f32 * ROW_H - app.scroll;
        let row = Rect::from_min_size(Pos2::new(list.left() + 10.0, y), egui::vec2(list.width() - 20.0, ROW_H - 8.0));
        if row.bottom() < list.top() - 10.0 || row.top() > list.bottom() + 10.0 {
            continue;
        }
        let d = &app.devices[idx];
        let selected = app.selected.as_deref() == Some(d.key.as_str());
        let resp = ui.interact(
            row.intersect(list),
            ui.scope_id().with(("row", idx)),
            Sense::click(),
        );

        let bg = if selected {
            alpha(ACCENT, 26)
        } else if resp.hovered() {
            BG_RAISE
        } else {
            Color32::TRANSPARENT
        };
        if bg != Color32::TRANSPARENT {
            clip.rect_filled(row, 12u8, bg);
        }
        if selected {
            clip.rect_stroke(row, 12u8, Stroke::new(1.0, alpha(ACCENT, 85)), StrokeKind::Inside);
        }

        // 图标块
        let kc = d.kind.color();
        let ir = Rect::from_min_size(
            Pos2::new(row.left() + 10.0, row.center().y - 16.0),
            egui::vec2(32.0, 32.0),
        );
        clip.rect_filled(ir, 10u8, alpha(kc, if d.present { 30 } else { 16 }));
        icons::kind_icon(&clip, ir.shrink(8.0), d.kind, if d.present { kc } else { alpha(kc, 110) });

        // 文本
        let tx = ir.right() + 11.0;
        let name_w = row.right() - tx - 24.0;
        let name = truncate(&clip, &d.name, 13.0, name_w);
        clip.text(
            Pos2::new(tx, row.center().y - 10.0),
            Align2::LEFT_CENTER,
            &name,
            font(13.0),
            if d.present { TEXT } else { TEXT_DIM },
        );
        let vp = if d.vid == "----" {
            "—".to_string()
        } else {
            format!("{}:{}", d.vid, d.pid)
        };
        let sub = if d.com_port.is_empty() {
            format!("{} · {}", d.kind.label(), vp)
        } else {
            format!("{} · {} · {}", d.kind.label(), vp, d.com_port)
        };
        let sub = truncate(&clip, &sub, 10.5, name_w);
        clip.text(
            Pos2::new(tx, row.center().y + 11.0),
            Align2::LEFT_CENTER,
            &sub,
            font(10.5),
            TEXT_FAINT,
        );

        // 状态点
        let dotc = if d.present { OK } else { alpha(TEXT_FAINT, 130) };
        clip.circle_filled(
            Pos2::new(row.right() - 12.0, row.center().y),
            3.4,
            dotc,
        );

        if resp.clicked() {
            app.selected = Some(d.key.clone());
        }
    }
}

fn right_pane(app: &mut App, ui: &mut Ui, rect: Rect) {
    let p = ui.painter().clone();

    let sel = app
        .selected
        .as_ref()
        .and_then(|k| app.devices.iter().find(|d| &d.key == k))
        .cloned();

    let Some(d) = sel else {
        p.text(
            Pos2::new(rect.center().x, rect.center().y - 8.0),
            Align2::CENTER_CENTER,
            "从左侧选择一个设备",
            font(13.0),
            TEXT_FAINT,
        );
        p.text(
            Pos2::new(rect.center().x, rect.center().y + 16.0),
            Align2::CENTER_CENTER,
            "可以看到 VID / PID、类型、厂商等完整明细",
            font(11.0),
            alpha(TEXT_FAINT, 150),
        );
        return;
    };

    let pad = 24.0;
    let inner = rect.shrink2(egui::vec2(pad, 20.0));

    // ---- 头部 ----
    let head_h = 76.0;
    let head = Rect::from_min_size(inner.min, egui::vec2(inner.width(), head_h));
    let kc = d.kind.color();
    let ir = Rect::from_min_size(
        Pos2::new(head.left(), head.center().y - 27.0),
        egui::vec2(54.0, 54.0),
    );
    p.rect_filled(ir, 16u8, alpha(kc, 28));
    p.rect_stroke(ir, 16u8, Stroke::new(1.0, alpha(kc, 60)), StrokeKind::Inside);
    icons::kind_icon(&p, ir.shrink(14.0), d.kind, kc);

    let tx = ir.right() + 16.0;
    let name = truncate(&p, &d.name, 19.0, head.right() - tx - 100.0);
    p.text(
        Pos2::new(tx, head.center().y - 14.0),
        Align2::LEFT_CENTER,
        &name,
        font(19.0),
        TEXT,
    );

    // 类型 + 状态
    let mut label = d.kind.label().to_string();
    if !d.com_port.is_empty() {
        label.push_str(" · ");
        label.push_str(&d.com_port);
    }
    let bw = measure(&p, &label, 11.0) + 18.0;
    let badge = Rect::from_min_size(Pos2::new(tx, head.center().y + 6.0), egui::vec2(bw, 20.0));
    pill(&p, badge, kc, &label, kc);

    let (st, sc) = if d.present { ("已连接", OK) } else { ("已断开", BAD) };
    let stx = badge.right() + 10.0;
    p.circle_filled(Pos2::new(stx + 3.0, badge.center().y), 3.2, sc);
    p.text(
        Pos2::new(stx + 11.0, badge.center().y),
        Align2::LEFT_CENTER,
        st,
        font(11.0),
        sc,
    );

    // 复制按钮
    let cb = Rect::from_min_size(
        Pos2::new(head.right() - 84.0, head.center().y - 14.0),
        egui::vec2(84.0, 28.0),
    );
    let cresp = ui.interact(cb, ui.scope_id().with("copy_btn"), Sense::click());
    p.rect(
        cb,
        8u8,
        if cresp.hovered() { BG_HOVER } else { BG_RAISE },
        Stroke::new(1.0, BORDER_SOFT),
        StrokeKind::Inside,
    );
    p.text(
        cb.center(),
        Align2::CENTER_CENTER,
        if app.copied_at > 0.0 && app.now - app.copied_at < 1.2 { "已复制" } else { "复制信息" },
        font(11.0),
        if app.copied_at > 0.0 && app.now - app.copied_at < 1.2 { OK } else { TEXT_DIM },
    );
    if cresp.clicked() {
        ui.ctx().copy_text(d.dump());
        app.copied_at = app.now;
    }

    p.line_segment(
        [
            Pos2::new(inner.left(), head.bottom() + 8.0),
            Pos2::new(inner.right(), head.bottom() + 8.0),
        ],
        Stroke::new(1.0, BORDER_SOFT),
    );

    // ---- VID / PID 大方块 ----
    let mut y = head.bottom() + 24.0;
    let bw2 = 136.0;
    let bh2 = 58.0;
    for (i, (k, v)) in [("VID", d.vid.as_str()), ("PID", d.pid.as_str())].iter().enumerate() {
        let br = Rect::from_min_size(
            Pos2::new(inner.left() + i as f32 * (bw2 + 12.0), y),
            egui::vec2(bw2, bh2),
        );
        let resp = ui.interact(br, ui.scope_id().with(("vpbox", i)), Sense::click());
        p.rect(
            br,
            12u8,
            if resp.hovered() { BG_SOFT } else { BG_RAISE },
            Stroke::new(1.0, if resp.hovered() { alpha(ACCENT, 70) } else { BORDER }),
            StrokeKind::Inside,
        );
        p.text(
            Pos2::new(br.left() + 14.0, br.top() + 14.0),
            Align2::LEFT_CENTER,
            k,
            font(10.5),
            TEXT_FAINT,
        );
        text_mono(
            &p,
            Pos2::new(br.left() + 13.0, br.top() + 38.0),
            Align2::LEFT_CENTER,
            v,
            23.0,
            ACCENT,
        );
        if resp.clicked() {
            ui.ctx().copy_text(format!("0x{}", v));
            app.copied_at = app.now;
        }
    }

    // ---- 属性表 ----
    y += bh2 + 22.0;
    let rows: Vec<(&str, String)> = vec![
        ("设备类型", d.kind.label().to_string()),
        ("设备类", if d.class.is_empty() { "—".into() } else { d.class.clone() }),
        ("制造商", if d.mfg.is_empty() { "—".into() } else { d.mfg.clone() }),
        ("服务", if d.service.is_empty() { "—".into() } else { d.service.clone() }),
        ("序列号", if d.serial.is_empty() { "—".into() } else { d.serial.clone() }),
        ("端口位置", if d.location.is_empty() { "—".into() } else { d.location.clone() }),
        ("描述", if d.desc.is_empty() { "—".into() } else { d.desc.clone() }),
        ("实例 ID", d.instance_id.clone()),
        ("插拔记录", format!("{} 次 · {}", d.plug_count, humanize(d.last_change_ms))),
    ];

    let row_h = 30.0;
    let key_w = 78.0;
    for (i, (k, v)) in rows.iter().enumerate() {
        let ry = y + i as f32 * row_h;
        if ry + row_h > inner.bottom() {
            break;
        }
        if i > 0 {
            p.line_segment(
                [
                    Pos2::new(inner.left() - 4.0, ry),
                    Pos2::new(inner.right() + 4.0, ry),
                ],
                Stroke::new(1.0, alpha(BORDER_SOFT, 200)),
            );
        }
        p.text(
            Pos2::new(inner.left(), ry + row_h * 0.5),
            Align2::LEFT_CENTER,
            k,
            font(11.5),
            TEXT_FAINT,
        );
        let vw = inner.width() - key_w - 8.0;
        let vtxt = truncate(&p, v, 12.0, vw);
        p.text(
            Pos2::new(inner.left() + key_w, ry + row_h * 0.5),
            Align2::LEFT_CENTER,
            &vtxt,
            if *k == "实例 ID" { font_mono(11.0) } else { font(12.0) },
            TEXT_DIM,
        );
    }
}

/// 把毫秒时间戳变成"3 分钟前"这种相对描述
fn humanize(ts_ms: u64) -> String {
    if ts_ms == 0 {
        return "启动时已连接".to_string();
    }
    let now = crate::usb::now_ms();
    let secs = now.saturating_sub(ts_ms) / 1000;
    if secs < 60 {
        format!("{} 秒前", secs)
    } else if secs < 3600 {
        format!("{} 分钟前", secs / 60)
    } else {
        format!("{} 小时前", secs / 3600)
    }
}

fn measure(p: &egui::Painter, s: &str, size: f32) -> f32 {
    p.layout_no_wrap(s.to_owned(), font(size), Color32::WHITE)
        .rect
        .width()
}

#[allow(dead_code)]
fn kind_of(d: &DeviceInfo) -> DeviceKind {
    d.kind
}

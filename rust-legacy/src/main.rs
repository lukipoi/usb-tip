//! USB TIP —— 插拔 USB 设备时右下角弹卡片，告诉你"这是个啥"
//!
//! 纯用户态实现：只读枚举设备信息，不装驱动、不要管理员权限。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod icons;
mod model;
mod panel;
mod theme;
mod toast;
mod tray;
mod usb;
mod win;

use std::collections::HashSet;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use egui::{Pos2, Vec2};
use model::DeviceInfo;
use theme::PAD;
use toast::Toast;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Hidden,
    Toast,
    Panel,
}

pub struct App {
    pub devices: Vec<DeviceInfo>,
    pub selected: Option<String>,
    pub search: String,
    pub filter: panel::Filter,
    pub scroll: f32,
    pub copied_at: f64,
    pub now: f64,
    pub want_close: bool,
    pub want_minimize: bool,

    mode: Mode,
    toasts: Vec<Toast>,
    toast_enabled: bool,
    rx: Receiver<Vec<DeviceInfo>>,
    tray: Option<tray::Tray>,
    start: Instant,
    last_size: Vec2,
    last_pos: Pos2,
    last_level: Option<bool>,
    seq: u64,
    demo: u8,
    start_panel: bool,
    last_visible: Option<bool>,
    show_pending: bool,
    tray_tried: bool,
    last_beat: f64,
    no_tray: bool,
    border_tries: u8,
}

const TOAST_W: f32 = 424.0;
const TOAST_H: f32 = 92.0;
const TOAST_TTL: f64 = 6.0;
const MAX_TOASTS: usize = 3;

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::setup_fonts(&cc.egui_ctx);

        let rx = usb::spawn_detector(cc.egui_ctx.clone());
        let devices = usb::initial_scan();


        let args: Vec<String> = std::env::args().collect();
        let silent = args.iter().any(|a| a == "--hidden");
        let demo = if args.iter().any(|a| a == "--demo-toast") {
            1
        } else if args.iter().any(|a| a == "--demo-panel") {
            2
        } else {
            0
        };

        Self {
            devices,
            selected: None,
            search: String::new(),
            filter: panel::Filter::All,
            scroll: 0.0,
            copied_at: 0.0,
            now: 0.0,
            want_close: false,
            want_minimize: false,
            mode: Mode::Hidden,
            toasts: Vec::new(),
            toast_enabled: true,
            rx,
            tray: None,
            start: Instant::now(),
            last_size: Vec2::new(-1.0, -1.0),
            last_pos: Pos2::new(-9999.0, -9999.0),
            last_level: None,
            seq: 0,
            demo,
            start_panel: !silent && demo == 0,
            last_visible: None,
            show_pending: false,
            tray_tried: false,
            last_beat: 0.0,
            no_tray: std::env::var("USBTIP_NO_TRAY").is_ok(),
            border_tries: 0,
        }
    }

    // ---------- 设备快照 ----------

    fn apply_snapshot(&mut self, snap: Vec<DeviceInfo>) {
        let ts = usb::now_ms();
        let mut seen: HashSet<String> = HashSet::new();
        let mut announce: Vec<(DeviceInfo, bool)> = Vec::new();

        for mut d in snap {
            seen.insert(d.key.clone());
            if let Some(existing) = self.devices.iter_mut().find(|x| x.key == d.key) {
                if !existing.present {
                    d.plug_count = existing.plug_count + 1;
                    d.last_change_ms = ts;
                    *existing = d;
                    if usb::is_noteworthy(existing) {
                        announce.push((existing.clone(), true));
                    }
                } else {
                    let pc = existing.plug_count;
                    let lc = existing.last_change_ms;
                    *existing = d;
                    existing.plug_count = pc;
                    existing.last_change_ms = lc;
                }
            } else {
                d.present = true;
                d.plug_count = 1;
                d.last_change_ms = ts;
                self.devices.push(d.clone());
                if usb::is_noteworthy(&d) {
                    announce.push((d, true));
                }
            }
        }

        let mut removed: Vec<DeviceInfo> = Vec::new();
        for d in self.devices.iter_mut() {
            if d.present && !seen.contains(&d.key) {
                d.present = false;
                d.last_change_ms = ts;
                removed.push(d.clone());
            }
        }
        for d in removed {
            announce.push((d, false));
        }

        for (d, arr) in announce {
            self.push_toast(d, arr);
        }
    }

    fn push_toast(&mut self, dev: DeviceInfo, arrival: bool) {
        if !self.toast_enabled {
            return;
        }
        self.seq += 1;
        let t = Toast {
            key: dev.key.clone(),
            dev,
            arrival,
            born: self.now,
            ttl: TOAST_TTL,
            id: self.seq,
            hover_start: None,
        };
        self.toasts.push(t);
        while self.toasts.len() > MAX_TOASTS {
            self.toasts.remove(0);
        }
        if self.mode == Mode::Hidden {
            self.mode = Mode::Toast;
            self.last_pos = Pos2::new(-9999.0, -9999.0);
        }
    }

    // ---------- 托盘 ----------

    fn handle_tray(&mut self, ctx: &egui::Context) {
        let mut open_panel = false;
        let mut toggle = false;
        let mut auto = false;
        let mut quit = false;

        while let Ok(ev) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if let Some(t) = &self.tray {
                win::trace(&format!(
                    "menu ev id={:?} open={:?} toggle={:?} auto={:?} quit={:?}",
                    ev.id, t.m_open.id(), t.m_toggle.id(), t.m_auto.id(), t.m_quit.id()
                ));
                if ev.id == *t.m_open.id() {
                    open_panel = true;
                } else if ev.id == *t.m_toggle.id() {
                    toggle = true;
                } else if ev.id == *t.m_auto.id() {
                    auto = true;
                } else if ev.id == *t.m_quit.id() {
                    quit = true;
                }
            }
        }

        while let Ok(ev) = tray_icon::TrayIconEvent::receiver().try_recv() {
            win::trace(&format!("tray ev {ev:?}"));
            if let tray_icon::TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                ..
            } = ev
            {
                open_panel = true;
            }
        }

        if toggle {
            self.toast_enabled = !self.toast_enabled;
            if !self.toast_enabled {
                self.toasts.clear();
                if self.mode == Mode::Toast {
                    self.mode = Mode::Hidden;
                }
            }
            if let Some(t) = &self.tray {
                t.m_toggle
                    .set_text(if self.toast_enabled { "暂停提示" } else { "恢复提示  ✓" });
            }
        }
        if auto {
            let on = !win::autostart_enabled();
            win::set_autostart(on);
            if let Some(t) = &self.tray {
                t.m_auto.set_text(if on { "开机自启  ✓" } else { "开机自启" });
            }
        }
        if quit {
            win::trace("QUIT requested by menu");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if open_panel {
            self.open_panel(ctx);
        }
    }

    fn open_panel(&mut self, ctx: &egui::Context) {
        self.mode = Mode::Panel;
        self.toasts.clear();
        self.last_pos = Pos2::new(-9999.0, -9999.0);
        if self.selected.is_none() {
            if let Some(d) = self.devices.iter().max_by_key(|d| d.last_change_ms) {
                self.selected = Some(d.key.clone());
            }
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        win::trace("open_panel done");
    }

    // ---------- 窗口几何 ----------

    fn content_size(&self) -> Vec2 {
        match self.mode {
            Mode::Hidden => Vec2::new(1.0, 1.0),
            Mode::Toast => {
                let n = self.toasts.len().max(1) as f32;
                Vec2::new(TOAST_W, n * TOAST_H + (n - 1.0) * 10.0)
            }
            Mode::Panel => Vec2::new(panel::PANEL_W, panel::PANEL_H),
        }
    }

    fn sync_window(&mut self, ctx: &egui::Context) {
        let want_visible = self.mode != Mode::Hidden;

        if self.last_visible != Some(want_visible) {
            self.last_visible = Some(want_visible);
            self.last_size = Vec2::new(-1.0, -1.0);
            self.last_pos = Pos2::new(-9999.0, -9999.0);
            if !want_visible {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                return;
            }
            // 从隐藏恢复：先把尺寸和位置摆好，最后一刻再显示，避免闪一下
            self.show_pending = true;
        }

        let wa = win::work_area();
        let cs = self.content_size();
        let win = cs + Vec2::splat(PAD * 2.0);

        if (self.last_size.x - win.x).abs() > 0.5 || (self.last_size.y - win.y).abs() > 0.5 {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(win));
            self.last_size = win;
        }

        let mut pos = match self.mode {
            Mode::Hidden => Pos2::new(wa.right() - 2.0, wa.bottom() - 2.0),
            Mode::Panel => {
                let c = wa.center();
                Pos2::new((c.x - win.x * 0.5).round(), (c.y - win.y * 0.5).round())
            }
            Mode::Toast => {
                let mut slide = 0.0;
                if let Some(t) = self.toasts.last() {
                    let age = (self.now - t.born) as f32;
                    if age < 0.28 {
                        slide = (1.0 - theme::ease_out_cubic(age / 0.28)) * 56.0;
                    }
                }
                Pos2::new(
                    (wa.right() - win.x - 12.0).round() + slide,
                    (wa.bottom() - win.y - 12.0).round(),
                )
            }
        };
        if self.mode == Mode::Panel {
            pos.x = pos.x.clamp(wa.left(), (wa.right() - win.x).max(wa.left()));
            pos.y = pos.y.clamp(wa.top(), (wa.bottom() - win.y).max(wa.top()));
        }

        if (self.last_pos.x - pos.x).abs() > 0.4 || (self.last_pos.y - pos.y).abs() > 0.4 {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
            self.last_pos = pos;
        }

        let want_top = self.mode == Mode::Toast;
        if self.last_level != Some(want_top) {
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if want_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            }));
            self.last_level = Some(want_top);
        }

        if self.show_pending {
            self.show_pending = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }
    }

    // ---------- 绘制 ----------

    fn draw_toasts(&mut self, ui: &mut egui::Ui) {
        let area = ui.max_rect().shrink(PAD);
        let mut clicked: Option<usize> = None;
        let mut hovered_any = false;

        let items: Vec<(usize, Toast)> = self
            .toasts
            .iter()
            .enumerate()
            .map(|(i, t)| (i, clone_toast(t)))
            .collect();

        for (i, t) in items {
            let y = area.top() + i as f32 * (TOAST_H + 10.0);
            let r = egui::Rect::from_min_size(
                Pos2::new(area.left(), y),
                Vec2::new(area.width(), TOAST_H),
            );
            let hovered = ui.rect_contains_pointer(r);
            if hovered {
                hovered_any = true;
            }
            let resp = toast::draw(ui, r, &t, t.progress(self.now), hovered);
            if resp.clicked() {
                clicked = Some(i);
            }
        }

        for t in self.toasts.iter_mut() {
            if hovered_any {
                if t.hover_start.is_none() {
                    t.hover_start = Some(self.now);
                }
            } else if let Some(hs) = t.hover_start.take() {
                t.born += self.now - hs;
            }
        }

        if let Some(i) = clicked {
            if let Some(t) = self.toasts.get(i) {
                self.selected = Some(t.key.clone());
            }
            self.toasts.clear();
            self.mode = Mode::Panel;
            self.last_pos = Pos2::new(-9999.0, -9999.0);
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
            ui.ctx().request_repaint();
        }
    }

    /// 面板打开时，插拔提示浮在面板右下角，不另开窗口
    fn draw_panel_toasts(&mut self, ui: &mut egui::Ui) {
        if self.toasts.is_empty() {
            return;
        }
        let area = ui.max_rect().shrink(PAD);
        let items: Vec<(usize, Toast)> = self
            .toasts
            .iter()
            .enumerate()
            .map(|(i, t)| (i, clone_toast(t)))
            .collect();
        let n = items.len();
        let mut clicked: Option<usize> = None;
        let mut hovered_any = false;

        for (idx, (i, t)) in items.iter().enumerate() {
            let from_bottom = (n - idx) as f32;
            let y = area.bottom() - 14.0 - from_bottom * (TOAST_H + 10.0) + 10.0;
            let r = egui::Rect::from_min_size(
                Pos2::new(area.right() - TOAST_W - 14.0, y),
                Vec2::new(TOAST_W, TOAST_H),
            );
            let hovered = ui.rect_contains_pointer(r);
            if hovered {
                hovered_any = true;
            }
            let resp = toast::draw(ui, r, &t, t.progress(self.now), hovered);
            if resp.clicked() {
                clicked = Some(*i);
            }
        }

        for t in self.toasts.iter_mut() {
            if hovered_any {
                if t.hover_start.is_none() {
                    t.hover_start = Some(self.now);
                }
            } else if let Some(hs) = t.hover_start.take() {
                t.born += self.now - hs;
            }
        }

        if let Some(i) = clicked {
            if let Some(t) = self.toasts.get(i) {
                self.selected = Some(t.key.clone());
                self.filter = panel::Filter::All;
                self.search.clear();
            }
            self.toasts.clear();
        }
    }

    fn expire_toasts(&mut self) {
        self.toasts.retain(|t| (self.now - t.born) < t.ttl);
        if self.mode == Mode::Toast && self.toasts.is_empty() {
            self.mode = Mode::Hidden;
        }
    }
}

fn clone_toast(t: &Toast) -> Toast {
    Toast {
        key: t.key.clone(),
        dev: t.dev.clone(),
        arrival: t.arrival,
        born: t.born,
        ttl: t.ttl,
        id: t.id,
        hover_start: t.hover_start,
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        win::trace("on_exit called");
    }

    /// 每帧逻辑（窗口隐藏时也会被调用，用于继续监听插拔）
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.now = self.start.elapsed().as_secs_f64();

        // 去掉 Windows 11 画在窗口外沿的原生边框（会显示成一圈白框）。
        // 窗口首次显示 / 尺寸变化时 winit 可能重新套用样式，所以分几次补打。
        const BORDER_AT: [f64; 3] = [0.25, 1.20, 3.00];
        if (self.border_tries as usize) < BORDER_AT.len()
            && self.now > BORDER_AT[self.border_tries as usize]
        {
            let n = self.border_tries + 1;
            self.border_tries += 1;
            let ok = win::hide_window_border();
            win::trace(&format!(
                "dwm border try#{n} -> {}",
                if ok { "ok" } else { "FAIL" }
            ));
        }

        // 托盘图标等事件循环真正跑起来之后再建，太早会失败
        if !self.tray_tried && self.now > 0.35 {
            self.tray_tried = true;
            if self.no_tray {
                win::trace("tray skipped (USBTIP_NO_TRAY)");
            } else {
                self.tray = tray::build();
            }
            win::trace(if self.tray.is_some() { "tray created" } else { "tray FAILED" });
            if let Some(t) = &self.tray {
                let on = win::autostart_enabled();
                t.m_auto
                    .set_text(if on { "开机自启  ✓" } else { "开机自启" });
            }
            ctx.request_repaint();
        }

        while let Ok(snap) = self.rx.try_recv() {
            self.apply_snapshot(snap);
        }

        self.handle_tray(ctx);

        if self.start_panel && self.now > 1.2 {
            self.start_panel = false;
            win::trace("open_panel from start_panel");
            self.open_panel(ctx);
        }

        if self.demo > 0 && self.now > 0.9 {
            let d = self.demo;
            self.demo = 0;
            if d == 2 {
                self.open_panel(ctx);
            } else {
                let dev = self
                    .devices
                    .iter()
                    .find(|x| x.kind == model::DeviceKind::Serial)
                    .or_else(|| self.devices.iter().find(|x| x.present))
                    .or_else(|| self.devices.first())
                    .cloned();
                if let Some(dev) = dev {
                    self.push_toast(dev, true);
                }
            }
        }

        if self.want_minimize {
            // 最小化到任务栏，随时能点回来
            self.want_minimize = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
        if self.want_close {
            // 收进托盘
            self.want_close = false;
            self.mode = Mode::Hidden;
            self.last_pos = Pos2::new(-9999.0, -9999.0);
        }

        self.expire_toasts();
        self.sync_window(ctx);

        let animating = self.mode == Mode::Toast
            && self
                .toasts
                .last()
                .map(|t| (self.now - t.born) < 0.32)
                .unwrap_or(false);

        if (self.now - self.last_beat) > 1.0 {
            self.last_beat = self.now;
            win::trace(&format!(
                "t={:.1} mode={} toasts={} devs={} vis={:?}",
                self.now,
                match self.mode { Mode::Hidden => "hidden", Mode::Toast => "toast", Mode::Panel => "panel" },
                self.toasts.len(),
                self.devices.len(),
                self.last_visible
            ));
        }
        if false {
            self.last_beat = self.now;
            win::trace(&format!(
                "alive mode={} toasts={} devs={}",
                match self.mode { Mode::Hidden => "hidden", Mode::Toast => "toast", Mode::Panel => "panel" },
                self.toasts.len(),
                self.devices.len()
            ));
        }

        let delay = if animating {
            Duration::from_millis(12)
        } else if self.mode == Mode::Hidden {
            Duration::from_millis(500)
        } else {
            Duration::from_millis(160)
        };
        ctx.request_repaint_after(delay);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 模式刚切换的那一帧，窗口尺寸还是上一个形态（可能只有 1x1），
        // 减去阴影边距后会算出负尺寸矩形。这一帧直接跳过，下一帧尺寸就位后再画。
        let area = ui.max_rect();
        if self.mode != Mode::Hidden && (area.width() < 200.0 || area.height() < 120.0) {
            win::trace(&format!(
                "skip draw: area {}x{}",
                area.width(),
                area.height()
            ));
            return;
        }
        match self.mode {
            Mode::Hidden => {}
            Mode::Toast => self.draw_toasts(ui),
            Mode::Panel => {
                panel::draw(self, ui);
                self.draw_panel_toasts(ui);
            }
        }
        if self.want_close || self.want_minimize {
            ui.ctx().request_repaint();
        }
    }
}

fn main() -> eframe::Result {
    // GUI 程序没有控制台，把崩溃信息落到临时目录方便排查
    std::panic::set_hook(Box::new(|info| {
        let mut path = std::env::temp_dir();
        path.push("usbtip-panic.log");
        let _ = std::fs::write(&path, format!("{info}\n"));
        eprintln!("{info}");
    }));

    win::trace_reset();
    win::trace("=== start ===");

    let icon = egui::IconData {
        rgba: icons::icon_rgba(64),
        width: 64,
        height: 64,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("USB TIP")
            .with_inner_size([1.0, 1.0])
            .with_position([0.0, 0.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_active(false)
            .with_icon(icon),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    let r = eframe::run_native(
        "USB TIP",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    );
    win::trace(match &r {
        Ok(()) => "run_native returned Ok (clean exit)",
        Err(_) => "run_native returned Err",
    });
    r
}

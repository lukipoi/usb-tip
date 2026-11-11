//! Win32 辅助：只留三件小事——工作区尺寸、开机自启、运行日志。
//!
//! 窗口的显示 / 隐藏 / 置顶 / 焦点全部交给 eframe 自己的 ViewportCommand，
//! 这里不碰任何窗口样式或框架状态（上次崩溃就是自作聪明改 EXSTYLE 引起的）。

use std::ffi::c_void;

const SPI_GETWORKAREA: u32 = 0x0030;

#[repr(C)]
#[derive(Default)]
struct Rect32 {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[link(name = "user32")]
extern "system" {
    fn SystemParametersInfoW(action: u32, ui_param: u32, pv: *mut c_void, win_ini: u32) -> i32;
    fn GetDpiForSystem() -> u32;
    fn FindWindowW(class_name: *const u16, window_name: *const u16) -> isize;
    fn GetWindowThreadProcessId(hwnd: isize, pid: *mut u32) -> u32;
}

#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(hwnd: isize, attr: u32, value: *const c_void, size: u32) -> i32;
}

/// Windows 11 会给每个顶层窗口画一圈原生边框（DWM border）。
/// 我们的窗口本体是透明的、真正的卡片比窗口小一圈（留给阴影），
/// 于是这条 1px 白线就露在卡片外面，看起来像"多出来的白框"。
/// 把边框颜色设成 DWMWA_COLOR_NONE 即可去掉，只改颜色，不动窗口样式。
pub fn hide_window_border() -> bool {
    const DWMWA_BORDER_COLOR: u32 = 38;
    const DWMWA_COLOR_NONE: u32 = 0xFFFF_FFFE;

    let title = wide("USB TIP");
    let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
    if hwnd == 0 {
        return false;
    }
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    if pid != std::process::id() {
        return false;
    }
    let color = DWMWA_COLOR_NONE;
    let rc = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &color as *const u32 as *const c_void,
            4,
        )
    };
    rc == 0
}

// ---------------- 日志 ----------------

/// 追加一行运行日志到 %TEMP%\usbtip.log
pub fn trace(msg: &str) {
    use std::io::Write;
    let mut path = std::env::temp_dir();
    path.push("usbtip.log");
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(f, "{ms} {msg}");
    }
}

pub fn trace_reset() {
    let mut path = std::env::temp_dir();
    path.push("usbtip.log");
    let _ = std::fs::write(&path, "");
}

// ---------------- 工作区 ----------------

/// 主显示器工作区（已排除任务栏），返回逻辑点坐标
pub fn work_area() -> egui::Rect {
    let mut r = Rect32::default();
    let ok = unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut r as *mut _ as *mut c_void, 0) };
    if ok == 0 || r.right <= r.left || r.bottom <= r.top {
        return egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1920.0, 1040.0));
    }
    let dpi = unsafe { GetDpiForSystem() } as f32;
    let scale = if dpi > 0.0 { dpi / 96.0 } else { 1.0 };
    egui::Rect::from_min_max(
        egui::pos2(r.left as f32 / scale, r.top as f32 / scale),
        egui::pos2(r.right as f32 / scale, r.bottom as f32 / scale),
    )
}

// ---------------- 开机自启 ----------------

const HKEY_CURRENT_USER: isize = 0x8000_0001u32 as isize;
const KEY_QUERY_VALUE: u32 = 0x0001;
const KEY_SET_VALUE: u32 = 0x0002;
const REG_SZ: u32 = 1;
const ERROR_SUCCESS: i32 = 0;

#[link(name = "advapi32")]
extern "system" {
    fn RegCreateKeyExW(
        hkey: isize,
        sub_key: *const u16,
        reserved: u32,
        class: *const u16,
        options: u32,
        sam: u32,
        security: *mut c_void,
        result: *mut isize,
        disposition: *mut u32,
    ) -> i32;
    fn RegOpenKeyExW(
        hkey: isize,
        sub_key: *const u16,
        options: u32,
        sam: u32,
        result: *mut isize,
    ) -> i32;
    fn RegSetValueExW(
        hkey: isize,
        name: *const u16,
        reserved: u32,
        typ: u32,
        data: *const u8,
        len: u32,
    ) -> i32;
    fn RegQueryValueExW(
        hkey: isize,
        name: *const u16,
        reserved: *mut u32,
        typ: *mut u32,
        data: *mut u8,
        len: *mut u32,
    ) -> i32;
    fn RegDeleteValueW(hkey: isize, name: *const u16) -> i32;
    fn RegCloseKey(hkey: isize) -> i32;
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE: &str = "UsbTip";

pub fn autostart_enabled() -> bool {
    let sub = wide(RUN_KEY);
    let mut hkey: isize = 0;
    let rc = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, KEY_QUERY_VALUE, &mut hkey) };
    if rc != ERROR_SUCCESS {
        return false;
    }
    let name = wide(VALUE);
    let mut typ: u32 = 0;
    let mut len: u32 = 0;
    let rc = unsafe {
        RegQueryValueExW(
            hkey,
            name.as_ptr(),
            std::ptr::null_mut(),
            &mut typ,
            std::ptr::null_mut(),
            &mut len,
        )
    };
    unsafe { RegCloseKey(hkey) };
    rc == ERROR_SUCCESS
}

pub fn set_autostart(on: bool) {
    let sub = wide(RUN_KEY);
    let mut hkey: isize = 0;
    let mut disp: u32 = 0;
    let rc = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            sub.as_ptr(),
            0,
            std::ptr::null(),
            0,
            KEY_SET_VALUE,
            std::ptr::null_mut(),
            &mut hkey,
            &mut disp,
        )
    };
    if rc != ERROR_SUCCESS {
        return;
    }
    let name = wide(VALUE);
    if on {
        let exe = std::env::current_exe().unwrap_or_default();
        let path = format!("\"{}\" --hidden", exe.display());
        let data: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = data.len() * 2;
        unsafe {
            RegSetValueExW(
                hkey,
                name.as_ptr(),
                0,
                REG_SZ,
                data.as_ptr() as *const u8,
                bytes as u32,
            );
        }
    } else {
        unsafe { RegDeleteValueW(hkey, name.as_ptr()) };
    }
    unsafe { RegCloseKey(hkey) };
}

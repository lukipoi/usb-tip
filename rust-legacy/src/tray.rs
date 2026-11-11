//! 系统托盘

use tray_icon::menu::{Menu, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::icons;

pub struct Tray {
    pub _icon: TrayIcon,
    pub m_open: MenuItem,
    pub m_toggle: MenuItem,
    pub m_auto: MenuItem,
    pub m_quit: MenuItem,
}

fn log(msg: &str) {
    let mut path = std::env::temp_dir();
    path.push("usbtip-tray.log");
    let _ = std::fs::write(&path, msg);
}

pub fn build() -> Option<Tray> {
    let menu = Menu::new();
    let m_open = MenuItem::new("打开面板", true, None);
    let m_toggle = MenuItem::new("暂停提示", true, None);
    let m_auto = MenuItem::new("开机自启", true, None);
    let m_quit = MenuItem::new("退出", true, None);
    let _ = menu.append(&m_open);
    let _ = menu.append(&m_toggle);
    let _ = menu.append(&m_auto);
    let _ = menu.append(&m_quit);

    let rgba = icons::icon_rgba(32);
    let icon = match Icon::from_rgba(rgba, 32, 32) {
        Ok(i) => i,
        Err(e) => {
            log(&format!("icon err: {e}"));
            return None;
        }
    };

    match TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("USB TIP · USB 插拔监视")
        .with_icon(icon)
        .build()
    {
        Ok(tray) => {
            log("ok");
            Some(Tray {
                _icon: tray,
                m_open,
                m_toggle,
                m_auto,
                m_quit,
            })
        }
        Err(e) => {
            log(&format!("build err: {e:?}"));
            None
        }
    }
}

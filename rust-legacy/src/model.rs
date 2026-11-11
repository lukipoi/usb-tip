//! 设备数据模型与类型推断

use egui::Color32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeviceKind {
    Serial,
    Storage,
    Hid,
    Network,
    Audio,
    Camera,
    Printer,
    Bluetooth,
    SmartCard,
    Android,
    Hub,
    Modem,
    Composite,
    Generic,
}

impl DeviceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Serial => "串口",
            Self::Storage => "存储",
            Self::Hid => "人机输入",
            Self::Network => "网络",
            Self::Audio => "音频",
            Self::Camera => "图像",
            Self::Printer => "打印",
            Self::Bluetooth => "蓝牙",
            Self::SmartCard => "智能卡",
            Self::Android => "安卓设备",
            Self::Hub => "集线器",
            Self::Modem => "调制解调器",
            Self::Composite => "复合设备",
            Self::Generic => "其他设备",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            Self::Serial => Color32::from_rgb(0x4A, 0xDE, 0x80),
            Self::Storage => Color32::from_rgb(0xFB, 0xBF, 0x24),
            Self::Hid => Color32::from_rgb(0xA7, 0x8B, 0xFA),
            Self::Network => Color32::from_rgb(0x38, 0xBD, 0xF8),
            Self::Audio => Color32::from_rgb(0xF4, 0x72, 0xB6),
            Self::Camera => Color32::from_rgb(0xFB, 0x92, 0x3C),
            Self::Printer => Color32::from_rgb(0x22, 0xD3, 0xEE),
            Self::Bluetooth => Color32::from_rgb(0x60, 0xA5, 0xFA),
            Self::SmartCard => Color32::from_rgb(0x94, 0xA3, 0xB8),
            Self::Android => Color32::from_rgb(0x34, 0xD3, 0x99),
            Self::Hub => Color32::from_rgb(0x8B, 0x9E, 0xB4),
            Self::Modem => Color32::from_rgb(0xC0, 0x8A, 0xFE),
            Self::Composite => Color32::from_rgb(0x7D, 0x8A, 0xA0),
            Self::Generic => Color32::from_rgb(0x94, 0xA3, 0xB8),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    /// 归并键（USB\VID_xxxx&PID_yyyy）
    pub key: String,
    pub name: String,
    pub desc: String,
    pub class: String,
    pub service: String,
    pub mfg: String,
    pub location: String,
    pub instance_id: String,
    pub vid: String,
    pub pid: String,
    pub serial: String,
    pub com_port: String,
    pub kind: DeviceKind,
    pub present: bool,
    pub last_change_ms: u64,
    pub plug_count: u32,
}

impl DeviceInfo {
    pub fn vid_pid(&self) -> String {
        format!("VID_{} · PID_{}", self.vid, self.pid)
    }

    /// 给剪贴板/详情用的纯文本
    pub fn dump(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("设备名称: {}\n", self.name));
        s.push_str(&format!("类型:     {}\n", self.kind.label()));
        s.push_str(&format!("VID:      {}\n", self.vid));
        s.push_str(&format!("PID:      {}\n", self.pid));
        if !self.com_port.is_empty() {
            s.push_str(&format!("串口:     {}\n", self.com_port));
        }
        if !self.serial.is_empty() {
            s.push_str(&format!("序列号:   {}\n", self.serial));
        }
        s.push_str(&format!("设备类:   {}\n", self.class));
        s.push_str(&format!("制造商:   {}\n", self.mfg));
        s.push_str(&format!("服务:     {}\n", self.service));
        s.push_str(&format!("端口位置: {}\n", self.location));
        s.push_str(&format!("实例 ID:  {}\n", self.instance_id));
        s.push_str(&format!("状态:     {}\n", if self.present { "已连接" } else { "已断开" }));
        s
    }
}

/// `USB\VID_1A86&PID_7523&MI_00\7&2f3a&0&0000` -> `USB\VID_1A86&PID_7523&MI_00`
pub fn device_id_of(instance_id: &str) -> &str {
    match instance_id.find('\\') {
        Some(_) => match instance_id.rfind('\\') {
            Some(p) => &instance_id[..p],
            None => instance_id,
        },
        None => instance_id,
    }
}

/// 去掉接口后缀，归并到物理设备
pub fn base_of(device_id: &str) -> String {
    if let Some(p) = device_id.find("&MI_") {
        return device_id[..p].to_string();
    }
    device_id.to_string()
}

pub fn parse_vid_pid(device_id: &str) -> (String, String) {
    let upper = device_id.to_ascii_uppercase();
    let vid = upper
        .find("VID_")
        .and_then(|p| upper.get(p + 4..p + 8))
        .unwrap_or("----")
        .to_string();
    let pid = upper
        .find("PID_")
        .and_then(|p| upper.get(p + 4..p + 8))
        .unwrap_or("----")
        .to_string();
    (vid, pid)
}

/// 实例路径第二段若不是 "7&1a2b&0&0000" 这种总线编号，就当序列号
pub fn parse_serial(instance_id: &str) -> String {
    let mut parts = instance_id.splitn(3, '\\');
    let _dev = parts.next();
    let inst = parts.next().unwrap_or("");
    if inst.is_empty() || inst.contains('&') {
        return String::new();
    }
    if inst.chars().all(|c| c.is_ascii_digit()) {
        return String::new();
    }
    inst.to_string()
}

pub fn extract_com(name: &str) -> String {
    let bytes = name.as_bytes();
    let mut i = 0;
    while i + 3 <= bytes.len() {
        if bytes[i].eq_ignore_ascii_case(&b'C')
            && bytes[i + 1].eq_ignore_ascii_case(&b'O')
            && bytes[i + 2].eq_ignore_ascii_case(&b'M')
        {
            let mut j = i + 3;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 3 {
                return String::from_utf8_lossy(&bytes[i..j]).to_string();
            }
        }
        i += 1;
    }
    String::new()
}

/// Windows 给同名设备会加括号，展示时去掉
pub fn clean_name(s: &str) -> String {
    let t = s.trim();
    if t.chars().count() > 2 && t.starts_with('(') && t.ends_with(')') {
        return t[1..t.len() - 1].trim().to_string();
    }
    t.to_string()
}

pub fn classify(class: &str, service: &str, name: &str, desc: &str) -> DeviceKind {
    let c = class.to_ascii_lowercase();
    let s = service.to_ascii_lowercase();
    let n = format!("{} {}", name, desc).to_lowercase();

    if c == "ports" {
        return DeviceKind::Serial;
    }
    if s.contains("usbstor") {
        return DeviceKind::Storage;
    }
    if c == "hidclass" || n.contains("hid") {
        return DeviceKind::Hid;
    }
    if c == "net" {
        return DeviceKind::Network;
    }
    if c == "media" || c == "audioendpoint" {
        return DeviceKind::Audio;
    }
    if c == "image" || c == "camera" {
        return DeviceKind::Camera;
    }
    if c == "printer" || s == "usbprint" {
        return DeviceKind::Printer;
    }
    if s == "bthusb" || c == "bluetooth" || n.contains("bluetooth") || n.contains("蓝牙") {
        return DeviceKind::Bluetooth;
    }
    if c == "smartcardreader" || c == "smartcard" {
        return DeviceKind::SmartCard;
    }
    if c == "androidusbdeviceclass" || n.contains("android") || n.contains("adb") {
        return DeviceKind::Android;
    }
    if s == "usbhub" || n.contains("hub") || n.contains("集线器") || n.contains("根集线器") {
        return DeviceKind::Hub;
    }
    if c == "modem" {
        return DeviceKind::Modem;
    }
    if n.contains("keyboard") || n.contains("键盘") {
        return DeviceKind::Hid;
    }
    if n.contains("mouse") || n.contains("鼠标") {
        return DeviceKind::Hid;
    }
    if c == "usb" || s == "usbccgp" {
        return DeviceKind::Composite;
    }
    DeviceKind::Generic
}

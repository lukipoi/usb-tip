//! USB 设备枚举（SetupAPI / CfgMgr32）与插拔检测线程
//!
//! 全部走 CfgMgr32 的只读接口，普通用户权限即可，不需要管理员、不装驱动。

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

use crate::model::{self, DeviceInfo, DeviceKind};

const CR_SUCCESS: u32 = 0;
const CR_BUFFER_SMALL: u32 = 0x1A;

const CM_GETIDLIST_FILTER_ENUMERATOR: u32 = 0x0000_0001;
const CM_GETIDLIST_FILTER_PRESENT: u32 = 0x0000_0100;

const CM_DRP_DEVICEDESC: u32 = 0x00;
const CM_DRP_SERVICE: u32 = 0x05;
const CM_DRP_CLASS: u32 = 0x08;
const CM_DRP_MFG: u32 = 0x0B;
const CM_DRP_FRIENDLYNAME: u32 = 0x0C;
const CM_DRP_LOCATION_INFORMATION: u32 = 0x0D;

#[link(name = "cfgmgr32")]
extern "system" {
    fn CM_Get_Device_ID_List_SizeW(len: *mut u32, filter: *const u16, flags: u32) -> u32;
    fn CM_Get_Device_ID_ListW(filter: *const u16, list: *mut u16, len: u32, flags: u32) -> u32;
    fn CM_Locate_DevNodeW(dn: *mut u32, id: *const u16, flags: u32) -> u32;
    fn CM_Get_DevNode_Registry_PropertyW(
        dn: u32,
        prop: u32,
        reg_type: *mut u32,
        buf: *mut c_void,
        len: *mut u32,
        flags: u32,
    ) -> u32;
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 枚举器下的设备实例 ID 列表
fn enum_ids(enumerator: &str, present_only: bool) -> Vec<String> {
    let filter = wide(enumerator);
    let mut flags = CM_GETIDLIST_FILTER_ENUMERATOR;
    if present_only {
        flags |= CM_GETIDLIST_FILTER_PRESENT;
    }

    for _ in 0..3 {
        let mut size: u32 = 0;
        let cr = unsafe { CM_Get_Device_ID_List_SizeW(&mut size, filter.as_ptr(), flags) };
        if cr != CR_SUCCESS || size == 0 {
            return Vec::new();
        }

        let mut buf = vec![0u16; size as usize];
        let cr = unsafe { CM_Get_Device_ID_ListW(filter.as_ptr(), buf.as_mut_ptr(), size, flags) };
        if cr == CR_BUFFER_SMALL {
            continue;
        }
        if cr != CR_SUCCESS {
            return Vec::new();
        }

        let mut out = Vec::new();
        let mut start = 0usize;
        for i in 0..buf.len() {
            if buf[i] == 0 {
                if i > start {
                    out.push(String::from_utf16_lossy(&buf[start..i]));
                }
                start = i + 1;
            }
        }
        return out;
    }
    Vec::new()
}

/// 读设备注册表属性（REG_SZ）
fn read_prop(dn: u32, prop: u32) -> String {
    let mut size: u32 = 512;
    let mut buf = vec![0u8; size as usize];
    let mut reg_type: u32 = 0;

    let mut cr = unsafe {
        CM_Get_DevNode_Registry_PropertyW(
            dn,
            prop,
            &mut reg_type,
            buf.as_mut_ptr() as *mut c_void,
            &mut size,
            0,
        )
    };
    if cr == CR_BUFFER_SMALL {
        buf = vec![0u8; size as usize];
        cr = unsafe {
            CM_Get_DevNode_Registry_PropertyW(
                dn,
                prop,
                &mut reg_type,
                buf.as_mut_ptr() as *mut c_void,
                &mut size,
                0,
            )
        };
    }
    if cr != CR_SUCCESS || size == 0 {
        return String::new();
    }

    // REG_SZ(1) / REG_MULTI_SZ(7) 都是 UTF-16
    let units: Vec<u16> = buf
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&u| u != 0)
        .collect();
    String::from_utf16_lossy(&units).trim().to_string()
}

/// 单个 USB 设备实例的原始信息
#[derive(Clone, Debug, Default)]
struct RawInstance {
    instance_id: String,
    desc: String,
    friendly: String,
    class: String,
    service: String,
    mfg: String,
    location: String,
}

fn read_instance(instance_id: &str) -> Option<RawInstance> {
    let id_w = wide(instance_id);
    let mut dn: u32 = 0;
    let cr = unsafe { CM_Locate_DevNodeW(&mut dn, id_w.as_ptr(), 0) };
    if cr != CR_SUCCESS {
        return None;
    }
    Some(RawInstance {
        instance_id: instance_id.to_string(),
        desc: read_prop(dn, CM_DRP_DEVICEDESC),
        friendly: read_prop(dn, CM_DRP_FRIENDLYNAME),
        class: read_prop(dn, CM_DRP_CLASS),
        service: read_prop(dn, CM_DRP_SERVICE),
        mfg: read_prop(dn, CM_DRP_MFG),
        location: read_prop(dn, CM_DRP_LOCATION_INFORMATION),
    })
}

/// 只枚举当前在位设备的实例 ID（很便宜，用于轮询比对）
fn present_instance_ids() -> Vec<String> {
    enum_ids("USB", true)
}

/// 把一组实例归并成逻辑设备
fn merge_group(group: &[RawInstance]) -> DeviceInfo {
    // 越靠前的类信息量越大
    fn score(r: &RawInstance) -> i32 {
        let c = r.class.to_ascii_lowercase();
        let mut s = match c.as_str() {
            "ports" => 100,
            "hidclass" => 90,
            "net" => 85,
            "media" | "audioendpoint" => 80,
            "image" => 78,
            "printer" => 76,
            "smartcardreader" => 70,
            "androidusbdeviceclass" => 68,
            "modem" => 60,
            "usb" => 10,
            _ => 40,
        };
        if !r.friendly.is_empty() {
            s += 20;
        }
        if !r.desc.is_empty() {
            s += 5;
        }
        s
    }

    let mut sorted: Vec<&RawInstance> = group.iter().collect();
    sorted.sort_by_key(|r| -score(r));

    let best = sorted[0];
    let parent = group
        .iter()
        .find(|r| !model::device_id_of(&r.instance_id).contains("&MI_"))
        .unwrap_or(best);

    let (vid, pid) = model::parse_vid_pid(model::device_id_of(&best.instance_id));

    let name = {
        let f = best.friendly.trim();
        let d = best.desc.trim();
        if !f.is_empty() {
            f.to_string()
        } else if !d.is_empty() {
            d.to_string()
        } else if !parent.desc.trim().is_empty() {
            parent.desc.trim().to_string()
        } else {
            "未知 USB 设备".to_string()
        }
    };

    let name = model::clean_name(&name);
    let com = model::extract_com(&name);
    let kind = model::classify(&best.class, &best.service, &name, &parent.desc);

    let mut mfg = best.mfg.trim().to_string();
    if mfg.is_empty() {
        mfg = parent.mfg.trim().to_string();
    }
    let location = if !parent.location.trim().is_empty() {
        parent.location.trim().to_string()
    } else {
        best.location.trim().to_string()
    };

    DeviceInfo {
        key: model::base_of(model::device_id_of(&best.instance_id)),
        name,
        desc: parent.desc.trim().to_string(),
        class: if best.class.is_empty() {
            parent.class.clone()
        } else {
            best.class.clone()
        },
        service: best.service.clone(),
        mfg,
        location,
        instance_id: parent.instance_id.clone(),
        vid,
        pid,
        serial: model::parse_serial(&parent.instance_id),
        com_port: com,
        kind,
        present: true,
        last_change_ms: 0,
        plug_count: 1,
    }
}

pub fn now_ms() -> u64 {
    use std::sync::OnceLock;
    static START: OnceLock<Instant> = OnceLock::new();
    let start = START.get_or_init(Instant::now);
    start.elapsed().as_millis() as u64
}

/// 启动后台检测线程：每 500ms 扫一次，把"当前在位设备表"发给 UI 线程
pub fn spawn_detector(ctx: egui::Context) -> Receiver<Vec<DeviceInfo>> {
    let (tx, rx): (Sender<Vec<DeviceInfo>>, Receiver<Vec<DeviceInfo>>) = channel();

    std::thread::spawn(move || {
        let mut cache: HashMap<String, RawInstance> = HashMap::new();
        let mut last_ids: Vec<String> = Vec::new();

        loop {
            let ids = present_instance_ids();
            let mut sorted_ids = ids.clone();
            sorted_ids.sort();

            if sorted_ids != last_ids {
                last_ids = sorted_ids;
                // 清理已消失的缓存
                cache.retain(|k, _| ids.iter().any(|i| i == k));
                for id in &ids {
                    if !cache.contains_key(id) {
                        if let Some(raw) = read_instance(id) {
                            cache.insert(id.clone(), raw);
                        }
                    }
                }

                // 按 base 归并
                let mut groups: HashMap<String, Vec<RawInstance>> = HashMap::new();
                for id in &ids {
                    if let Some(raw) = cache.get(id) {
                        let base = model::base_of(model::device_id_of(&raw.instance_id));
                        groups.entry(base).or_default().push(raw.clone());
                    }
                }

                let mut devices: Vec<DeviceInfo> = groups
                    .values()
                    .filter(|g| !g.is_empty())
                    .map(|g| merge_group(g))
                    .collect();
                devices.sort_by(|a, b| a.name.cmp(&b.name));

                if tx.send(devices).is_err() {
                    return; // UI 已退出
                }
                ctx.request_repaint();
            }

            std::thread::sleep(Duration::from_millis(500));
        }
    });

    rx
}

/// 启动时做一次同步扫描（用于初始化，不弹窗）
pub fn initial_scan() -> Vec<DeviceInfo> {
    let ids = present_instance_ids();
    let mut groups: HashMap<String, Vec<RawInstance>> = HashMap::new();
    for id in &ids {
        if let Some(raw) = read_instance(id) {
            let base = model::base_of(model::device_id_of(&raw.instance_id));
            groups.entry(base).or_default().push(raw);
        }
    }
    let mut devices: Vec<DeviceInfo> = groups
        .values()
        .filter(|g| !g.is_empty())
        .map(|g| merge_group(g))
        .collect();
    devices.sort_by(|a, b| a.name.cmp(&b.name));
    devices
}

/// 判断一个设备是否值得弹窗（过滤掉根集线器之类的噪音）
pub fn is_noteworthy(d: &DeviceInfo) -> bool {
    if d.vid == "----" && d.pid == "----" {
        return false;
    }
    let n = d.name.to_lowercase();
    if n.contains("root hub") || n.contains("根集线器") {
        return false;
    }
    if d.kind == DeviceKind::Hub && n.contains("generic usb hub") && d.name.trim().is_empty() {
        return false;
    }
    true
}

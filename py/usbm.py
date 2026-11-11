"""USB 设备枚举（ctypes 直调 CfgMgr32）。

全部走只读接口，普通用户权限即可：不需要管理员、不装驱动、不改注册表。
"""

from __future__ import annotations

import ctypes
import re
from ctypes import wintypes
from dataclasses import dataclass, field

# ---- CfgMgr32 常量 ----
CR_SUCCESS = 0
CR_BUFFER_SMALL = 0x1A

CM_GETIDLIST_FILTER_ENUMERATOR = 0x0001
CM_GETIDLIST_FILTER_PRESENT = 0x0100

DRP_DEVICEDESC = 0x00
DRP_SERVICE = 0x05
DRP_CLASS = 0x08
DRP_MFG = 0x0C
DRP_FRIENDLYNAME = 0x0D
DRP_LOCATION_INFORMATION = 0x0E

_cfg = ctypes.WinDLL("cfgmgr32", use_last_error=True)

_cfg.CM_Get_Device_ID_List_SizeW.argtypes = [
    ctypes.POINTER(wintypes.ULONG),
    wintypes.LPCWSTR,
    wintypes.ULONG,
]
_cfg.CM_Get_Device_ID_List_SizeW.restype = wintypes.ULONG

_cfg.CM_Get_Device_ID_ListW.argtypes = [
    wintypes.LPCWSTR,
    ctypes.c_void_p,
    wintypes.ULONG,
    wintypes.ULONG,
]
_cfg.CM_Get_Device_ID_ListW.restype = wintypes.ULONG

_cfg.CM_Locate_DevNodeW.argtypes = [
    ctypes.POINTER(wintypes.ULONG),
    wintypes.LPCWSTR,
    wintypes.ULONG,
]
_cfg.CM_Locate_DevNodeW.restype = wintypes.ULONG

_cfg.CM_Get_DevNode_Registry_PropertyW.argtypes = [
    wintypes.ULONG,
    wintypes.ULONG,
    ctypes.POINTER(wintypes.ULONG),
    ctypes.c_void_p,
    ctypes.POINTER(wintypes.ULONG),
    wintypes.ULONG,
]
_cfg.CM_Get_DevNode_Registry_PropertyW.restype = wintypes.ULONG


# ---------------------------------------------------------------- 类型与配色

KIND_LABEL = {
    "serial": "串口",
    "storage": "存储",
    "hid": "人机输入",
    "network": "网络",
    "audio": "音频",
    "camera": "图像",
    "printer": "打印",
    "bluetooth": "蓝牙",
    "smartcard": "智能卡",
    "android": "安卓设备",
    "hub": "集线器",
    "modem": "调制解调器",
    "composite": "复合设备",
    "generic": "其他设备",
}

KIND_COLOR = {
    "serial": "#4ADE80",
    "storage": "#FBBF24",
    "hid": "#A78BFA",
    "network": "#38BDF8",
    "audio": "#F472B6",
    "camera": "#FB923C",
    "printer": "#22D3EE",
    "bluetooth": "#60A5FA",
    "smartcard": "#94A3B8",
    "android": "#34D399",
    "hub": "#8B9EB4",
    "modem": "#C08AFE",
    "composite": "#7D8AA0",
    "generic": "#94A3B8",
}

# 拿不到好名字时按类型给个能读的默认名
KIND_DEFAULT = {
    "serial": "串口设备",
    "storage": "USB 存储设备",
    "hid": "USB 输入设备",
    "network": "USB 网卡",
    "audio": "USB 音频设备",
    "camera": "USB 摄像头",
    "printer": "USB 打印机",
    "bluetooth": "蓝牙适配器",
    "smartcard": "智能卡读卡器",
    "android": "安卓设备",
    "hub": "USB 集线器",
    "modem": "USB 调制解调器",
    "composite": "USB 复合设备",
    "generic": "USB 设备",
}

# Windows 给驱动节点起的通用名，没有信息量，不要拿来当设备名
VAGUE_NAMES = {
    "标准系统设备",
    "标准 usb 主控制器",
    "标准 usb 集线器",
    "标准 usb 主机控制器",
    "标准串行端口设备",
    "usb 复合设备",
    "usb composite device",
    "composite device",
    "generic usb hub",
    "usb root hub",
    "usb 打印支持",
    "microsoft",
    "unknown",
    "未知",
}


@dataclass
class Iface:
    """复合设备里的一个功能接口（MI_xx）。"""

    mi: int
    tag: str        # 短标签：串口 COM5 / WCH 调试 / HID
    name: str       # 该接口的完整名字，可能为空
    cls: str = ""
    service: str = ""
    com: str = ""

    @property
    def detail(self) -> str:
        bits = [self.tag]
        if self.name and self.name != self.tag:
            bits.append(self.name)
        if self.service:
            bits.append(self.service)
        return " · ".join(bits)


def iface_tag(cls: str, service: str, com: str) -> str:
    c = (cls or "").lower()
    s = (service or "").lower()
    if c == "ports":
        return f"串口 {com}" if com else "串口"
    if c == "hidclass":
        return "HID"
    if c in ("camera", "image"):
        return "摄像头"
    if c in ("media", "audioendpoint"):
        return "音频"
    if c == "net":
        return "网卡"
    if c == "printer":
        return "打印"
    if s == "usbprint":
        return "打印"
    if s == "usbvideo":
        return "摄像头"
    if s == "usbstor":
        return "存储"
    if "wchlink" in s or c == "wch":
        return "WCH 调试"
    if c == "usb" and s == "usbccgp":
        return "根接口"
    if cls:
        return cls
    return service or "接口"


IFACE_COLOR = {
    "串口": "#4ADE80",
    "WCH 调试": "#C08AFE",
    "HID": "#A78BFA",
    "摄像头": "#FB923C",
    "音频": "#F472B6",
    "网卡": "#38BDF8",
    "打印": "#22D3EE",
    "存储": "#FBBF24",
}


def iface_color(tag: str) -> str:
    for k, v in IFACE_COLOR.items():
        if tag.startswith(k):
            return v
    return "#94A3B8"


@dataclass
class Device:
    key: str = ""            # 归并键：唯一标识一台物理设备（含端口路径）
    base: str = ""           # USB\VID_xxxx&PID_yyyy
    name: str = ""
    desc: str = ""
    cls: str = ""
    service: str = ""
    mfg: str = ""
    location: str = ""
    instance_id: str = ""
    vid: str = "----"
    pid: str = "----"
    serial: str = ""
    com: str = ""
    kind: str = "generic"
    ifaces: list[Iface] = field(default_factory=list)
    composite: bool = False
    present: bool = True
    first_seen: float = 0.0
    last_change: float = 0.0
    plugs: int = 1

    # ---- 展示辅助 ----
    @property
    def kind_label(self) -> str:
        return KIND_LABEL.get(self.kind, KIND_LABEL["generic"])

    @property
    def label(self) -> str:
        """类型标签；多接口设备追加“复合”"""
        return f"{self.kind_label} · 复合" if self.composite else self.kind_label

    @property
    def kind_color(self) -> str:
        return KIND_COLOR.get(self.kind, KIND_COLOR["generic"])

    @property
    def vid_pid(self) -> str:
        return f"VID_{self.vid} · PID_{self.pid}"

    def dump(self) -> str:
        lines = [
            f"设备名称: {self.name}",
            f"类型:     {self.kind_label}",
            f"VID:      {self.vid}",
            f"PID:      {self.pid}",
        ]
        if self.com:
            lines.append(f"串口:     {self.com}")
        if self.serial:
            lines.append(f"序列号:   {self.serial}")
        lines += [
            f"设备类:   {self.cls}",
            f"制造商:   {self.mfg}",
            f"服务:     {self.service}",
            f"端口位置: {self.location}",
            f"实例 ID:  {self.instance_id}",
            f"状态:     {'已连接' if self.present else '已断开'}",
        ]
        if self.ifaces:
            lines.append("接口:")
            for i in self.ifaces:
                lines.append(f"  MI_{i.mi:02X}  {i.detail}")
        return "\n".join(lines)


# ---------------------------------------------------------------- 字符串工具


def device_id_of(instance_id: str) -> str:
    """USB\\VID_1A86&PID_7523&MI_00\\7&2f3a&0&0000 -> 前半段"""
    p = instance_id.rfind("\\")
    return instance_id[:p] if p >= 0 else instance_id


def base_of(device_id: str) -> str:
    """去掉 &MI_xx 接口后缀，归并到物理设备"""
    p = device_id.find("&MI_")
    return device_id[:p] if p >= 0 else device_id


def group_key(instance_id: str) -> str:
    """归并键：同一 VID/PID 的父节点与各接口归并为“一台设备”。

    局限：同时插两个同型号（同 VID/PID、无序列号）设备时会被合并成一条，
    因为 Windows 没有提供接口节点与端口路径的对应关系。
    """
    return base_of(device_id_of(instance_id))


def parse_vid_pid(device_id: str) -> tuple[str, str]:
    up = device_id.upper()
    vid = pid = "----"
    i = up.find("VID_")
    if i >= 0 and len(up) >= i + 8:
        vid = up[i + 4 : i + 8]
    i = up.find("PID_")
    if i >= 0 and len(up) >= i + 8:
        pid = up[i + 4 : i + 8]
    return vid, pid


def parse_serial(instance_id: str) -> str:
    """实例路径最后一段若不像总线编号（不含 &），就当作序列号。

    USB\\VID_1A86&PID_8010\\7FB68F060553  -> 7FB68F060553
    USB\\VID_1A86&PID_7523\\5&2f3a&0&1    -> （总线编号，无序列号）
    """
    parts = instance_id.split("\\")
    if len(parts) < 3:
        return ""
    inst = parts[-1]
    if not inst or "&" in inst or inst.isdigit():
        return ""
    return inst


_RE_COM = re.compile(r"COM\d+", re.I)


def extract_com(name: str) -> str:
    m = _RE_COM.search(name or "")
    return m.group(0).upper() if m else ""


def clean_name(s: str) -> str:
    t = (s or "").strip()
    if len(t) > 2 and t.startswith("(") and t.endswith(")"):
        t = t[1:-1].strip()
    # Windows 有时给的名字里带连续空格（"DeTong  DP27"），收成一个
    return re.sub(r"\s{2,}", " ", t)


def classify(cls: str, service: str, name: str, desc: str) -> str:
    c = (cls or "").lower()
    s = (service or "").lower()
    n = f"{name or ''} {desc or ''}".lower()

    if c == "ports":
        return "serial"
    if "usbstor" in s:
        return "storage"
    if c == "hidclass" or "hid" in n:
        return "hid"
    if c == "net":
        return "network"
    if c in ("media", "audioendpoint"):
        return "audio"
    if c in ("image", "camera"):
        return "camera"
    if c == "printer" or s == "usbprint":
        return "printer"
    if s == "bthusb" or c == "bluetooth" or "bluetooth" in n or "蓝牙" in n:
        return "bluetooth"
    if c in ("smartcardreader", "smartcard"):
        return "smartcard"
    if c == "androidusbdeviceclass" or "android" in n or "adb" in n:
        return "android"
    if s.startswith("usbhub") or "hub" in n or "集线器" in n:
        return "hub"
    if c == "modem":
        return "modem"
    if "keyboard" in n or "键盘" in n or "mouse" in n or "鼠标" in n:
        return "hid"
    if c == "usb" or s == "usbccgp":
        return "composite"
    return "generic"


# ---------------------------------------------------------------- 枚举


@dataclass
class _Raw:
    instance_id: str
    desc: str = ""
    friendly: str = ""
    cls: str = ""
    service: str = ""
    mfg: str = ""
    location: str = ""


def _enum_ids(enumerator: str = "USB", present_only: bool = True) -> list[str]:
    flags = CM_GETIDLIST_FILTER_ENUMERATOR
    if present_only:
        flags |= CM_GETIDLIST_FILTER_PRESENT

    for _ in range(3):
        size = wintypes.ULONG(0)
        if _cfg.CM_Get_Device_ID_List_SizeW(
            ctypes.byref(size), enumerator, flags
        ) != CR_SUCCESS or size.value == 0:
            return []

        buf = ctypes.create_string_buffer(size.value * 2)
        cr = _cfg.CM_Get_Device_ID_ListW(
            enumerator, buf, size.value, flags
        )
        if cr == CR_BUFFER_SMALL:
            continue
        if cr != CR_SUCCESS:
            return []

        txt = buf.raw.decode("utf-16-le", "ignore")
        return [s for s in txt.split("\x00") if s]
    return []


def _read_prop(dn: int, prop: int) -> str:
    size = wintypes.ULONG(512)
    rtype = wintypes.ULONG(0)
    buf = ctypes.create_string_buffer(512)

    cr = _cfg.CM_Get_DevNode_Registry_PropertyW(
        dn, prop, ctypes.byref(rtype), buf, ctypes.byref(size), 0
    )
    if cr == CR_BUFFER_SMALL:
        n = max(size.value, 8)
        buf = ctypes.create_string_buffer(n)
        size = wintypes.ULONG(n)
        cr = _cfg.CM_Get_DevNode_Registry_PropertyW(
            dn, prop, ctypes.byref(rtype), buf, ctypes.byref(size), 0
        )
    if cr != CR_SUCCESS or size.value == 0:
        return ""

    raw = buf.raw[: size.value]
    if rtype.value in (1, 2, 7):  # REG_SZ / EXPAND_SZ / MULTI_SZ
        return raw.decode("utf-16-le", "ignore").split("\x00")[0].strip()
    if rtype.value == 4:  # REG_DWORD
        return str(int.from_bytes(raw[:4], "little"))
    return ""


def _read_instance(instance_id: str) -> _Raw | None:
    dn = wintypes.ULONG(0)
    if (
        _cfg.CM_Locate_DevNodeW(ctypes.byref(dn), instance_id, 0)
        != CR_SUCCESS
    ):
        return None
    return _Raw(
        instance_id=instance_id,
        desc=_read_prop(dn, DRP_DEVICEDESC),
        friendly=_read_prop(dn, DRP_FRIENDLYNAME),
        cls=_read_prop(dn, DRP_CLASS),
        service=_read_prop(dn, DRP_SERVICE),
        mfg=_read_prop(dn, DRP_MFG),
        location=_read_prop(dn, DRP_LOCATION_INFORMATION),
    )


def _score(r: _Raw) -> int:
    c = (r.cls or "").lower()
    s = (r.service or "").lower()
    base = {
        "ports": 100,
        "hidclass": 90,
        "net": 85,
        "media": 80,
        "audioendpoint": 80,
        "image": 78,
        "printer": 76,
        "smartcardreader": 70,
        "androidusbdeviceclass": 68,
        "modem": 60,
        "usb": 10,
    }.get(c, 40)

    # 有些功能接口的 class 是通用的（比如 usbprint 挂在 class=USB 下），
    # 只看 class 会让 HID 之类的接口抢走“主接口”位置，所以服务名也要算分。
    svc = {
        "usbprint": 96,
        "usbvideo": 88,
        "usbccgp": 0,
        "usbhub3": 0,
    }.get(s, 60 if s else 0)

    score = max(base, svc)
    if r.friendly:
        score += 20
    if r.desc:
        score += 5
    return score


def is_vague(s: str) -> bool:
    t = clean_name(s or "")
    if len(t) <= 3:
        return True
    return t.lower() in VAGUE_NAMES


_RE_MI = re.compile(r"&MI_([0-9A-Fa-f]{2})")
_RE_COMPAREN = re.compile(r"\s*[\(（]\s*COM\d+\s*[\)）]\s*$", re.I)


def strip_com_paren(s: str) -> str:
    return _RE_COMPAREN.sub("", s or "").strip()


def _merge(key: str, group: list[_Raw]) -> Device:
    group = sorted(group, key=_score, reverse=True)
    best = group[0]
    parent = next(
        (r for r in group if "&MI_" not in device_id_of(r.instance_id)), best
    )

    base = base_of(device_id_of(best.instance_id))
    vid, pid = parse_vid_pid(device_id_of(best.instance_id))

    # 收集各功能接口
    ifaces: list[Iface] = []
    for r in group:
        m = _RE_MI.search(device_id_of(r.instance_id))
        if not m:
            continue
        com = extract_com(r.friendly) or extract_com(r.desc)
        ifaces.append(
            Iface(
                mi=int(m.group(1), 16),
                tag=iface_tag(r.cls, r.service, com),
                name=clean_name(r.friendly) or clean_name(r.desc),
                cls=r.cls,
                service=r.service,
                com=com,
            )
        )
    ifaces.sort(key=lambda i: i.mi)

    tags: list[str] = []
    for i in ifaces:
        if i.tag not in tags:
            tags.append(i.tag)
    # 只有真的存在多种功能才算复合设备
    composite = len(tags) >= 2

    # 名字从具体到笼统依次尝试，跳过 Windows 的通用名
    raw_name = ""
    for cand in (best.friendly, best.desc, parent.friendly, parent.desc):
        if cand and not is_vague(cand):
            raw_name = cand.strip()
            break

    kind = classify(best.cls, best.service, raw_name, parent.desc)
    name = clean_name(raw_name) or KIND_DEFAULT.get(kind, "USB 设备")
    if composite:
        name = strip_com_paren(name) or name

    com = extract_com(name) or extract_com(best.friendly) or extract_com(parent.friendly)
    if not com:
        com = next((i.com for i in ifaces if i.com), "")
    mfg = clean_name(best.mfg.strip() or parent.mfg.strip())
    if is_vague(mfg):
        mfg = ""
    location = parent.location.strip() or best.location.strip()

    # 复合设备的“设备类/服务”用父节点的，单接口设备的用该接口的
    if composite:
        cls = parent.cls or best.cls
        service = parent.service or best.service
    else:
        cls = best.cls or parent.cls
        service = best.service

    return Device(
        key=key,
        base=base,
        name=name,
        desc=parent.desc.strip() or best.desc.strip(),
        cls=cls,
        service=service,
        mfg=mfg,
        location=location,
        instance_id=parent.instance_id,
        vid=vid,
        pid=pid,
        serial=parse_serial(parent.instance_id),
        com=com,
        kind=kind,
        ifaces=ifaces,
        composite=composite,
    )


# 实例属性缓存：只在设备表变化时补读新增项，轮询开销很小
_CACHE: dict[str, _Raw] = {}


def scan(present_only: bool = True) -> dict[str, Device] | None:
    """枚举当前 USB 设备。返回 None 表示这次枚举失败（调用方应忽略，别当成“全拔了”）。"""
    ids = _enum_ids("USB", present_only)
    if not ids:
        return None

    alive = set(ids)
    for k in [k for k in _CACHE if k not in alive]:
        del _CACHE[k]

    groups: dict[str, list[_Raw]] = {}
    for iid in ids:
        raw = _CACHE.get(iid)
        if raw is None:
            raw = _read_instance(iid)
            if raw is None:
                continue
            _CACHE[iid] = raw
        groups.setdefault(group_key(iid), []).append(raw)

    out: dict[str, Device] = {}
    for k, g in groups.items():
        dev = _merge(k, g)
        if not is_real_usb(k, dev):
            continue
        out[k] = dev
    return out


def is_real_usb(key: str, dev: Device) -> bool:
    """排除根集线器和没有 VID/PID 的东西——那些不是“插上去的 USB 设备”。"""
    up = key.upper()
    if "ROOT_HUB" in up:
        return False
    if dev.vid == "----" or dev.pid == "----":
        return False
    return True


def noteworthy(d: Device) -> bool:
    """是否值得弹窗（根集线器之类已经不在列表里了，这里再兜一层）"""
    if d.vid == "----" and d.pid == "----":
        return False
    n = (d.name or "").lower()
    if "root hub" in n or "根集线器" in n:
        return False
    return True

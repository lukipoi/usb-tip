"""离屏渲染面板/通知，用来检查排版（不打扰桌面）。

用法：
    python tools/preview.py        # 在当前目录生成 shot_panel.png 等预览图
"""

import os
import sys

os.environ["QT_QPA_PLATFORM"] = "offscreen"

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(_HERE, "..", "py"))

from PySide6.QtGui import QFontDatabase  # noqa: E402
from PySide6.QtWidgets import QApplication  # noqa: E402

import theme  # noqa: E402
import usbm  # noqa: E402
from panelwin import PanelWindow  # noqa: E402
from toastwin import ToastWindow  # noqa: E402

app = QApplication(sys.argv)
for _f in (
    "C:/Windows/Fonts/msyh.ttc",
    "C:/Windows/Fonts/msyhbd.ttc",
    "C:/Windows/Fonts/consola.ttf",
    "C:/Windows/Fonts/CascadiaMono.ttf",
    "C:/Windows/Fonts/CascadiaCode.ttf",
):
    print(_f, QFontDatabase.addApplicationFont(_f))
print("families", len(QFontDatabase.families()))
app.setStyle("Fusion")
app.setFont(theme.font(12))
app.setStyleSheet(theme.QSS)

devs = usbm.scan() or {}
items = sorted(devs.values(), key=lambda d: (0 if d.present else 1, d.name))
items.append(
    usbm.Device(
        key="fake1",
        name="USB-SERIAL CH340",
        vid="1A86",
        pid="7523",
        kind="serial",
        com="COM17",
        mfg="wch.cn",
        cls="Ports",
        location="Port_#0003.Hub_#0001",
        instance_id="USB\\VID_1A86&PID_7523\\5&2f3a&0&1",
        serial="",
        present=False,
        plugs=3,
    )
)
items.append(
    usbm.Device(
        key="fake2",
        name="USB Mass Storage Device",
        vid="0781",
        pid="5567",
        kind="storage",
        mfg="SanDisk",
        cls="USB",
        location="Port_#0002.Hub_#0001",
        instance_id="USB\\VID_0781&PID_5567\\4C530001",
        serial="4C530001",
        present=True,
        plugs=1,
    )
)

comp = next((d for d in items if d.composite), None)
if comp is None:
    comp = usbm.Device(
        key="fake3",
        name="WCH-Link SERIAL",
        vid="1A86",
        pid="8010",
        kind="composite",
        composite=True,
        ifaces=[
            usbm.Iface(0, "WCH 调试", "", "WCH", "WCHLink_A64"),
            usbm.Iface(1, "串口 COM5", "WCH-Link SERIAL (COM5)", "Ports", "usbser", "COM5"),
        ],
        mfg="wch.cn",
        cls="USB",
        location="Port_#0008.Hub_#0001",
        instance_id="USB\\VID_1A86&PID_8010\\7FB68F060553",
        serial="7FB68F060553",
        present=True,
    )
    items.append(comp)

prn = next((d for d in items if d.pid == "5A11"), None) or comp

p = PanelWindow()
p.set_devices({d.key: d for d in items}, [d.key for d in items])
p.select(prn.key)
p.show()
app.processEvents()
p.grab().save("shot_panel.png")

t = ToastWindow(prn, True, lambda *_: None, lambda *_: None)
t.move(40, 40)
t.appear()
app.processEvents()
t.grab().save("shot_toast.png")

t2 = ToastWindow(comp, True, lambda *_: None, lambda *_: None)
t2.move(40, 40)
t2.appear()
app.processEvents()
t2.grab().save("shot_toast2.png")
print("ok", p.size(), t.size(), t2.size())

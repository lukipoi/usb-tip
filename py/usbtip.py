"""USB TIP —— 插拔 USB 就在右下角告诉你是啥。

用法：
    python usbtip.py            启动并打开面板
    python usbtip.py --hidden   只驻留托盘，不弹面板
    python usbtip.py --demo     弹一条假通知，用来调样式
"""

from __future__ import annotations

import os
import sys
import time
import traceback

from PySide6.QtCore import QObject, QThread, QTimer, Signal
from PySide6.QtGui import QAction, QGuiApplication
from PySide6.QtWidgets import QApplication, QMenu, QSystemTrayIcon

import icons
import theme
import usbm
from panelwin import PanelWindow
from toastwin import ToastManager
from usbm import Device

APP_NAME = "USB TIP"
LOG = os.path.join(os.environ.get("TEMP", "."), "usbtip-py.log")


def log(msg: str) -> None:
    try:
        with open(LOG, "a", encoding="utf-8") as f:
            f.write(f"{time.strftime('%H:%M:%S')} {msg}\n")
    except Exception:
        pass


# ---------------------------------------------------------------- 监测线程


class Monitor(QThread):
    snapshot = Signal(dict)

    def __init__(self):
        super().__init__()
        self._stop = False

    def run(self):
        last: set[str] = set()
        fails = 0
        while not self._stop:
            try:
                devs = usbm.scan()
                if devs is None:
                    # 枚举失败：保持上一次状态，不要误判成“全部拔出”
                    fails += 1
                    if fails == 1:
                        log("scan returned None (retry)")
                else:
                    if fails:
                        log(f"scan recovered after {fails} fail(s)")
                        fails = 0
                    keys = set(devs)
                    if keys != last:
                        last = keys
                        self.snapshot.emit(devs)
            except Exception:
                log("scan error: " + traceback.format_exc(limit=2))
            for _ in range(4):
                if self._stop:
                    return
                self.msleep(90)

    def stop(self):
        self._stop = True


# ---------------------------------------------------------------- 应用


class App(QObject):
    def __init__(self, qapp: QApplication, hidden: bool):
        super().__init__()
        self.qapp = qapp
        self.devices: dict[str, Device] = {}
        self.history: dict[str, Device] = {}
        self.paused = False
        self._primed = False
        self.toasts = ToastManager(self.open_device)

        self.panel = PanelWindow()
        self.panel.set_devices({}, [])
        self._center_panel()

        self.tray = self._build_tray()

        self.monitor = Monitor()
        self.monitor.snapshot.connect(self._on_snapshot)
        self.monitor.start()

        if not hidden:
            self.panel.show_panel()

    # ---------------- 托盘 ----------------
    def _build_tray(self) -> QSystemTrayIcon | None:
        if not QSystemTrayIcon.isSystemTrayAvailable():
            log("no system tray")
            return None
        tray = QSystemTrayIcon(icons.app_icon(), self)
        tray.setToolTip("USB TIP · USB 插拔监视")

        menu = QMenu()
        menu.setStyleSheet(theme.QSS)

        self.act_open = QAction("打开面板", menu)
        self.act_open.triggered.connect(self.panel.show_panel)
        menu.addAction(self.act_open)

        self.act_pause = QAction("暂停提示", menu)
        self.act_pause.setCheckable(True)
        self.act_pause.toggled.connect(self._set_paused)
        menu.addAction(self.act_pause)

        self.act_auto = QAction("开机自启", menu)
        self.act_auto.setCheckable(True)
        self.act_auto.setChecked(autostart_on())
        self.act_auto.toggled.connect(self._set_autostart)
        menu.addAction(self.act_auto)

        menu.addSeparator()
        act_quit = QAction("退出", menu)
        act_quit.triggered.connect(self.quit)
        menu.addAction(act_quit)

        tray.setContextMenu(menu)
        tray.activated.connect(self._tray_clicked)
        tray.show()
        log("tray created")
        return tray

    def _tray_clicked(self, reason):
        if reason in (
            QSystemTrayIcon.ActivationReason.Trigger,
            QSystemTrayIcon.ActivationReason.DoubleClick,
        ):
            if self.panel.isVisible() and not self.panel.isMinimized():
                self.panel.hide_to_tray()
            else:
                self.panel.show_panel()

    def _set_paused(self, on: bool):
        self.paused = on
        self.act_pause.setText("继续提示" if on else "暂停提示")

    def _set_autostart(self, on: bool):
        set_autostart(on)
        log(f"autostart -> {on}")

    # ---------------- 设备表 ----------------
    def _center_panel(self):
        scr = QGuiApplication.primaryScreen()
        if scr is None:
            return
        g = scr.availableGeometry()
        self.panel.move(
            g.center().x() - self.panel.width() // 2,
            g.center().y() - self.panel.height() // 2,
        )

    def _on_snapshot(self, devs: dict):
        primed = self._primed
        self._primed = True
        now = time.time()

        for key, d in devs.items():
            old = self.history.get(key)

            if old is None:
                # 首次见到
                d.first_seen = now
                d.last_change = 0.0 if not primed else now
                d.plugs = 1
                self.history[key] = d
                if primed and usbm.noteworthy(d) and not self.paused:
                    log(f"plug   {d.name} {d.vid}:{d.pid}")
                    self.toasts.show(d, True)
            else:
                was_present = old.present
                # 刷新字段（名字、接口、COM 都可能变）
                old.present = True
                old.name = d.name
                old.desc = d.desc
                old.com = d.com
                old.location = d.location
                old.cls = d.cls
                old.service = d.service
                old.mfg = d.mfg
                old.serial = d.serial
                old.instance_id = d.instance_id
                old.kind = d.kind
                old.ifaces = d.ifaces
                old.composite = d.composite

                # 之前是拔出状态 -> 这次算重新插入，必须再提示一次
                if not was_present:
                    old.plugs += 1
                    old.last_change = now
                    if primed and usbm.noteworthy(old) and not self.paused:
                        log(f"plug   {old.name} {old.vid}:{old.pid} (again)")
                        self.toasts.show(old, True)

        # 拔出
        for key, d in self.history.items():
            if d.present and key not in devs:
                d.present = False
                d.last_change = now
                if primed and usbm.noteworthy(d) and not self.paused:
                    log(f"unplug {d.name}")
                    self.toasts.show(d, False)

        self._refresh_panel()

    def _refresh_panel(self):
        items = [d for d in self.history.values()]
        items.sort(
            key=lambda d: (
                0 if d.present else 1,
                0 if d.vid != "----" else 1,
                -d.last_change,
            )
        )
        self.devices = {d.key: d for d in items}
        self.panel.set_devices(self.devices, [d.key for d in items])

    def open_device(self, dev: Device):
        self.panel.show_panel()
        self.panel.select(dev.key)

    # ---------------- 退出 ----------------
    def quit(self):
        log("quit")
        self.monitor.stop()
        self.monitor.wait(1500)
        if self.tray is not None:
            self.tray.hide()
        self.qapp.quit()


# ---------------------------------------------------------------- 开机自启

RUN_KEY = r"Software\Microsoft\Windows\CurrentVersion\Run"
VALUE = "UsbTip"


def _launch_cmd() -> str:
    exe = sys.executable or ""
    pyw = os.path.join(os.path.dirname(exe), "pythonw.exe")
    if os.path.exists(pyw):
        exe = pyw
    script = os.path.abspath(__file__)
    return f'"{exe}" "{script}" --hidden'


def autostart_on() -> bool:
    import winreg

    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, RUN_KEY) as k:
            winreg.QueryValueEx(k, VALUE)
            return True
    except OSError:
        return False


def set_autostart(on: bool) -> None:
    import winreg

    try:
        with winreg.CreateKeyEx(
            winreg.HKEY_CURRENT_USER, RUN_KEY, 0, winreg.KEY_SET_VALUE
        ) as k:
            if on:
                winreg.SetValueEx(k, VALUE, 0, winreg.REG_SZ, _launch_cmd())
            else:
                try:
                    winreg.DeleteValue(k, VALUE)
                except OSError:
                    pass
    except OSError:
        log("autostart write failed")


# ---------------------------------------------------------------- 入口


def main() -> int:
    with open(LOG, "w", encoding="utf-8") as f:
        f.write("")

    app = QApplication(sys.argv)
    app.setApplicationName(APP_NAME)
    app.setQuitOnLastWindowClosed(False)
    app.setStyle("Fusion")
    app.setFont(theme.font(12))
    app.setWindowIcon(icons.app_icon())

    hidden = "--hidden" in sys.argv
    tip = App(app, hidden)
    log(f"start hidden={hidden}")

    if "--demo" in sys.argv:
        demo = Device(
            key="demo",
            name="USB-SERIAL CH340",
            vid="1A86",
            pid="7523",
            kind="serial",
            com="COM17",
            mfg="wch.cn",
            cls="Ports",
            present=True,
        )
        QTimer.singleShot(700, lambda: tip.toasts.show(demo, True))

    return app.exec()


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception:
        log("FATAL " + traceback.format_exc())
        raise

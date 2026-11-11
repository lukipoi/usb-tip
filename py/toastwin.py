"""右下角弹出通知（独立无边框透明窗口，带淡入淡出与自动消失）。"""

from __future__ import annotations

from PySide6.QtCore import (
    QEasingCurve,
    QObject,
    QPoint,
    QPropertyAnimation,
    Qt,
    QTimer,
)
from PySide6.QtGui import QColor, QGuiApplication
from PySide6.QtWidgets import QGraphicsDropShadowEffect, QWidget

from toastcard import ToastCard
from usbm import Device

PAD = 16
TTL_MS = 6500
MAX_TOASTS = 3
GAP = 8


class ToastWindow(QWidget):
    def __init__(self, dev: Device, plugged: bool, on_click, on_gone):
        super().__init__(None)
        self.setWindowFlags(
            Qt.WindowType.FramelessWindowHint
            | Qt.WindowType.WindowStaysOnTopHint
            | Qt.WindowType.Tool
            | Qt.WindowType.NoDropShadowWindowHint
        )
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        self.setAttribute(Qt.WidgetAttribute.WA_ShowWithoutActivating)

        self.dev = dev
        self._on_click = on_click
        self._on_gone = on_gone
        self._elapsed = 0.0
        self._paused = False
        self._closing = False

        self.card = ToastCard(dev, plugged, self)
        self.card.move(PAD, PAD)
        self.card.clicked.connect(self._clicked)
        self.card.hover_changed.connect(self._hover)

        self.setFixedSize(self.card.width() + PAD * 2, self.card.height() + PAD * 2)

        sh = QGraphicsDropShadowEffect(self)
        sh.setBlurRadius(34)
        sh.setOffset(0, 7)
        sh.setColor(QColor(0, 0, 0, 165))
        self.card.setGraphicsEffect(sh)

        self._tick = QTimer(self)
        self._tick.setInterval(40)
        self._tick.timeout.connect(self._on_tick)

        self._fade = QPropertyAnimation(self, b"windowOpacity", self)
        self._fade.setDuration(180)
        self._fade.setEasingCurve(QEasingCurve.Type.OutCubic)
        self._fade.finished.connect(self._fade_done)

        self.setWindowOpacity(0.0)

    # ---- 生命周期 ----
    def appear(self) -> None:
        self.show()
        self.raise_()
        self._fade.stop()
        self._fade.setStartValue(0.0)
        self._fade.setEndValue(1.0)
        self._fade.start()
        self._tick.start()

    def dismiss(self) -> None:
        if self._closing:
            return
        self._closing = True
        self._tick.stop()
        self._fade.stop()
        self._fade.setStartValue(self.windowOpacity())
        self._fade.setEndValue(0.0)
        self._fade.start()

    def _fade_done(self) -> None:
        if self._closing:
            self.close()
            self._on_gone(self)

    def _clicked(self) -> None:
        if self._on_click:
            self._on_click(self.dev)
        self.dismiss()

    def _hover(self, on: bool) -> None:
        self._paused = on

    def _on_tick(self) -> None:
        if not self._paused:
            self._elapsed += self._tick.interval()
            self.card.progress = min(1.0, self._elapsed / TTL_MS)
            self.card.update()
            if self.card.progress >= 1.0:
                self.dismiss()

    def move_to(self, x: int, y: int) -> None:
        if self.pos() != QPoint(x, y):
            self.move(x, y)


class ToastManager(QObject):
    """管理叠放的通知窗口。"""

    def __init__(self, on_click):
        super().__init__()
        self._items: list[ToastWindow] = []
        self._on_click = on_click

    def show(self, dev: Device, plugged: bool) -> None:
        # 先清掉已经关掉/正在关闭的，避免它们占着叠放位置
        self._items = [w for w in self._items if not w._closing]

        w = ToastWindow(dev, plugged, self._on_click, self._forget)
        self._items.append(w)

        while len(self._items) > MAX_TOASTS:
            self._items[0].dismiss()
            self._items.pop(0)

        self._layout()
        w.appear()

    def _forget(self, w: ToastWindow) -> None:
        if w in self._items:
            self._items.remove(w)
        self._layout()

    def _layout(self) -> None:
        scr = QGuiApplication.primaryScreen()
        if scr is None:
            return
        g = scr.availableGeometry()
        live = [w for w in self._items if not w._closing]
        # 最新的排在最下面，依次往上叠
        for i, w in enumerate(reversed(live)):
            x = g.right() - w.width() - 6
            y = g.bottom() - w.height() - 6 - i * (w.height() + GAP)
            w.move_to(x, y)

    def clear(self) -> None:
        for w in list(self._items):
            w.dismiss()
        self._items.clear()

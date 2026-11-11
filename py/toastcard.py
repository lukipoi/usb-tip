"""通知卡片本体（可放在独立窗口里，也可作为面板内的浮层）。"""

from __future__ import annotations

from PySide6.QtCore import QRectF, Qt, Signal
from PySide6.QtGui import QColor, QFont, QFontMetrics, QPainter, QPen
from PySide6.QtWidgets import QWidget

import icons
import theme
from usbm import Device, iface_color

CARD_W = 404
CARD_H = 84
TAG_ZONE = 22
FW = QFont.Weight


def card_height(dev: Device) -> int:
    return CARD_H + (TAG_ZONE if dev.composite and dev.ifaces else 0)


class ToastCard(QWidget):
    clicked = Signal()
    hover_changed = Signal(bool)

    def __init__(self, dev: Device, plugged: bool, parent=None):
        super().__init__(parent)
        self.dev = dev
        self.plugged = plugged
        self.progress = 0.0
        self.setFixedSize(CARD_W, card_height(dev))
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.setAttribute(Qt.WidgetAttribute.WA_Hover, True)

    # ---- 交互 ----
    def enterEvent(self, e):
        self.hover_changed.emit(True)
        super().enterEvent(e)

    def leaveEvent(self, e):
        self.hover_changed.emit(False)
        super().leaveEvent(e)

    def mouseReleaseEvent(self, e):
        if e.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit()
        super().mouseReleaseEvent(e)

    # ---- 绘制 ----
    def paintEvent(self, _e):
        p = QPainter(self)
        p.setRenderHint(QPainter.RenderHint.Antialiasing)
        p.setRenderHint(QPainter.RenderHint.TextAntialiasing)

        w, h = self.width(), self.height()
        outer = QRectF(0.5, 0.5, w - 1.0, h - 1.0)

        p.setPen(Qt.PenStyle.NoPen)
        p.setBrush(QColor(0, 0, 0, 0))
        p.drawRect(self.rect())

        p.setBrush(QColor(theme.RAISE))
        p.setPen(QPen(QColor(theme.BORDER), 1.0))
        p.drawRoundedRect(outer, 14.0, 14.0)

        # 顶部一条极淡的高光，让卡片有厚度
        p.setPen(QPen(QColor(255, 255, 255, 14), 1.0))
        p.setBrush(Qt.BrushStyle.NoBrush)
        p.drawRoundedRect(outer.adjusted(1, 1, -1, -1), 13.0, 13.0)

        accent = QColor(dev_color(self.dev))
        # 左侧图标块
        tile = QRectF(14, 20, 44, 44)
        bg = QColor(accent)
        bg.setAlpha(38)
        p.setPen(QPen(QColor(accent.red(), accent.green(), accent.blue(), 80), 1.0))
        p.setBrush(bg)
        p.drawRoundedRect(tile, 12.0, 12.0)
        pm = icons.kind_pixmap(self.dev.kind, 26, dev_color(self.dev))
        p.drawPixmap(int(tile.x() + 9), int(tile.y() + 9), pm)

        # 状态胶囊（右上）
        label = "已插入" if self.plugged else "已拔出"
        col = QColor(theme.OK if self.plugged else theme.BAD)
        fm_pill = QFontMetrics(theme.font(11, FW.DemiBold))
        tw = fm_pill.horizontalAdvance(label)
        pill = QRectF(w - 14 - tw - 20, 16, tw + 20, 22)
        fill = QColor(col)
        fill.setAlpha(30)
        p.setPen(QPen(QColor(col.red(), col.green(), col.blue(), 90), 1.0))
        p.setBrush(fill)
        p.drawRoundedRect(pill, 11.0, 11.0)
        p.setPen(col)
        p.setFont(theme.font(11, FW.DemiBold))
        p.drawText(pill, Qt.AlignmentFlag.AlignCenter, label)

        # 设备名
        text_x = 70
        text_w = int(pill.x() - text_x - 10)
        f_name = theme.font(14, FW.DemiBold)
        fm = QFontMetrics(f_name)
        name = fm.elidedText(self.dev.name or "未知 USB 设备", Qt.TextElideMode.ElideRight, text_w)
        p.setFont(f_name)
        p.setPen(QColor(theme.TEXT))
        p.drawText(
            QRectF(text_x, 20, text_w, 20),
            Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter,
            name,
        )

        # 第二行：类型 + VIDEO/PID
        sub_y = 44
        kind_txt = self.dev.label
        f_sub = theme.font(11)
        fm_sub = QFontMetrics(f_sub)
        p.setFont(f_sub)
        p.setPen(accent)
        p.drawText(
            QRectF(text_x, sub_y, 200, 18),
            Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter,
            kind_txt,
        )
        kx = text_x + fm_sub.horizontalAdvance(kind_txt) + 8
        p.setPen(QColor(theme.FAINT))
        p.drawText(
            QRectF(kx, sub_y, text_w - (kx - text_x), 18),
            Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter,
            self.dev.vid_pid,
        )

        # 复合设备：把各个功能接口列出来（WCH-LinkE 的调试口 + 串口）
        if self.dev.composite and self.dev.ifaces:
            self._draw_tags(p, 64)

        # 底部进度条
        if 0.0 < self.progress < 1.0:
            bar = QRectF(14, h - 5, (w - 28) * (1.0 - self.progress), 2.0)
            c = QColor(accent)
            c.setAlpha(150)
            p.setPen(Qt.PenStyle.NoPen)
            p.setBrush(c)
            p.drawRoundedRect(bar, 1.0, 1.0)

        p.end()

    def _draw_tags(self, p: QPainter, y: float) -> None:
        """一行小胶囊，标出复合设备里的每个功能接口。"""
        seen: list[str] = []
        for i in self.dev.ifaces:
            if i.tag not in seen:
                seen.append(i.tag)
        # 和类型标签重复的那个就不画了（比如打印机：类型已是“打印”）
        trimmed = [t for t in seen if t != self.dev.kind_label]
        if trimmed:
            seen = trimmed

        f = theme.font(10, FW.Medium)
        fm = QFontMetrics(f)
        p.setFont(f)

        x = 70.0
        right = self.width() - 14.0
        for idx, tag in enumerate(seen):
            tw = fm.horizontalAdvance(tag) + 14
            if x + tw > right:
                p.setPen(QColor(theme.FAINT))
                p.drawText(
                    QRectF(x, y, right - x, 18),
                    Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter,
                    f"+{len(seen) - idx}",
                )
                return
            col = QColor(iface_color(tag))
            bg = QColor(col)
            bg.setAlpha(24)
            box = QRectF(x, y, tw, 18)
            p.setPen(QPen(QColor(col.red(), col.green(), col.blue(), 85), 1.0))
            p.setBrush(bg)
            p.drawRoundedRect(box, 9.0, 9.0)
            p.setPen(col)
            p.drawText(box, Qt.AlignmentFlag.AlignCenter, tag)
            x += tw + 5


def dev_color(dev: Device) -> str:
    from usbm import KIND_COLOR

    return KIND_COLOR.get(dev.kind, KIND_COLOR["generic"])
"""矢量图标：设备类型图标、应用/托盘图标。全部用 QPainter 画，无外部资源。"""

from __future__ import annotations

from functools import lru_cache

from PySide6.QtCore import QPointF, QRectF, Qt
from PySide6.QtGui import (
    QColor,
    QGuiApplication,
    QIcon,
    QLinearGradient,
    QPainter,
    QPainterPath,
    QPen,
    QPixmap,
)


def _p(r: QRectF, x: float, y: float) -> QPointF:
    return QPointF(r.x() + x * r.width(), r.y() + y * r.height())


def _pen(p: QPainter, color: QColor, w: float) -> QPen:
    pen = QPen(color, w)
    pen.setCapStyle(Qt.PenCapStyle.RoundCap)
    pen.setJoinStyle(Qt.PenJoinStyle.RoundJoin)
    p.setPen(pen)
    p.setBrush(Qt.BrushStyle.NoBrush)
    return pen


def _arrow(p: QPainter, a: QPointF, b: QPointF, s: float) -> None:
    import math

    p.drawLine(a, b)
    ang = math.atan2(b.y() - a.y(), b.x() - a.x())
    for d in (2.6, -2.6):
        p.drawLine(
            b,
            QPointF(
                b.x() + s * math.cos(ang + d),
                b.y() + s * math.sin(ang + d),
            ),
        )


# ---------------------------------------------------------------- 类型图标


def _serial(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    u = r.width()
    _arrow(p, _p(r, 0.14, 0.33), _p(r, 0.80, 0.33), u * 0.14)
    _arrow(p, _p(r, 0.86, 0.67), _p(r, 0.20, 0.67), u * 0.14)


def _storage(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    p.drawRoundedRect(
        QRectF(_p(r, 0.28, 0.34), _p(r, 0.72, 0.88)), r.width() * 0.09, r.width() * 0.09
    )
    p.drawRect(QRectF(_p(r, 0.40, 0.12), _p(r, 0.60, 0.34)))
    p.setBrush(c)
    p.setPen(Qt.PenStyle.NoPen)
    p.drawEllipse(_p(r, 0.5, 0.52), w * 0.85, w * 0.85)
    p.drawLine(_p(r, 0.5, 0.62), _p(r, 0.5, 0.78))


def _hid(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    p.drawRoundedRect(
        QRectF(_p(r, 0.08, 0.30), _p(r, 0.92, 0.72)), r.width() * 0.08, r.width() * 0.08
    )
    p.setBrush(c)
    p.setPen(Qt.PenStyle.NoPen)
    for dx in (0.22, 0.38, 0.54, 0.70):
        p.drawEllipse(_p(r, dx, 0.44), w * 0.6, w * 0.6)
    p.setPen(QPen(c, w * 0.9))
    p.drawLine(_p(r, 0.30, 0.60), _p(r, 0.70, 0.60))


def _network(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    cx, cy = 0.5, 0.80
    for rad in (0.20, 0.42, 0.64):
        d = rad
        box = QRectF(
            _p(r, cx - d, cy - d * 0.92),
            _p(r, cx + d, cy + d * 0.92),
        )
        p.drawArc(box, 45 * 16, 90 * 16)
    p.setBrush(c)
    p.setPen(Qt.PenStyle.NoPen)
    p.drawEllipse(_p(r, cx, cy - 0.03), w * 0.75, w * 0.75)


def _audio(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    path = QPainterPath(_p(r, 0.12, 0.40))
    path.lineTo(_p(r, 0.30, 0.40))
    path.lineTo(_p(r, 0.52, 0.20))
    path.lineTo(_p(r, 0.52, 0.80))
    path.lineTo(_p(r, 0.30, 0.60))
    path.lineTo(_p(r, 0.12, 0.60))
    path.closeSubpath()
    p.drawPath(path)
    for rad in (0.20, 0.36):
        box = QRectF(_p(r, 0.62 - rad, 0.5 - rad), _p(r, 0.62 + rad, 0.5 + rad))
        p.drawArc(box, -60 * 16, 120 * 16)


def _camera(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    p.drawRoundedRect(
        QRectF(_p(r, 0.06, 0.28), _p(r, 0.94, 0.80)), r.width() * 0.10, r.width() * 0.10
    )
    p.drawRoundedRect(
        QRectF(_p(r, 0.34, 0.16), _p(r, 0.62, 0.30)), r.width() * 0.04, r.width() * 0.04
    )
    p.drawEllipse(_p(r, 0.5, 0.55), r.width() * 0.17, r.width() * 0.17)


def _printer(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    p.drawRect(QRectF(_p(r, 0.28, 0.12), _p(r, 0.72, 0.34)))
    p.drawRoundedRect(
        QRectF(_p(r, 0.12, 0.34), _p(r, 0.88, 0.72)), r.width() * 0.07, r.width() * 0.07
    )
    p.drawRect(QRectF(_p(r, 0.30, 0.66), _p(r, 0.70, 0.88)))
    p.setBrush(c)
    p.setPen(Qt.PenStyle.NoPen)
    p.drawEllipse(_p(r, 0.76, 0.46), w * 0.55, w * 0.55)


def _bluetooth(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    path = QPainterPath(_p(r, 0.50, 0.14))
    path.lineTo(_p(r, 0.50, 0.86))
    path.moveTo(_p(r, 0.50, 0.14))
    path.lineTo(_p(r, 0.74, 0.34))
    path.lineTo(_p(r, 0.26, 0.66))
    path.moveTo(_p(r, 0.50, 0.86))
    path.lineTo(_p(r, 0.74, 0.66))
    path.lineTo(_p(r, 0.26, 0.34))
    p.drawPath(path)


def _smartcard(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    p.drawRoundedRect(
        QRectF(_p(r, 0.10, 0.20), _p(r, 0.90, 0.80)), r.width() * 0.09, r.width() * 0.09
    )
    p.setBrush(c)
    p.setPen(Qt.PenStyle.NoPen)
    p.drawRoundedRect(
        QRectF(_p(r, 0.24, 0.34), _p(r, 0.52, 0.66)),
        r.width() * 0.04,
        r.width() * 0.04,
    )
    p.setPen(QPen(c, w * 0.9))
    p.drawLine(_p(r, 0.62, 0.40), _p(r, 0.78, 0.40))
    p.drawLine(_p(r, 0.62, 0.52), _p(r, 0.78, 0.52))


def _android(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    box = QRectF(_p(r, 0.16, 0.26), _p(r, 0.84, 0.94))
    p.drawArc(box, 0, 180 * 16)
    p.drawLine(_p(r, 0.16, 0.60), _p(r, 0.84, 0.60))
    p.drawLine(_p(r, 0.30, 0.28), _p(r, 0.22, 0.12))
    p.drawLine(_p(r, 0.70, 0.28), _p(r, 0.78, 0.12))
    p.setBrush(c)
    p.setPen(Qt.PenStyle.NoPen)
    p.drawEllipse(_p(r, 0.36, 0.46), w * 0.6, w * 0.6)
    p.drawEllipse(_p(r, 0.64, 0.46), w * 0.6, w * 0.6)


def _hub(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    path = QPainterPath(_p(r, 0.50, 0.14))
    path.lineTo(_p(r, 0.50, 0.48))
    path.moveTo(_p(r, 0.16, 0.48))
    path.lineTo(_p(r, 0.84, 0.48))
    for x in (0.16, 0.50, 0.84):
        path.moveTo(_p(r, x, 0.48))
        path.lineTo(_p(r, x, 0.72))
    p.drawPath(path)
    p.setBrush(c)
    p.setPen(Qt.PenStyle.NoPen)
    for x in (0.16, 0.50, 0.84):
        p.drawEllipse(_p(r, x, 0.78), w * 0.8, w * 0.8)


def _modem(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    path = QPainterPath(_p(r, 0.10, 0.50))
    path.cubicTo(_p(r, 0.30, 0.16), _p(r, 0.42, 0.16), _p(r, 0.50, 0.50))
    path.cubicTo(_p(r, 0.58, 0.84), _p(r, 0.70, 0.84), _p(r, 0.90, 0.50))
    p.drawPath(path)


def _composite(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    d = r.width() * 0.055
    p.drawRoundedRect(QRectF(_p(r, 0.12, 0.12), _p(r, 0.46, 0.46)), d, d)
    p.drawRoundedRect(QRectF(_p(r, 0.54, 0.12), _p(r, 0.88, 0.46)), d, d)
    p.drawRoundedRect(QRectF(_p(r, 0.12, 0.54), _p(r, 0.46, 0.88)), d, d)
    p.setBrush(c)
    p.setPen(Qt.PenStyle.NoPen)
    p.drawRoundedRect(QRectF(_p(r, 0.64, 0.64), _p(r, 0.78, 0.78)), d, d)


def _generic(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    _pen(p, c, w)
    p.drawEllipse(_p(r, 0.5, 0.5), r.width() * 0.30, r.width() * 0.30)
    p.setBrush(c)
    p.setPen(Qt.PenStyle.NoPen)
    p.drawEllipse(_p(r, 0.5, 0.5), w * 0.8, w * 0.8)


_PAINTERS = {
    "serial": _serial,
    "storage": _storage,
    "hid": _hid,
    "network": _network,
    "audio": _audio,
    "camera": _camera,
    "printer": _printer,
    "bluetooth": _bluetooth,
    "smartcard": _smartcard,
    "android": _android,
    "hub": _hub,
    "modem": _modem,
    "composite": _composite,
    "generic": _generic,
}


def draw_kind(p: QPainter, rect: QRectF, kind: str, color: str, scale: float = 1.0) -> None:
    fn = _PAINTERS.get(kind, _generic)
    c = QColor(color)
    fn(p, rect, c, max(1.2, rect.width() * 0.095 * scale))


@lru_cache(maxsize=256)
def kind_pixmap(kind: str, size: int, color: str) -> QPixmap:
    dpr = QGuiApplication.primaryScreen().devicePixelRatio()
    pm = QPixmap(int(size * dpr), int(size * dpr))
    pm.fill(Qt.GlobalColor.transparent)
    pm.setDevicePixelRatio(dpr)
    p = QPainter(pm)
    p.setRenderHint(QPainter.RenderHint.Antialiasing)
    draw_kind(p, QRectF(0, 0, size, size), kind, color)
    p.end()
    return pm


# ---------------------------------------------------------------- 应用图标


def _trident(p: QPainter, r: QRectF, c: QColor, w: float) -> None:
    pen = QPen(c, w)
    pen.setCapStyle(Qt.PenCapStyle.RoundCap)
    pen.setJoinStyle(Qt.PenJoinStyle.RoundJoin)
    p.setPen(pen)
    p.setBrush(Qt.BrushStyle.NoBrush)

    p.drawLine(_p(r, 0.50, 0.16), _p(r, 0.50, 0.84))
    path = QPainterPath(_p(r, 0.50, 0.58))
    path.cubicTo(_p(r, 0.50, 0.40), _p(r, 0.28, 0.44), _p(r, 0.26, 0.30))
    p.drawPath(path)
    path2 = QPainterPath(_p(r, 0.50, 0.50))
    path2.cubicTo(_p(r, 0.50, 0.34), _p(r, 0.74, 0.38), _p(r, 0.74, 0.24))
    p.drawPath(path2)

    p.setBrush(c)
    p.setPen(Qt.PenStyle.NoPen)
    p.drawEllipse(_p(r, 0.50, 0.12), w * 1.05, w * 1.05)
    p.drawEllipse(_p(r, 0.74, 0.20), w * 0.85, w * 0.85)
    p.drawRect(
        QRectF(
            _p(r, 0.26, 0.28).x() - w * 0.85,
            _p(r, 0.26, 0.28).y() - w * 0.85,
            w * 1.7,
            w * 1.7,
        )
    )
    p.setBrush(Qt.BrushStyle.NoBrush)
    p.setPen(pen)
    p.drawLine(_p(r, 0.38, 0.84), _p(r, 0.62, 0.84))


def app_pixmap(size: int) -> QPixmap:
    dpr = QGuiApplication.primaryScreen().devicePixelRatio()
    pm = QPixmap(int(size * dpr), int(size * dpr))
    pm.fill(Qt.GlobalColor.transparent)
    pm.setDevicePixelRatio(dpr)
    p = QPainter(pm)
    p.setRenderHint(QPainter.RenderHint.Antialiasing)

    r = QRectF(0.5, 0.5, size - 1.0, size - 1.0)
    grad = QLinearGradient(r.topLeft(), r.bottomRight())
    grad.setColorAt(0.0, QColor("#4F8CFF"))
    grad.setColorAt(1.0, QColor("#8B5CF6"))
    p.setPen(Qt.PenStyle.NoPen)
    p.setBrush(grad)
    p.drawRoundedRect(r, size * 0.24, size * 0.24)

    inset = size * 0.20
    _trident(
        p,
        QRectF(inset, inset, size - 2 * inset, size - 2 * inset),
        QColor(255, 255, 255, 245),
        max(1.0, size * 0.045),
    )
    p.end()
    return pm


def app_icon() -> QIcon:
    ic = QIcon()
    for s in (16, 24, 32, 48, 64, 128, 256):
        ic.addPixmap(app_pixmap(s))
    return ic

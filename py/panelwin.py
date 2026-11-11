"""主面板：设备列表 + 详情。"""

from __future__ import annotations

from PySide6.QtCore import QPoint, QRectF, QSize, Qt, Signal
from PySide6.QtGui import (
    QColor,
    QFont,
    QFontMetrics,
    QGuiApplication,
    QPainter,
    QPen,
    QPixmap,
)
from PySide6.QtWidgets import (
    QButtonGroup,
    QFrame,
    QGraphicsDropShadowEffect,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QPushButton,
    QScrollArea,
    QSizePolicy,
    QVBoxLayout,
    QWidget,
)

import icons
import theme
from usbm import KIND_COLOR, Device, iface_color

PANEL_W = 924
PANEL_H = 564
PAD = 18
TITLE_H = 46
LEFT_W = 300
ROW_H = 58
FW = QFont.Weight


def _magnifier(size: int = 14) -> QPixmap:
    dpr = QGuiApplication.primaryScreen().devicePixelRatio()
    pm = QPixmap(int(size * dpr), int(size * dpr))
    pm.fill(Qt.GlobalColor.transparent)
    pm.setDevicePixelRatio(dpr)
    p = QPainter(pm)
    p.setRenderHint(QPainter.RenderHint.Antialiasing)
    pen = QPen(QColor(theme.FAINT), 1.4)
    pen.setCapStyle(Qt.PenCapStyle.RoundCap)
    p.setPen(pen)
    p.setBrush(Qt.BrushStyle.NoBrush)
    p.drawEllipse(QRectF(2.0, 2.0, size - 7.0, size - 7.0))
    p.drawLine(
        QPoint(int(size - 5), int(size - 5)),
        QPoint(int(size - 2), int(size - 2)),
    )
    p.end()
    return pm


# --------------------------------------------------------------- 标题栏


class TitleBar(QWidget):
    def __init__(self, panel: "PanelWindow"):
        super().__init__(panel)
        self.panel = panel
        self.setFixedHeight(TITLE_H)

        lay = QHBoxLayout(self)
        lay.setContentsMargins(16, 0, 10, 0)
        lay.setSpacing(8)

        logo = QLabel(self)
        logo.setPixmap(icons.app_pixmap(20))
        logo.setFixedSize(20, 20)
        lay.addWidget(logo)

        t = QLabel("USB TIP", self)
        t.setObjectName("hdr")
        lay.addWidget(t)

        dot = QLabel("|", self)
        dot.setObjectName("hdrSub")
        lay.addWidget(dot)

        sub = QLabel("USB 插拔监视", self)
        sub.setObjectName("hdrSub")
        lay.addWidget(sub)

        lay.addStretch(1)

        self.counter = QLabel("", self)
        self.counter.setObjectName("counter")
        lay.addWidget(self.counter)
        lay.addSpacing(6)

        b_min = QPushButton("–", self)
        b_min.setObjectName("tbtn")
        b_min.setFixedSize(30, 30)
        b_min.setCursor(Qt.CursorShape.PointingHandCursor)
        b_min.clicked.connect(panel.showMinimized)
        lay.addWidget(b_min)

        b_close = QPushButton("×", self)
        b_close.setObjectName("close")
        b_close.setStyleSheet(
            "QPushButton{background:transparent;border:none;border-radius:8px;"
            f"color:{theme.DIM};font-size:17px;}}"
            "QPushButton:hover{background:#E5484D;color:#FFFFFF;}"
        )
        b_close.setFixedSize(30, 30)
        b_close.setCursor(Qt.CursorShape.PointingHandCursor)
        b_close.clicked.connect(panel.hide_to_tray)
        lay.addWidget(b_close)

    # 拖动 / 双击最小化
    def mousePressEvent(self, e):
        if e.button() == Qt.MouseButton.LeftButton:
            wh = self.window().windowHandle()
            if wh is not None:
                wh.startSystemMove()
                return
        super().mousePressEvent(e)

    def mouseDoubleClickEvent(self, e):
        self.window().showMinimized()


# --------------------------------------------------------------- 列表行


class DeviceRow(QFrame):
    clicked = Signal(str)

    def __init__(self, dev: Device, parent=None):
        super().__init__(parent)
        self.dev = dev
        self.setObjectName("row")
        self.setFixedHeight(ROW_H)
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.setProperty("sel", "false")

        lay = QHBoxLayout(self)
        lay.setContentsMargins(10, 0, 12, 0)
        lay.setSpacing(10)

        ic = QLabel(self)
        ic.setFixedSize(26, 26)
        ic.setPixmap(icons.kind_pixmap(dev.kind, 24, dev_color(dev)))
        lay.addWidget(ic)

        box = QVBoxLayout()
        box.setContentsMargins(0, 0, 0, 0)
        box.setSpacing(1)

        name = QLabel(self)
        name.setObjectName("name")
        fm = QFontMetrics(theme.font(13, FW.DemiBold))
        name.setText(fm.elidedText(dev.name, Qt.TextElideMode.ElideRight, LEFT_W - 110))
        box.addWidget(name)

        sub = QLabel(self)
        sub.setObjectName("sub")
        bits = [dev.label]
        if dev.com:
            bits.append(dev.com)
        if dev.vid != "----":
            bits.append(f"{dev.vid}:{dev.pid}")
        sub.setText(" · ".join(bits))
        box.addWidget(sub)
        lay.addLayout(box, 1)

        dot = QLabel("●", self)
        dot.setStyleSheet(
            f"color:{theme.OK if dev.present else theme.BAD};font-size:9px;"
        )
        lay.addWidget(dot)

    def set_selected(self, on: bool):
        self.setProperty("sel", "true" if on else "false")
        self.style().unpolish(self)
        self.style().polish(self)

    def mouseReleaseEvent(self, e):
        if e.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit(self.dev.key)
        super().mouseReleaseEvent(e)


def dev_color(dev: Device) -> str:
    return KIND_COLOR.get(dev.kind, KIND_COLOR["generic"])


class _ShadowLayer(QWidget):
    """只画一块不透明圆角矩形，交给阴影特效去模糊；被上面的卡片完全盖住。"""

    def paintEvent(self, _e):
        p = QPainter(self)
        p.setRenderHint(QPainter.RenderHint.Antialiasing)
        p.setPen(Qt.PenStyle.NoPen)
        p.setBrush(QColor(theme.BG))
        p.drawRoundedRect(QRectF(self.rect()).adjusted(0, 0, -1, -1), 16.0, 16.0)
        p.end()


# --------------------------------------------------------------- 面板


class PanelWindow(QWidget):
    def __init__(self):
        super().__init__(None)
        self.setWindowFlags(Qt.WindowType.Window | Qt.WindowType.FramelessWindowHint)
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        self.setWindowTitle("USB TIP")
        self.setWindowIcon(icons.app_icon())
        self.setFixedSize(PANEL_W + PAD * 2, PANEL_H + PAD * 2)

        self.devices: dict[str, Device] = {}
        self.order: list[str] = []
        self.selected: str | None = None
        self.filter = "all"
        self._rows: dict[str, DeviceRow] = {}
        self._moved = False

        # 阴影层（单独一层，保证卡片内文字锐利）
        self._shadow = _ShadowLayer(self)
        self._shadow.setGeometry(PAD, PAD, PANEL_W, PANEL_H)
        self._shadow.setAttribute(Qt.WidgetAttribute.WA_TransparentForMouseEvents)
        sh = QGraphicsDropShadowEffect(self._shadow)
        sh.setBlurRadius(46)
        sh.setOffset(0, 10)
        sh.setColor(QColor(0, 0, 0, 185))
        self._shadow.setGraphicsEffect(sh)

        self._card = QFrame(self)
        self._card.setObjectName("card")
        self._card.setGeometry(PAD, PAD, PANEL_W, PANEL_H)

        root = QVBoxLayout(self._card)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(0)

        self.title = TitleBar(self)
        root.addWidget(self.title)

        line = QFrame(self._card)
        line.setObjectName("sep")
        line.setFixedHeight(1)
        root.addWidget(line)

        body = QWidget(self._card)
        root.addWidget(body, 1)
        bl = QHBoxLayout(body)
        bl.setContentsMargins(16, 12, 16, 14)
        bl.setSpacing(14)

        bl.addWidget(self._build_left(body))

        vsep = QFrame(body)
        vsep.setObjectName("sep")
        vsep.setFixedWidth(1)
        bl.addWidget(vsep)

        bl.addWidget(self._build_right(body), 1)

        self._apply_style()
        self._show_empty()

    # ------------------------------------------------ 左侧
    def _build_left(self, parent) -> QWidget:
        box = QWidget(parent)
        box.setFixedWidth(LEFT_W)
        lay = QVBoxLayout(box)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(10)

        self.search = QLineEdit(box)
        self.search.setPlaceholderText("搜索名称 / VID / PID / 串口")
        self.search.setClearButtonEnabled(True)
        self.search.addAction(
            QIcon_from(_magnifier(14)), QLineEdit.ActionPosition.LeadingPosition
        )
        self.search.textChanged.connect(self._refresh_list)
        lay.addWidget(self.search)

        chips = QWidget(box)
        chl = QHBoxLayout(chips)
        chl.setContentsMargins(0, 0, 0, 0)
        chl.setSpacing(6)
        self._chip_group = QButtonGroup(self)
        self._chip_group.setExclusive(True)
        for key, text in (("all", "全部"), ("on", "在线"), ("off", "已拔出")):
            b = QPushButton(text, chips)
            b.setObjectName("chip")
            b.setCheckable(True)
            b.setCursor(Qt.CursorShape.PointingHandCursor)
            b.setChecked(key == "all")
            b.clicked.connect(lambda _=False, k=key: self._set_filter(k))
            self._chip_group.addButton(b)
            chl.addWidget(b)
        chl.addStretch(1)
        lay.addWidget(chips)

        self.scroll = QScrollArea(box)
        self.scroll.setWidgetResizable(True)
        self.scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.scroll.setFrameShape(QFrame.Shape.NoFrame)
        self.list_host = QWidget()
        self.list_lay = QVBoxLayout(self.list_host)
        self.list_lay.setContentsMargins(0, 0, 4, 0)
        self.list_lay.setSpacing(2)
        self.list_lay.addStretch(1)
        self.scroll.setWidget(self.list_host)
        lay.addWidget(self.scroll, 1)
        return box

    # ------------------------------------------------ 右侧
    def _build_right(self, parent) -> QWidget:
        self._stack = QWidget(parent)
        lay = QVBoxLayout(self._stack)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(0)

        self.empty = QLabel("选择左侧设备查看详细信息", self._stack)
        self.empty.setObjectName("hint")
        self.empty.setAlignment(Qt.AlignmentFlag.AlignCenter)
        lay.addWidget(self.empty)

        self.detail = QWidget(self._stack)
        self.detail.hide()
        dl = QVBoxLayout(self.detail)
        dl.setContentsMargins(0, 0, 0, 0)
        dl.setSpacing(12)

        # 头部
        head = QWidget(self.detail)
        hl = QHBoxLayout(head)
        hl.setContentsMargins(0, 0, 0, 0)
        hl.setSpacing(12)

        self.tile = QFrame(head)
        self.tile.setObjectName("tile")
        self.tile.setFixedSize(60, 60)
        tl = QVBoxLayout(self.tile)
        tl.setContentsMargins(0, 0, 0, 0)
        self.tile_icon = QLabel(self.tile)
        self.tile_icon.setAlignment(Qt.AlignmentFlag.AlignCenter)
        tl.addWidget(self.tile_icon)
        hl.addWidget(self.tile)

        mid = QVBoxLayout()
        mid.setContentsMargins(0, 2, 0, 2)
        mid.setSpacing(5)
        self.d_name = QLabel("", head)
        self.d_name.setObjectName("big")
        mid.addWidget(self.d_name)
        prow = QHBoxLayout()
        prow.setContentsMargins(0, 0, 0, 0)
        prow.setSpacing(8)
        self.d_pill = QLabel("", head)
        prow.addWidget(self.d_pill)
        self.d_state = QLabel("", head)
        self.d_state.setObjectName("sub")
        prow.addWidget(self.d_state)
        prow.addStretch(1)
        mid.addLayout(prow)
        hl.addLayout(mid, 1)

        self.btn_copy = QPushButton("复制信息", head)
        self.btn_copy.setObjectName("act")
        self.btn_copy.setCursor(Qt.CursorShape.PointingHandCursor)
        self.btn_copy.clicked.connect(self._copy)
        hl.addWidget(self.btn_copy, 0, Qt.AlignmentFlag.AlignTop)
        dl.addWidget(head)

        # VID / PID
        vp = QWidget(self.detail)
        vl = QHBoxLayout(vp)
        vl.setContentsMargins(0, 0, 0, 0)
        vl.setSpacing(10)
        self.vid_val, vid_card = self._vc("VID", vp)
        self.pid_val, pid_card = self._vc("PID", vp)
        vl.addWidget(vid_card)
        vl.addWidget(pid_card)
        dl.addWidget(vp)

        sec = QLabel("设备属性", self.detail)
        sec.setObjectName("section")
        dl.addWidget(sec)

        self.prop_host = QWidget(self.detail)
        self.prop_host.setObjectName("prophost")
        self.prop_lay = QVBoxLayout(self.prop_host)
        self.prop_lay.setContentsMargins(0, 0, 6, 0)
        self.prop_lay.setSpacing(0)

        self.prop_scroll = QScrollArea(self.detail)
        self.prop_scroll.setWidgetResizable(True)
        self.prop_scroll.setFrameShape(QFrame.Shape.NoFrame)
        self.prop_scroll.setHorizontalScrollBarPolicy(
            Qt.ScrollBarPolicy.ScrollBarAlwaysOff
        )
        self.prop_scroll.setWidget(self.prop_host)
        dl.addWidget(self.prop_scroll, 1)

        lay.addWidget(self.detail, 1)
        return self._stack

    def _vc(self, cap: str, parent) -> tuple[QLabel, QFrame]:
        card = QFrame(parent)
        card.setObjectName("vcard")
        card.setFixedHeight(62)
        lay = QVBoxLayout(card)
        lay.setContentsMargins(14, 8, 14, 8)
        lay.setSpacing(2)

        k = QLabel(cap, card)
        k.setObjectName("key")
        lay.addWidget(k)

        v = QLabel("----", card)
        v.setFont(theme.mono(21, FW.DemiBold))
        v.setStyleSheet(f"color:{theme.ACCENT};")
        lay.addWidget(v)
        return v, card

    def _apply_style(self):
        self.setStyleSheet(theme.QSS)

    # ------------------------------------------------ 数据

    def set_devices(self, devices: dict[str, Device], order: list[str]):
        self.devices = devices
        self.order = order
        online = sum(1 for d in devices.values() if d.present)
        self.title.counter.setText(f"在线 {online} · 记录 {len(devices)}")
        self._refresh_list()
        if self.selected and self.selected in devices:
            self._fill_detail(devices[self.selected])

    def _set_filter(self, key: str):
        self.filter = key
        self._refresh_list()

    def _visible(self) -> list[Device]:
        q = self.search.text().strip().lower()
        out = []
        for k in self.order:
            d = self.devices.get(k)
            if d is None:
                continue
            if self.filter == "on" and not d.present:
                continue
            if self.filter == "off" and d.present:
                continue
            if q:
                hay = " ".join(
                    [d.name, d.vid, d.pid, d.serial, d.com, d.mfg, d.label]
                ).lower()
                if q not in hay:
                    continue
            out.append(d)
        return out

    def _refresh_list(self):
        for w in self._rows.values():
            w.setParent(None)
            w.deleteLater()
        self._rows.clear()

        for d in self._visible():
            row = DeviceRow(d, self.list_host)
            row.set_selected(d.key == self.selected)
            row.clicked.connect(self.select)
            self.list_lay.insertWidget(self.list_lay.count() - 1, row)
            self._rows[d.key] = row

        if not self._rows:
            hint = QLabel("没有匹配的设备", self.list_host)
            hint.setObjectName("hint")
            hint.setAlignment(Qt.AlignmentFlag.AlignCenter)
            hint.setMinimumHeight(80)
            self.list_lay.insertWidget(self.list_lay.count() - 1, hint)
            self._rows["__hint__"] = hint  # type: ignore[assignment]

    def select(self, key: str):
        self.selected = key
        for k, row in self._rows.items():
            if isinstance(row, DeviceRow):
                row.set_selected(k == key)
        d = self.devices.get(key)
        if d is not None:
            self._fill_detail(d)

    # ------------------------------------------------ 详情

    def _show_empty(self):
        self.empty.show()
        self.detail.hide()

    def _fill_detail(self, d: Device):
        self.empty.hide()
        self.detail.show()

        self.tile_icon.setPixmap(icons.kind_pixmap(d.kind, 34, dev_color(d)))
        self.d_name.setText(d.name or "未知 USB 设备")

        col = dev_color(d)
        self.d_pill.setText(f"  {d.label}  ")
        self.d_pill.setStyleSheet(
            f"color:{col};background:rgba({QColor(col).red()},{QColor(col).green()},"
            f"{QColor(col).blue()},38);border:1px solid rgba({QColor(col).red()},"
            f"{QColor(col).green()},{QColor(col).blue()},90);border-radius:8px;"
            "font-size:11px;padding:2px 2px;"
        )

        ok = d.present
        self.d_state.setText(
            f"<span style='color:{theme.OK if ok else theme.BAD}'>●</span> "
            f"<span style='color:{theme.DIM}'>{'已连接' if ok else '已拔出'}</span>"
            f"<span style='color:{theme.FAINT}'> · 插拔 {d.plugs} 次</span>"
        )

        self.vid_val.setText(d.vid)
        self.pid_val.setText(d.pid)

        self._clear_props()
        rows = [
            ("设备类型", d.label),
            ("设备类", d.cls),
            ("制造商", d.mfg),
            ("服务", d.service),
            ("序列号", d.serial),
            ("串口", d.com),
            ("端口位置", d.location),
            ("描述", d.desc),
        ]
        for k, v in rows:
            if not v:
                continue
            self._add_prop(k, v)

        # 复合设备：逐个列出功能接口
        for i in d.ifaces:
            self._add_prop(f"MI_{i.mi:02X}", i.detail, iface_color(i.tag))

        if d.instance_id:
            self._add_prop("实例 ID", d.instance_id)
        self.prop_lay.addStretch(1)

    def _clear_props(self):
        while self.prop_lay.count():
            item = self.prop_lay.takeAt(0)
            w = item.widget()
            if w is not None:
                w.setParent(None)
                w.deleteLater()

    def _add_prop(self, key: str, val: str, color: str | None = None):
        row = QWidget(self.prop_host)
        rl = QHBoxLayout(row)
        rl.setContentsMargins(0, 8, 0, 8)
        rl.setSpacing(10)

        k = QLabel(key, row)
        k.setObjectName("key")
        k.setFixedWidth(72)
        k.setAlignment(
            Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignTop
        )
        rl.addWidget(k)

        v = QLabel(val, row)
        v.setObjectName("val")
        v.setWordWrap(True)
        if color:
            v.setStyleSheet(f"color:{color};")
        v.setTextInteractionFlags(Qt.TextInteractionFlag.TextSelectableByMouse)
        v.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)
        rl.addWidget(v, 1)

        self.prop_lay.addWidget(row)

        sep = QFrame(self.prop_host)
        sep.setObjectName("sep")
        sep.setFixedHeight(1)
        self.prop_lay.addWidget(sep)

    def _copy(self):
        d = self.devices.get(self.selected or "")
        if d is None:
            return
        QGuiApplication.clipboard().setText(d.dump())
        self.btn_copy.setText("已复制")
        from PySide6.QtCore import QTimer

        QTimer.singleShot(1200, lambda: self.btn_copy.setText("复制信息"))

    # ------------------------------------------------ 窗口行为

    def show_panel(self):
        if self.isMinimized():
            self.showNormal()
        self.show()
        self.raise_()
        self.activateWindow()

    def hide_to_tray(self):
        self.hide()

    def keyPressEvent(self, e):
        if e.key() == Qt.Key.Key_Escape:
            self.hide_to_tray()
            return
        super().keyPressEvent(e)


def QIcon_from(pm: QPixmap):
    from PySide6.QtGui import QIcon

    return QIcon(pm)

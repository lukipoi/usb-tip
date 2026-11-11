"""配色、字体与全局样式表。"""

from __future__ import annotations

from PySide6.QtGui import QColor, QFont, QFontDatabase

BG = "#15171C"
RAISE = "#1B1E25"
SOFT = "#20242C"
HOVER = "#262B35"
BORDER = "#2C313C"
BORDER_SOFT = "#232730"
TEXT = "#E9EBF1"
DIM = "#9EA7BB"
FAINT = "#717C92"
ACCENT = "#6EA8FE"
OK = "#3DDC97"
BAD = "#FF7B7B"

C_BG = QColor(BG)
C_TEXT = QColor(TEXT)
C_DIM = QColor(DIM)
C_FAINT = QColor(FAINT)
C_BORDER = QColor(BORDER)
C_ACCENT = QColor(ACCENT)
C_OK = QColor(OK)
C_BAD = QColor(BAD)

UI_FAMILY = "Microsoft YaHei UI"
MONO_FAMILY = "Cascadia Mono"


def pick_family() -> str:
    fams = set(QFontDatabase.families())
    for f in ("Microsoft YaHei UI", "Microsoft YaHei", "Segoe UI", "SimHei"):
        if f in fams:
            return f
    return UI_FAMILY


def pick_mono() -> str:
    fams = set(QFontDatabase.families())
    for f in ("Cascadia Mono", "Consolas", "JetBrains Mono", "Courier New"):
        if f in fams:
            return f
    return MONO_FAMILY


_UI = ""
_MONO = ""


def font(px: int, weight: QFont.Weight = QFont.Weight.Normal) -> QFont:
    global _UI
    if not _UI:
        _UI = pick_family()
    f = QFont(_UI)
    f.setPixelSize(px)
    f.setWeight(weight)
    f.setHintingPreference(QFont.HintingPreference.PreferDefaultHinting)
    return f


def mono(px: int, weight: QFont.Weight = QFont.Weight.Normal) -> QFont:
    global _MONO
    if not _MONO:
        _MONO = pick_mono()
    f = QFont(_MONO)
    f.setPixelSize(px)
    f.setWeight(weight)
    return f


QSS = f"""
QWidget {{
    color: {TEXT};
    font-family: "{UI_FAMILY}";
}}
QFrame#card {{
    background: {BG};
    border: 1px solid {BORDER};
    border-radius: 16px;
}}
QLabel {{ background: transparent; }}
QLabel#hdr      {{ color: {TEXT};    font-size: 15px; font-weight: 600; }}
QLabel#hdrSub   {{ color: {FAINT};   font-size: 11px; }}
QLabel#counter  {{ color: {DIM};     font-size: 11px; }}
QLabel#section  {{ color: {FAINT};   font-size: 11px; font-weight: 600;
                   letter-spacing: 1px; }}
QLabel#name     {{ color: {TEXT};    font-size: 13px; font-weight: 600; }}
QLabel#sub      {{ color: {FAINT};   font-size: 11px; }}
QLabel#big      {{ color: {TEXT};    font-size: 19px; font-weight: 600; }}
QLabel#key      {{ color: {FAINT};   font-size: 11px; }}
QLabel#val      {{ color: {TEXT};    font-size: 12px; }}
QLabel#hint     {{ color: {FAINT};   font-size: 12px; }}

QLineEdit {{
    background: {RAISE};
    border: 1px solid {BORDER_SOFT};
    border-radius: 9px;
    padding: 7px 10px;
    color: {TEXT};
    font-size: 12px;
    selection-background-color: #2B4C7E;
}}
QLineEdit:focus {{ border: 1px solid #3C4A63; }}

QPushButton#chip {{
    background: transparent;
    border: 1px solid {BORDER_SOFT};
    border-radius: 8px;
    padding: 4px 12px;
    color: {DIM};
    font-size: 12px;
}}
QPushButton#chip:hover   {{ background: {RAISE}; color: {TEXT}; }}
QPushButton#chip:checked {{ background: #22304A; border-color: #34507A; color: #C9D8F5; }}

QPushButton#tbtn {{
    background: transparent;
    border: none;
    border-radius: 8px;
    color: {DIM};
    font-size: 14px;
}}
QPushButton#tbtn:hover {{ background: {HOVER}; color: {TEXT}; }}
QPushButton#close:hover {{ background: #E5484D; color: #FFFFFF; }}

QPushButton#act {{
    background: {SOFT};
    border: 1px solid {BORDER};
    border-radius: 9px;
    padding: 6px 14px;
    color: {TEXT};
    font-size: 12px;
}}
QPushButton#act:hover  {{ background: {HOVER}; border-color: #3A404E; }}
QPushButton#act:pressed {{ background: #1A1D24; }}

QFrame#row {{ background: transparent; border-radius: 10px; }}
QFrame#row:hover {{ background: #1B1F27; }}
QFrame#row[sel="true"] {{ background: #1E2634; }}

QFrame#tile {{
    background: {RAISE};
    border: 1px solid {BORDER_SOFT};
    border-radius: 14px;
}}
QFrame#vcard {{
    background: {RAISE};
    border: 1px solid {BORDER_SOFT};
    border-radius: 12px;
}}
QFrame#sep {{ background: {BORDER_SOFT}; border: none; }}

QScrollArea {{ background: transparent; border: none; }}
QScrollArea > QWidget > QWidget {{ background: transparent; }}
QScrollBar:vertical {{ background: transparent; width: 8px; margin: 0; }}
QScrollBar::handle:vertical {{ background: {BORDER}; border-radius: 4px; min-height: 30px; }}
QScrollBar::handle:vertical:hover {{ background: #3A404E; }}
QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {{ height: 0; }}
QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical {{ background: transparent; }}
QScrollBar:horizontal {{ height: 0; }}

QMenu {{
    background: {RAISE};
    border: 1px solid {BORDER};
    border-radius: 10px;
    padding: 6px;
    color: {TEXT};
}}
QMenu::item {{
    padding: 7px 22px 7px 14px;
    border-radius: 7px;
    font-size: 12px;
}}
QMenu::item:selected {{ background: {HOVER}; }}
QMenu::separator {{ height: 1px; background: {BORDER_SOFT}; margin: 5px 8px; }}

QToolTip {{
    background: {RAISE};
    color: {TEXT};
    border: 1px solid {BORDER};
    padding: 4px 8px;
}}
"""

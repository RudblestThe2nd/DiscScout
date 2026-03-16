#!/usr/bin/env python3


import sys
import os
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional, List

from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QSplitter, QTreeWidget, QTreeWidgetItem, QPushButton, QLabel,
    QFileDialog, QHeaderView, QProgressBar, QFrame, QToolTip,
    QStatusBar, QSizePolicy, QAbstractItemView
)
from PyQt6.QtCore import (
    Qt, QThread, pyqtSignal, QRectF, QPointF, QTimer
)
from PyQt6.QtGui import (
    QPainter, QColor, QPen, QFont, QBrush, QFontMetrics,
    QPainterPath, QLinearGradient
)


# ─────────────────────────────────────────────
#  Veri Modeli
# ─────────────────────────────────────────────

@dataclass
class FileNode:
    name: str
    path: str
    size: int = 0
    is_dir: bool = False
    file_count: int = 0
    dir_count: int = 0
    children: List = field(default_factory=list)
    parent: Optional["FileNode"] = None

    def add_child(self, child: "FileNode"):
        child.parent = self
        self.children.append(child)


# ─────────────────────────────────────────────
#  Tarayıcı Thread
# ─────────────────────────────────────────────

class ScanWorker(QThread):
    progress  = pyqtSignal(str)
    finished  = pyqtSignal(object)

    def __init__(self, path: str):
        super().__init__()
        self.path  = path
        self._stop = False

    def stop(self):
        self._stop = True

    def run(self):
        root = self._scan(self.path, None)
        if not self._stop:
            self.finished.emit(root)

    def _scan(self, path: str, parent) -> FileNode:
        node = FileNode(
            name=os.path.basename(path) or path,
            path=path,
            is_dir=True,
            parent=parent,
        )
        self.progress.emit(path)

        try:
            with os.scandir(path) as it:
                entries = list(it)
        except PermissionError:
            return node

        for entry in entries:
            if self._stop:
                break
            try:
                if entry.is_dir(follow_symlinks=False):
                    child = self._scan(entry.path, node)
                    node.children.append(child)
                    node.size       += child.size
                    node.file_count += child.file_count
                    node.dir_count  += child.dir_count + 1
                else:
                    st = entry.stat(follow_symlinks=False)
                    child = FileNode(
                        name=entry.name,
                        path=entry.path,
                        size=st.st_size,
                        is_dir=False,
                        file_count=1,
                        parent=node,
                    )
                    node.children.append(child)
                    node.size       += st.st_size
                    node.file_count += 1
            except (PermissionError, OSError):
                continue

        return node


# ─────────────────────────────────────────────
#  Squarified Treemap Algoritması
# ─────────────────────────────────────────────

def squarify(items, rect: QRectF, total: float) -> list:
    """(QRectF, FileNode) çiftleri döner."""
    if not items or rect.width() < 1 or rect.height() < 1 or total == 0:
        return []

    # Boyuta göre büyükten küçüğe sırala
    sorted_items = sorted(items, key=lambda n: n.size, reverse=True)
    result = []
    _squarify(sorted_items, rect, total, result)
    return result


def _worst(row, w, total_area):
    if not row or w == 0:
        return float("inf")
    s = sum(n.size for n in row)
    if s == 0:
        return float("inf")
    rmax = max(n.size for n in row)
    rmin = min(n.size for n in row)
    if rmin == 0:
        return float("inf")
    area = s / total_area
    return max(w * w * rmax / (s * s), s * s / (w * w * rmin))


def _layout_row(row, bounds: QRectF, total: float, result: list, horiz: bool):
    s = sum(n.size for n in row)
    if s == 0 or total == 0:
        return
    frac = s / total

    if horiz:
        h = bounds.height() * frac
        x = bounds.x()
        for node in row:
            nf = node.size / s
            w = bounds.width() * nf
            result.append((QRectF(x, bounds.y(), w, h), node))
            x += w
        bounds.setY(bounds.y() + h)
        bounds.setHeight(bounds.height() - h)
    else:
        w = bounds.width() * frac
        y = bounds.y()
        for node in row:
            nf = node.size / s
            h = bounds.height() * nf
            result.append((QRectF(bounds.x(), y, w, h), node))
            y += h
        bounds.setX(bounds.x() + w)
        bounds.setWidth(bounds.width() - w)


def _squarify(items, bounds: QRectF, total: float, result: list):
    if not items:
        return

    b = QRectF(bounds)
    remaining = list(items)

    while remaining:
        horiz = b.height() >= b.width()
        w = b.height() if horiz else b.width()

        row: list = []
        for i, item in enumerate(remaining):
            test_row = row + [item]
            if len(row) >= 1 and _worst(test_row, w, total) > _worst(row, w, total):
                break
            row = test_row

        _layout_row(row, b, total, result, horiz)
        remaining = remaining[len(row):]


# ─────────────────────────────────────────────
#  Renk Paleti
# ─────────────────────────────────────────────

PALETTE = [
    "#E05C5C", "#5C8EE0", "#5CC97A", "#E0A85C",
    "#A05CE0", "#5CCFCF", "#E07A5C", "#5CA8E0",
    "#8DE05C", "#E05CAF", "#5CE0B8", "#C4E05C",
    "#7A5CE0", "#E0C45C", "#5C6EE0",
]


def node_color(index: int, depth: int = 0) -> QColor:
    base = QColor(PALETTE[index % len(PALETTE)])
    if depth > 0:
        f = max(0.5, 1.0 - depth * 0.12)
        return QColor(
            int(base.red() * f),
            int(base.green() * f),
            int(base.blue() * f),
        )
    return base


# ─────────────────────────────────────────────
#  Treemap Widget
# ─────────────────────────────────────────────

class TreemapWidget(QWidget):
    node_clicked   = pyqtSignal(object)
    node_hovered   = pyqtSignal(object)

    def __init__(self):
        super().__init__()
        self.current_node: Optional[FileNode] = None
        self._rects: list = []          # (QRectF, FileNode, color_idx)
        self._hovered: Optional[FileNode] = None
        self.setMouseTracking(True)
        self.setMinimumHeight(280)
        self.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        self.setStyleSheet("background:#0d1117;")

    # ── Public ──────────────────────────────

    def set_node(self, node: Optional[FileNode]):
        self.current_node = node
        self._rebuild()
        self.update()

    # ── Layout ──────────────────────────────

    def _rebuild(self):
        self._rects = []
        if not self.current_node:
            return
        valid = [c for c in self.current_node.children if c.size > 0]
        if not valid:
            return
        total = self.current_node.size or 1
        rect  = QRectF(2, 2, self.width() - 4, self.height() - 4)
        pairs = squarify(valid, rect, total)
        for i, (r, node) in enumerate(pairs):
            self._rects.append((r, node, i))

    def resizeEvent(self, event):
        self._rebuild()
        self.update()

    # ── Paint ───────────────────────────────

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing, False)

        # Arkaplan
        painter.fillRect(self.rect(), QColor("#0d1117"))

        if not self._rects:
            painter.setPen(QColor("#3d4450"))
            f = QFont("Monospace", 11)
            painter.setFont(f)
            painter.drawText(
                self.rect(), Qt.AlignmentFlag.AlignCenter,
                "📂  Taranacak dizini seçin\n\nCtrl+O veya 'Dizin Seç' butonuna tıklayın"
            )
            painter.end()
            return

        for rect, node, idx in self._rects:
            if rect.width() < 2 or rect.height() < 2:
                continue

            color = node_color(idx)
            is_hov = node is self._hovered

            if is_hov:
                color = color.lighter(140)

            # Fill
            painter.fillRect(rect, color)

            # Gradient overlay for depth
            grad = QLinearGradient(rect.topLeft(), rect.bottomLeft())
            grad.setColorAt(0, QColor(255, 255, 255, 30))
            grad.setColorAt(1, QColor(0, 0, 0, 40))
            painter.fillRect(rect, QBrush(grad))

            # Border
            border_color = QColor("#0d1117") if not is_hov else QColor("white")
            painter.setPen(QPen(border_color, 1 if not is_hov else 2))
            painter.drawRect(rect.adjusted(0, 0, -1, -1))

            # Label
            self._draw_label(painter, rect, node, color)

        painter.end()

    def _draw_label(self, painter: QPainter, rect: QRectF, node: FileNode, bg: QColor):
        w, h = rect.width(), rect.height()
        if w < 24 or h < 16:
            return

        margin = 4
        inner = rect.adjusted(margin, margin, -margin, -margin)

        # Name
        name_font = QFont("Monospace", 9 if w > 120 else 7)
        name_font.setBold(True)
        painter.setFont(name_font)
        painter.setPen(QColor("white"))

        fm = QFontMetrics(name_font)
        name = node.name
        name_text = fm.elidedText(name, Qt.TextElideMode.ElideMiddle, int(inner.width()))

        if h > 36:
            painter.drawText(
                QRectF(inner.x(), inner.y(), inner.width(), fm.height()),
                Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignTop,
                name_text,
            )
            # Size line
            size_font = QFont("Monospace", 7)
            painter.setFont(size_font)
            painter.setPen(QColor(255, 255, 255, 180))
            painter.drawText(
                QRectF(inner.x(), inner.bottom() - QFontMetrics(size_font).height(), inner.width(), QFontMetrics(size_font).height()),
                Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignBottom,
                fmt_size(node.size),
            )
        else:
            combined = f"{name_text}  {fmt_size(node.size)}"
            painter.drawText(inner, Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter, combined)

    # ── Mouse ────────────────────────────────

    def mouseMoveEvent(self, event):
        pos = event.position()
        for rect, node, _ in self._rects:
            if rect.contains(pos):
                if self._hovered is not node:
                    self._hovered = node
                    self.update()
                    self.node_hovered.emit(node)
                    QToolTip.showText(
                        event.globalPosition().toPoint(),
                        f"<b>{node.name}</b><br>"
                        f"Boyut: {fmt_size(node.size)}<br>"
                        f"{'Klasör' if node.is_dir else 'Dosya'}"
                        + (f"<br>İçerik: {node.file_count:,} dosya, {node.dir_count:,} klasör" if node.is_dir else ""),
                        self,
                    )
                return
        if self._hovered:
            self._hovered = None
            self.update()

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            pos = event.position()
            for rect, node, _ in self._rects:
                if rect.contains(pos):
                    self.node_clicked.emit(node)
                    return

    def mouseDoubleClickEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            if self.current_node and self.current_node.parent:
                self.node_clicked.emit(self.current_node.parent)


# ─────────────────────────────────────────────
#  Sortable Tree Item
# ─────────────────────────────────────────────

class SortableItem(QTreeWidgetItem):
    def __lt__(self, other):
        col = self.treeWidget().sortColumn()
        my_val  = self.data(col, Qt.ItemDataRole.UserRole + 1)
        oth_val = other.data(col, Qt.ItemDataRole.UserRole + 1)
        if my_val is not None and oth_val is not None:
            return my_val < oth_val
        return super().__lt__(other)


# ─────────────────────────────────────────────
#  Yardımcı Fonksiyonlar
# ─────────────────────────────────────────────

def fmt_size(size: int) -> str:
    for unit in ("B", "KB", "MB", "GB", "TB"):
        if size < 1024.0:
            return f"{size:.1f} {unit}"
        size /= 1024.0
    return f"{size:.1f} PB"


# ─────────────────────────────────────────────
#  Ana Pencere
# ─────────────────────────────────────────────

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("DiskMapper")
        self.resize(1280, 820)
        self.root_node: Optional[FileNode] = None
        self.current_node: Optional[FileNode] = None
        self._scanner: Optional[ScanWorker] = None
        self._scan_count = 0
        self._timer = QTimer()
        self._timer.timeout.connect(self._tick)

        self._build_ui()
        self._apply_style()

    # ── UI İnşası ───────────────────────────

    def _build_ui(self):
        root = QWidget()
        self.setCentralWidget(root)
        vbox = QVBoxLayout(root)
        vbox.setContentsMargins(0, 0, 0, 0)
        vbox.setSpacing(0)

        # ── Toolbar ─────────────────────
        tb = QWidget()
        tb.setObjectName("toolbar")
        tb.setFixedHeight(52)
        tb_lay = QHBoxLayout(tb)
        tb_lay.setContentsMargins(12, 6, 12, 6)
        tb_lay.setSpacing(8)

        self.btn_scan = QPushButton("📂  Dizin Seç & Tara")
        self.btn_scan.setObjectName("btn_primary")
        self.btn_scan.setShortcut("Ctrl+O")
        self.btn_scan.clicked.connect(self.select_and_scan)

        self.btn_up = QPushButton("⬆  Yukarı")
        self.btn_up.setObjectName("btn_secondary")
        self.btn_up.setEnabled(False)
        self.btn_up.setShortcut("Backspace")
        self.btn_up.clicked.connect(self.go_up)

        self.btn_stop = QPushButton("⏹  Durdur")
        self.btn_stop.setObjectName("btn_danger")
        self.btn_stop.setEnabled(False)
        self.btn_stop.clicked.connect(self.stop_scan)

        self.lbl_path = QLabel("—")
        self.lbl_path.setObjectName("lbl_path")

        self.lbl_stat = QLabel("")
        self.lbl_stat.setObjectName("lbl_stat")
        self.lbl_stat.setAlignment(Qt.AlignmentFlag.AlignRight | Qt.AlignmentFlag.AlignVCenter)

        for w in (self.btn_scan, self.btn_up, self.btn_stop):
            w.setFixedHeight(36)

        tb_lay.addWidget(self.btn_scan)
        tb_lay.addWidget(self.btn_up)
        tb_lay.addWidget(self.btn_stop)
        tb_lay.addSpacing(8)
        tb_lay.addWidget(self.lbl_path, stretch=1)
        tb_lay.addWidget(self.lbl_stat)
        vbox.addWidget(tb)

        # ── Breadcrumb ──────────────────
        self.bc_bar = QWidget()
        self.bc_bar.setObjectName("bc_bar")
        self.bc_bar.setFixedHeight(28)
        self.bc_lay = QHBoxLayout(self.bc_bar)
        self.bc_lay.setContentsMargins(12, 0, 12, 0)
        self.bc_lay.setSpacing(0)
        self.bc_lay.addStretch()
        vbox.addWidget(self.bc_bar)

        # ── Progress ────────────────────
        self.progress = QProgressBar()
        self.progress.setObjectName("progress")
        self.progress.setFixedHeight(3)
        self.progress.setRange(0, 0)
        self.progress.setVisible(False)
        self.progress.setTextVisible(False)
        vbox.addWidget(self.progress)

        # ── Splitter ────────────────────
        splitter = QSplitter(Qt.Orientation.Vertical)
        splitter.setHandleWidth(4)
        splitter.setObjectName("main_splitter")

        self.treemap = TreemapWidget()
        self.treemap.node_clicked.connect(self.on_treemap_click)
        self.treemap.node_hovered.connect(self.on_treemap_hover)
        splitter.addWidget(self.treemap)

        # ── File List ───────────────────
        list_frame = QFrame()
        list_frame.setObjectName("list_frame")
        lf_lay = QVBoxLayout(list_frame)
        lf_lay.setContentsMargins(0, 0, 0, 0)
        lf_lay.setSpacing(0)

        # Column header bar
        col_header = QWidget()
        col_header.setObjectName("col_header_bar")
        col_header.setFixedHeight(22)
        lf_lay.addWidget(col_header)

        self.file_list = QTreeWidget()
        self.file_list.setObjectName("file_list")
        self.file_list.setColumnCount(6)
        self.file_list.setHeaderLabels([
            "İsim", "Boyut", "Toplam", "% Oran",
            "Dosya Sayısı", "Yol"
        ])
        self.file_list.setSortingEnabled(True)
        self.file_list.sortByColumn(1, Qt.SortOrder.DescendingOrder)
        self.file_list.setAlternatingRowColors(True)
        self.file_list.setSelectionMode(QAbstractItemView.SelectionMode.SingleSelection)
        self.file_list.itemDoubleClicked.connect(self.on_list_double_click)
        self.file_list.itemClicked.connect(self.on_list_click)

        hdr = self.file_list.header()
        hdr.setSectionResizeMode(0, QHeaderView.ResizeMode.Stretch)
        hdr.setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)
        hdr.setSectionResizeMode(2, QHeaderView.ResizeMode.ResizeToContents)
        hdr.setSectionResizeMode(3, QHeaderView.ResizeMode.ResizeToContents)
        hdr.setSectionResizeMode(4, QHeaderView.ResizeMode.ResizeToContents)
        hdr.setSectionResizeMode(5, QHeaderView.ResizeMode.Interactive)

        lf_lay.addWidget(self.file_list)
        splitter.addWidget(list_frame)

        splitter.setSizes([420, 340])
        vbox.addWidget(splitter, stretch=1)

        # ── Status Bar ──────────────────
        sb = self.statusBar()
        sb.setObjectName("status_bar")
        self.lbl_hover = QLabel("")
        self.lbl_hover.setObjectName("lbl_hover")
        sb.addPermanentWidget(self.lbl_hover)

    # ── Style ───────────────────────────────

    def _apply_style(self):
        self.setStyleSheet("""
        /* ─── Global ─── */
        QMainWindow, QWidget { background:#0d1117; color:#c9d1d9; font-family:Monospace; }

        /* ─── Toolbar ─── */
        #toolbar {
            background: qlineargradient(x1:0,y1:0,x2:0,y2:1,
                stop:0 #161b22, stop:1 #0d1117);
            border-bottom: 1px solid #30363d;
        }

        /* ─── Buttons ─── */
        QPushButton {
            border-radius:5px; font-size:12px; font-weight:600;
            padding:0 14px; cursor:pointer;
        }
        #btn_primary {
            background:#238636; color:#ffffff;
            border:1px solid #2ea043;
        }
        #btn_primary:hover  { background:#2ea043; }
        #btn_primary:pressed{ background:#1a7f37; }

        #btn_secondary {
            background:#21262d; color:#c9d1d9;
            border:1px solid #30363d;
        }
        #btn_secondary:hover  { background:#30363d; border-color:#8b949e; }
        #btn_secondary:disabled { color:#484f58; border-color:#21262d; }

        #btn_danger {
            background:#21262d; color:#f85149;
            border:1px solid #da3633;
        }
        #btn_danger:hover  { background:#da3633; color:#fff; }
        #btn_danger:disabled { color:#484f58; border-color:#21262d; }

        /* ─── Labels ─── */
        #lbl_path {
            color:#58a6ff; font-size:12px; font-family:Monospace;
            padding-left:4px;
        }
        #lbl_stat { color:#8b949e; font-size:11px; }
        #lbl_hover{ color:#8b949e; font-size:11px; padding-right:6px; }

        /* ─── Breadcrumb ─── */
        #bc_bar {
            background:#161b22;
            border-bottom:1px solid #21262d;
        }

        /* ─── Progress ─── */
        #progress {
            background:#0d1117;
            border:none;
        }
        #progress::chunk { background:#238636; }

        /* ─── Splitter ─── */
        #main_splitter::handle {
            background:#21262d;
        }

        /* ─── File List ─── */
        #list_frame { background:#0d1117; }

        QTreeWidget#file_list {
            background:#0d1117;
            alternate-background-color:#0d1117;
            color:#c9d1d9;
            border:none;
            font-size:12px;
            font-family:Monospace;
            outline:none;
        }
        QTreeWidget#file_list::item {
            height:24px;
            border-bottom:1px solid #161b22;
            padding-left:2px;
        }
        QTreeWidget#file_list::item:alternate {
            background:#161b22;
        }
        QTreeWidget#file_list::item:hover {
            background:#1f2937;
        }
        QTreeWidget#file_list::item:selected {
            background:#1f6feb;
            color:#ffffff;
        }

        QHeaderView::section {
            background:#161b22;
            color:#8b949e;
            padding:4px 8px;
            border:none;
            border-right:1px solid #21262d;
            border-bottom:1px solid #30363d;
            font-size:11px;
            font-weight:600;
            font-family:Monospace;
        }
        QHeaderView::section:hover { color:#c9d1d9; background:#1c2128; }

        /* ─── Status Bar ─── */
        QStatusBar {
            background:#161b22;
            border-top:1px solid #21262d;
            color:#8b949e;
            font-size:11px;
        }

        /* ─── Tooltip ─── */
        QToolTip {
            background:#161b22; color:#c9d1d9;
            border:1px solid #30363d;
            font-size:12px; padding:6px;
        }

        /* ─── Scrollbar ─── */
        QScrollBar:vertical {
            background:#0d1117; width:8px; border:none;
        }
        QScrollBar::handle:vertical {
            background:#30363d; border-radius:4px; min-height:20px;
        }
        QScrollBar::handle:vertical:hover { background:#484f58; }
        QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical { height:0; }

        QScrollBar:horizontal {
            background:#0d1117; height:8px; border:none;
        }
        QScrollBar::handle:horizontal {
            background:#30363d; border-radius:4px; min-width:20px;
        }
        QScrollBar::handle:horizontal:hover { background:#484f58; }
        QScrollBar::add-line:horizontal, QScrollBar::sub-line:horizontal { width:0; }
        """)

    # ── Tarama ──────────────────────────────

    def select_and_scan(self):
        path = QFileDialog.getExistingDirectory(
            self, "Taranacak Dizini Seçin", str(Path.home())
        )
        if path:
            self.start_scan(path)

    def start_scan(self, path: str):
        if self._scanner and self._scanner.isRunning():
            self._scanner.stop()
            self._scanner.wait()

        self.file_list.clear()
        self.treemap.set_node(None)
        self.root_node = None
        self.current_node = None
        self.lbl_path.setText(path)
        self.btn_stop.setEnabled(True)
        self.btn_scan.setEnabled(False)
        self.btn_up.setEnabled(False)
        self.progress.setVisible(True)
        self._scan_count = 0
        self._timer.start(200)
        self._update_breadcrumb(path, None)

        self._scanner = ScanWorker(path)
        self._scanner.progress.connect(self._on_progress)
        self._scanner.finished.connect(self._on_finished)
        self._scanner.start()

    def stop_scan(self):
        if self._scanner:
            self._scanner.stop()
        self._timer.stop()
        self.progress.setVisible(False)
        self.btn_stop.setEnabled(False)
        self.btn_scan.setEnabled(True)
        self.statusBar().showMessage("Tarama durduruldu.")

    def _on_progress(self, path: str):
        self._scan_count += 1
        self.lbl_path.setText(path[-80:] if len(path) > 80 else path)

    def _tick(self):
        self.statusBar().showMessage(f"Taranıyor… {self._scan_count:,} öğe işlendi")

    def _on_finished(self, root: FileNode):
        self._timer.stop()
        self.root_node = root
        self.progress.setVisible(False)
        self.btn_stop.setEnabled(False)
        self.btn_scan.setEnabled(True)
        self.lbl_path.setText(root.path)
        self.lbl_stat.setText(
            f"{fmt_size(root.size)}  ·  {root.file_count:,} dosya  ·  {root.dir_count:,} klasör"
        )
        self.statusBar().showMessage(
            f"✅  Tamamlandı — {fmt_size(root.size)} — "
            f"{root.file_count:,} dosya, {root.dir_count:,} klasör"
        )
        self.navigate_to(root)

    # ── Navigasyon ──────────────────────────

    def navigate_to(self, node: FileNode):
        self.current_node = node
        self.treemap.set_node(node)
        self._populate_list(node)
        self.btn_up.setEnabled(node.parent is not None)
        self._update_breadcrumb(node.path, node)

    def go_up(self):
        if self.current_node and self.current_node.parent:
            self.navigate_to(self.current_node.parent)

    def on_treemap_click(self, node: FileNode):
        if node.is_dir and node.children:
            self.navigate_to(node)
        elif not node.is_dir:
            # Dosya ise highlight et
            self._highlight_in_list(node)

    def on_treemap_hover(self, node: FileNode):
        pct = ""
        if self.current_node and self.current_node.size:
            pct = f"  ({node.size/self.current_node.size*100:.1f}%)"
        self.lbl_hover.setText(f"{node.name}  —  {fmt_size(node.size)}{pct}")

    def on_list_double_click(self, item, col):
        node: Optional[FileNode] = item.data(0, Qt.ItemDataRole.UserRole)
        if node and node.is_dir:
            self.navigate_to(node)

    def on_list_click(self, item, col):
        node: Optional[FileNode] = item.data(0, Qt.ItemDataRole.UserRole)
        if node:
            self.statusBar().showMessage(f"{node.path}  —  {fmt_size(node.size)}")

    # ── Liste Doldurma ───────────────────────

    def _populate_list(self, node: FileNode):
        self.file_list.setSortingEnabled(False)
        self.file_list.clear()

        parent_size = node.size or 1

        for child in node.children:
            item = SortableItem()

            # İsim
            icon = "📁 " if child.is_dir else "📄 "
            item.setText(0, icon + child.name)
            item.setData(0, Qt.ItemDataRole.UserRole, child)
            item.setData(0, Qt.ItemDataRole.UserRole + 1, child.name.lower())

            # Boyut (ham bytes ile sıralama)
            item.setText(1, fmt_size(child.size))
            item.setData(1, Qt.ItemDataRole.UserRole + 1, child.size)

            # Toplam boyut (aynı ama farklı hesaplanabilir ileride)
            item.setText(2, fmt_size(child.size))
            item.setData(2, Qt.ItemDataRole.UserRole + 1, child.size)

            # % Oran
            pct = child.size / parent_size * 100
            item.setText(3, f"{pct:.2f}%")
            item.setData(3, Qt.ItemDataRole.UserRole + 1, pct)

            # Dosya sayısı
            fc = child.file_count if child.is_dir else 1
            item.setText(4, f"{fc:,}")
            item.setData(4, Qt.ItemDataRole.UserRole + 1, fc)

            # Yol
            item.setText(5, child.path)
            item.setData(5, Qt.ItemDataRole.UserRole + 1, child.path)

            # Renklendirme
            if child.is_dir:
                item.setForeground(0, QColor("#79c0ff"))
            else:
                item.setForeground(0, QColor("#c9d1d9"))

            # % bar rengi
            if pct > 50:
                item.setForeground(3, QColor("#f85149"))
            elif pct > 20:
                item.setForeground(3, QColor("#d29922"))
            else:
                item.setForeground(3, QColor("#3fb950"))

            self.file_list.addTopLevelItem(item)

        self.file_list.setSortingEnabled(True)
        self.file_list.sortByColumn(1, Qt.SortOrder.DescendingOrder)

    def _highlight_in_list(self, target: FileNode):
        for i in range(self.file_list.topLevelItemCount()):
            item = self.file_list.topLevelItem(i)
            if item.data(0, Qt.ItemDataRole.UserRole) is target:
                self.file_list.setCurrentItem(item)
                self.file_list.scrollToItem(item)
                return

    # ── Breadcrumb ───────────────────────────

    def _update_breadcrumb(self, path: str, node: Optional[FileNode]):
        # Temizle
        while self.bc_lay.count():
            w = self.bc_lay.takeAt(0).widget()
            if w:
                w.deleteLater()

        parts = []
        p = path
        while True:
            head, tail = os.path.split(p)
            if tail:
                parts.insert(0, (tail, p))
                p = head
            else:
                if head:
                    parts.insert(0, (head, head))
                break

        for i, (name, fp) in enumerate(parts):
            btn = QPushButton(name)
            btn.setFlat(True)
            btn.setStyleSheet(
                "QPushButton { background:transparent; color:#58a6ff; border:none;"
                "  padding:0 4px; font-size:11px; font-family:Monospace; }"
                "QPushButton:hover { color:#ffffff; text-decoration:underline; }"
            )
            btn.clicked.connect(lambda _, full=fp: self._bc_navigate(full))
            self.bc_lay.addWidget(btn)

            if i < len(parts) - 1:
                sep = QLabel(" /")
                sep.setStyleSheet("color:#30363d; font-size:11px;")
                self.bc_lay.addWidget(sep)

        self.bc_lay.addStretch()

    def _bc_navigate(self, path: str):
        if not self.root_node:
            return
        node = self._find_node(self.root_node, path)
        if node:
            self.navigate_to(node)

    def _find_node(self, node: FileNode, path: str) -> Optional[FileNode]:
        if node.path == path:
            return node
        for child in node.children:
            found = self._find_node(child, path)
            if found:
                return found
        return None


# ─────────────────────────────────────────────
#  Giriş Noktası
# ─────────────────────────────────────────────

def main():
    app = QApplication(sys.argv)
    app.setApplicationName("DiskMapper")
    app.setApplicationVersion("1.0")
    app.setStyle("Fusion")

    win = MainWindow()
    win.show()

    # Komut satırından dizin verilmişse hemen tara
    if len(sys.argv) > 1 and os.path.isdir(sys.argv[1]):
        win.start_scan(sys.argv[1])

    sys.exit(app.exec())


if __name__ == "__main__":
    main()

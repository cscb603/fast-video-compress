import logging
from collections.abc import Callable
from pathlib import Path
from typing import Any

from PyQt5.QtCore import QSettings, Qt, pyqtSignal
from PyQt5.QtGui import QDragEnterEvent, QDropEvent
from PyQt5.QtWidgets import (
    QAbstractItemView,
    QCheckBox,
    QComboBox,
    QFileDialog,
    QFormLayout,
    QGroupBox,
    QHBoxLayout,
    QHeaderView,
    QLabel,
    QMainWindow,
    QMenu,
    QProgressBar,
    QPushButton,
    QSpinBox,
    QTreeWidget,
    QTreeWidgetItem,
    QVBoxLayout,
    QWidget,
)

from .models import TranscodeRequest
from .worker import JobManager

logger = logging.getLogger(__name__)

VIDEO_EXTS = {".mp4", ".mov", ".mkv", ".avi", ".webm", ".m4v", ".flv", ".ts", ".3gp"}


class DropArea(QLabel):
    filesDropped = pyqtSignal(list)

    def __init__(self, callback: Callable[[list[str]], None] | None = None) -> None:
        super().__init__("拖拽文件或文件夹到此处")
        self.callback = callback
        self.setAlignment(Qt.AlignCenter)
        self.setAcceptDrops(True)
        self.setStyleSheet("""
            QLabel {
                border: 2px dashed #aaa;
                border-radius: 10px;
                padding: 20px;
                background: #f9f9f9;
                color: #666;
                font-size: 14px;
            }
            QLabel:hover {
                background: #f0f0f0;
                border-color: #888;
            }
        """)

    def dragEnterEvent(self, event: QDragEnterEvent) -> None:
        if event.mimeData().hasUrls():
            event.accept()
            self.setStyleSheet(self.styleSheet().replace("#aaa", "#4CAF50"))
        else:
            event.ignore()

    def dragLeaveEvent(self, event: Any) -> None:
        self.setStyleSheet(self.styleSheet().replace("#4CAF50", "#aaa"))

    def dropEvent(self, event: QDropEvent) -> None:
        self.setStyleSheet(self.styleSheet().replace("#4CAF50", "#aaa"))
        urls = event.mimeData().urls()
        paths: list[str] = []
        for url in urls:
            p = url.toLocalFile()
            path_obj = Path(p)
            if path_obj.is_file():
                if path_obj.suffix.lower() in VIDEO_EXTS:
                    paths.append(p)
            elif path_obj.is_dir():
                # 递归扫描文件夹
                for ext in VIDEO_EXTS:
                    for f in path_obj.rglob(f"*{ext}"):
                        paths.append(str(f))

        if paths:
            if self.callback:
                self.callback(paths)
            self.filesDropped.emit(paths)


class MainWindow(QMainWindow):
    def __init__(self) -> None:
        super().__init__()
        self.setWindowTitle("星TAP批量视频压缩 Pro")
        self.resize(1000, 700)

        self.settings = QSettings("XTAP", "VideoCompressPro")
        self.mgr = JobManager()
        self.path_to_item: dict[str, QTreeWidgetItem] = {}
        self.init_ui()
        self.load_settings()

        # 连接信号
        self.mgr.itemProgress.connect(self._update_progress)
        self.mgr.itemStatus.connect(self._update_status)
        self.mgr.allFinished.connect(self._on_all_finished)

    def init_ui(self) -> None:
        central = QWidget()
        self.setCentralWidget(central)
        layout = QVBoxLayout(central)

        # 顶部工具栏
        top_bar = QHBoxLayout()
        self.btn_add = QPushButton("添加文件")
        self.btn_add_dir = QPushButton("添加文件夹")
        self.btn_clear = QPushButton("清空列表")
        self.btn_start = QPushButton("开始转换")
        self.btn_start.setStyleSheet(
            "background: #4CAF50; color: white; font-weight: bold; padding: 5px 15px;"
        )

        top_bar.addWidget(self.btn_add)
        top_bar.addWidget(self.btn_add_dir)
        top_bar.addStretch()
        top_bar.addWidget(self.btn_clear)
        top_bar.addWidget(self.btn_start)
        layout.addLayout(top_bar)

        # 中间部分：列表 + 设置
        content = QHBoxLayout()

        # 左侧列表
        self.tree = QTreeWidget()
        self.tree.setColumnCount(3)
        self.tree.setHeaderLabels(["文件名", "状态", "进度"])
        self.tree.header().setSectionResizeMode(0, QHeaderView.Stretch)
        self.tree.setSelectionMode(QAbstractItemView.ExtendedSelection)
        self.tree.setAcceptDrops(True)
        # 让 TreeWidget 也支持拖拽
        self.tree.dragEnterEvent = self.dragEnterEvent  # type: ignore
        self.tree.dropEvent = self.dropEvent  # type: ignore

        content.addWidget(self.tree, 3)

        # 右侧设置面板
        side_panel = QVBoxLayout()

        # 1. 编码方案
        group_enc = QGroupBox("编码设置")
        form_enc = QFormLayout(group_enc)
        self.combo_mode = QComboBox()
        self.combo_mode.addItems(["极速 (H.264)", "高压缩 (H.265/HEVC)", "无损 (Constant Quality)"])

        self.combo_res = QComboBox()
        self.combo_res.addItems(["保持原始", "4K (2160p)", "2K (1440p)", "1080p", "720p"])

        self.spin_bitrate = QSpinBox()
        self.spin_bitrate.setRange(0, 50000)
        self.spin_bitrate.setSuffix(" kbps")
        self.spin_bitrate.setSpecialValueText("自动")

        form_enc.addRow("模式:", self.combo_mode)
        form_enc.addRow("分辨率:", self.combo_res)
        form_enc.addRow("码率:", self.spin_bitrate)
        side_panel.addWidget(group_enc)

        # 2. 增强选项
        group_ext = QGroupBox("增强功能")
        vbox_ext = QVBoxLayout(group_ext)
        self.cb_stab = QCheckBox("视频稳像 (vidstab)")
        self.cb_audio = QCheckBox("音频降噪/人声增强")
        self.cb_metadata = QCheckBox("保留所有元数据 (DJI/Exif)")
        self.cb_metadata.setChecked(True)
        self.cb_skip = QCheckBox("跳过已存在的文件")
        self.cb_skip.setChecked(True)

        vbox_ext.addWidget(self.cb_stab)
        vbox_ext.addWidget(self.cb_audio)
        vbox_ext.addWidget(self.cb_metadata)
        vbox_ext.addWidget(self.cb_skip)
        side_panel.addWidget(group_ext)

        # 3. 输出设置
        group_out = QGroupBox("输出设置")
        vbox_out = QVBoxLayout(group_out)
        self.combo_out_type = QComboBox()
        self.combo_out_type.addItems(["原文件目录", "自定义目录"])

        self.btn_select_out = QPushButton("选择目录")
        self.btn_select_out.setVisible(False)
        self.label_out_path = QLabel("跟源文件一致")
        self.label_out_path.setStyleSheet("color: #666; font-size: 11px;")
        self.label_out_path.setWordWrap(True)

        vbox_out.addWidget(self.combo_out_type)
        vbox_out.addWidget(self.btn_select_out)
        vbox_out.addWidget(self.label_out_path)
        side_panel.addWidget(group_out)

        # 4. 系统设置
        group_sys = QGroupBox("系统")
        form_sys = QFormLayout(group_sys)
        self.spin_threads = QSpinBox()
        self.spin_threads.setRange(0, 8)
        self.spin_threads.setValue(0)
        self.spin_threads.setSpecialValueText("自动 (智能判定)")
        form_sys.addRow("并行任务数:", self.spin_threads)
        side_panel.addWidget(group_sys)

        side_panel.addStretch()
        content.addLayout(side_panel, 1)
        layout.addLayout(content)

        # 底部拖拽区
        self.drop_area = DropArea(self.add_files)
        layout.addWidget(self.drop_area)

        # 状态栏
        self.statusBar().showMessage("准备就绪")

        # 按钮事件
        self.btn_add.clicked.connect(self.on_add_files)
        self.btn_add_dir.clicked.connect(self.on_add_dir)
        self.btn_clear.clicked.connect(self.on_clear)
        self.btn_start.clicked.connect(self.on_start)
        self.combo_out_type.currentIndexChanged.connect(self.on_out_type_changed)
        self.btn_select_out.clicked.connect(self.on_select_out_dir)
        self.tree.setContextMenuPolicy(Qt.CustomContextMenu)
        self.tree.customContextMenuRequested.connect(self.on_context_menu)

    def load_settings(self) -> None:
        """从 QSettings 加载上次的配置"""
        self.combo_mode.setCurrentIndex(int(self.settings.value("mode", 1)))
        self.combo_res.setCurrentIndex(int(self.settings.value("res", 3)))  # 默认 1080p
        self.spin_bitrate.setValue(int(self.settings.value("bitrate", 0)))
        self.cb_stab.setChecked(self.settings.value("stab", False, type=bool))
        self.cb_audio.setChecked(self.settings.value("audio", False, type=bool))
        self.spin_threads.setValue(int(self.settings.value("threads", 0)))

        out_type = int(self.settings.value("out_type", 0))
        self.combo_out_type.setCurrentIndex(out_type)
        self.custom_out_dir = self.settings.value("out_dir", "")
        self.on_out_type_changed(out_type)

    def save_settings(self) -> None:
        """保存当前配置到 QSettings"""
        self.settings.setValue("mode", self.combo_mode.currentIndex())
        self.settings.setValue("res", self.combo_res.currentIndex())
        self.settings.setValue("bitrate", self.spin_bitrate.value())
        self.settings.setValue("stab", self.cb_stab.isChecked())
        self.settings.setValue("audio", self.cb_audio.isChecked())
        self.settings.setValue("pro_metadata", self.cb_metadata.isChecked())
        self.settings.setValue("pro_skip", self.cb_skip.isChecked())
        self.settings.setValue("threads", self.spin_threads.value())
        self.settings.setValue("out_type", self.combo_out_type.currentIndex())
        self.settings.setValue("out_dir", getattr(self, "custom_out_dir", ""))

    def on_out_type_changed(self, index: int) -> None:
        """输出类型切换"""
        is_custom = index == 1
        self.btn_select_out.setVisible(is_custom)
        if is_custom:
            path = getattr(self, "custom_out_dir", "")
            self.label_out_path.setText(path if path else "请选择目录...")
        else:
            self.label_out_path.setText("跟源文件一致 (文件名加 _xiao)")

    def on_select_out_dir(self) -> None:
        """选择自定义输出目录"""
        d = QFileDialog.getExistingDirectory(
            self, "选择输出文件夹", getattr(self, "custom_out_dir", "")
        )
        if d:
            self.custom_out_dir = d
            self.label_out_path.setText(d)

    def on_add_files(self) -> None:
        fs, _ = QFileDialog.getOpenFileNames(
            self, "选择视频文件", "", "Video (*.mp4 *.mov *.mkv *.avi *.ts)"
        )
        if fs:
            self.add_files(fs)

    def on_add_dir(self) -> None:
        d = QFileDialog.getExistingDirectory(self, "选择文件夹")
        if d:
            paths = []
            d_path = Path(d)
            for f in d_path.rglob("*"):
                if f.is_file() and f.suffix.lower() in VIDEO_EXTS:
                    paths.append(str(f))
            self.add_files(paths)

    def add_files(self, paths: list[str]) -> None:
        if not paths:
            self.statusBar().showMessage("未发现可识别的视频文件")
            return
        count = 0
        for p in paths:
            if p in self.path_to_item:
                continue
            item = QTreeWidgetItem(self.tree)
            item.setText(0, p)
            item.setText(1, "等待中")
            bar = QProgressBar()
            bar.setRange(0, 100)
            bar.setValue(0)
            bar.setTextVisible(True)
            self.tree.setItemWidget(item, 2, bar)
            self.path_to_item[p] = item
            count += 1
        self.statusBar().showMessage(f"已成功添加 {count} 个文件 (忽略重复)")

    def on_clear(self) -> None:
        self.tree.clear()
        self.path_to_item.clear()

    def on_start(self) -> None:
        self.save_settings()
        self.mgr.setConcurrency(self.spin_threads.value())

        reqs = []
        for i in range(self.tree.topLevelItemCount()):
            item = self.tree.topLevelItem(i)
            if not item:
                continue
            if item.text(1) in ("完成", "转码中", "已停止"):
                continue

            p = item.text(0)
            mode_idx = self.combo_mode.currentIndex()
            res_map = {0: 0, 1: 2160, 2: 1440, 3: 1080, 4: 720}

            output_dir = None
            if self.combo_out_type.currentIndex() == 1:
                output_dir = getattr(self, "custom_out_dir", None)

            req = TranscodeRequest(
                p,
                output_dir=output_dir,
                pro=True,
                pro_encoder="h265" if mode_idx == 1 else "h264" if mode_idx == 0 else "libx265",
                bitrate_kbps=-1 if mode_idx == 2 else self.spin_bitrate.value(),
                pro_height=res_map.get(self.combo_res.currentIndex(), 1080),
                pro_stab=self.cb_stab.isChecked(),
                pro_audio_enhance=self.cb_audio.isChecked(),
                skip_existing=self.cb_skip.isChecked(),
            )
            reqs.append(req)
            if item:
                item.setText(1, "队列中")

        if reqs:
            self.btn_start.setEnabled(False)
            self.btn_start.setText("正在处理...")
            self.mgr.enqueue(reqs)
            self.statusBar().showMessage(f"已添加 {len(reqs)} 个任务到队列")

    def on_context_menu(self, pos: Any) -> None:
        menu = QMenu()
        act_stop = menu.addAction("停止选中任务")
        act_remove = menu.addAction("移除选中项")

        action = menu.exec_(self.tree.mapToGlobal(pos))
        if not action:
            return

        selected = self.tree.selectedItems()
        if action == act_stop:
            for item in selected:
                self.mgr.stop(item.text(0))
        elif action == act_remove:
            for item in selected:
                path = item.text(0)
                self.mgr.stop(path)
                if path in self.path_to_item:
                    del self.path_to_item[path]
                self.tree.takeTopLevelItem(self.tree.indexOfTopLevelItem(item))

    def _update_progress(self, path: str, val: int) -> None:
        item = self.path_to_item.get(path)
        if item:
            bar = self.tree.itemWidget(item, 2)
            if isinstance(bar, QProgressBar):
                bar.setValue(val)

    def _update_status(self, path: str, status: str) -> None:
        item = self.path_to_item.get(path)
        if item:
            item.setText(1, status)

    def _on_all_finished(self) -> None:
        self.btn_start.setEnabled(True)
        self.btn_start.setText("开始转换")
        self.statusBar().showMessage("所有任务处理完成")

    def closeEvent(self, event: Any) -> None:
        self.save_settings()
        event.accept()

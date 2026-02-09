import json
import logging
import tempfile
import time
from enum import Enum, auto
from pathlib import Path
from typing import Dict, List, Optional

from PyQt5.QtCore import (
    QMutex,
    QMutexLocker,
    QObject,
    QProcess,
    QThread,
    QTimer,
    pyqtSignal,
)

from .core import FFmpegHandler
from .models import TranscodeRequest

logger = logging.getLogger(__name__)


class JobState(Enum):
    """任务生命周期状态"""

    PENDING = auto()
    PROBING = auto()
    STABBING = auto()
    ENCODING = auto()
    FINISHED = auto()
    FAILED = auto()
    STOPPED = auto()
    SKIPPED = auto()


class Job(QObject):
    """
    单个视频转码任务。采用状态机模式管理 FFmpeg 流程。
    """

    progress = pyqtSignal(str, int)  # (input_path, percent)
    status_changed = pyqtSignal(str, str)  # (input_path, status_text)
    finished = pyqtSignal(str, JobState)  # (input_path, final_state)
    error = pyqtSignal(str, str)  # (input_path, error_msg)

    def __init__(self, req: TranscodeRequest, ff: FFmpegHandler, threads: int = 0) -> None:
        super().__init__()
        self.req = req
        self.ff = ff
        self.threads = threads
        self.state = JobState.PENDING

        # 核心进程管理
        self.process = QProcess(self)
        self.process.setProcessChannelMode(QProcess.SeparateChannels)
        self.process.readyReadStandardOutput.connect(self._handle_stdout)
        self.process.readyReadStandardError.connect(self._handle_stderr)
        self.process.finished.connect(self._handle_process_finished)
        self.process.errorOccurred.connect(self._handle_process_error)

        # 运行数据
        self.duration_ms = 0
        self.has_audio = False
        self.has_video = False
        self.start_time = 0.0
        self.last_ui_ts = 0.0
        self.trf_file: Optional[Path] = None
        self.log_buffer: List[str] = []
        self.stdout_buffer = ""
        self.output_path = self._resolve_output_path()

    def _resolve_output_path(self) -> str:
        """解析最终输出绝对路径"""
        base = Path(self.req.input_path)
        name = base.stem + "_xiao.mp4"
        if self.req.output_dir:
            target = Path(self.req.output_dir) / name
        else:
            target = base.with_name(name)
        return str(target.absolute())

    def start(self) -> None:
        """进入状态机入口"""
        if self.state != JobState.PENDING:
            return

        # 检查跳过逻辑
        if self.req.skip_existing and Path(self.output_path).exists():
            if Path(self.output_path).stat().st_size > 1024: # 简单校验非虚假文件
                self._transit_state(JobState.SKIPPED, "已跳过 (文件已存在)")
                return

        # 启动分析
        self._transit_state(JobState.PROBING, "分析中")
        self._run_probe()

    def stop(self) -> None:
        """安全停止进程"""
        if self.state in (JobState.FINISHED, JobState.FAILED, JobState.STOPPED, JobState.SKIPPED):
            return

        self._transit_state(JobState.STOPPED, "已停止")
        if self.process.state() != QProcess.NotRunning:
            self.process.terminate()
            if not self.process.waitForFinished(1000):
                self.process.kill()

    def _transit_state(self, new_state: JobState, status_text: Optional[str] = None) -> None:
        """状态转换核心逻辑"""
        self.state = new_state
        logger.info(f"Job state changed: {self.req.input_path} -> {new_state} ({status_text})")
        if status_text:
            # 在主线程中更新 UI
            QTimer.singleShot(0, lambda: self.status_changed.emit(self.req.input_path, status_text))

        if new_state in (JobState.FINISHED, JobState.FAILED, JobState.STOPPED, JobState.SKIPPED):
            if new_state in (JobState.FINISHED, JobState.SKIPPED):
                QTimer.singleShot(0, lambda: self.progress.emit(self.req.input_path, 100))
            # 延迟发射完成信号，确保所有 UI 更新已排队
            QTimer.singleShot(0, lambda: self.finished.emit(self.req.input_path, new_state))

    def _run_probe(self) -> None:
        """执行 ffprobe"""
        if not self.ff.bin_ffprobe:
            self._handle_fatal("缺少 ffprobe 组件")
            return

        logger.info(f"Running ffprobe for {self.req.input_path}")
        args = [
            "-v", "error",
            "-show_entries", "format=duration:stream=codec_type",
            "-of", "json", self.req.input_path
        ]
        self.process.start(self.ff.bin_ffprobe, args)
        
        # 为 Probe 添加超时保护，防止损坏文件导致挂起
        QTimer.singleShot(10000, self._check_probe_timeout)

    def _check_probe_timeout(self) -> None:
        if self.state == JobState.PROBING and self.process.state() != QProcess.NotRunning:
            logger.warning(f"Probe timeout for {self.req.input_path}, killing...")
            self.process.kill()
            self._handle_fatal("分析文件超时 (文件可能损坏)")

    def _run_stab(self) -> None:
        """阶段1：稳像分析"""
        self._transit_state(JobState.STABBING, "稳像分析 (1/2)")
        self.trf_file = Path(tempfile.gettempdir()) / f"stab_{int(time.time())}_{id(self)}.trf"
        trf_arg = self.ff.escape_path_filter(str(self.trf_file))

        args = [
            "-hide_banner", "-y", "-hwaccel", "auto",
            "-i", self.req.input_path,
            "-vf", f"vidstabdetect=stepsize=6:shakiness=8:accuracy=15:result='{trf_arg}'",
            "-f", "null", "-", "-progress", "pipe:1"
        ]
        self.process.start(self.ff.bin_ffmpeg, args)

    def _run_encode(self) -> None:
        """阶段2/核心：视频转码"""
        status = "稳像转码 (2/2)" if self.req.pro_stab else "转码中"
        self._transit_state(JobState.ENCODING, status)
        self.start_time = time.time()
        logger.info(f"Starting encode for {self.req.input_path}")

        # 确保输出目录可用且有写权限
        try:
            out_p = Path(self.output_path)
            out_p.parent.mkdir(parents=True, exist_ok=True)
            
            # 测试写权限
            test_file = out_p.parent / f".perm_test_{int(time.time())}"
            try:
                test_file.touch()
                test_file.unlink()
            except (PermissionError, OSError) as e:
                self._handle_fatal(f"目录无写入权限: {out_p.parent}")
                return
        except Exception as e:
            self._handle_fatal(f"无法访问输出目录: {e}")
            return

        # 构建 FFmpeg 参数
        enc = self.ff.get_pro_encoder(self.req.pro_encoder) if self.req.pro else "libx264"
        is_hw = any(x in enc for x in ("nvenc", "videotoolbox", "qsv", "vaapi"))

        args = ["-hide_banner", "-y", "-hwaccel", "auto"]
        if not is_hw:
            threads = self.threads if self.threads > 0 else 4
            args += ["-threads", str(threads)]

        args += ["-i", self.req.input_path]

        # 滤镜链
        vf = []
        if self.req.pro_stab and self.trf_file:
            trf_arg = self.ff.escape_path_filter(str(self.trf_file))
            smooth = 30 if self.req.pro_stab_quality == "hq" else 15
            vf.append(f"vidstabtransform=smoothing={smooth}:input='{trf_arg}'")
            # 移除默认的 unsharp，因为它会显著降低速度
            # vf.append("unsharp=5:5:0.8:3:3:0.4")

        if self.req.pro_height > 0:
            vf.append(f"scale=-2:{self.req.pro_height}")

        vf.append("format=yuv420p")  # 最大兼容性

        af = self.ff.build_audio_filter(self.req.pro_audio_enhance)

        if vf: args += ["-vf", ",".join(vf)]
        # af 已经移入下方的 has_audio 判断中

        args += ["-c:v", enc]
        if "265" in enc or "hevc" in enc:
            args += ["-tag:v", "hvc1"]

        # 质量控制
        if self.req.bitrate_kbps > 0:
            b = self.req.bitrate_kbps
            args += ["-b:v", f"{b}k", "-maxrate", f"{b}k", "-bufsize", f"{b * 2}k"]
        else:
            if "videotoolbox" in enc:
                args += ["-global_quality", "50"]
            else:
                args += ["-crf", "23", "-preset", "faster"]

        # 音频处理
        if self.has_audio:
            args += ["-c:a", "aac", "-b:a", "192k"]
            if af: args += ["-af", af]
        else:
            args += ["-an"]

        # 映射与元数据
        args += ["-map", "0:v:0"]
        if self.has_audio:
            args += ["-map", "0:a"]
        
        # 移除可能导致失败的 -map 0:d? 和 -map_metadata 0
        # 如果需要保留元数据，只保留最基础的
        args += ["-movflags", "+faststart"]
        args += ["-progress", "pipe:1", self.output_path]

        logger.info(f"FFmpeg cmd: {self.ff.bin_ffmpeg} {' '.join(args)}")
        self.process.start(self.ff.bin_ffmpeg, args)

    def _handle_stdout(self) -> None:
        """解析进度信息或分析结果"""
        data = bytes(self.process.readAllStandardOutput()).decode(errors="ignore")
        if self.state == JobState.PROBING:
            self.stdout_buffer += data
            return

        for line in data.splitlines():
            if line.startswith("out_time_ms="):
                try:
                    ms = int(line.split("=")[1])
                    if self.duration_ms > 0:
                        pct = int((ms / self.duration_ms) * 100)
                    else:
                        pct = 0
                    pct = max(0, min(99, pct))

                    # 复合进度显示
                    if self.state == JobState.STABBING:
                        display_pct = int(pct * 0.4)
                    elif self.state == JobState.ENCODING and self.req.pro_stab:
                        display_pct = 40 + int(pct * 0.6)
                    else:
                        display_pct = pct

                    now = time.time()
                    if now - self.last_ui_ts > 0.1:  # 稍微提高更新频率
                        # 确保进度信号在主线程触发
                        QTimer.singleShot(0, lambda p=display_pct: self.progress.emit(self.req.input_path, p))
                        self.last_ui_ts = now
                except Exception:
                    pass

    def _handle_stderr(self) -> None:
        """收集错误日志"""
        data = bytes(self.process.readAllStandardError()).decode(errors="ignore")
        if data:
            for line in data.strip().splitlines():
                self.log_buffer.append(line)
                if len(self.log_buffer) > 50:
                    self.log_buffer.pop(0)

    def _handle_process_finished(self, code: int) -> None:
        """进程结束后的逻辑分发"""
        logger.info(f"Process finished: {self.req.input_path}, state: {self.state}, code: {code}")
        if self.state == JobState.STOPPED:
            return

        if self.state == JobState.PROBING:
            # 延迟解析 Probe 结果
            try:
                info = json.loads(self.stdout_buffer)
                fmt = info.get("format", {})
                self.duration_ms = int(float(fmt.get("duration", 0)) * 1000)
                
                streams = info.get("streams", [])
                self.has_video = any(s.get("codec_type") == "video" for s in streams)
                self.has_audio = any(s.get("codec_type") == "audio" for s in streams)
                
                logger.info(f"Probe success: {self.req.input_path}, duration: {self.duration_ms}ms, video: {self.has_video}, audio: {self.has_audio}")
            except Exception as e:
                logger.warning(f"Probe parse failed: {e}")
                # 即使失败也继续，尝试转码（有些文件 ffprobe 报错但 ffmpeg 能跑）
                if self.duration_ms <= 0:
                    self.duration_ms = 0

        if code != 0:
            self._handle_fatal(self._parse_last_error())
            return

        # 状态流转
        if self.state == JobState.PROBING:
            if self.req.pro_stab:
                self._run_stab()
            else:
                self._run_encode()
        elif self.state == JobState.STABBING:
            self._run_encode()
        elif self.state == JobState.ENCODING:
            # 严格验证输出文件
            out_p = Path(self.output_path)
            if out_p.exists() and out_p.stat().st_size > 1024: # 至少 1KB
                elapsed = time.time() - self.start_time
                self._transit_state(JobState.FINISHED, f"完成 ({elapsed:.1f}s)")
            else:
                msg = "输出文件异常 (文件丢失或大小为0)"
                logger.error(f"Job failed verification: {self.req.input_path}, size: {out_p.stat().st_size if out_p.exists() else 'N/A'}")
                self._handle_fatal(msg)

    def _handle_process_error(self, error: QProcess.ProcessError) -> None:
        """底层系统错误"""
        logger.error(f"Process error: {self.req.input_path}, error_code: {error}")
        if self.state == JobState.STOPPED:
            return
        err_map = {
            QProcess.FailedToStart: "无法启动 FFmpeg/FFprobe",
            QProcess.Crashed: "FFmpeg 意外崩溃",
            QProcess.Timedout: "操作超时",
        }
        self._handle_fatal(err_map.get(error, f"系统进程错误 ({error})"))

    def _handle_fatal(self, msg: str) -> None:
        """发生致命错误"""
        self._transit_state(JobState.FAILED, msg)
        self.error.emit(self.req.input_path, msg)

    def _parse_last_error(self) -> str:
        """从日志缓冲区分析错误原因"""
        full_log = "\n".join(self.log_buffer).lower()
        if "permission denied" in full_log:
            return "权限不足"
        if "no space left" in full_log:
            return "磁盘空间不足"
        if "error opening output" in full_log:
            return "输出文件被占用"
        # 返回最后一行非进度信息
        for line in reversed(self.log_buffer):
            if line.strip() and "out_time" not in line:
                return line.strip()[:50]
        return "处理异常中断"

    def __del__(self):
        # 确保 TRF 临时文件被清理
        if self.trf_file and self.trf_file.exists():
            try:
                self.trf_file.unlink()
            except Exception:
                pass


class JobManager(QObject):
    """
    任务管理器。负责全局队列调度与资源限制。
    """
    itemProgress = pyqtSignal(str, int)
    itemStatus = pyqtSignal(str, str)
    allFinished = pyqtSignal()

    def __init__(self, concurrency: int = 0) -> None:
        super().__init__()
        self.queue: List[TranscodeRequest] = []
        self.running: Dict[str, Job] = {}
        self.concurrency = concurrency if concurrency > 0 else self._auto_detect_concurrency()
        self.mutex = QMutex()
        self.ff = FFmpegHandler()
        self._is_pumping = False
        self._had_tasks = False
        
        # 守护定时器：确保即便信号意外丢失也能继续泵动
        self.watchdog = QTimer(self)
        self.watchdog.setInterval(5000)  # 5秒检查一次
        self.watchdog.timeout.connect(self.pump)
        self.watchdog.start()

    def _auto_detect_concurrency(self) -> int:
        cores = QThread.idealThreadCount()
        if cores <= 4:
            return 1
        if cores <= 8:
            return 2
        return 3

    def setConcurrency(self, n: int) -> None:
        self.concurrency = max(1, n)
        self.pump()

    def enqueue(self, reqs: List[TranscodeRequest]) -> None:
        with QMutexLocker(self.mutex):
            # 防止重复添加已经在队列或正在运行的任务
            existing = {r.input_path for r in self.queue} | set(self.running.keys())
            for r in reqs:
                if r.input_path not in existing:
                    self.queue.append(r)
        self.pump()

    def stop(self, path: str) -> None:
        with QMutexLocker(self.mutex):
            if path in self.running:
                self.running[path].stop()
            self.queue = [r for r in self.queue if r.input_path != path]

    def pump(self) -> None:
        """异步触发调度循环"""
        QTimer.singleShot(0, self._do_pump)

    def _do_pump(self) -> None:
        with QMutexLocker(self.mutex):
            if self._is_pumping:
                return
            self._is_pumping = True

            try:
                # 检查是否全部完成
                if not self.running and not self.queue:
                    if self._had_tasks:
                        logger.info("All tasks finished.")
                        self.allFinished.emit()
                        self._had_tasks = False
                    return

                self._had_tasks = True
                logger.info(f"Pumping: {len(self.running)} running, {len(self.queue)} in queue, concurrency: {self.concurrency}")

                # 填充运行槽位
                while len(self.running) < self.concurrency and self.queue:
                    req = self.queue.pop(0)
                    job = Job(req, self.ff)
                    
                    # 绑定信号
                    job.progress.connect(self.itemProgress.emit)
                    job.status_changed.connect(self.itemStatus.emit)
                    job.finished.connect(self._on_job_done)
                    
                    logger.info(f"Starting job: {req.input_path}")
                    self.running[req.input_path] = job
                    job.start()
            finally:
                self._is_pumping = False

    def _on_job_done(self, path: str, _state: JobState) -> None:
        """任务结束回调（无论是成功、失败还是跳过）"""
        logger.info(f"Job done: {path}, final_state: {_state}")
        with QMutexLocker(self.mutex):
            job = self.running.pop(path, None)
            if job:
                job.deleteLater()
        
        # 继续拉动队列
        self.pump()

import logging
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

logger = logging.getLogger(__name__)


class FFmpegHandler:
    """FFmpeg 核心处理类，负责探测环境、扫描能力及构建命令行参数。"""

    def __init__(self) -> None:
        """初始化 FFmpegHandler，自动探测二进制文件并扫描支持的能力。"""
        self.bin_ffmpeg: str | None = None
        self.bin_ffprobe: str | None = None
        self.version_info: tuple[int, int, int] = (0, 0, 0)
        self.filters: set[str] = set()
        self.encoders: set[str] = set()
        self.detect_binaries()
        self.scan_capabilities()

    def detect_binaries(self) -> None:
        """多平台自动探测 ffmpeg/ffprobe 二进制文件。"""
        candidates: list[Path | None] = []

        # 0. 在 Mac 上优先使用 Homebrew 安装的原生 arm64 版本
        sys_name = platform.system()
        if sys_name == "Darwin":
            candidates.append(Path("/opt/homebrew/bin"))
            candidates.append(Path("/usr/local/bin"))

        # 1. 尝试本地 bin 目录 (用户手动放置的版本)
        base = Path(__file__).parent.parent.parent
        candidates.append(base / "bin")
        candidates.append(base)

        # 1. 环境变量
        env_home = os.environ.get("FFMPEG_HOME") or os.environ.get("FFMPEG_DIR")
        if env_home:
            candidates.append(Path(env_home) / "bin")

        # 2. PATH (Homebrew 环境)
        candidates.append(None)

        # 3. 打包后的执行目录
        try:
            exe_dir = Path(sys.executable).parent
            candidates.append(exe_dir)
            candidates.append(exe_dir / "bin")
        except Exception:
            pass

        # 4. 系统公共路径
        if sys_name == "Windows":
            try:
                c_drive = Path("C:/")
                if c_drive.exists():
                    candidates.extend(list(c_drive.glob("ffmpeg-*/bin")))
            except Exception:
                pass
            candidates.append(Path("C:/ffmpeg/bin"))

            cfg = base / "ffmpeg_path.cfg"
            if cfg.exists():
                try:
                    p = cfg.read_text(encoding="utf-8").strip()
                    if p:
                        candidates.append(Path(p).parent)
                except Exception:
                    pass

        # 5. 系统 PATH (已经移动到最前面)
        # candidates.append(None)

        exe_ext = ".exe" if sys_name == "Windows" else ""

        for folder in candidates:
            curr_ffmpeg: str | None = None
            curr_ffprobe: str | None = None
            if folder is None:
                curr_ffmpeg = shutil.which("ffmpeg")
                curr_ffprobe = shutil.which("ffprobe")
            else:
                curr_ffmpeg = str(folder / ("ffmpeg" + exe_ext))
                curr_ffprobe = str(folder / ("ffprobe" + exe_ext))

            if (
                curr_ffmpeg
                and os.path.exists(curr_ffmpeg)
                # curr_ffprobe 可能不存在，我们容忍它，后面使用时再检查
            ):
                ver = self._get_version(curr_ffmpeg)
                if ver != (0, 0, 0):
                    self.bin_ffmpeg = curr_ffmpeg
                    # 如果当前目录下没有 ffprobe，尝试从其他地方找一个凑合用
                    if curr_ffprobe and os.path.exists(curr_ffprobe):
                        self.bin_ffprobe = curr_ffprobe
                    else:
                        self.bin_ffprobe = shutil.which("ffprobe")
                    
                    self.version_info = ver
                    logger.info(f"Detected FFmpeg: {curr_ffmpeg} (v{ver[0]}.{ver[1]})")
                    return

        logger.error("FFmpeg/FFprobe binaries not found in any candidate paths.")

    def _get_version(self, bin_path: str) -> tuple[int, int, int]:
        """获取 FFmpeg 版本号"""
        try:
            cmd = [bin_path, "-version"]
            startupinfo = None
            if sys.platform == "win32":
                startupinfo = subprocess.STARTUPINFO()  # type: ignore
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW  # type: ignore

            r = subprocess.run(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                startupinfo=startupinfo,
                timeout=5,
            )

            if r.stdout:
                line = r.stdout.splitlines()[0]
                parts = line.split()
                if len(parts) > 2:
                    v_str = parts[2].split("-")[0]  # Handle "6.0-full"
                    v_parts = [p for p in v_str.split(".") if p.isdigit()]
                    if v_parts:
                        return (int(v_parts[0]), int(v_parts[1]) if len(v_parts) > 1 else 0, 0)
        except Exception as e:
            logger.warning(f"Failed to check version for {bin_path}: {e}")
        return (0, 0, 0)

    def scan_capabilities(self) -> None:
        """扫描 FFmpeg 支持的编码器和滤镜"""
        if not self.bin_ffmpeg:
            return

        try:
            # 扫描编码器
            out = subprocess.check_output([self.bin_ffmpeg, "-hide_banner", "-encoders"], text=True)
            for line in out.splitlines():
                parts = line.split()
                if len(parts) > 1 and "V" in parts[0]:
                    self.encoders.add(parts[1])

            # 扫描滤镜
            out = subprocess.check_output([self.bin_ffmpeg, "-hide_banner", "-filters"], text=True)
            for line in out.splitlines():
                parts = line.split()
                if len(parts) > 1:
                    self.filters.add(parts[1])
            logger.info(
                f"Capabilities scanned: {len(self.encoders)} encoders, {len(self.filters)} filters."
            )
        except Exception as e:
            logger.error(f"Failed to scan FFmpeg capabilities: {e}")

    def get_auto_encoder(self) -> str:
        """
        根据当前平台和硬件能力自动选择最佳的 H.264 硬件加速编码器。

        Returns:
            str: 编码器名称 (如 h264_videotoolbox 或 libx264)
        """
        sys_name = platform.system().lower()
        if "darwin" in sys_name:
            # 在 Mac 上优先检查 VideoToolbox
            if "h264_videotoolbox" in self.encoders:
                logger.info("Auto-selected Mac hardware encoder: h264_videotoolbox")
                return "h264_videotoolbox"
        elif "windows" in sys_name:
            if "h264_nvenc" in self.encoders:
                return "h264_nvenc"
            if "h264_qsv" in self.encoders:
                return "h264_qsv"
        return "libx264"

    def get_pro_encoder(self, codec: str = "h265") -> str:
        """
        获取指定编码格式的专业级/硬件加速编码器。

        Args:
            codec: 目标编码格式 ('h265' 或 'h264')

        Returns:
            str: 最佳可用的编码器名称
        """
        if codec.startswith("lib") or "_nvenc" in codec or "_videotoolbox" in codec:
            return codec

        sys_name = platform.system().lower()
        if codec == "h265":
            if "darwin" in sys_name and "hevc_videotoolbox" in self.encoders:
                return "hevc_videotoolbox"
            if "windows" in sys_name and "hevc_nvenc" in self.encoders:
                return "hevc_nvenc"
            return "libx265"

        if codec == "h264":
            return self.get_auto_encoder()

        return self.get_auto_encoder()

    def build_audio_filter(
        self, enhance: bool = False, nr_mode: str = "medium"
    ) -> str | None:
        """构建音频处理滤镜链 (简化版以提高性能)"""
        if not enhance:
            return None

        chain = []

        # 1. 基础降噪 (如果支持)
        if "afftdn" in self.filters:
            nr_val = {"light": 10, "medium": 15, "heavy": 25}.get(nr_mode, 15)
            chain.append(f"afftdn=nr={nr_val}:nf=-25")

        # 2. 简单的 EQ 增强人声
        chain.append("equalizer=f=1000:g=2:w=1")

        # 3. 响度标准化与动态压缩 (轻量级)
        chain.append("acompressor=threshold=-18dB:ratio=2:attack=50:release=500")
        chain.append("loudnorm=I=-16:TP=-1.5:LRA=11")

        return ",".join(chain)

    def escape_path_filter(self, path_str: str) -> str:
        """转义滤镜字符串中的文件路径"""
        # 对于 vidstab 等滤镜，需要处理冒号、反斜杠和单引号
        p = path_str.replace("\\", "/")
        p = p.replace(":", "\\:")
        p = p.replace("'", "'\\''")
        return p

import logging
from logging.handlers import RotatingFileHandler
from pathlib import Path


def setup_logging(log_file: str = "app.log") -> logging.Logger:
    """配置日志系统，支持文件滚动和控制台输出"""
    logger = logging.getLogger()
    if not logger.handlers:
        logger.setLevel(logging.INFO)
        log_path = Path.cwd() / log_file
        # 如果当前目录无法写入，尝试用户目录
        try:
            h = RotatingFileHandler(
                str(log_path), maxBytes=2 * 1024 * 1024, backupCount=3, encoding="utf-8"
            )
        except (PermissionError, OSError):
            log_path = Path.home() / "VideoCompressPro.log"
            h = RotatingFileHandler(
                str(log_path), maxBytes=2 * 1024 * 1024, backupCount=3, encoding="utf-8"
            )
        h.setFormatter(logging.Formatter("%(asctime)s %(levelname)s %(name)s: %(message)s"))
        logger.addHandler(h)

        # 同时输出到控制台方便调试
        console = logging.StreamHandler()
        console.setFormatter(logging.Formatter("%(levelname)s: %(message)s"))
        logger.addHandler(console)
    return logger


def format_time(seconds: float) -> str:
    """格式化秒数为时:分:秒或分:秒格式"""
    m, s = divmod(seconds, 60)
    h, m = divmod(m, 60)
    if h > 0:
        return f"{int(h):02d}:{int(m):02d}:{int(s):02d}"
    return f"{int(m):02d}:{int(s):02d}"

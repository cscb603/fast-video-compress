from dataclasses import dataclass
from typing import Any


@dataclass
class TranscodeRequest:
    """转码请求配置模型"""

    input_path: str
    output_dir: str | None = None
    rotation_filter: str | None = None
    quality_mode: str = "hd"
    bitrate_kbps: int = 0
    compat: bool = False
    pro: bool = False
    pro_encoder: str = "h265"
    pro_height: int = 1080
    audio_multi: bool = False
    pro_stab: bool = False
    pro_audio_enhance: bool = False
    pro_stab_quality: str = "hq"
    audio_nr: str = "medium"
    stereo_mode: str = "center"
    skip_existing: bool = True

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "TranscodeRequest":
        """从字典创建实例，支持旧版本的参数映射"""
        return cls(**data)

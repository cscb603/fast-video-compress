"""
自定义异常模块，统一管理项目中的错误类型。
"""


class AppError(Exception):
    """项目自定义异常基类"""

    pass


class FFmpegError(AppError):
    """FFmpeg 相关错误，例如编码失败、二进制文件缺失等"""

    pass


class ConfigError(AppError):
    """配置或环境错误，例如路径不正确、参数非法等"""

    pass


class ValidationError(AppError):
    """输入验证错误"""

    pass

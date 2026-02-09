# 自动批量视频压缩 (Auto Batch Video Compressor)

[中文](#中文) | [English](#English)

---

## 中文

### 项目简介
这是一个基于 FFmpeg 的全自动批量视频压缩工具。旨在提供极速、高效且画质损失极小的视频压缩方案。支持多种硬件加速编码器（如 NVIDIA NVENC, Intel QSV, Apple Toolbox），并能够根据设备环境自动选择最佳压缩策略。

### 主要功能
- **自动环境探测**：自动寻找系统中安装的 FFmpeg 或本地二进制文件。
- **多硬件加速支持**：
  - NVIDIA: `h264_nvenc`, `hevc_nvenc`
  - Intel: `h264_qsv`, `hevc_qsv`
  - Apple: `h264_videotoolbox`, `hevc_videotoolbox`
- **批量处理**：一键扫描文件夹，批量压缩视频文件。
- **智能预设**：提供画质优先、速度优先、体积优先等多种配置方案。
- **多平台支持**：支持 Windows, macOS 和 Linux。

### 快速开始
1. **安装依赖**：
   ```bash
   pip install -r requirements.txt
   ```
2. **运行程序**：
   ```bash
   python app_refactored.py
   ```

### 依赖项
- Python 3.12+
- FFmpeg (程序会自动尝试探测，建议安装在系统路径或放在项目 `bin` 目录下)
- PyQt5 (用于图形界面)

---

## English

### Project Introduction
An automatic batch video compression tool based on FFmpeg. Designed to provide fast, efficient video compression with minimal quality loss. Supports multiple hardware-accelerated encoders (e.g., NVIDIA NVENC, Intel QSV, Apple Toolbox) and automatically selects the best compression strategy based on the device environment.

### Key Features
- **Auto Environment Detection**: Automatically finds installed FFmpeg in the system or local binary files.
- **Hardware Acceleration Support**:
  - NVIDIA: `h264_nvenc`, `hevc_nvenc`
  - Intel: `h264_qsv`, `hevc_qsv`
  - Apple: `h264_videotoolbox`, `hevc_videotoolbox`
- **Batch Processing**: One-click folder scanning and batch video compression.
- **Smart Presets**: Multiple configurations for Quality-first, Speed-first, or Size-first compression.
- **Cross-platform**: Supports Windows, macOS, and Linux.

### Quick Start
1. **Install Dependencies**:
   ```bash
   pip install -r requirements.txt
   ```
2. **Run Application**:
   ```bash
   python app_refactored.py
   ```

### Requirements
- Python 3.12+
- FFmpeg (Automatically detected; recommended to be in system PATH or project `bin` directory)
- PyQt5 (For GUI)

---

## 开源协议 (License)
MIT License

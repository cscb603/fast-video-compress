# 🚀 自动批量视频压缩 (Auto Batch Video Compressor)

> **告别存储焦虑，让视频“瘦身”而不失真。**

[中文介绍](#-为什么需要它) | [English Introduction](#-why-this-project)

---

## 🌟 为什么需要它？

你是否遇到过这些尴尬瞬间：
- 📸 **手机/电脑空间告急**：几百个视频占满了硬盘，想留着回忆却没地方放？
- ⏳ **上传进度条纹丝不动**：想发个高清视频给朋友，结果因为文件太大，传了半天还失败？
- 💻 **压缩软件太复杂**：网上的工具要么收费，要么操作繁琐，调个参数像在考研？

**这个工具就是为你准备的！** 它能帮你一键把“巨无霸”视频变成“小而美”，且肉眼几乎看不出画质损失。

### ✨ 它能解决什么问题？
- **全自动批量处理**：哪怕你有 100 个视频，丢进去点一下，它就自动排队帮你压好。
- **聪明地“压榨”性能**：它会自动识别你的电脑是否有 NVIDIA、Intel 或 Apple 的显卡硬件加速，像专业剪辑师一样榨干电脑性能，速度飞快。
- **小白级操作**：不需要懂什么是 Bitrate（码率）或 H.265，选个“画质优先”或“速度优先”预设，剩下的交给我。
- **完全免费且隐私**：所有处理都在你本地电脑完成，视频不会上传到任何服务器，安全可靠。

---

## 🛠 快速上手 (Quick Start)

1. **安装环境**：确保你的电脑安装了 Python。
2. **下载依赖**：
   ```bash
   pip install -r requirements.txt
   ```
3. **启动程序**：
   ```bash
   python app_refactored.py
   ```

---

## 🤓 技术细节 (For Techies)

如果你是开发者或技术爱好者，这里有你关心的：

- **核心引擎**：基于 [FFmpeg](https://ffmpeg.org/)。
- **自动能力探测**：内置 `FFmpegHandler`，启动时自动扫描系统路径及本地 `bin` 目录，检测可用编码器。
- **硬件加速支持**：
  - **NVIDIA**: `h264_nvenc`, `hevc_nvenc`
  - **Intel**: `h264_qsv`, `hevc_qsv`
  - **Apple**: `h264_videotoolbox`, `hevc_videotoolbox`
- **并发处理**：采用多线程架构，UI 与压缩逻辑分离，确保处理过程中界面不卡顿。
- **智能预设系统**：根据目标 CRF 值和编码器特性自动构建命令行参数。

---

## 📢 有经验发布者的温馨提示

作为一个成熟的项目发布者，除了代码，我们还关注这些：

1. **License (授权协议)**：本项目采用 **MIT 协议**，意味着你可以自由地使用、修改和分发，非常友好。
2. **Issue 交流**：如果你发现 Bug 或者有新点子，欢迎在 [Issues](https://github.com/cscb603/fast-video-compress/issues) 留言，我会尽快回复。
3. **版本管理**：建议关注 `Releases` 页面，我们会定期发布打包好的版本。

---

## English Introduction

### 🚀 Why this project?
Tired of "Storage Full" warnings? This tool provides a one-click solution to compress massive video files without noticeable quality loss. It's designed for everyone—from casual users to tech enthusiasts.

### ✨ Key Features
- **Batch Processing**: Handle hundreds of videos automatically.
- **Hardware Acceleration**: Automatically detects and uses NVIDIA, Intel, or Apple GPUs for ultra-fast encoding.
- **Privacy First**: All processing happens locally on your machine.
- **Zero Configuration**: Smart presets for non-technical users.

---

## 开源协议 (License)
MIT License

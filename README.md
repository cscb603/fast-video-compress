# 星TAP 极速视频压缩 | StarTAP Video Compressor

[![Platform](https://img.shields.io/badge/platform-Windows-blue.svg)](https://github.com/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

### **[中文]**

**视频太占空间？发给朋友太慢？这个工具能帮你瞬间“瘦身”。**

![界面演示](视频压缩图标.png)

**星TAP | 极速视频压缩 (StarTAP Video Compressor)** 是一款专为视频创作者和存储压力大的用户打造的极速压缩工具。基于 Rust 核心和 FFmpeg 引擎，它能在几乎不损失画质的前提下，把几 GB 的视频压缩到几百 MB，特别适合发朋友圈、发邮件或节省硬盘空间。

- **✅ 暴力压缩**: 采用先进编码算法，视频体积最高可减少 80%，画质依然清晰。
- **✅ 极速处理**: 充分压榨电脑 CPU 性能，支持多视频并行压缩，告别漫长等待。
- **✅ 绿色便携**: 
  - **国内用户（推荐）**: 我们在 Release 页面提供了“内置依赖版”，下载解压即用，无需额外配置 FFmpeg。
  - **小白友好**: 支持拖拽文件，一键开始，不需要看懂复杂的参数。
- **✅ 安全可靠**: 本地处理，不上传云端，保护你的隐私。

#### **如何使用 (Win 版)**
1. 在 [GitHub Releases](https://github.com/cscb603/StarTap-Video-Compressor/releases) 下载 `StarTAP_Video_Compressor_v1.0.0_Win_Portable.zip`。
2. 解压到任意文件夹。
3. 双击 `星TAP极速视频压缩.exe` 即可开始。
*注意：如果你的电脑没有安装 FFmpeg，请务必下载带依赖的完整版。*

---

### **[English]**

**Video files too large? Slow to share? Let your videos "slim down" instantly.**

**StarTAP | Video Compressor** is a high-speed compression tool powered by Rust and FFmpeg. It delivers exceptional compression ratios (up to 80% reduction) while maintaining high visual quality. Perfect for social media, email, or saving disk space.

- **Key Features**:
  - **Efficient Compression**: Significant size reduction with minimal quality loss.
  - **High Performance**: Multi-threaded engine for blazing-fast processing.
  - **Standalone & Portable**: 
    - **Global Users**: For the best experience, we recommend downloading the "Full Version" which includes FFmpeg binaries.
    - **No Setup Required**: Just download, extract, and run.
  - **Privacy First**: All processing happens locally on your machine.

#### **Quick Start**
1. Download `StarTAP_Video_Compressor_v1.0.0_Win_Portable.zip` from [Releases](https://github.com/cscb603/StarTap-Video-Compressor/releases).
2. Extract the archive.
3. Run `StarTAP_Video_Compressor.exe`.

---

## 🛠️ 技术规格 (Technical Specs)
- **Engine**: Rust + FFmpeg
- **Backend**: Libx264 / Libx265
- **OS Support**: Windows 10/11

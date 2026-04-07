# 🚀 星TAP 极简视频压缩 V3 | StarTAP Simple Video Compressor V3

![主界面](视频压缩界面v3-1.png)

---

## 📖 中文说明 / Chinese Docs

### 这玩意儿是干啥的？ / What is this

**一句话介绍**：一个极简、高效、零安装的视频压缩神器，把你的视频变得**更小巧、更易分享、更省空间**！

---

### 为什么你需要它？ / Why do you need it

#### 😰 你是否遇到过这些痛点？

- **「微信发视频又被压缩了！」**：原片 200MB，微信发不出去，发出去画质糊成渣
- **「硬盘又满了！」**：手机拍的 4K 视频，存了几十个就占了几十 GB
- **「传百度网盘要传半天！」**：大文件上传下载，等得花儿都谢了
- **「视频压缩软件好复杂！」**：一堆参数看不懂，不知道怎么调
- **「还要安装？我怕麻烦！」**：不想装一堆软件，想用就用，用完就走

#### ✨ 这个工具帮你解决！

- **微信秒发**：1080p 视频压缩到 10-20MB，微信直接发，画质还不错
- **节省空间**：相同画质下，体积比原片小 60%-80%
- **极速传输**：文件小了，上传下载都快
- **小白友好**：5 个档位一键选，不用懂参数
- **绿色便携**：不用安装，双击即用，拷到 U 盘带走

---

### 有哪些厉害的功能？ / Key Features

| 功能                          | 说明                                             |
| ----------------------------- | ------------------------------------------------ |
| 🖱️ **极简 GUI 界面**          | 不用记命令，小白也能秒上手                       |
| 💻 **CLI 命令行 + JSON 输出** | 完美支持 OpenClaw 小龙虾、各种 AI 大模型工具调用 |
| 🎯 **5 档预设**               | 一键选择，不用调参数                             |
| 📱 **竖屏视频智能适配**       | 抖音/小红书视频，自动限制宽度，不变形            |
| ⚡ **NVIDIA 硬件加速**        | 有 N 卡？速度飞起！                              |
| 🧠 **智能 FFmpeg 检测**       | 自动找 FFmpeg，不用配置                          |
| 📂 **批量处理**               | 一次拖入 100 个视频，自动排队压缩                |

---

### 5 档预设，一键选择！ / 5 Preset Profiles

| 档位                               | 适用场景             | 预期效果                 |
| ---------------------------------- | -------------------- | ------------------------ |
| 🎬 **原始大小 - 不缩放**           | 专业素材、保留原画质 | 保持原分辨率，只压缩码率 |
| 📦 **1080p (临时分享 · 极致压缩)** | 微信/QQ 快速分享     | 文件极小，适合临时预览   |
| ⚖️ **1080p (推荐 · 平衡)**         | **日常使用首选**     | 画质和体积的完美平衡     |
| 📺 **1440p (2K) - 高清**           | 2K 视频存档          | 高清质量，节省空间       |
| 🎬 **2160p (4K) - 原画**           | 4K 专业素材          | 接近原画画质，体积减半   |

---

### 怎么用？ / How to use

#### 🖱️ GUI 模式（推荐给普通用户）

1. **打开程序**：双击 `星TAP视频压缩v3.exe`
2. **添加视频**：
   - 点击「➕ 添加文件」选择视频
   - 点击「📂 添加文件夹」选择整个文件夹
   - 直接把视频或文件夹拖到界面上！
3. **选择档位**：推荐「1080p (推荐 · 平衡)」
4. **开始压缩**：点击「🚀 开始批量压缩」

#### 💻 CLI 模式（给技术用户和 AI 工具）

```bash
# 快速压缩一个视频
fast-video-cli.exe --input 视频.mp4 --output-dir 输出文件夹

# 临时分享模式（极致压缩）
fast-video-cli.exe --input 视频.mp4 --quick-share --max-height 1080 --quality 32 --speed-preset 10

# 压缩整个文件夹
fast-video-cli.exe --input 视频文件夹 --output-dir 输出文件夹

# JSON 输出（AI 调用）
fast-video-cli.exe --input 视频.mp4 --json
```

---

### 常见问题 / FAQ

**Q: 压缩后的视频在哪里？** A: 如果你没选导出目录，就在原视频旁边，文件名会加
`_s` 后缀。

**Q: 为什么有些视频无法播放？** A: AV1 格式需要新设备（Win11 22H2+/macOS
13+）或 VLC 播放器（下载地址：<https://www.videolan.org/vlc/）。>

**Q: 压缩速度太慢怎么办？** A: 如果你有 NVIDIA 显卡，选 NVENC 编码器！极快！

**Q: 竖屏视频会怎么处理？**
A: 程序会自动判断！竖屏视频会限制宽度而不是高度，保持画面比例！

**Q: 可以同时压缩多个视频吗？**
A: 可以！你可以添加多个视频，程序会自动排队处理！

---

## 📖 English Documentation

### What is this

**One-sentence intro**: A simple, efficient, zero-install video compressor that
makes your videos **smaller, easier to share, and space-saving**!

---

### Why do you need it

#### 😰 Do you have these pain points

- **"WeChat compressed my video again!"**: Original 200MB video can't be sent on
  WeChat, and if sent, quality is ruined
- **"Hard drive is full again!"**: 4K videos from phone take up tens of GBs
  after just a few
- **"Uploading to cloud takes forever!"**: Large files take ages to upload and
  download
- **"Video compression software is too complicated!"**: Too many parameters,
  don't know how to adjust
- **"Need to install? Too much trouble!"**: Don't want to install software, use
  it on demand

#### ✨ This tool solves it

- **Instant WeChat sharing**: 1080p videos compressed to 10-20MB, send directly
  on WeChat with decent quality
- **Save space**: 60-80% smaller than original with same quality
- **Fast transfer**: Smaller files mean faster uploads and downloads
- **Beginner friendly**: 5 presets, no need to understand parameters
- **Green portable**: No installation needed, double-click to use, carry on USB

---

### Key Features

| Feature                                | Description                                            |
| -------------------------------------- | ------------------------------------------------------ |
| 🖱️ **Simple GUI**                      | No commands to remember, beginners can use instantly   |
| 💻 **CLI + JSON Output**               | Perfect for OpenClaw and various AI LLM tool calling   |
| 🎯 **5 Preset Profiles**               | One-click selection, no parameter tuning               |
| 📱 **Vertical Video Smart Adaptation** | TikTok/RedNote videos, auto limit width, no distortion |
| ⚡ **NVIDIA Hardware Acceleration**    | Have an NVIDIA GPU? Blazing fast!                      |
| 🧠 **Smart FFmpeg Detection**          | Auto finds FFmpeg, no configuration needed             |
| 📂 **Batch Processing**                | Drag in 100 videos at once, auto queue compression     |

---

### 5 Preset Profiles

| Preset                                           | Use Case                                    | Expected Result                                 |
| ------------------------------------------------ | ------------------------------------------- | ----------------------------------------------- |
| 🎬 **Original Size - No Scaling**                | Professional footage, keep original quality | Keep original resolution, only compress bitrate |
| 📦 **1080p (Quick Share · Extreme Compression)** | WeChat/QQ quick sharing                     | Minimal file size, good for temporary preview   |
| ⚖️ **1080p (Recommended · Balanced)**            | **Daily use recommended**                   | Perfect balance of quality and size             |
| 📺 **1440p (2K) - HD**                           | 2K video archive                            | HD quality, space saving                        |
| 🎬 **2160p (4K) - Original**                     | 4K professional footage                     | Near original quality, half the size            |

---

### How to use

#### 🖱️ GUI Mode (Recommended for regular users)

1. **Open the program**: Double-click `星TAP视频压缩v3.exe`
2. **Add videos**:
   - Click "➕ Add Files" to select videos
   - Click "📂 Add Folder" to select an entire folder
   - Just drag and drop videos or folders onto the interface!
3. **Select preset**: Recommend "1080p (Recommended · Balanced)"
4. **Start compression**: Click "🚀 Start Batch Compression"

#### 💻 CLI Mode (For technical users and AI tools)

```bash
# Quick compress one video
fast-video-cli.exe --input video.mp4 --output-dir ./output

# Quick share mode (extreme compression)
fast-video-cli.exe --input video.mp4 --quick-share --max-height 1080 --quality 32 --speed-preset 10

# Compress entire folder
fast-video-cli.exe --input ./videos --output-dir ./output

# JSON output (for AI integration)
fast-video-cli.exe --input video.mp4 --json
```

---

## 📦 File Description

- `星TAP视频压缩v3.exe` - GUI 版本 / GUI Version
- `fast-video-cli.exe` - CLI 版本 / CLI Version
- `ffmpeg.exe` - FFmpeg 编码器 / FFmpeg Encoder
- `ffprobe.exe` - FFprobe 视频信息检测工具 / FFprobe Video Info Tool

## ⚙️ System Requirements

- Windows 10 或更高版本 / Windows 10 or later
- NVIDIA 显卡（使用 NVENC 需要）/ NVIDIA GPU (for NVENC)

---

## 🎉 开始使用吧！/ Get Started

双击 **「星TAP视频压缩v3.exe」** 开始你的视频压缩之旅！

---

**StarTAP Labs © 2026** | 极致速度，极简生活。

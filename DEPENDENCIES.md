# 依赖说明 | Dependencies

## macOS 编译依赖

### 系统要求
- macOS 10.15 或更高版本
- Xcode Command Line Tools

### 安装 Xcode Command Line Tools
```bash
xcode-select --install
```

### Rust 环境
- Rust 1.70 或更高版本
- Cargo 包管理器

安装 Rust：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### FFmpeg (必需)
项目需要 FFmpeg 进行视频编码。

**使用 Homebrew 安装：**
```bash
brew install ffmpeg
```

**或手动下载：**
- 访问 https://ffmpeg.org/download.html
- 下载 macOS 版本的 FFmpeg
- 解压后将 `ffmpeg` 和 `ffprobe` 放入项目目录或系统 PATH

## Rust Crate 依赖

### 核心依赖
| Crate | 版本 | 用途 |
|-------|------|------|
| tokio | 1.0 | 异步运行时 |
| clap | 4.5 | 命令行参数解析 |
| serde | 1.0 | 序列化/反序列化 |
| serde_json | 1.0 | JSON 处理 |
| walkdir | 2.3 | 目录遍历 |
| anyhow | 1.0 | 错误处理 |
| indicatif | 0.17 | 进度条 |
| futures | 0.3 | 异步工具 |

### GUI 依赖
| Crate | 版本 | 用途 |
|-------|------|------|
| eframe | 0.28 | GUI 框架 (egui) |
| rfd | 0.14 | 文件对话框 |
| confy | 0.6 | 配置管理 |
| image | 0.25 | 图像处理 |

### 工具依赖
| Crate | 版本 | 用途 |
|-------|------|------|
| rayon | 1.10 | 并行计算 |
| num_cpus | 1.16 | CPU 核心数检测 |
| dirs | 5.0 | 系统目录 |
| sysinfo | 0.30 | 系统信息 |
| once_cell | 1.19 | 延迟初始化 |
| opener | 0.6 | 打开文件/URL |
| which | 4.4 | 查找可执行文件 |
| log | 0.4 | 日志 |
| simplelog | 0.12 | 简单日志实现 |

## 编译命令

### 开发模式
```bash
cargo build
```

### 发布模式
```bash
cargo build --release
```

### 运行 GUI
```bash
cargo run --bin fast-video-compress-gui
```

### 运行 CLI
```bash
cargo run --bin fast-video-compress-cli -- --help
```

## macOS 打包

### 创建 .app 包 (可选)
可以使用 `cargo-bundle` 工具创建 macOS .app 包：

```bash
# 安装 cargo-bundle
cargo install cargo-bundle

# 打包
cargo bundle --release
```

## 注意事项

1. **FFmpeg 路径**：程序会自动检测系统中的 FFmpeg，也可以手动指定路径
2. **硬件加速**：macOS 支持 VideoToolbox 硬件加速
3. **图标**：macOS 需要 .icns 格式图标，可以使用 `iconutil` 从 .png 转换

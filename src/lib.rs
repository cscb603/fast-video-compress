use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use walkdir::WalkDir;
use std::fs;
use dirs;

#[cfg(windows)]
#[allow(unused_imports)]
use std::os::windows::process::CommandExt;

pub fn get_config_file_path() -> Result<PathBuf> {
    if let Some(mut path) = dirs::config_dir() {
        path.push("fast_video_compressor");
        fs::create_dir_all(&path)?;
        path.push("config_v4.toml");
        Ok(path)
    } else {
        Ok(PathBuf::from("video_compressor_config_v4.toml"))
    }
}

pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_file_path()?;
    if config_path.exists() {
        let config_str = fs::read_to_string(config_path)?;
        let config = toml::from_str(&config_str)?;
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_file_path()?;
    let config_str = toml::to_string_pretty(config)?;
    fs::write(config_path, config_str)?;
    Ok(())
}

pub fn path_self_healing(input_path: &Path) -> PathBuf {
    let path_str = input_path.to_string_lossy();

    if input_path.exists() && input_path.is_file() {
        return input_path.to_path_buf();
    }

    if let Some(file_name) = input_path.file_name().and_then(|n| n.to_str()) {
        if let Some(parent) = input_path.parent() {
            if parent.exists() {
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.eq_ignore_ascii_case(file_name) {
                                let candidate = entry.path();
                                if candidate.is_file() {
                                    return candidate;
                                }
                            }
                        }
                    }
                }
            }

            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.to_lowercase() == file_name.to_lowercase() {
                            let candidate = entry.path();
                            if candidate.is_file() {
                                return candidate;
                            }
                        }
                    }
                }
            }
        }
    }

    let normalized = path_str.replace("\\", "/");
    if normalized != path_str {
        let alt_path = Path::new(&normalized);
        if alt_path.exists() && alt_path.is_file() {
            return alt_path.to_path_buf();
        }
    }

    input_path.to_path_buf()
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Encoder {
    LibsvtAv1,
    Rav1e,
    HevcNvenc,
    HevcVideotoolbox,
    Libx265,
}

#[allow(clippy::derivable_impls)]
impl Default for Encoder {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        {
            Encoder::LibsvtAv1
        }
        #[cfg(target_os = "macos")]
        {
            Encoder::HevcVideotoolbox
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            Encoder::LibsvtAv1
        }
    }
}

impl std::fmt::Display for Encoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Encoder::LibsvtAv1 => write!(f, "SVT-AV1 (推荐)"),
            Encoder::Rav1e => write!(f, "rav1e (极致压缩)"),
            Encoder::HevcNvenc => write!(f, "HEVC NVENC (NVIDIA 硬件)"),
            Encoder::HevcVideotoolbox => write!(f, "HEVC VideoToolbox (Apple 硬件)"),
            Encoder::Libx265 => write!(f, "x265 (兼容性优先)"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub enum AudioCodec {
    #[default]
    Opus,
    Aac,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub output_dir: String,
    pub quality: u32,
    pub concurrency: usize,
    pub encoder: Encoder,
    pub speed_preset: u8,
    pub audio_codec: AudioCodec,
    pub audio_bitrate: String,
    pub max_height: i32,
    pub is_quick_share: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            output_dir: String::new(),
            quality: 24,
            concurrency: 2,
            encoder: Encoder::default(),
            speed_preset: 8,
            audio_codec: AudioCodec::default(),
            audio_bitrate: "128k".to_string(),
            max_height: 1080,
            is_quick_share: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProcessConfig {
    pub output_dir: Option<PathBuf>,
    pub quality: u32,
    pub concurrency: usize,
    pub encoder: Encoder,
    pub speed_preset: u8,
    pub audio_codec: AudioCodec,
    pub audio_bitrate: String,
    pub max_height: i32,
    pub is_quick_share: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub width: i32,
    pub height: i32,
    pub fps: f64,
}

#[derive(Clone, Debug, Default)]
pub struct HardwareCapabilities {
    pub has_videotoolbox: bool,
    pub has_nvenc: bool,
    pub has_quick_sync: bool,
    pub has_vaapi: bool,
}

impl HardwareCapabilities {
    pub fn detect() -> Self {
        let mut caps = Self::default();
        
        #[cfg(target_os = "macos")]
        {
            caps.has_videotoolbox = true;
        }
        
        #[cfg(windows)]
        {
            if let Ok(output) = std::process::Command::new("nvidia-smi")
                .arg("--query-gpu=name")
                .arg("--format=csv,noheader")
                .output()
            {
                caps.has_nvenc = output.status.success();
            }
        }
        
        caps
    }
    
    pub fn get_recommended_encoder(&self) -> Encoder {
        #[cfg(target_os = "macos")]
        {
            if self.has_videotoolbox {
                return Encoder::HevcVideotoolbox;
            }
        }
        
        #[cfg(windows)]
        {
            if self.has_nvenc {
                return Encoder::HevcNvenc;
            }
        }
        
        Encoder::LibsvtAv1
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FFProbeOutput {
    streams: Vec<Stream>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Stream {
    width: Option<i32>,
    height: Option<i32>,
    r_frame_rate: Option<String>,
    codec_type: String,
}

pub async fn get_video_metadata(path: &Path) -> Result<VideoMetadata> {
    let healed_path = path_self_healing(path);
    let ffprobe_path = find_ffmpeg_tool("ffprobe")?;
    let mut cmd = Command::new(ffprobe_path);
    cmd.args([
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height,r_frame_rate,codec_type",
        "-of",
        "json",
    ])
    .arg(&healed_path);

    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000);
    }

    let output = cmd.output().await?;
    let probe: FFProbeOutput = serde_json::from_slice(&output.stdout)?;
    let s = probe
        .streams
        .iter()
        .find(|s| s.codec_type == "video")
        .context("No video stream found")?;
    let fps = s.r_frame_rate.as_ref().map(|r| {
        let p: Vec<&str> = r.split('/').collect();
        if p.len() == 2 {
            p[0].parse::<f64>().unwrap_or(30.0) / p[1].parse::<f64>().unwrap_or(1.0)
        } else {
            r.parse().unwrap_or(30.0)
        }
    }).unwrap_or(30.0);
    Ok(VideoMetadata {
        width: s.width.unwrap_or(0),
        height: s.height.unwrap_or(0),
        fps,
    })
}

pub fn find_ffmpeg_tool(tool_name: &str) -> Result<PathBuf> {
    let mut candidate_paths = Vec::new();

    // 优先在 app 包内查找（用于分发版）
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            candidate_paths.push(parent.join(tool_name));
            candidate_paths.push(parent.join(format!("{}.exe", tool_name)));
        }
    }

    #[cfg(windows)]
    {
        candidate_paths.push(PathBuf::from(format!(r"C:\ffmpeg-8.0\bin\{}.exe", tool_name)));
        candidate_paths.push(PathBuf::from(format!(r"C:\ffmpeg-7.1\bin\{}.exe", tool_name)));
        candidate_paths.push(PathBuf::from(format!(r"C:\ffmpeg-7.0\bin\{}.exe", tool_name)));
        candidate_paths.push(PathBuf::from(format!(r"C:\ffmpeg\bin\{}.exe", tool_name)));
        candidate_paths.push(PathBuf::from(format!(r"C:\Program Files\ffmpeg\bin\{}.exe", tool_name)));
        candidate_paths.push(PathBuf::from(format!(r"C:\Program Files (x86)\ffmpeg\bin\{}.exe", tool_name)));
    }

    #[cfg(target_os = "macos")]
    {
        candidate_paths.push(PathBuf::from(format!("/usr/local/bin/{}", tool_name)));
        candidate_paths.push(PathBuf::from(format!("/opt/homebrew/bin/{}", tool_name)));
        candidate_paths.push(PathBuf::from(format!("/opt/homebrew/opt/ffmpeg/bin/{}", tool_name)));
        candidate_paths.push(PathBuf::from(format!("/usr/bin/{}", tool_name)));
    }

    for path in candidate_paths {
        if path.exists() {
            if let Ok(version) = check_ffmpeg_version(&path, tool_name) {
                log::debug!("找到 FFmpeg {}: {:?} (版本: {})", tool_name, path, version);
                return Ok(path);
            }
        }
    }

    let path_in_env = which::which(tool_name);
    if let Ok(path) = path_in_env {
        if let Ok(version) = check_ffmpeg_version(&path, tool_name) {
            log::debug!("在 PATH 找到 FFmpeg {}: {:?} (版本: {})", tool_name, path, version);
            return Ok(path);
        }
    }

    Err(anyhow::anyhow!(
        "未找到 FFmpeg {}！请确保 FFmpeg 已安装并在 PATH 中，或放在程序同级目录下。\nMac 用户: brew install ffmpeg\nWindows 用户: 下载并添加到 PATH\n推荐下载地址: https://ffmpeg.org/download.html",
        tool_name
    ))
}

fn check_ffmpeg_version(path: &Path, tool_name: &str) -> Result<String> {
    let mut cmd = std::process::Command::new(path);
    cmd.arg("-version");

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let output = cmd.output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("{} 执行失败", tool_name));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let first_line = output_str.lines().next().unwrap_or("");
    
    if first_line.contains("ffmpeg") || first_line.contains("ffprobe") {
        let version_str = first_line.split_whitespace().nth(2).unwrap_or("unknown");
        Ok(version_str.to_string())
    } else {
        Err(anyhow::anyhow!("无效的 {} 可执行文件", tool_name))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub success: bool,
    pub error: Option<String>,
    pub original_size: u64,
    pub compressed_size: u64,
}

#[derive(Clone)]
pub struct VideoProcessor {
    config: ProcessConfig,
}

impl VideoProcessor {
    pub fn new(config: ProcessConfig) -> Self {
        Self { config }
    }

    fn get_size_suffix(&self) -> &'static str {
        "_s"
    }

    pub async fn compress_video(&self, input_path: &Path) -> Result<CompressionResult> {
        let healed_path = path_self_healing(input_path);
        let original_size = std::fs::metadata(&healed_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let file_stem = healed_path.file_stem().unwrap().to_string_lossy();
        let output_dir = self
            .config
            .output_dir
            .clone();
        
        let output_path = if let Some(output_dir) = output_dir {
            if !output_dir.exists() {
                std::fs::create_dir_all(&output_dir)?;
            }
            
            let size_suffix = self.get_size_suffix();
            let mut output_path = output_dir.join(format!("{}{}.mp4", file_stem, size_suffix));
            
            let mut counter = 1;
            while output_path.exists() {
                output_path = output_dir.join(format!("{}{}_{}.mp4", file_stem, size_suffix, counter));
                counter += 1;
            }
            output_path
        } else {
            let parent_dir = input_path.parent().unwrap_or(Path::new(".")).to_path_buf();
            let size_suffix = self.get_size_suffix();
            let mut output_path = parent_dir.join(format!("{}{}.mp4", file_stem, size_suffix));
            
            let mut counter = 1;
            while output_path.exists() {
                output_path = parent_dir.join(format!("{}{}_{}.mp4", file_stem, size_suffix, counter));
                counter += 1;
            }
            output_path
        };

        let metadata = match get_video_metadata(&healed_path).await {
            Ok(m) => m,
            Err(e) => {
                return Ok(CompressionResult {
                    input_path: healed_path.to_path_buf(),
                    output_path: output_path.clone(),
                    success: false,
                    error: Some(format!("Failed to get metadata: {}", e)),
                    original_size,
                    compressed_size: 0,
                });
            }
        };

        // 简单逻辑：如果是 0 → 原始大小；如果是竖屏（高>宽）→ 限制宽度；如果是横屏（宽>高）→ 限制高度
        let vf = if self.config.max_height == 0 {
            // 0 表示原始大小，不缩放，但还是要确保偶数
            "scale=-2:trunc(ih/2)*2".to_string()
        } else if metadata.height > metadata.width {
            // 竖屏视频 → 限制宽度为 max_height（其实是最大宽度）
            format!("scale={}:-2", self.config.max_height)
        } else {
            // 横屏视频 → 限制高度为 max_height（正常情况）
            format!("scale=-2:{}", self.config.max_height)
        };

        let ffmpeg_path = find_ffmpeg_tool("ffmpeg")?;
        let mut args: Vec<String> = vec![
            "-hide_banner".to_string(),
            "-loglevel".to_string(),
            "error".to_string(),
            "-y".to_string(),
            "-hwaccel".to_string(),
            "auto".to_string(),
            "-i".to_string(),
            healed_path.to_string_lossy().to_string(),
            "-vf".to_string(),
            format!("{},format=yuv420p", vf),
        ];

        match self.config.encoder {
            Encoder::LibsvtAv1 => {
                args.extend([
                    "-c:v".to_string(),
                    "libsvtav1".to_string(),
                    "-crf".to_string(),
                    self.config.quality.to_string(),
                    "-preset".to_string(),
                    self.config.speed_preset.to_string(),
                    "-g".to_string(),
                    "240".to_string(),
                ]);
            }
            Encoder::Rav1e => {
                args.extend([
                    "-c:v".to_string(),
                    "librav1e".to_string(),
                    "-crf".to_string(),
                    self.config.quality.to_string(),
                    "-speed".to_string(),
                    self.config.speed_preset.to_string(),
                ]);
            }
            Encoder::HevcNvenc => {
                let cq = (51.0 * (1.0 - (self.config.quality as f32 / 100.0))) as u32;
                args.extend([
                    "-c:v".to_string(),
                    "hevc_nvenc".to_string(),
                    "-rc".to_string(),
                    "vbr".to_string(),
                    "-cq".to_string(),
                    cq.to_string(),
                    "-preset".to_string(),
                    "p6".to_string(),
                    "-spatial-aq".to_string(),
                    "1".to_string(),
                ]);
            }
            Encoder::HevcVideotoolbox => {
                args.extend([
                    "-c:v".to_string(),
                    "hevc_videotoolbox".to_string(),
                    "-q:v".to_string(),
                    self.config.quality.to_string(),
                    "-preset".to_string(),
                    "hq".to_string(),
                    "-tag:v".to_string(),
                    "hvc1".to_string(),
                ]);
            }
            Encoder::Libx265 => {
                args.extend([
                    "-c:v".to_string(),
                    "libx265".to_string(),
                    "-crf".to_string(),
                    self.config.quality.to_string(),
                    "-preset".to_string(),
                    "medium".to_string(),
                ]);
            }
        }

        match self.config.audio_codec {
            AudioCodec::Opus => {
                args.extend([
                    "-c:a".to_string(),
                    "libopus".to_string(),
                    "-b:a".to_string(),
                    self.config.audio_bitrate.clone(),
                ]);
            }
            AudioCodec::Aac => {
                args.extend([
                    "-c:a".to_string(),
                    "aac".to_string(),
                    "-b:a".to_string(),
                    self.config.audio_bitrate.clone(),
                ]);
            }
        }

        args.extend([
            "-map_metadata".to_string(),
            "0".to_string(),
            "-movflags".to_string(),
            "+faststart+use_metadata_tags".to_string(),
            output_path.to_string_lossy().to_string(),
        ]);

        let mut cmd = Command::new(ffmpeg_path);
        cmd.args(args);

        #[cfg(windows)]
        {
            cmd.creation_flags(0x08000000);
        }

        let status = cmd.status().await;
        let (success, error) = match status {
            Ok(s) if s.success() => (true, None),
            Ok(s) => (false, Some(format!("FFmpeg exited with code: {}", s))),
            Err(e) => (false, Some(format!("FFmpeg error: {}", e))),
        };

        let compressed_size = if success {
            std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        Ok(CompressionResult {
            input_path: healed_path.to_path_buf(),
            output_path,
            success,
            error,
            original_size,
            compressed_size,
        })
    }
}

pub fn collect_video_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let p = entry.path();
                if let Some(ext) = p.extension() {
                    if [
                        "mp4", "mkv", "mov", "avi", "wmv", "flv", "webm", "ts", "m2ts",
                    ]
                    .contains(&ext.to_string_lossy().to_lowercase().as_str())
                    {
                        files.push(p.to_path_buf());
                    }
                }
            }
        } else {
            files.push(path.to_path_buf());
        }
    }
    files
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonInput {
    pub version: String,
    pub encoder: Option<String>,
    pub quality: Option<u32>,
    pub speed_preset: Option<u8>,
    pub concurrency: Option<usize>,
    pub output_dir: Option<String>,
    pub audio_codec: Option<String>,
    pub audio_bitrate: Option<String>,
    pub max_height: Option<i32>,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutput {
    pub success: bool,
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub results: Vec<FileResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileResult {
    pub input: String,
    pub output: Option<String>,
    pub success: bool,
    pub error: Option<String>,
    pub original_size: Option<u64>,
    pub compressed_size: Option<u64>,
    pub compression_ratio: Option<f64>,
}

pub fn app_config_to_process_config(
    config: &AppConfig,
    output_dir: Option<PathBuf>,
) -> ProcessConfig {
    let dir = if output_dir.is_some() {
        output_dir
    } else if !config.output_dir.is_empty() {
        Some(PathBuf::from(&config.output_dir))
    } else {
        None
    };
    ProcessConfig {
        output_dir: dir,
        quality: config.quality,
        concurrency: config.concurrency,
        encoder: config.encoder,
        speed_preset: config.speed_preset,
        audio_codec: config.audio_codec,
        audio_bitrate: config.audio_bitrate.clone(),
        max_height: config.max_height,
        is_quick_share: config.is_quick_share,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.quality, 24);
        assert_eq!(config.speed_preset, 8);
    }
}

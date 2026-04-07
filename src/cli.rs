use clap::Parser;
use fast_video_compress_rs_v2::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fast-video-compress-cli")]
#[command(about = "星TAP 视频压缩工具 CLI 版", long_about = None)]
pub struct Cli {
    #[arg(long, short = 'i', value_name = "FILE/DIR")]
    pub input: Vec<PathBuf>,

    #[arg(long, value_name = "DIR")]
    pub output_dir: Option<PathBuf>,

    #[arg(long, value_enum, default_value = "svt-av1")]
    pub encoder: CliEncoder,

    #[arg(long, default_value_t = 24)]
    pub quality: u32,

    #[arg(long, default_value_t = 8)]
    pub speed_preset: u8,

    #[arg(long, default_value_t = 2)]
    pub concurrency: usize,

    #[arg(long, value_enum, default_value = "opus")]
    pub audio_codec: CliAudioCodec,

    #[arg(long, default_value = "128k")]
    pub audio_bitrate: String,

    #[arg(long, default_value_t = 1080)]
    pub max_height: i32,

    #[arg(long)]
    pub quick_share: bool,

    #[arg(long)]
    pub json: bool,

    #[arg(long, short = 'q')]
    pub quiet: bool,

    #[arg(value_name = "FILE/DIR")]
    pub positional: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CliEncoder {
    SvtAv1,
    Rav1e,
    HevcNvenc,
    HevcVideotoolbox,
    X265,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CliAudioCodec {
    Opus,
    Aac,
}

impl From<CliEncoder> for Encoder {
    fn from(encoder: CliEncoder) -> Self {
        match encoder {
            CliEncoder::SvtAv1 => Encoder::LibsvtAv1,
            CliEncoder::Rav1e => Encoder::Rav1e,
            CliEncoder::HevcNvenc => Encoder::HevcNvenc,
            CliEncoder::HevcVideotoolbox => Encoder::HevcVideotoolbox,
            CliEncoder::X265 => Encoder::Libx265,
        }
    }
}

impl From<CliAudioCodec> for AudioCodec {
    fn from(codec: CliAudioCodec) -> Self {
        match codec {
            CliAudioCodec::Opus => AudioCodec::Opus,
            CliAudioCodec::Aac => AudioCodec::Aac,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut all_inputs = cli.input.clone();
    all_inputs.extend(cli.positional.clone());

    if all_inputs.is_empty() {
        eprintln!("错误: 请指定输入文件或目录");
        std::process::exit(1);
    }

    let video_files = collect_video_files(&all_inputs);

    if video_files.is_empty() {
        eprintln!("错误: 未找到视频文件");
        std::process::exit(1);
    }

    let config = ProcessConfig {
        output_dir: cli.output_dir.clone(),
        quality: cli.quality,
        concurrency: cli.concurrency,
        encoder: cli.encoder.into(),
        speed_preset: cli.speed_preset,
        audio_codec: cli.audio_codec.into(),
        audio_bitrate: cli.audio_bitrate.clone(),
        max_height: cli.max_height,
        is_quick_share: cli.quick_share,
    };

    let processor = VideoProcessor::new(config);

    if !cli.quiet && !cli.json {
        println!("星TAP 视频压缩工具 V3");
        println!("======================");
        println!("找到 {} 个视频文件", video_files.len());
        println!("编码器: {:?}", cli.encoder);
        println!("画质: {}", cli.quality);
        println!("速度预设: {}", cli.speed_preset);
        println!("并发数: {}", cli.concurrency);
        println!();
    }

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(cli.concurrency));
    let mut tasks = Vec::new();

    for file in video_files {
        let processor = processor.clone();
        let sem = semaphore.clone();
        let quiet = cli.quiet;
        let json_mode = cli.json;

        let task = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            
            if !quiet && !json_mode {
                println!("处理中: {}", file.to_string_lossy());
            }

            let result = processor.compress_video(&file).await;
            
            if !quiet && !json_mode {
                match &result {
                    Ok(r) if r.success => {
                        println!(
                            "  ✓ 完成: {} → {} (节省 {:.1}%)",
                            r.input_path.to_string_lossy(),
                            format_size(r.compressed_size),
                            if r.original_size > 0 {
                                (1.0 - r.compressed_size as f64 / r.original_size as f64) * 100.0
                            } else {
                                0.0
                            }
                        );
                    }
                    Ok(r) => {
                        println!("  ✗ 失败: {} - {:?}", r.input_path.to_string_lossy(), r.error);
                    }
                    Err(e) => {
                        println!("  ✗ 错误: {} - {}", file.to_string_lossy(), e);
                    }
                }
            }

            result
        });

        tasks.push(task);
    }

    let mut results = Vec::new();
    for task in tasks {
        if let Ok(result) = task.await {
            results.push(result);
        }
    }

    let total = results.len();
    let completed = results.iter().filter(|r| r.as_ref().map(|x| x.success).unwrap_or(false)).count();
    let failed = total - completed;

    if cli.json {
        let mut json_results = Vec::new();
        for result in results {
            match result {
                Ok(r) => {
                    let compression_ratio = if r.original_size > 0 {
                        Some(1.0 - r.compressed_size as f64 / r.original_size as f64)
                    } else {
                        None
                    };
                    json_results.push(FileResult {
                        input: r.input_path.to_string_lossy().to_string(),
                        output: if r.success {
                            Some(r.output_path.to_string_lossy().to_string())
                        } else {
                            None
                        },
                        success: r.success,
                        error: r.error.clone(),
                        original_size: Some(r.original_size),
                        compressed_size: if r.success { Some(r.compressed_size) } else { None },
                        compression_ratio,
                    });
                }
                Err(e) => {
                    json_results.push(FileResult {
                        input: String::new(),
                        output: None,
                        success: false,
                        error: Some(e.to_string()),
                        original_size: None,
                        compressed_size: None,
                        compression_ratio: None,
                    });
                }
            }
        }

        let json_output = JsonOutput {
            success: failed == 0,
            total,
            completed,
            failed,
            results: json_results,
        };

        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else if !cli.quiet {
        println!();
        println!("======================");
        println!("总计: {} 个文件", total);
        println!("成功: {} 个", completed);
        println!("失败: {} 个", failed);
    }

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let units = ["B", "KB", "MB", "GB", "TB"];
    let i = (bytes as f64).log(1024.0).floor() as usize;
    format!(
        "{:.2} {}",
        bytes as f64 / 1024.0f64.powi(i as i32),
        units[i]
    )
}

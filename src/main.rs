#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context, Result};
use eframe::egui;
use eframe::egui::NumExt;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use walkdir::WalkDir;

// --- 数据模型 ---

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppConfig {
    output_dir: String,
    quality: u32,
    concurrency: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            output_dir: String::new(),
            quality: 85,
            concurrency: 2,
        }
    }
}

#[derive(PartialEq, Clone)]
enum JobStatus {
    Pending,
    Processing(f32),
    Finished,
    Failed(String),
}

struct VideoJob {
    path: PathBuf,
    status: JobStatus,
    original_size: u64,
    compressed_size: u64,
}

struct AppStats {
    total: usize,
    success: usize,
    failed: usize,
    original_total_size: u64,
    compressed_total_size: u64,
}

// --- 视频处理逻辑 (从命令行版移植并优化) ---

#[derive(Deserialize, Debug)]
struct FFProbeOutput {
    streams: Vec<Stream>,
}

#[derive(Deserialize, Debug)]
struct Stream {
    width: Option<i32>,
    height: Option<i32>,
    r_frame_rate: Option<String>,
    codec_type: String,
}

async fn get_video_metadata(path: &Path) -> Result<(i32, i32, f64)> {
    // 智能获取 ffprobe 路径：优先寻找当前目录，然后是 C:\ffmpeg-8.0\bin\，最后是系统路径
    let ffprobe_path = if let Ok(exe_path) = std::env::current_exe() {
        let local_path = exe_path.parent().unwrap().join("ffprobe.exe");
        if local_path.exists() {
            local_path.to_string_lossy().to_string()
        } else if Path::new(r"C:\ffmpeg-8.0\bin\ffprobe.exe").exists() {
            r"C:\ffmpeg-8.0\bin\ffprobe.exe".to_string()
        } else {
            "ffprobe".to_string()
        }
    } else {
        "ffprobe".to_string()
    };

    let mut cmd = Command::new(ffprobe_path);
    cmd.args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height,r_frame_rate,codec_type", "-of", "json"])
        .arg(path);

    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let output = cmd.output().await?;
    let probe: FFProbeOutput = serde_json::from_slice(&output.stdout)?;
    let s = probe.streams.iter().find(|s| s.codec_type == "video").context("No video")?;
    let fps = s.r_frame_rate.as_ref().map(|r| {
        let p: Vec<&str> = r.split('/').collect();
        if p.len() == 2 { p[0].parse::<f64>().unwrap_or(30.0) / p[1].parse::<f64>().unwrap_or(1.0) }
        else { r.parse().unwrap_or(30.0) }
    }).unwrap_or(30.0);
    Ok((s.width.unwrap_or(0), s.height.unwrap_or(0), fps))
}

// --- UI 应用程序 ---

struct VideoCompressApp {
    config: AppConfig,
    jobs: Arc<Mutex<Vec<VideoJob>>>,
    stats: Arc<Mutex<AppStats>>,
    is_running: bool,
    runtime: tokio::runtime::Runtime,
}

impl VideoCompressApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // --- 莫兰迪配色方案 ---
        let mut visuals = egui::Visuals::light();
        let morandi_blue = egui::Color32::from_rgb(180, 195, 205);      // 莫兰迪淡蓝
        let morandi_bg = egui::Color32::from_rgb(245, 247, 248);        // 极浅灰蓝背景
        
        visuals.panel_fill = morandi_bg;
        visuals.window_rounding = 16.0.into();
        visuals.widgets.noninteractive.bg_fill = egui::Color32::WHITE;
        visuals.widgets.noninteractive.rounding = 12.0.into();
        visuals.widgets.inactive.bg_fill = morandi_blue.linear_multiply(0.3);
        visuals.widgets.inactive.rounding = 12.0.into();
        visuals.widgets.hovered.bg_fill = morandi_blue.linear_multiply(0.5);
        visuals.widgets.active.bg_fill = morandi_blue;
        
        cc.egui_ctx.set_visuals(visuals);

        // 配置中文字体
        let mut fonts = egui::FontDefinitions::default();
        let font_paths = if cfg!(target_os = "windows") {
            vec![
                "C:\\Windows\\Fonts\\msyh.ttc", // 微软雅黑
                "C:\\Windows\\Fonts\\simhei.ttf", // 黑体
            ]
        } else {
            vec![
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/STHeiti Light.ttc",
                "/System/Library/Fonts/Supplemental/Songti.ttc",
            ]
        };

        for path in font_paths {
            if let Ok(font_data) = std::fs::read(path) {
                fonts.font_data.insert(
                    "chinese_font".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "chinese_font".to_owned());
                fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("chinese_font".to_owned());
                cc.egui_ctx.set_fonts(fonts);
                break;
            }
        }

        let config: AppConfig = confy::load("fast-video-compress-rs", None).unwrap_or_default();
        Self {
            config,
            jobs: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(AppStats {
                total: 0,
                success: 0,
                failed: 0,
                original_total_size: 0,
                compressed_total_size: 0,
            })),
            is_running: false,
            runtime: tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap(),
        }
    }

    fn add_paths(&mut self, paths: Vec<PathBuf>) {
        let mut jobs = self.jobs.lock().unwrap();
        for path in paths {
            if path.is_dir() {
                for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if let Some(ext) = p.extension() {
                        if ["mp4", "mkv", "mov", "avi", "wmv", "flv", "webm", "ts", "m2ts"].contains(&ext.to_string_lossy().to_lowercase().as_str()) {
                            let size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
                            jobs.push(VideoJob { path: p.to_path_buf(), status: JobStatus::Pending, original_size: size, compressed_size: 0 });
                        }
                    }
                }
            } else {
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                jobs.push(VideoJob { path, status: JobStatus::Pending, original_size: size, compressed_size: 0 });
            }
        }
    }

    fn run_compression(&mut self) {
        if self.is_running { return; }
        self.is_running = true;
        
        let jobs_arc = self.jobs.clone();
        let stats_arc = self.stats.clone();
        let config = self.config.clone();
        let _ = confy::store("fast-video-compress-rs", None, &config);

        // 重置统计
        {
            let mut stats = stats_arc.lock().unwrap();
            stats.total = 0;
            stats.success = 0;
            stats.failed = 0;
            stats.original_total_size = 0;
            stats.compressed_total_size = 0;
        }

        self.runtime.spawn(async move {
            let pending_indices: Vec<usize> = {
                let jobs = jobs_arc.lock().unwrap();
                jobs.iter().enumerate()
                    .filter(|(_, j)| j.status == JobStatus::Pending)
                    .map(|(i, _)| i).collect()
            };

            {
                let mut stats = stats_arc.lock().unwrap();
                stats.total = pending_indices.len();
            }

            let semaphore = Arc::new(tokio::sync::Semaphore::new(config.concurrency));
            let mut stream = futures::stream::iter(pending_indices).map(|idx| {
                let jobs = jobs_arc.clone();
                let stats = stats_arc.clone();
                let sem = semaphore.clone();
                let conf = config.clone();
                async move {
                    let _permit = sem.acquire().await.unwrap();
                    let (path, out_path, orig_size) = {
                        let mut jobs_lock = jobs.lock().unwrap();
                        let job = &mut jobs_lock[idx];
                        job.status = JobStatus::Processing(0.0);
                        (job.path.clone(), Path::new(&conf.output_dir).join(job.path.file_name().unwrap()), job.original_size)
                    };

                    let res = match get_video_metadata(&path).await {
                        Ok((_w, h, _fps)) => {
                            let vf = if h >= 1440 { "scale=-2:1440" } else if h >= 1080 { "scale=-2:1080" } else { "scale=-2:trunc(ih/2)*2" };
                            
                            // 智能获取 ffmpeg 路径：优先寻找当前目录，然后是 C:\ffmpeg-8.0\bin\，最后是系统路径
                            let ffmpeg_path = if let Ok(exe_path) = std::env::current_exe() {
                                let local_path = exe_path.parent().unwrap().join("ffmpeg.exe");
                                if local_path.exists() {
                                    local_path.to_string_lossy().to_string()
                                } else if Path::new(r"C:\ffmpeg-8.0\bin\ffmpeg.exe").exists() {
                                    r"C:\ffmpeg-8.0\bin\ffmpeg.exe".to_string()
                                } else {
                                    "ffmpeg".to_string()
                                }
                            } else {
                                "ffmpeg".to_string()
                            };

                            // 针对不同平台选择编码参数
                            let mut args = vec![
                                "-hide_banner".to_string(),
                                "-loglevel".to_string(),
                                "error".to_string(),
                                "-y".to_string(),
                                "-hwaccel".to_string(),
                                "auto".to_string(), // 使用 auto 自动选择最佳解码器
                                "-i".to_string(),
                                path.to_string_lossy().to_string(),
                                "-vf".to_string(),
                                format!("{},format=yuv420p", vf), // 增加显式像素格式转换，确保显卡编码器兼容
                            ];

                            // 平台特定参数
                            if cfg!(target_os = "macos") {
                                args.extend([
                                    "-c:v".to_string(), "hevc_videotoolbox".to_string(),
                                    "-q:v".to_string(), conf.quality.to_string(),
                                    "-preset".to_string(), "hq".to_string(),
                                    "-tag:v".to_string(), "hvc1".to_string(),
                                ]);
                            } else if cfg!(target_os = "windows") {
                                let cq = (51.0 * (1.0 - (conf.quality as f32 / 100.0))) as u32;
                                args.extend([
                                    "-c:v".to_string(), "hevc_nvenc".to_string(),
                                    "-rc".to_string(), "vbr".to_string(),
                                    "-cq".to_string(), cq.to_string(),
                                    "-preset".to_string(), "p6".to_string(),
                                    "-spatial-aq".to_string(), "1".to_string(),
                                ]);
                            } else {
                                args.extend([
                                    "-c:v".to_string(), "libx265".to_string(),
                                    "-crf".to_string(), "23".to_string(),
                                    "-preset".to_string(), "medium".to_string(),
                                ]);
                            }

                            args.extend([
                                "-c:a".to_string(), "aac".to_string(),
                                "-b:a".to_string(), "128k".to_string(),
                                "-map_metadata".to_string(), "0".to_string(),
                                "-movflags".to_string(), "+faststart+use_metadata_tags".to_string(),
                                out_path.to_string_lossy().to_string(),
                            ]);

                            let mut cmd = Command::new(ffmpeg_path);
                            cmd.args(&args);

                            #[cfg(windows)]
                            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

                            let status = cmd.status().await;
                            match status {
                                Ok(s) if s.success() => {
                                    let c_size = std::fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
                                    Ok(c_size)
                                },
                                _ => Err("FFmpeg error".to_string()),
                            }
                        }
                        Err(e) => Err(e.to_string()),
                    };

                    let mut jobs_lock = jobs.lock().unwrap();
                    let mut stats_lock = stats.lock().unwrap();
                    match res {
                        Ok(c_size) => {
                            jobs_lock[idx].status = JobStatus::Finished;
                            jobs_lock[idx].compressed_size = c_size;
                            stats_lock.success += 1;
                            stats_lock.original_total_size += orig_size;
                            stats_lock.compressed_total_size += c_size;
                        },
                        Err(e) => {
                            jobs_lock[idx].status = JobStatus::Failed(e);
                            stats_lock.failed += 1;
                        }
                    };
                }
            }).buffer_unordered(config.concurrency);

            while (stream.next().await).is_some() {}
            
            // 处理完成后自动打开文件夹
            if cfg!(target_os = "windows") {
                let mut cmd = Command::new("explorer");
                cmd.arg(&config.output_dir);
                #[cfg(windows)]
                cmd.creation_flags(0x08000000);
                let _ = cmd.status().await;
            } else if cfg!(target_os = "macos") {
                let _ = Command::new("open").arg(&config.output_dir).status().await;
            }
        });
    }
}

fn format_size(bytes: u64) -> String {
    if bytes == 0 { return "0 B".to_string(); }
    let units = ["B", "KB", "MB", "GB", "TB"];
    let i = (bytes as f64).log(1024.0).floor() as usize;
    format!("{:.2} {}", bytes as f64 / 1024.0f64.powi(i as i32), units[i])
}

impl eframe::App for VideoCompressApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(ctx.style().visuals.panel_fill).inner_margin(egui::Margin {
                left: 30.0,
                right: 30.0,
                top: 25.0,
                bottom: 25.0,
            }))
            .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("🚀 星TAP 极简视频压缩").size(24.0).strong().color(egui::Color32::from_rgb(70, 85, 95)));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("极简 · 高效 · 智能").weak().size(13.0));
            });
            
            ui.add_space(20.0);

            // 统一内容宽度限制，防止拉伸过宽
            let max_content_width = 600.0;
            let available_width = ui.available_width(); // 获取当前 UI 的真实可用宽度
            
            ui.vertical_centered(|ui| {
                // 内容区域的宽度限制：不能超过 max_content_width，也不能超过当前可用宽度
                let content_width = available_width.at_most(max_content_width);
                ui.set_max_width(content_width);

                // 设置区域
                egui::Frame::group(ui.style())
                    .fill(egui::Color32::WHITE)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 225, 230)))
                    .rounding(16.0)
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width()); 
                        egui::Grid::new("config_grid")
                            .num_columns(2)
                            .spacing([15.0, 18.0])
                            .show(ui, |ui| {
                                ui.add_sized([80.0, 20.0], egui::Label::new(egui::RichText::new("导出目录:").strong()));
                                ui.horizontal(|ui| {
                                    // 动态计算输入框宽度，确保不溢出
                                    let btn_width = 85.0;
                                    let edit_width = (ui.available_width() - btn_width - 10.0).at_least(100.0);
                                    ui.add(egui::TextEdit::singleline(&mut self.config.output_dir).desired_width(edit_width).margin(egui::vec2(8.0, 4.0)));
                                    if ui.button(" 选择... ").clicked() {
                                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                            self.config.output_dir = path.display().to_string();
                                        }
                                    }
                                });
                                ui.end_row();

                                ui.add_sized([80.0, 20.0], egui::Label::new(egui::RichText::new("视频画质:").strong()));
                                ui.horizontal(|ui| {
                                    let quality_slider = ui.add(egui::Slider::new(&mut self.config.quality, 1..=100).show_value(true));
                                    
                                    let quality_text = if self.config.quality >= 90 {
                                        "🌟 极清 (文件大)"
                                    } else if self.config.quality >= 75 {
                                        "✨ 高清 (推荐)"
                                    } else if self.config.quality >= 50 {
                                        "📱 标清 (平衡)"
                                    } else {
                                        "📉 低清 (极小)"
                                    };
                                    
                                    ui.label(egui::RichText::new(quality_text).size(12.0).weak());
                                    
                                    quality_slider.on_hover_text("数值越高，画质越好，但压缩后的文件也越大。\n推荐选择 75-85 之间。");

                                    ui.add_space(20.0);
                                    ui.label(egui::RichText::new("并发任务:").strong());
                                    ui.add(egui::Slider::new(&mut self.config.concurrency, 1..=8))
                                        .on_hover_text("同时处理的视频数量。建议根据显卡性能设置，通常 2-4 即可。");
                                });
                                ui.end_row();
                            });
                    });

                ui.add_space(15.0);

                // 拖拽与选择区域
                let is_hovering = ctx.input(|i| !i.raw.dropped_files.is_empty());
                let border_color = if is_hovering { egui::Color32::from_rgb(150, 180, 200) } else { egui::Color32::from_rgb(210, 215, 220) };
                
                egui::Frame::canvas(ui.style())
                    .fill(if is_hovering { egui::Color32::from_rgb(235, 242, 248) } else { egui::Color32::WHITE })
                    .stroke(egui::Stroke::new(2.0, border_color))
                    .rounding(16.0)
                    .inner_margin(30.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("📥").size(40.0));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("将视频或文件夹拖到此处").size(14.0).strong());
                            ui.add_space(15.0);
                            
                            ui.horizontal(|ui| {
                                // 使用动态比例居中按钮
                                let total_btn_width = 200.0; // 估算两个按钮总宽
                                let space = (ui.available_width() - total_btn_width) / 2.0;
                                ui.add_space(space.at_least(0.0)); 
                                if ui.button("➕ 添加文件").clicked() {
                                    if let Some(files) = rfd::FileDialog::new()
                                        .add_filter("视频", &["mp4", "mkv", "mov", "avi", "wmv", "flv", "webm", "ts", "m2ts"])
                                        .pick_files() {
                                        self.add_paths(files);
                                    }
                                }
                                ui.add_space(10.0);
                                if ui.button("📂 添加文件夹").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        self.add_paths(vec![path]);
                                    }
                                }
                            });
                        });
                    });
            });

            if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                let paths = ctx.input(|i| {
                    i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect()
                });
                self.add_paths(paths);
            }

            ui.add_space(20.0);

            // 任务列表和统计
            ui.vertical_centered(|ui| {
                let content_width = ui.available_width().at_most(max_content_width);
                ui.set_max_width(content_width);
                
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("待处理任务 ({})", self.jobs.lock().unwrap().len())).strong().size(15.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑 清空列表").clicked() {
                            self.jobs.lock().unwrap().clear();
                            self.is_running = false;
                        }
                    });
                });
                ui.add_space(8.0);

                let stats = self.stats.lock().unwrap();
                if stats.total > 0 {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("✅ 成功: {}", stats.success)).size(12.0));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(format!("❌ 失败: {}", stats.failed)).size(12.0));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(format!("📊 压缩率: {:.1}%", 
                            if stats.original_total_size > 0 {
                                (1.0 - (stats.compressed_total_size as f64 / stats.original_total_size as f64)) * 100.0
                            } else { 0.0 }
                        )).size(12.0));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(format!("💾 节省空间: {}", format_size(stats.original_total_size.saturating_sub(stats.compressed_total_size)))).size(12.0).weak());
                    });
                    ui.add_space(8.0);
                }
                drop(stats);
                
                let scroll_height = ui.available_height() - 80.0; 
                egui::ScrollArea::vertical()
                    .max_height(scroll_height.at_least(100.0))
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                    let jobs = self.jobs.lock().unwrap();
                    for job in jobs.iter() {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(250, 251, 252))
                            .rounding(8.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    let name = job.path.file_name().unwrap().to_string_lossy();
                                    ui.label(egui::RichText::new(name).size(13.0));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        match &job.status {
                                            JobStatus::Pending => { ui.label(egui::RichText::new("⏳ 等待").color(egui::Color32::GRAY)); }
                                            JobStatus::Processing(_) => { ui.spinner(); ui.label(egui::RichText::new("⚙️ 压缩中").color(egui::Color32::from_rgb(100, 150, 200))); }
                                            JobStatus::Finished => { 
                                                ui.label(egui::RichText::new("✅ 完成").color(egui::Color32::from_rgb(80, 180, 100))); 
                                                ui.label(egui::RichText::new(format!("{} → {}", format_size(job.original_size), format_size(job.compressed_size))).size(11.0).weak());
                                            }
                                            JobStatus::Failed(e) => { ui.label(egui::RichText::new(format!("❌ 失败: {}", e)).color(egui::Color32::from_rgb(220, 100, 100))); }
                                        }
                                    });
                                });
                            });
                        ui.add_space(4.0);
                    }
                });

                ui.add_space(ui.available_height() - 60.0); 
                let btn_text = if self.is_running {
                    let stats = self.stats.lock().unwrap();
                    if stats.success + stats.failed == stats.total && stats.total > 0 {
                        "🎉 处理完成"
                    } else {
                        "🚀 正在处理..."
                    }
                } else {
                    "开始批量压缩"
                };
                
                let btn = egui::Button::new(egui::RichText::new(btn_text).size(18.0).strong().color(egui::Color32::WHITE))
                    .min_size(egui::vec2(280.0, 50.0))
                    .fill(egui::Color32::from_rgb(130, 155, 175))
                    .rounding(25.0);
                
                if ui.add_enabled(!self.is_running && !self.jobs.lock().unwrap().is_empty(), btn).clicked() {
                    self.run_compression();
                }
            });
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

fn main() -> Result<(), eframe::Error> {
    // 智能加载图标：优先使用嵌入到二进制的 ico 数据，确保在任何位置都能显示图标
    let icon_bytes = include_bytes!("../视频压缩图标.ico");
    
    let icon_data = image::load_from_memory(icon_bytes).ok().map(|img| {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([680.0, 780.0])
        .with_min_inner_size([500.0, 650.0]);
    
    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "星TAP 极简视频压缩",
        options,
        Box::new(|cc| Ok(Box::new(VideoCompressApp::new(cc)))),
    )
}

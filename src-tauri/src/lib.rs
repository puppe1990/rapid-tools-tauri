mod domain;

use domain::zip_archive::ZipEntry;
use domain::{ConversionResult, ZipResult};
use std::path::PathBuf;

fn default_output_dir(tool: &str) -> String {
    let base = std::env::temp_dir().join("rapid_tools_tauri").join(tool);
    let _ = std::fs::create_dir_all(&base);
    base.to_string_lossy().into_owned()
}

/// ffmpeg/magick must not run on the UI thread — a wrong file (e.g. MKV in Image
/// Converter) previously froze the app long enough for macOS to report a hang.
async fn run_blocking<T, F>(f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| format!("worker_join_failed: {e}"))?
}

#[tauri::command]
fn list_tools() -> Vec<serde_json::Value> {
    vec![
        json_tool("image", "Image converter", "png,jpg,webp,heic,avif,enc"),
        json_tool("image_resizer", "Image resizer", "jpg,png,webp"),
        json_tool("video", "Video converter", "mp4,mov,webm,mkv,avi,3gp"),
        json_tool("video_compressor", "Video compressor", "mp4"),
        json_tool("extract_audio", "Extract audio", "mp3,wav,ogg,aac,flac"),
        json_tool("audio", "Audio converter", "mp3,wav,ogg,aac,flac"),
        json_tool("photos_to_pdf", "Photos to PDF", "pdf"),
        json_tool("pdf_to_images", "PDF to images", "png,jpg"),
        json_tool("together_audios", "Together audios", "mp3,wav,ogg,aac,flac"),
        json_tool(
            "together_videos",
            "Together videos",
            "mp4,mov,webm,mkv,avi,3gp",
        ),
        json_tool("images_to_video", "Images to video", "mp4,gif"),
        json_tool("qr_reader", "QR reader", "text"),
    ]
}

fn json_tool(id: &str, name: &str, formats: &str) -> serde_json::Value {
    serde_json::json!({ "id": id, "name": name, "formats": formats })
}

#[tauri::command]
async fn convert_image(
    source_path: String,
    target_format: String,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("image"));
    run_blocking(move || domain::image_converter::convert(&source_path, &target_format, &out)).await
}

#[tauri::command]
async fn convert_video(
    source_path: String,
    target_format: String,
    output_dir: Option<String>,
    orientation: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("video"));
    run_blocking(move || {
        domain::video_converter::convert(&source_path, &target_format, &out, orientation.as_deref())
    })
    .await
}

#[tauri::command]
async fn convert_audio(
    source_path: String,
    target_format: String,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("audio"));
    run_blocking(move || domain::audio_converter::convert(&source_path, &target_format, &out)).await
}

#[tauri::command]
async fn resize_image(
    source_path: String,
    width: u32,
    height: u32,
    output_dir: Option<String>,
    target_format: Option<String>,
    fit: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("image_resizer"));
    run_blocking(move || {
        domain::image_resizer::resize(
            &source_path,
            width,
            height,
            &out,
            target_format.as_deref(),
            fit.as_deref(),
        )
    })
    .await
}

#[tauri::command]
async fn extract_audio(
    source_path: String,
    target_format: String,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("extract_audio"));
    run_blocking(move || domain::audio_extractor::extract(&source_path, &target_format, &out)).await
}

#[tauri::command]
async fn compress_video(
    source_path: String,
    output_dir: Option<String>,
    preset: Option<String>,
    max_resolution: Option<String>,
    mute: Option<bool>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("video_compressor"));
    run_blocking(move || {
        domain::video_compressor::compress(
            &source_path,
            &out,
            preset.as_deref(),
            max_resolution.as_deref(),
            mute.unwrap_or(false),
        )
    })
    .await
}

#[tauri::command]
async fn photos_to_pdf(
    source_paths: Vec<String>,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("photos_to_pdf"));
    run_blocking(move || domain::pdf_converter::images_to_pdf(&source_paths, &out)).await
}

#[tauri::command]
async fn pdf_to_images(
    source_path: String,
    target_format: String,
    output_dir: Option<String>,
) -> Result<Vec<ConversionResult>, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("pdf_to_images"));
    run_blocking(move || domain::pdf_converter::pdf_to_images(&source_path, &target_format, &out))
        .await
}

#[tauri::command]
async fn join_audios(
    source_paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("together_audios"));
    run_blocking(move || domain::audio_joiner::join(&source_paths, &target_format, &out)).await
}

#[tauri::command]
async fn join_videos(
    source_paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("together_videos"));
    run_blocking(move || domain::video_joiner::join(&source_paths, &target_format, &out)).await
}

#[tauri::command]
async fn images_to_video(
    source_paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
    interval_secs: Option<f64>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("images_to_video"));
    run_blocking(move || {
        domain::images_to_video::convert(
            &source_paths,
            &target_format,
            &out,
            interval_secs.unwrap_or(2.0),
        )
    })
    .await
}

#[tauri::command]
async fn read_qr(source_path: String) -> Result<String, String> {
    run_blocking(move || domain::qr_reader::read(&source_path)).await
}

#[tauri::command]
async fn build_zip(
    id: String,
    entries: Vec<ZipEntryDto>,
    output_dir: Option<String>,
) -> Result<ZipResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("zips"));
    run_blocking(move || {
        let mapped: Vec<ZipEntry> = entries
            .into_iter()
            .map(|e| ZipEntry {
                path: e.path,
                filename: e.filename,
            })
            .collect();
        domain::zip_archive::build(&id, &mapped, &out)
    })
    .await
}

#[derive(serde::Deserialize)]
struct ZipEntryDto {
    path: String,
    filename: String,
}

#[tauri::command]
fn reveal_path(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err("path_not_found".into());
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .status()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = p.parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .status()
                .map_err(|e| e.to_string())?;
        }
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .status()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Open maximized by default. Config `maximized: true` is unreliable on some platforms
            // (notably macOS), so we also force it here after the window exists.
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                // Prefer native maximize; if it no-ops, fall back to filling the monitor work area.
                if window.maximize().is_err() || !window.is_maximized().unwrap_or(false) {
                    if let Ok(Some(monitor)) = window.current_monitor() {
                        let area = monitor.work_area();
                        let _ = window.set_position(tauri::PhysicalPosition::new(
                            area.position.x,
                            area.position.y,
                        ));
                        let _ = window
                            .set_size(tauri::PhysicalSize::new(area.size.width, area.size.height));
                    }
                    let _ = window.maximize();
                }
                let _ = window.set_focus();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_tools,
            convert_image,
            convert_video,
            convert_audio,
            resize_image,
            extract_audio,
            compress_video,
            photos_to_pdf,
            pdf_to_images,
            join_audios,
            join_videos,
            images_to_video,
            read_qr,
            build_zip,
            reveal_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

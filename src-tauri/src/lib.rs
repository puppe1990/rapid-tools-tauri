mod domain;

use domain::zip_archive::ZipEntry;
use domain::{ConversionResult, ZipResult};
use std::path::PathBuf;

fn default_output_dir(tool: &str) -> String {
    let base = std::env::temp_dir().join("rapid_tools_tauri").join(tool);
    let _ = std::fs::create_dir_all(&base);
    base.to_string_lossy().into_owned()
}

#[tauri::command]
fn list_tools() -> Vec<serde_json::Value> {
    vec![
        json_tool("image", "Image converter", "png,jpg,webp,heic,avif,enc"),
        json_tool("image_resizer", "Image resizer", "jpg,png,webp"),
        json_tool("video", "Video converter", "mp4,mov,webm,mkv,avi"),
        json_tool("video_compressor", "Video compressor", "mp4"),
        json_tool("extract_audio", "Extract audio", "mp3,wav,ogg,aac,flac"),
        json_tool("audio", "Audio converter", "mp3,wav,ogg,aac,flac"),
        json_tool("photos_to_pdf", "Photos to PDF", "pdf"),
        json_tool("pdf_to_images", "PDF to images", "png,jpg"),
        json_tool("together_audios", "Together audios", "mp3,wav,ogg,aac,flac"),
        json_tool("together_videos", "Together videos", "mp4,mov,webm,mkv,avi"),
        json_tool("images_to_video", "Images to video", "mp4,gif"),
        json_tool("qr_reader", "QR reader", "text"),
    ]
}

fn json_tool(id: &str, name: &str, formats: &str) -> serde_json::Value {
    serde_json::json!({ "id": id, "name": name, "formats": formats })
}

#[tauri::command]
fn convert_image(
    source_path: String,
    target_format: String,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("image"));
    domain::image_converter::convert(&source_path, &target_format, &out)
}

#[tauri::command]
fn convert_video(
    source_path: String,
    target_format: String,
    output_dir: Option<String>,
    orientation: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("video"));
    domain::video_converter::convert(
        &source_path,
        &target_format,
        &out,
        orientation.as_deref(),
    )
}

#[tauri::command]
fn convert_audio(
    source_path: String,
    target_format: String,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("audio"));
    domain::audio_converter::convert(&source_path, &target_format, &out)
}

#[tauri::command]
fn resize_image(
    source_path: String,
    width: u32,
    height: u32,
    output_dir: Option<String>,
    target_format: Option<String>,
    fit: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("image_resizer"));
    domain::image_resizer::resize(
        &source_path,
        width,
        height,
        &out,
        target_format.as_deref(),
        fit.as_deref(),
    )
}

#[tauri::command]
fn extract_audio(
    source_path: String,
    target_format: String,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("extract_audio"));
    domain::audio_extractor::extract(&source_path, &target_format, &out)
}

#[tauri::command]
fn compress_video(
    source_path: String,
    output_dir: Option<String>,
    preset: Option<String>,
    max_resolution: Option<String>,
    mute: Option<bool>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("video_compressor"));
    domain::video_compressor::compress(
        &source_path,
        &out,
        preset.as_deref(),
        max_resolution.as_deref(),
        mute.unwrap_or(false),
    )
}

#[tauri::command]
fn photos_to_pdf(
    source_paths: Vec<String>,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("photos_to_pdf"));
    domain::pdf_converter::images_to_pdf(&source_paths, &out)
}

#[tauri::command]
fn pdf_to_images(
    source_path: String,
    target_format: String,
    output_dir: Option<String>,
) -> Result<Vec<ConversionResult>, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("pdf_to_images"));
    domain::pdf_converter::pdf_to_images(&source_path, &target_format, &out)
}

#[tauri::command]
fn join_audios(
    source_paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("together_audios"));
    domain::audio_joiner::join(&source_paths, &target_format, &out)
}

#[tauri::command]
fn join_videos(
    source_paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("together_videos"));
    domain::video_joiner::join(&source_paths, &target_format, &out)
}

#[tauri::command]
fn images_to_video(
    source_paths: Vec<String>,
    target_format: String,
    output_dir: Option<String>,
    interval_secs: Option<f64>,
) -> Result<ConversionResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("images_to_video"));
    domain::images_to_video::convert(
        &source_paths,
        &target_format,
        &out,
        interval_secs.unwrap_or(2.0),
    )
}

#[tauri::command]
fn read_qr(source_path: String) -> Result<String, String> {
    domain::qr_reader::read(&source_path)
}

#[tauri::command]
fn build_zip(
    id: String,
    entries: Vec<ZipEntryDto>,
    output_dir: Option<String>,
) -> Result<ZipResult, String> {
    let out = output_dir.unwrap_or_else(|| default_output_dir("zips"));
    let mapped: Vec<ZipEntry> = entries
        .into_iter()
        .map(|e| ZipEntry {
            path: e.path,
            filename: e.filename,
        })
        .collect();
    domain::zip_archive::build(&id, &mapped, &out)
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

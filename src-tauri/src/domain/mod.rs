//! Pure conversion/orchestration logic. Tests call these units directly.
//! Tauri commands are thin wrappers around this module.

pub mod audio_converter;
pub mod audio_extractor;
pub mod audio_joiner;
pub mod image_converter;
pub mod image_resizer;
pub mod images_to_video;
pub mod pdf_converter;
pub mod qr_reader;
pub mod util;
pub mod video_compressor;
pub mod video_converter;
pub mod video_joiner;
pub mod zip_archive;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversionResult {
    pub output_path: String,
    pub filename: String,
    pub media_type: String,
    pub target_format: String,
}

impl ConversionResult {
    pub fn new(
        output_path: impl Into<String>,
        media_type: impl Into<String>,
        target_format: impl Into<String>,
    ) -> Self {
        let output_path = output_path.into();
        let filename = std::path::Path::new(&output_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string();
        Self {
            output_path,
            filename,
            media_type: media_type.into(),
            target_format: target_format.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZipResult {
    pub path: String,
    pub filename: String,
    pub media_type: String,
}

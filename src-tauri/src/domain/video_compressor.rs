use super::util::{
    ensure_output_dir, ensure_source_exists, ensure_source_extension, find_executable,
    normalize_format, run_command,
};
use super::ConversionResult;
use std::path::PathBuf;

pub const SUPPORTED_PRESETS: &[&str] = &["small", "balanced", "high"];
pub const SUPPORTED_RESOLUTIONS: &[&str] = &["original", "1080", "720", "480"];
pub const SUPPORTED_INPUTS: &[&str] = &["mp4", "mov", "webm", "mkv", "avi", "3gp"];

pub fn compress(
    source_path: &str,
    output_dir: &str,
    preset: Option<&str>,
    max_resolution: Option<&str>,
    mute: bool,
) -> Result<ConversionResult, String> {
    let preset = normalize_format(preset.unwrap_or("balanced"));
    if !SUPPORTED_PRESETS.contains(&preset.as_str()) {
        return Err(format!("unsupported_preset: {preset}"));
    }
    let max_resolution = normalize_format(max_resolution.unwrap_or("original"));
    if !SUPPORTED_RESOLUTIONS.contains(&max_resolution.as_str()) {
        return Err(format!("unsupported_resolution: {max_resolution}"));
    }
    ensure_source_exists(source_path)?;
    ensure_source_extension(source_path, SUPPORTED_INPUTS)?;
    let out_dir = ensure_output_dir(output_dir)?;
    let stem = std::path::Path::new(source_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");
    let output_path: PathBuf = out_dir.join(format!("{stem}-compressed.mp4"));
    let ffmpeg = find_executable(&["ffmpeg"]).map_err(|_| "ffmpeg_not_found".to_string())?;

    let mut args = vec![
        "-y".into(),
        "-i".into(),
        source_path.into(),
        "-c:v".into(),
        "libx264".into(),
    ];
    args.extend(resolution_args(&max_resolution));
    args.extend(preset_args(&preset));
    if mute {
        args.push("-an".into());
    } else {
        args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), "128k".into()]);
    }
    args.extend([
        "-movflags".into(),
        "+faststart".into(),
        output_path.to_string_lossy().into_owned(),
    ]);

    run_command(&ffmpeg, &args)?;

    if !output_path.is_file()
        || std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0)
            == 0
    {
        return Err("compression_failed: empty or missing output".into());
    }

    Ok(ConversionResult::new(
        output_path.to_string_lossy(),
        "video/mp4",
        "mp4",
    ))
}

fn preset_args(preset: &str) -> Vec<String> {
    match preset {
        "small" => vec![
            "-preset".into(),
            "veryfast".into(),
            "-crf".into(),
            "34".into(),
        ],
        "balanced" => vec![
            "-preset".into(),
            "medium".into(),
            "-crf".into(),
            "29".into(),
        ],
        "high" => vec!["-preset".into(), "slow".into(), "-crf".into(), "24".into()],
        _ => vec![
            "-preset".into(),
            "medium".into(),
            "-crf".into(),
            "29".into(),
        ],
    }
}

fn resolution_args(max_resolution: &str) -> Vec<String> {
    match max_resolution {
        "original" => vec![],
        "1080" => vec![
            "-vf".into(),
            "scale=w='min(1920,iw)':h='min(1080,ih)':force_original_aspect_ratio=decrease".into(),
        ],
        "720" => vec![
            "-vf".into(),
            "scale=w='min(1280,iw)':h='min(720,ih)':force_original_aspect_ratio=decrease".into(),
        ],
        "480" => vec![
            "-vf".into(),
            "scale=w='min(854,iw)':h='min(480,ih)':force_original_aspect_ratio=decrease".into(),
        ],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::util::test_fixtures::make_mp4;
    use tempfile::TempDir;

    #[test]
    fn compresses_mp4_with_small_preset() {
        let dir = TempDir::new().unwrap();
        let source = make_mp4(&dir, "clip.mp4");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let result = compress(
            source.to_str().unwrap(),
            out.to_str().unwrap(),
            Some("small"),
            Some("480"),
            false,
        )
        .expect("compress");

        assert!(std::path::Path::new(&result.output_path).is_file());
        assert!(std::fs::metadata(&result.output_path).unwrap().len() > 0);
        assert!(result.output_path.ends_with("-compressed.mp4"));
        assert_eq!(result.target_format, "mp4");
    }
}

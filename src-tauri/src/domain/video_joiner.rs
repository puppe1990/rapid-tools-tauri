use super::util::{
    ensure_output_dir, ensure_sources_exist, find_executable, media_type_for_video, normalize_format,
    run_command,
};
use super::ConversionResult;
use std::path::PathBuf;

pub const SUPPORTED_FORMATS: &[&str] = &["mp4", "mov", "webm", "mkv", "avi"];

pub fn join(
    source_paths: &[String],
    target_format: &str,
    output_dir: &str,
) -> Result<ConversionResult, String> {
    if source_paths.len() < 2 {
        return Err("not_enough_source_files".into());
    }
    let target_format = normalize_format(target_format);
    if !SUPPORTED_FORMATS.contains(&target_format.as_str()) {
        return Err(format!("unsupported_target_format: {target_format}"));
    }
    ensure_sources_exist(source_paths)?;
    let out_dir = ensure_output_dir(output_dir)?;
    let ffmpeg = find_executable(&["ffmpeg"]).map_err(|_| "ffmpeg_not_found".to_string())?;

    // Normalize each input to intermediate mp4 (robust across codecs)
    let mut segment_paths = Vec::new();
    for (i, path) in source_paths.iter().enumerate() {
        let segment = out_dir.join(format!("segment_{i}.mp4"));
        run_command(
            &ffmpeg,
            &[
                "-y".into(),
                "-i".into(),
                path.clone(),
                "-c:v".into(),
                "libx264".into(),
                "-pix_fmt".into(),
                "yuv420p".into(),
                "-c:a".into(),
                "aac".into(),
                "-movflags".into(),
                "+faststart".into(),
                segment.to_string_lossy().into_owned(),
            ],
        )?;
        segment_paths.push(segment);
    }

    let concat_path = out_dir.join("concat_list.txt");
    let mut lines = String::new();
    for seg in &segment_paths {
        lines.push_str(&format!("file '{}'\n", seg.display()));
    }
    std::fs::write(&concat_path, lines).map_err(|e| format!("concat_file_error: {e}"))?;

    // Always concat to a temp .mp4 first — never write H.264+AAC with -c copy
    // into container formats that reject those codecs (e.g. WebM).
    let temp_joined: PathBuf = out_dir.join("together-videos-temp.mp4");
    run_command(
        &ffmpeg,
        &[
            "-y".into(),
            "-f".into(),
            "concat".into(),
            "-safe".into(),
            "0".into(),
            "-i".into(),
            concat_path.to_string_lossy().into_owned(),
            "-c".into(),
            "copy".into(),
            temp_joined.to_string_lossy().into_owned(),
        ],
    )?;

    if !temp_joined.is_file()
        || std::fs::metadata(&temp_joined).map(|m| m.len()).unwrap_or(0) == 0
    {
        return Err("join_failed: empty or missing intermediate mp4".into());
    }

    let final_path: PathBuf = out_dir.join(format!("together-videos.{target_format}"));

    if target_format == "mp4" {
        std::fs::rename(&temp_joined, &final_path)
            .or_else(|_| {
                std::fs::copy(&temp_joined, &final_path).map(|_| {
                    let _ = std::fs::remove_file(&temp_joined);
                })
            })
            .map_err(|e| format!("join_failed: {e}"))?;
    } else {
        let mut args = vec![
            "-y".into(),
            "-i".into(),
            temp_joined.to_string_lossy().into_owned(),
        ];
        args.extend(transcode_args_for_target(&target_format));
        args.push(final_path.to_string_lossy().into_owned());
        run_command(&ffmpeg, &args)?;
        let _ = std::fs::remove_file(&temp_joined);
    }

    if !final_path.is_file() || std::fs::metadata(&final_path).map(|m| m.len()).unwrap_or(0) == 0 {
        return Err("join_failed: empty or missing output".into());
    }

    Ok(ConversionResult::new(
        final_path.to_string_lossy(),
        media_type_for_video(&target_format),
        target_format,
    ))
}

/// Codec args for remux/transcode from intermediate H.264+AAC mp4 into target container.
fn transcode_args_for_target(target_format: &str) -> Vec<String> {
    match target_format {
        // WebM only accepts VP8/VP9/AV1 + Vorbis/Opus — not H.264/AAC stream copy.
        "webm" => vec![
            "-c:v".into(),
            "libvpx-vp9".into(),
            "-b:v".into(),
            "0".into(),
            "-crf".into(),
            "35".into(),
            "-c:a".into(),
            "libopus".into(),
            "-b:a".into(),
            "96k".into(),
        ],
        "mov" => vec![
            "-c:v".into(),
            "libx264".into(),
            "-pix_fmt".into(),
            "yuv420p".into(),
            "-c:a".into(),
            "aac".into(),
            "-movflags".into(),
            "+faststart".into(),
        ],
        "mkv" => vec![
            "-c:v".into(),
            "libx264".into(),
            "-pix_fmt".into(),
            "yuv420p".into(),
            "-c:a".into(),
            "aac".into(),
        ],
        "avi" => vec![
            "-c:v".into(),
            "mpeg4".into(),
            "-q:v".into(),
            "5".into(),
            "-c:a".into(),
            "libmp3lame".into(),
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
    fn joins_two_mp4_files() {
        let dir = TempDir::new().unwrap();
        let a = make_mp4(&dir, "a.mp4");
        let b = make_mp4(&dir, "b.mp4");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let result = join(
            &[
                a.to_string_lossy().into_owned(),
                b.to_string_lossy().into_owned(),
            ],
            "mp4",
            out.to_str().unwrap(),
        )
        .expect("join video");

        assert!(std::path::Path::new(&result.output_path).is_file());
        assert!(std::fs::metadata(&result.output_path).unwrap().len() > 0);
        assert!(result.output_path.ends_with("together-videos.mp4"));
        assert_eq!(result.target_format, "mp4");
    }

    #[test]
    fn joins_two_mp4_files_to_webm() {
        let dir = TempDir::new().unwrap();
        let a = make_mp4(&dir, "a.mp4");
        let b = make_mp4(&dir, "b.mp4");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let result = join(
            &[
                a.to_string_lossy().into_owned(),
                b.to_string_lossy().into_owned(),
            ],
            "webm",
            out.to_str().unwrap(),
        )
        .expect("join video to webm via shipped path");

        assert!(
            std::path::Path::new(&result.output_path).is_file(),
            "webm output must exist: {}",
            result.output_path
        );
        assert!(
            std::fs::metadata(&result.output_path).unwrap().len() > 0,
            "webm output must be non-empty"
        );
        assert!(
            result.output_path.ends_with(".webm"),
            "expected .webm extension, got {}",
            result.output_path
        );
        assert_eq!(result.target_format, "webm");
        assert_eq!(result.media_type, "video/webm");
        assert_eq!(result.filename, "together-videos.webm");
    }
}

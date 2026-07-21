use super::util::{
    ensure_output_dir, ensure_sources_exist, find_executable, normalize_format, run_command,
};
use super::ConversionResult;

pub const SUPPORTED_FORMATS: &[&str] = &["mp4", "gif"];

pub fn convert(
    source_paths: &[String],
    target_format: &str,
    output_dir: &str,
    interval_secs: f64,
) -> Result<ConversionResult, String> {
    if source_paths.is_empty() {
        return Err("not_enough_source_files".into());
    }
    let target_format = normalize_format(target_format);
    if !SUPPORTED_FORMATS.contains(&target_format.as_str()) {
        return Err(format!("unsupported_target_format: {target_format}"));
    }
    let interval = if interval_secs <= 0.0 {
        2.0
    } else {
        interval_secs
    };
    ensure_sources_exist(source_paths)?;
    let out_dir = ensure_output_dir(output_dir)?;
    let output_path = out_dir.join(format!("images-to-video.{target_format}"));
    let ffmpeg = find_executable(&["ffmpeg"]).map_err(|_| "ffmpeg_not_found".to_string())?;

    let mut segment_paths = Vec::new();
    for (i, path) in source_paths.iter().enumerate() {
        let segment = out_dir.join(format!("segment_{i}.mp4"));
        run_command(
            &ffmpeg,
            &[
                "-y".into(),
                "-loop".into(),
                "1".into(),
                "-i".into(),
                path.clone(),
                "-vf".into(),
                "scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:720:(ow-iw)/2:(oh-ih)/2:black".into(),
                "-c:v".into(),
                "libx264".into(),
                "-t".into(),
                format!("{interval}"),
                "-pix_fmt".into(),
                "yuv420p".into(),
                "-an".into(),
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

    if target_format == "mp4" {
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
                output_path.to_string_lossy().into_owned(),
            ],
        )?;
    } else {
        // gif via palette
        let temp_mp4 = out_dir.join("images-to-video-temp.mp4");
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
                temp_mp4.to_string_lossy().into_owned(),
            ],
        )?;
        run_command(
            &ffmpeg,
            &[
                "-y".into(),
                "-i".into(),
                temp_mp4.to_string_lossy().into_owned(),
                "-vf".into(),
                "fps=10,scale=480:-1:flags=lanczos".into(),
                output_path.to_string_lossy().into_owned(),
            ],
        )?;
    }

    if !output_path.is_file()
        || std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0)
            == 0
    {
        return Err("conversion_failed: empty or missing output".into());
    }

    let media = if target_format == "gif" {
        "image/gif"
    } else {
        "video/mp4"
    };

    Ok(ConversionResult::new(
        output_path.to_string_lossy(),
        media,
        target_format,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::util::test_fixtures::make_png;
    use tempfile::TempDir;

    #[test]
    fn converts_images_to_mp4() {
        let dir = TempDir::new().unwrap();
        let a = make_png(&dir, "a.png");
        let b = make_png(&dir, "b.png");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let result = convert(
            &[
                a.to_string_lossy().into_owned(),
                b.to_string_lossy().into_owned(),
            ],
            "mp4",
            out.to_str().unwrap(),
            0.2,
        )
        .expect("images to video");

        assert!(std::path::Path::new(&result.output_path).is_file());
        assert!(std::fs::metadata(&result.output_path).unwrap().len() > 0);
        assert!(result.output_path.ends_with(".mp4"));
    }
}

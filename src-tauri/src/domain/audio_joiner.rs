use super::util::{
    audio_codec_args, ensure_output_dir, ensure_sources_exist, find_executable,
    media_type_for_audio, normalize_format, run_command,
};
use super::ConversionResult;

pub const SUPPORTED_FORMATS: &[&str] = &["mp3", "wav", "ogg", "aac", "flac"];

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
    let output_path = out_dir.join(format!("together-audios.{target_format}"));
    let ffmpeg = find_executable(&["ffmpeg"]).map_err(|_| "ffmpeg_not_found".to_string())?;

    let n = source_paths.len();
    let mut filter = String::new();
    for i in 0..n {
        filter.push_str(&format!("[{i}:a]"));
    }
    filter.push_str(&format!("concat=n={n}:v=0:a=1[outa]"));

    let mut args = vec!["-y".into()];
    for path in source_paths {
        args.push("-i".into());
        args.push(path.clone());
    }
    args.extend([
        "-filter_complex".into(),
        filter,
        "-map".into(),
        "[outa]".into(),
        "-vn".into(),
    ]);
    args.extend(audio_codec_args(&target_format)?);
    args.push(output_path.to_string_lossy().into_owned());

    run_command(&ffmpeg, &args)?;

    if !output_path.is_file()
        || std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0)
            == 0
    {
        return Err("join_failed: empty or missing output".into());
    }

    Ok(ConversionResult::new(
        output_path.to_string_lossy(),
        media_type_for_audio(&target_format),
        target_format,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::util::test_fixtures::make_wav;
    use tempfile::TempDir;

    #[test]
    fn joins_two_wav_files_to_mp3() {
        let dir = TempDir::new().unwrap();
        let a = make_wav(&dir, "a.wav");
        let b = make_wav(&dir, "b.wav");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let result = join(
            &[
                a.to_string_lossy().into_owned(),
                b.to_string_lossy().into_owned(),
            ],
            "mp3",
            out.to_str().unwrap(),
        )
        .expect("join audio");

        assert!(std::path::Path::new(&result.output_path).is_file());
        assert!(std::fs::metadata(&result.output_path).unwrap().len() > 0);
        assert!(result.output_path.ends_with("together-audios.mp3"));
    }
}

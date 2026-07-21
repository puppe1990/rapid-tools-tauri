use super::util::{
    audio_codec_args, ensure_output_dir, ensure_source_exists, find_executable,
    media_type_for_audio, normalize_format, output_path_for, run_command,
};
use super::ConversionResult;

pub const SUPPORTED_FORMATS: &[&str] = &["mp3", "wav", "ogg", "aac", "flac"];

pub fn convert(
    source_path: &str,
    target_format: &str,
    output_dir: &str,
) -> Result<ConversionResult, String> {
    let target_format = normalize_format(target_format);
    if !SUPPORTED_FORMATS.contains(&target_format.as_str()) {
        return Err(format!("unsupported_target_format: {target_format}"));
    }
    ensure_source_exists(source_path)?;
    let out_dir = ensure_output_dir(output_dir)?;
    let output_path = output_path_for(source_path, &out_dir, &target_format);
    let ffmpeg = find_executable(&["ffmpeg"]).map_err(|_| "ffmpeg_not_found".to_string())?;

    let mut args = vec!["-y".into(), "-i".into(), source_path.into(), "-vn".into()];
    args.extend(audio_codec_args(&target_format)?);
    args.push(output_path.to_string_lossy().into_owned());

    run_command(&ffmpeg, &args)?;

    if !output_path.is_file() {
        return Err("conversion_failed: output missing".into());
    }
    if std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0)
        == 0
    {
        return Err("conversion_failed: empty output".into());
    }

    Ok(ConversionResult::new(
        output_path.to_string_lossy(),
        media_type_for_audio(&target_format),
        target_format,
    ))
}

#[allow(dead_code)]
pub fn supported_formats() -> Vec<String> {
    SUPPORTED_FORMATS.iter().map(|s| (*s).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::util::test_fixtures::make_wav;
    use tempfile::TempDir;

    #[test]
    fn converts_wav_to_mp3_with_real_ffmpeg() {
        let dir = TempDir::new().unwrap();
        let source = make_wav(&dir, "tone.wav");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let result =
            convert(source.to_str().unwrap(), "mp3", out.to_str().unwrap()).expect("audio convert");

        assert!(std::path::Path::new(&result.output_path).is_file());
        assert!(std::fs::metadata(&result.output_path).unwrap().len() > 0);
        assert!(result.output_path.ends_with(".mp3"));
        assert_eq!(result.target_format, "mp3");
        assert_eq!(result.media_type, "audio/mpeg");
    }

    #[test]
    fn converts_wav_to_flac() {
        let dir = TempDir::new().unwrap();
        let source = make_wav(&dir, "tone.wav");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();
        let result =
            convert(source.to_str().unwrap(), "flac", out.to_str().unwrap()).expect("flac convert");
        assert!(result.output_path.ends_with(".flac"));
        assert!(std::fs::metadata(&result.output_path).unwrap().len() > 0);
    }

    #[test]
    fn rejects_unsupported_format() {
        let dir = TempDir::new().unwrap();
        let source = make_wav(&dir, "tone.wav");
        let err = convert(
            source.to_str().unwrap(),
            "wma",
            dir.path().to_str().unwrap(),
        )
        .unwrap_err();
        assert!(err.contains("unsupported_target_format"));
    }
}

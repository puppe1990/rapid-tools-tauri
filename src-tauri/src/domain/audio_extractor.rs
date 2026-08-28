use super::util::{
    audio_codec_args, ensure_output_dir, ensure_source_exists, ensure_source_extension,
    find_executable, media_type_for_audio, normalize_format, output_path_for, run_command,
    tool_command,
};
use super::ConversionResult;

pub const SUPPORTED_FORMATS: &[&str] = &["mp3", "wav", "ogg", "aac", "flac"];
pub const SUPPORTED_INPUTS: &[&str] = &["mp4", "mov", "webm", "mkv", "avi", "ts", "3gp"];

pub fn extract(
    source_path: &str,
    target_format: &str,
    output_dir: &str,
) -> Result<ConversionResult, String> {
    let target_format = normalize_format(target_format);
    if !SUPPORTED_FORMATS.contains(&target_format.as_str()) {
        return Err(format!("unsupported_target_format: {target_format}"));
    }
    ensure_source_exists(source_path)?;
    ensure_source_extension(source_path, SUPPORTED_INPUTS)?;
    ensure_has_audio_stream(source_path)?;

    let out_dir = ensure_output_dir(output_dir)?;
    let output_path = output_path_for(source_path, &out_dir, &target_format);
    let ffmpeg = find_executable(&["ffmpeg"]).map_err(|_| "ffmpeg_not_found".to_string())?;

    let mut args = vec!["-y".into(), "-i".into(), source_path.into(), "-vn".into()];
    args.extend(audio_codec_args(&target_format)?);
    args.push(output_path.to_string_lossy().into_owned());

    run_command(&ffmpeg, &args)?;

    if !output_path.is_file()
        || std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0)
            == 0
    {
        return Err("conversion_failed: empty or missing output".into());
    }

    Ok(ConversionResult::new(
        output_path.to_string_lossy(),
        media_type_for_audio(&target_format),
        target_format,
    ))
}

fn ensure_has_audio_stream(source_path: &str) -> Result<(), String> {
    let Ok(ffprobe) = find_executable(&["ffprobe"]) else {
        return Ok(());
    };
    let output = tool_command(&ffprobe)
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            source_path,
        ])
        .output()
        .map_err(|e| format!("ffprobe_failed: {e}"))?;

    if !output.status.success() {
        return Err("no_audio_stream".into());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    if text.contains("audio") {
        Ok(())
    } else {
        Err("no_audio_stream".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::util::test_fixtures::make_mp4;
    use tempfile::TempDir;

    #[test]
    fn extracts_audio_from_mp4_to_mp3() {
        let dir = TempDir::new().unwrap();
        let source = make_mp4(&dir, "clip.mp4");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let result =
            extract(source.to_str().unwrap(), "mp3", out.to_str().unwrap()).expect("extract audio");

        assert!(std::path::Path::new(&result.output_path).is_file());
        assert!(std::fs::metadata(&result.output_path).unwrap().len() > 0);
        assert!(result.output_path.ends_with(".mp3"));
    }
}

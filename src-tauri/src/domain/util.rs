use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Directories GUI-launched macOS apps often miss (Finder PATH is not a shell PATH).
const EXTRA_TOOL_PATHS: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"];

pub fn normalize_format(format: &str) -> String {
    format.trim().to_lowercase()
}

pub fn ensure_source_exists(path: &str) -> Result<(), String> {
    if Path::new(path).is_file() {
        Ok(())
    } else {
        Err("source_file_not_found".into())
    }
}

pub fn ensure_sources_exist(paths: &[String]) -> Result<(), String> {
    for path in paths {
        ensure_source_exists(path)?;
    }
    Ok(())
}

pub fn source_extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .map(normalize_format)
        .unwrap_or_default()
}

fn canonicalize_ext(ext: &str) -> String {
    let e = normalize_format(ext);
    if e == "jpeg" {
        "jpg".into()
    } else {
        e
    }
}

/// Rejects files whose extension is not in `allowed` (jpeg is treated as jpg).
pub fn ensure_source_extension(path: &str, allowed: &[&str]) -> Result<(), String> {
    let ext = canonicalize_ext(&source_extension(path));
    if ext.is_empty() {
        return Err("unsupported_source_format: unknown".into());
    }
    let allowed: Vec<String> = allowed.iter().copied().map(canonicalize_ext).collect();
    if allowed.iter().any(|candidate| candidate == &ext) {
        Ok(())
    } else {
        Err(format!("unsupported_source_format: {ext}"))
    }
}

pub fn ensure_sources_extensions(paths: &[String], allowed: &[&str]) -> Result<(), String> {
    for path in paths {
        ensure_source_extension(path, allowed)?;
    }
    Ok(())
}

pub fn ensure_output_dir(output_dir: &str) -> Result<PathBuf, String> {
    let dir = PathBuf::from(output_dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("output_dir_error: {e}"))?;
    Ok(dir)
}

pub fn find_executable(names: &[&str]) -> Result<PathBuf, String> {
    for name in names {
        if let Ok(path) = which(name) {
            return Ok(path);
        }
    }
    Err(format!("{} not found", names.join(" or ")))
}

/// PATH used to find tools and inherited by magick/ffmpeg child processes.
pub fn tool_search_path() -> OsString {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut dirs: Vec<PathBuf> = EXTRA_TOOL_PATHS.iter().map(PathBuf::from).collect();
    for dir in std::env::split_paths(&current) {
        if !dir.as_os_str().is_empty() && !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    std::env::join_paths(&dirs).unwrap_or(current)
}

pub fn tool_command(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.env("PATH", tool_search_path());
    cmd
}

fn which(name: &str) -> Result<PathBuf, ()> {
    for dir in std::env::split_paths(&tool_search_path()) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
        #[cfg(windows)]
        {
            let with_exe = dir.join(format!("{name}.exe"));
            if with_exe.is_file() {
                return Ok(with_exe);
            }
        }
    }
    Err(())
}

fn classify_tool_failure(combined: &str, code: Option<i32>) -> String {
    let lower = combined.to_lowercase();
    if lower.contains("ffmpeg: command not found")
        || lower.contains("ffmpeg: not found")
        || lower.contains("unable to execute file: ffmpeg")
    {
        return "ffmpeg_not_found".into();
    }
    if lower.contains("gs: command not found")
        || lower.contains("gs: not found")
        || (lower.contains("failedtoexecutecommand") && lower.contains("'gs'"))
    {
        return "ghostscript_not_found".into();
    }
    format!("conversion_failed (code {code:?}): {combined}")
}

pub fn run_command(program: &Path, args: &[String]) -> Result<(), String> {
    let output = tool_command(program)
        .args(args)
        .output()
        .map_err(|e| format!("command_failed_to_start: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(classify_tool_failure(
            &format!("{stdout}{stderr}"),
            output.status.code(),
        ))
    }
}

pub fn output_path_for(source_path: &str, output_dir: &Path, target_format: &str) -> PathBuf {
    let stem = Path::new(source_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    output_dir.join(format!("{stem}.{target_format}"))
}

pub fn media_type_for_image(format: &str) -> &'static str {
    match format {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "heic" => "image/heic",
        "avif" => "image/avif",
        "enc" => "image/enc",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    }
}

pub fn media_type_for_video(format: &str) -> &'static str {
    match format {
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "3gp" => "video/3gpp",
        _ => "application/octet-stream",
    }
}

pub fn media_type_for_audio(format: &str) -> &'static str {
    match format {
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        _ => "application/octet-stream",
    }
}

pub fn audio_codec_args(format: &str) -> Result<Vec<String>, String> {
    let args = match format {
        "mp3" => vec!["-c:a", "libmp3lame"],
        "wav" => vec!["-c:a", "pcm_s16le"],
        "ogg" => vec!["-c:a", "libopus"],
        "aac" => vec!["-c:a", "aac"],
        "flac" => vec!["-c:a", "flac"],
        _ => return Err(format!("unsupported_target_format: {format}")),
    };
    Ok(args.into_iter().map(String::from).collect())
}

#[cfg(test)]
pub mod test_fixtures {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    pub fn require_magick() -> PathBuf {
        find_executable(&["magick", "convert"]).expect("ImageMagick (magick/convert) required")
    }

    pub fn require_ffmpeg() -> PathBuf {
        find_executable(&["ffmpeg"]).expect("ffmpeg required")
    }

    pub fn make_png(dir: &TempDir, name: &str) -> PathBuf {
        let magick = require_magick();
        let path = dir.path().join(name);
        run_command(
            &magick,
            &[
                "-size".into(),
                "32x32".into(),
                "xc:red".into(),
                path.to_string_lossy().into(),
            ],
        )
        .expect("create png fixture");
        assert!(path.is_file());
        path
    }

    pub fn make_wav(dir: &TempDir, name: &str) -> PathBuf {
        let ffmpeg = require_ffmpeg();
        let path = dir.path().join(name);
        run_command(
            &ffmpeg,
            &[
                "-y".into(),
                "-f".into(),
                "lavfi".into(),
                "-i".into(),
                "sine=frequency=440:duration=0.3".into(),
                path.to_string_lossy().into(),
            ],
        )
        .expect("create wav fixture");
        assert!(path.is_file());
        path
    }

    pub fn make_mp4(dir: &TempDir, name: &str) -> PathBuf {
        let ffmpeg = require_ffmpeg();
        let path = dir.path().join(name);
        run_command(
            &ffmpeg,
            &[
                "-y".into(),
                "-f".into(),
                "lavfi".into(),
                "-i".into(),
                "color=c=blue:s=64x64:d=0.5".into(),
                "-f".into(),
                "lavfi".into(),
                "-i".into(),
                "sine=frequency=440:duration=0.5".into(),
                "-shortest".into(),
                "-c:v".into(),
                "libx264".into(),
                "-pix_fmt".into(),
                "yuv420p".into(),
                "-c:a".into(),
                "aac".into(),
                path.to_string_lossy().into(),
            ],
        )
        .expect("create mp4 fixture");
        assert!(path.is_file());
        path
    }

    pub fn make_qr_png(dir: &TempDir, name: &str, payload: &str) -> PathBuf {
        let qrencode = find_executable(&["qrencode"]).expect("qrencode required for QR fixtures");
        let path = dir.path().join(name);
        run_command(
            &qrencode,
            &["-o".into(), path.to_string_lossy().into(), payload.into()],
        )
        .expect("create qr png");
        assert!(path.is_file());
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn rejects_disallowed_source_extension() {
        let err = ensure_source_extension("clip.mkv", &["png", "jpg", "webp"]).unwrap_err();
        assert_eq!(err, "unsupported_source_format: mkv");
    }

    #[test]
    fn accepts_jpeg_as_jpg() {
        ensure_source_extension("photo.JPEG", &["png", "jpg", "webp"]).unwrap();
    }

    #[test]
    fn rejects_missing_extension() {
        let err = ensure_source_extension("readme", &["png"]).unwrap_err();
        assert_eq!(err, "unsupported_source_format: unknown");
    }

    #[test]
    fn tool_search_path_includes_homebrew_prefix() {
        let path = tool_search_path().to_string_lossy().into_owned();
        assert!(
            path.contains("/opt/homebrew/bin"),
            "expected Homebrew prefix in {path}"
        );
    }

    #[test]
    fn classifies_missing_ffmpeg_delegate() {
        let err = classify_tool_failure(
            "sh: ffmpeg: command not found\nmagick: no images found for operation `-auto-orient'",
            Some(1),
        );
        assert_eq!(err, "ffmpeg_not_found");
    }

    #[test]
    fn classifies_missing_ghostscript_delegate() {
        let err = classify_tool_failure(
            "sh: gs: command not found\nmagick: FailedToExecuteCommand `'gs' -sDEVICE=png16malpha'",
            Some(1),
        );
        assert_eq!(err, "ghostscript_not_found");
    }

    #[test]
    fn ensure_sources_extensions_reports_first_bad_file() {
        let dir = TempDir::new().unwrap();
        let ok = dir.path().join("a.png");
        let bad = dir.path().join("b.mkv");
        std::fs::write(&ok, b"x").unwrap();
        std::fs::write(&bad, b"x").unwrap();
        let paths = vec![
            ok.to_string_lossy().into_owned(),
            bad.to_string_lossy().into_owned(),
        ];
        let err = ensure_sources_extensions(&paths, &["png", "jpg"]).unwrap_err();
        assert_eq!(err, "unsupported_source_format: mkv");
    }
}

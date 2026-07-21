use std::path::{Path, PathBuf};
use std::process::Command;

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

fn which(name: &str) -> Result<PathBuf, ()> {
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Ok(candidate);
            }
            // macOS/Homebrew sometimes needs no extension; also try as-is
            #[cfg(windows)]
            {
                let with_exe = dir.join(format!("{name}.exe"));
                if with_exe.is_file() {
                    return Ok(with_exe);
                }
            }
        }
    }
    // Absolute common paths (homebrew)
    for prefix in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"] {
        let candidate = PathBuf::from(prefix).join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(())
}

pub fn run_command(program: &Path, args: &[String]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("command_failed_to_start: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!(
            "conversion_failed (code {:?}): {stdout}{stderr}",
            output.status.code()
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

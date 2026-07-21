use super::util::{
    ensure_output_dir, ensure_source_exists, find_executable, media_type_for_image, normalize_format,
    run_command,
};
use super::ConversionResult;
use std::path::PathBuf;

pub const SUPPORTED_FORMATS: &[&str] = &["original", "jpg", "png", "webp"];
pub const SUPPORTED_FITS: &[&str] = &["contain", "cover", "stretch"];

pub fn resize(
    source_path: &str,
    width: u32,
    height: u32,
    output_dir: &str,
    target_format: Option<&str>,
    fit: Option<&str>,
) -> Result<ConversionResult, String> {
    if width == 0 || height == 0 {
        return Err("invalid_dimension".into());
    }
    let target_format = normalize_format(target_format.unwrap_or("original"));
    if !SUPPORTED_FORMATS.contains(&target_format.as_str()) {
        return Err(format!("unsupported_target_format: {target_format}"));
    }
    let fit = normalize_format(fit.unwrap_or("contain"));
    if !SUPPORTED_FITS.contains(&fit.as_str()) {
        return Err(format!("unsupported_fit: {fit}"));
    }
    ensure_source_exists(source_path)?;
    let out_dir = ensure_output_dir(output_dir)?;
    let actual_format = output_format(source_path, &target_format);
    let output_path: PathBuf = out_dir.join(format!(
        "{}-{}x{}.{}",
        std::path::Path::new(source_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image"),
        width,
        height,
        actual_format
    ));

    let magick = find_executable(&["magick", "convert"])
        .map_err(|_| "imagemagick_not_found".to_string())?;

    let mut args = vec![source_path.to_string()];
    args.extend(resize_args(width, height, &fit));
    args.push(output_path.to_string_lossy().into_owned());

    run_command(&magick, &args)?;

    if !output_path.is_file() || std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0) == 0
    {
        return Err("resize_failed: empty or missing output".into());
    }

    Ok(ConversionResult::new(
        output_path.to_string_lossy(),
        media_type_for_image(&actual_format),
        actual_format,
    ))
}

fn output_format(source_path: &str, target_format: &str) -> String {
    if target_format == "original" {
        let ext = std::path::Path::new(source_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png");
        let f = normalize_format(ext);
        if f == "jpeg" {
            "jpg".into()
        } else {
            f
        }
    } else {
        target_format.to_string()
    }
}

fn resize_args(width: u32, height: u32, fit: &str) -> Vec<String> {
    match fit {
        "contain" => vec!["-resize".into(), format!("{width}x{height}")],
        "cover" => vec![
            "-resize".into(),
            format!("{width}x{height}^"),
            "-gravity".into(),
            "center".into(),
            "-extent".into(),
            format!("{width}x{height}"),
        ],
        "stretch" => vec!["-resize".into(), format!("{width}x{height}!")],
        _ => vec!["-resize".into(), format!("{width}x{height}")],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::util::test_fixtures::make_png;
    use tempfile::TempDir;

    #[test]
    fn resizes_png_to_jpg_contain() {
        let dir = TempDir::new().unwrap();
        let source = make_png(&dir, "photo.png");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let result = resize(
            source.to_str().unwrap(),
            16,
            16,
            out.to_str().unwrap(),
            Some("jpg"),
            Some("contain"),
        )
        .expect("resize");

        assert!(std::path::Path::new(&result.output_path).is_file());
        assert!(std::fs::metadata(&result.output_path).unwrap().len() > 0);
        assert!(result.output_path.ends_with(".jpg"));
        assert!(result.filename.contains("16x16"));
    }
}

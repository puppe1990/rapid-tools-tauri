use super::util::{
    ensure_output_dir, ensure_source_exists, find_executable, media_type_for_image, normalize_format,
    output_path_for, run_command,
};
use super::ConversionResult;

pub const SUPPORTED_FORMATS: &[&str] = &["png", "jpg", "webp", "heic", "avif", "enc"];

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

    let magick = find_executable(&["magick", "convert"])
        .map_err(|_| "imagemagick_not_found".to_string())?;

    let command_output = if target_format == "enc" {
        format!("EPS:{}", output_path.display())
    } else {
        output_path.to_string_lossy().into_owned()
    };

    run_command(
        &magick,
        &[
            source_path.to_string(),
            "-auto-orient".into(),
            command_output,
        ],
    )?;

    if !output_path.is_file() {
        return Err("conversion_failed: output missing".into());
    }

    let meta = std::fs::metadata(&output_path).map_err(|e| e.to_string())?;
    if meta.len() == 0 {
        return Err("conversion_failed: empty output".into());
    }

    Ok(ConversionResult::new(
        output_path.to_string_lossy(),
        media_type_for_image(&target_format),
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
    use crate::domain::util::test_fixtures::make_png;
    use tempfile::TempDir;

    #[test]
    fn converts_png_to_jpg_with_real_imagemagick() {
        let dir = TempDir::new().unwrap();
        let source = make_png(&dir, "sample.png");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let result = convert(
            source.to_str().unwrap(),
            "jpg",
            out.to_str().unwrap(),
        )
        .expect("image convert should succeed");

        assert!(std::path::Path::new(&result.output_path).is_file());
        assert!(std::fs::metadata(&result.output_path).unwrap().len() > 0);
        assert!(result.output_path.ends_with(".jpg"));
        assert_eq!(result.target_format, "jpg");
        assert_eq!(result.media_type, "image/jpeg");
        assert_eq!(result.filename, "sample.jpg");
    }

    #[test]
    fn converts_png_to_webp() {
        let dir = TempDir::new().unwrap();
        let source = make_png(&dir, "sample.png");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let result = convert(source.to_str().unwrap(), "webp", out.to_str().unwrap())
            .expect("webp convert");
        assert!(result.output_path.ends_with(".webp"));
        assert!(std::fs::metadata(&result.output_path).unwrap().len() > 0);
    }

    #[test]
    fn rejects_unsupported_format() {
        let dir = TempDir::new().unwrap();
        let source = make_png(&dir, "sample.png");
        let err = convert(source.to_str().unwrap(), "bmp", dir.path().to_str().unwrap())
            .unwrap_err();
        assert!(err.contains("unsupported_target_format"));
    }

    #[test]
    fn rejects_missing_source() {
        let dir = TempDir::new().unwrap();
        let err = convert("/no/such/file.png", "jpg", dir.path().to_str().unwrap()).unwrap_err();
        assert_eq!(err, "source_file_not_found");
    }
}

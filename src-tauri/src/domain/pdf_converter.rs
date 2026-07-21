use super::util::{
    ensure_output_dir, ensure_source_exists, ensure_sources_exist, find_executable, run_command,
};
use super::ConversionResult;
use std::path::PathBuf;

pub fn images_to_pdf(
    source_paths: &[String],
    output_dir: &str,
) -> Result<ConversionResult, String> {
    if source_paths.is_empty() {
        return Err("source_files_not_found".into());
    }
    ensure_sources_exist(source_paths)?;
    let out_dir = ensure_output_dir(output_dir)?;
    let output_path = out_dir.join("photos.pdf");
    let magick =
        find_executable(&["magick", "convert"]).map_err(|_| "imagemagick_not_found".to_string())?;

    let mut args: Vec<String> = source_paths.to_vec();
    args.push(output_path.to_string_lossy().into_owned());
    run_command(&magick, &args)?;

    if !output_path.is_file()
        || std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0)
            == 0
    {
        return Err("conversion_failed: empty or missing pdf".into());
    }

    Ok(ConversionResult::new(
        output_path.to_string_lossy(),
        "application/pdf",
        "pdf",
    ))
}

pub fn pdf_to_images(
    source_path: &str,
    target_format: &str,
    output_dir: &str,
) -> Result<Vec<ConversionResult>, String> {
    let target_format = target_format.trim().to_lowercase();
    if target_format != "png" && target_format != "jpg" {
        return Err(format!("unsupported_target_format: {target_format}"));
    }
    ensure_source_exists(source_path)?;
    let out_dir = ensure_output_dir(output_dir)?;
    let magick =
        find_executable(&["magick", "convert"]).map_err(|_| "imagemagick_not_found".to_string())?;

    let stem = std::path::Path::new(source_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");
    let template: PathBuf = out_dir.join(format!("{stem}-%02d.{target_format}"));

    run_command(
        &magick,
        &[
            "-density".into(),
            "144".into(),
            source_path.into(),
            "-quality".into(),
            "92".into(),
            template.to_string_lossy().into_owned(),
        ],
    )?;

    let mut results = Vec::new();
    let mut index = 0u32;
    loop {
        let candidate = out_dir.join(format!("{stem}-{index:02}.{target_format}"));
        if !candidate.is_file() {
            break;
        }
        let media = if target_format == "jpg" {
            "image/jpeg"
        } else {
            "image/png"
        };
        results.push(ConversionResult::new(
            candidate.to_string_lossy(),
            media,
            target_format.clone(),
        ));
        index += 1;
    }

    if results.is_empty() {
        return Err("conversion_failed: no pages produced".into());
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::util::test_fixtures::make_png;
    use tempfile::TempDir;

    #[test]
    fn photos_to_pdf_and_pdf_to_png_roundtrip() {
        let dir = TempDir::new().unwrap();
        let a = make_png(&dir, "p1.png");
        let b = make_png(&dir, "p2.png");
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let pdf = images_to_pdf(
            &[
                a.to_string_lossy().into_owned(),
                b.to_string_lossy().into_owned(),
            ],
            out.to_str().unwrap(),
        )
        .expect("images to pdf");

        assert!(std::path::Path::new(&pdf.output_path).is_file());
        assert!(pdf.output_path.ends_with(".pdf"));
        assert!(std::fs::metadata(&pdf.output_path).unwrap().len() > 0);

        let pages_dir = dir.path().join("pages");
        std::fs::create_dir_all(&pages_dir).unwrap();
        let pages = pdf_to_images(&pdf.output_path, "png", pages_dir.to_str().unwrap())
            .expect("pdf to images");
        assert!(!pages.is_empty());
        for page in &pages {
            assert!(std::path::Path::new(&page.output_path).is_file());
            assert!(std::fs::metadata(&page.output_path).unwrap().len() > 0);
            assert!(page.output_path.ends_with(".png"));
        }
    }
}

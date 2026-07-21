use super::ZipResult;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

#[derive(Debug, Clone)]
pub struct ZipEntry {
    pub path: String,
    pub filename: String,
}

pub fn build(id: &str, entries: &[ZipEntry], output_dir: &str) -> Result<ZipResult, String> {
    if entries.is_empty() {
        return Err("empty_entries".into());
    }

    std::fs::create_dir_all(output_dir).map_err(|e| format!("mkdir_failed: {e}"))?;
    let zip_path = PathBuf::from(output_dir).join(format!("rapid-tools-{id}.zip"));
    if zip_path.exists() {
        let _ = std::fs::remove_file(&zip_path);
    }

    let file = File::create(&zip_path).map_err(|e| format!("zip_create_failed: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut used: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for entry in entries {
        let source = std::path::Path::new(&entry.path);
        if !source.is_file() {
            return Err(format!("stage_failed: missing {}", entry.path));
        }

        let filename = unique_filename(&safe_filename(&entry.filename), &mut used);
        let mut data = Vec::new();
        File::open(source)
            .and_then(|mut f| f.read_to_end(&mut data))
            .map_err(|e| format!("stage_failed: {e}"))?;

        zip.start_file(&filename, options)
            .map_err(|e| format!("zip_failed: {e}"))?;
        zip.write_all(&data)
            .map_err(|e| format!("zip_failed: {e}"))?;
    }

    zip.finish().map_err(|e| format!("zip_failed: {e}"))?;

    let meta = std::fs::metadata(&zip_path).map_err(|e| e.to_string())?;
    if meta.len() == 0 {
        return Err("zip_failed: empty archive".into());
    }

    let path_str = zip_path.to_string_lossy().into_owned();
    let filename = zip_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("archive.zip")
        .to_string();

    Ok(ZipResult {
        path: path_str,
        filename,
        media_type: "application/zip".into(),
    })
}

fn safe_filename(filename: &str) -> String {
    let base = std::path::Path::new(filename)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let cleaned: String = base
        .chars()
        .map(|c| if c.is_control() { '_' } else { c })
        .collect();
    if cleaned.is_empty() {
        "file".into()
    } else {
        cleaned
    }
}

fn unique_filename(filename: &str, used: &mut std::collections::HashMap<String, u32>) -> String {
    match used.get(filename).copied() {
        None => {
            used.insert(filename.to_string(), 1);
            filename.to_string()
        }
        Some(count) => {
            let path = std::path::Path::new(filename);
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();
            let next = format!("{stem} ({}){ext}", count + 1);
            used.insert(filename.to_string(), count + 1);
            next
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audio_converter;
    use crate::domain::image_converter;
    use crate::domain::util::test_fixtures::{make_png, make_wav};
    use tempfile::TempDir;

    #[test]
    fn builds_zip_from_real_converted_outputs() {
        let dir = TempDir::new().unwrap();
        let png = make_png(&dir, "a.png");
        let wav = make_wav(&dir, "b.wav");
        let conv_out = dir.path().join("conv");
        std::fs::create_dir_all(&conv_out).unwrap();

        let img = image_converter::convert(
            png.to_str().unwrap(),
            "jpg",
            conv_out.to_str().unwrap(),
        )
        .unwrap();
        let aud = audio_converter::convert(
            wav.to_str().unwrap(),
            "mp3",
            conv_out.to_str().unwrap(),
        )
        .unwrap();

        let zip_dir = dir.path().join("zips");
        let result = build(
            "batch-1",
            &[
                ZipEntry {
                    path: img.output_path.clone(),
                    filename: img.filename.clone(),
                },
                ZipEntry {
                    path: aud.output_path.clone(),
                    filename: aud.filename.clone(),
                },
            ],
            zip_dir.to_str().unwrap(),
        )
        .expect("zip build");

        assert!(std::path::Path::new(&result.path).is_file());
        assert!(std::fs::metadata(&result.path).unwrap().len() > 0);
        assert!(result.path.ends_with(".zip"));
        assert_eq!(result.media_type, "application/zip");
        assert_eq!(result.filename, "rapid-tools-batch-1.zip");

        // Verify archive contains both files
        let file = File::open(&result.path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        assert_eq!(archive.len(), 2);
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(names.iter().any(|n| n.ends_with(".jpg")));
        assert!(names.iter().any(|n| n.ends_with(".mp3")));
    }

    #[test]
    fn rejects_empty_entries() {
        let dir = TempDir::new().unwrap();
        let err = build("x", &[], dir.path().to_str().unwrap()).unwrap_err();
        assert_eq!(err, "empty_entries");
    }
}

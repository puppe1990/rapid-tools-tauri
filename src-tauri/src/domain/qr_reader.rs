use super::util::{ensure_source_exists, ensure_source_extension, find_executable, tool_command};

pub const SUPPORTED_INPUTS: &[&str] = &["png", "jpg", "jpeg", "webp"];

#[allow(dead_code)]
pub fn available() -> bool {
    find_executable(&["zbarimg"]).is_ok()
}

pub fn read(source_path: &str) -> Result<String, String> {
    ensure_source_exists(source_path)?;
    ensure_source_extension(source_path, SUPPORTED_INPUTS)?;
    let zbar = find_executable(&["zbarimg"]).map_err(|_| "zbar_unavailable".to_string())?;

    let output = tool_command(&zbar)
        .args(["-q", "--raw", source_path])
        .output()
        .map_err(|e| format!("zbar_failed: {e}"))?;

    if !output.status.success() {
        return Err("no_qr_found".into());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let first = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|s| s.to_string());

    first.ok_or_else(|| "no_qr_found".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::util::test_fixtures::make_qr_png;
    use tempfile::TempDir;

    #[test]
    fn reads_qr_payload_from_real_image() {
        if !available() {
            panic!("zbarimg required for QR reader tests");
        }
        let dir = TempDir::new().unwrap();
        let payload = "rapid-tools-tauri-qr-test";
        let qr = make_qr_png(&dir, "code.png", payload);

        let text = read(qr.to_str().unwrap()).expect("read qr");
        assert_eq!(text, payload);
    }
}

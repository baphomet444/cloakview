//! One screenshot of the main display, for the screen-reading copilot.
//!
//! Taken only when the user asks. The picture is written to a temporary file by
//! macOS's own `screencapture`, shrunk with `sips`, read into memory and deleted
//! at once: it is never kept. CloakView's own windows are not in it, because they
//! are excluded from every capture.

use crate::error::{AppError, AppResult};
use base64::Engine;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The longest side sent, in pixels: enough to read text, small enough to upload fast.
const MAX_SIDE: &str = "2000";

/// Deletes the temporary screenshot whatever happens after it is written.
struct TempShot(PathBuf);

impl Drop for TempShot {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn run(program: &str, args: &[&str], path: &Path) -> AppResult<()> {
    let output = Command::new(program)
        .args(args)
        .arg(path)
        .output()
        .map_err(|e| AppError::Audio(format!("{program}: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::Audio(format!("{program} failed: {}", String::from_utf8_lossy(&output.stderr).trim())))
    }
}

/// `data:image/jpeg;base64,…`, the form the answers request takes an image in.
pub fn data_url(jpeg: &[u8]) -> String {
    format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(jpeg))
}

/// Captures the main display. Blocking (it runs two short system tools), so call
/// it off the async runtime. Fails with `PermissionDenied` (after asking macOS for
/// it) when screen recording is not allowed.
pub fn capture() -> AppResult<String> {
    if !objc2_core_graphics::CGPreflightScreenCaptureAccess() {
        objc2_core_graphics::CGRequestScreenCaptureAccess();
        return Err(AppError::PermissionDenied);
    }
    let shot = TempShot(std::env::temp_dir().join(format!("cloakview-{}.jpg", crate::db::uuid())));
    // -x: no sound, -m: main display only, -t jpg: format.
    run("/usr/sbin/screencapture", &["-x", "-m", "-t", "jpg"], &shot.0)?;
    run("/usr/bin/sips", &["-Z", MAX_SIDE, "-s", "formatOptions", "70"], &shot.0)?;
    let bytes = std::fs::read(&shot.0)?;
    if bytes.is_empty() {
        return Err(AppError::Audio("the screenshot came out empty".into()));
    }
    Ok(data_url(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_image_is_sent_as_a_jpeg_data_url() {
        assert_eq!(data_url(&[0xFF, 0xD8, 0xFF]), "data:image/jpeg;base64,/9j/");
    }

    #[test]
    fn the_temporary_screenshot_is_deleted_when_dropped() {
        let path = std::env::temp_dir().join(format!("cloakview-test-{}.jpg", crate::db::uuid()));
        std::fs::write(&path, b"x").unwrap();
        drop(TempShot(path.clone()));
        assert!(!path.exists());
    }
}

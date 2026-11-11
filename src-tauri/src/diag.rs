//! A small local diagnostics log: `~/Library/Logs/CloakView/cloakview.log`.
//!
//! It records what the listen pipeline did (started, connected, heard a turn,
//! failed and why) so a problem can be diagnosed after the fact. It never
//! records what was said or answered: only event names, sizes and error
//! messages. The file stays on this Mac and is capped in size.

use std::io::Write;
use std::path::PathBuf;

/// Once the file passes this size it is started over.
const MAX_BYTES: u64 = 512 * 1024;

fn log_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("Library/Logs/CloakView/cloakview.log"))
}

fn line(now_ms: i64, message: &str) -> String {
    // One line per entry, whatever the message contains.
    let message = message.replace(['\n', '\r'], " ");
    format!("{now_ms} {message}\n")
}

/// Appends one line. Failing to log is never an error for the caller.
pub fn log(message: &str) {
    let Some(path) = log_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let too_big = std::fs::metadata(&path).is_ok_and(|m| m.len() > MAX_BYTES);
    let file = std::fs::OpenOptions::new().create(true).append(!too_big).write(true).truncate(too_big).open(&path);
    if let Ok(mut file) = file {
        let _ = file.write_all(line(crate::db::now_ms(), message).as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entry_is_always_one_line() {
        assert_eq!(line(42, "error: bad\nrequest\r"), "42 error: bad request \n");
    }
}

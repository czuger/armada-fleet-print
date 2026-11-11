//! Atomic file writes: readers never see a partially written file.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Suffix counter keeping temporary file names unique within the process.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Error returned when a file cannot be written.
#[derive(Debug, thiserror::Error)]
#[error("writing {path} failed")]
pub struct AtomicWriteError {
    /// File or directory involved.
    pub path: PathBuf,
    /// Underlying error.
    #[source]
    pub source: io::Error,
}

/// Writes `bytes` to `path` through a temporary sibling file renamed over it, creating the
/// parent directory if needed.
///
/// # Errors
/// Returns [`AtomicWriteError`] naming the directory or file that could not be written.
pub async fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), AtomicWriteError> {
    let io_error = |path: &Path| {
        let path = path.to_path_buf();
        move |source| AtomicWriteError { path, source }
    };
    if let Some(dir) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(io_error(dir))?;
    }
    let suffix = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut temp_name = path.as_os_str().to_owned();
    temp_name.push(format!(".{}-{suffix}.part", std::process::id()));
    let temp_path = PathBuf::from(temp_name);
    tokio::fs::write(&temp_path, bytes)
        .await
        .map_err(io_error(&temp_path))?;
    tokio::fs::rename(&temp_path, path)
        .await
        .map_err(io_error(path))
}

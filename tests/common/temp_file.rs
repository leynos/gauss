//! Temporary-file cleanup for GPUI integration tests.

use camino::Utf8PathBuf;
use cap_std::fs_utf8::Dir;

/// Owns a temporary file and removes it when the guard is dropped.
pub struct TempFileGuard {
    /// Capability-scoped directory containing the temporary file.
    pub dir: Dir,
    /// File name relative to [`Self::dir`] used for cleanup.
    pub file_name: Utf8PathBuf,
}

impl TempFileGuard {
    /// Creates a cleanup guard for `file_name` within `dir`.
    pub const fn new(dir: Dir, file_name: Utf8PathBuf) -> Self {
        Self { dir, file_name }
    }

    /// Returns the capability-scoped directory containing the temporary file.
    pub const fn dir(&self) -> &Dir {
        &self.dir
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _cleanup = self.dir.remove_file(self.file_name.as_path());
    }
}

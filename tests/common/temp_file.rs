//! Temporary-file cleanup for GPUI integration tests.

use camino::Utf8PathBuf;
use cap_std::fs_utf8::Dir;

/// Owns a temporary file and removes it when the guard is dropped.
pub struct TempFileGuard {
    /// Capability-scoped directory containing the temporary file.
    pub dir: Dir,
    /// File name relative to [`Self::dir`] used for cleanup.
    pub file_name: Utf8PathBuf,
    /// Optional full path to the final file for tests that need to reopen it.
    pub path: Option<Utf8PathBuf>,
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _cleanup = self.dir.remove_file(self.file_name.as_path());
    }
}

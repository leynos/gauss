//! Temporary save-target cleanup for GPUI integration tests.

use camino::Utf8PathBuf;
use cap_std::fs_utf8::Dir;

/// Owns a temporary save target and removes it when cleanup is requested.
pub struct TempFileGuard {
    /// Capability-scoped directory containing the temporary file.
    pub dir: Dir,
    /// File name relative to [`Self::dir`].
    pub file_name: Utf8PathBuf,
}

impl TempFileGuard {
    /// Removes the temporary file, returning any filesystem error to the test.
    ///
    /// # Errors
    ///
    /// Returns the error reported when the guarded file cannot be removed.
    pub fn cleanup(&self) -> std::io::Result<()> {
        self.dir.remove_file(self.file_name.as_path())
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _cleanup = self.cleanup();
    }
}

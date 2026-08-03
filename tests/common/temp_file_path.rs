//! Path-backed temporary-file cleanup for GPUI integration tests.

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::fs_utf8::Dir;

/// Owns a temporary file and removes it when the guard is dropped.
///
/// The guard retains both the capability-relative cleanup name and the full
/// path exposed to the integration test.
pub struct TempFileGuard {
    dir: Dir,
    file_name: Utf8PathBuf,
    path: Utf8PathBuf,
}

impl TempFileGuard {
    /// Creates a guard from its directory, relative file name, and full path.
    pub const fn new(dir: Dir, file_name: Utf8PathBuf, path: Utf8PathBuf) -> Self {
        Self {
            dir,
            file_name,
            path,
        }
    }

    /// Returns the guarded file's full path by reference.
    pub fn path(&self) -> &Utf8Path {
        self.path.as_path()
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _cleanup = self.dir.remove_file(self.file_name.as_path());
    }
}

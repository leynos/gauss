//! Temporary save-target cleanup for GPUI integration tests.

use camino::Utf8PathBuf;
use cap_std::fs_utf8::Dir;

pub struct TempFileGuard {
    pub dir: Dir,
    pub file_name: Utf8PathBuf,
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _cleanup = self.dir.remove_file(self.file_name.as_path());
    }
}

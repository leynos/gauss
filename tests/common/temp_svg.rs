//! A scenario-owned temporary SVG file reached through a cap-std capability.
//!
//! The operations on [`TempSvgFile`] are split across sibling modules so each
//! scenario binary compiles only the ones it drives: see `temp_svg_write.rs`,
//! `temp_svg_read.rs` and `temp_svg_exists.rs`. The fields are `pub(crate)` so
//! those modules can reach them. Everything defined here is used by every binary
//! that includes it, so no `dead_code` expectation is needed.

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
use test_support::{TestSupportError, TestSupportResult};
use uuid::Uuid;

use crate::common::TempFileGuard;

/// A uniquely named temporary SVG whose lifetime is owned by the scenario.
pub struct TempSvgFile {
    /// Name within the owning directory capability, used by the operation modules.
    pub(crate) file_name: Utf8PathBuf,
    /// Directory capability and drop-time cleanup, used by the operation modules.
    pub(crate) cleanup: TempFileGuard,
}

impl TempSvgFile {
    /// Creates a uniquely named, UUID-suffixed SVG file path under the system temporary
    /// directory and prepares it for cleanup on drop.
    ///
    /// The file itself is not created on disc until it is written; only the owning directory
    /// capability and cleanup guard are established here.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the system temporary directory is not valid UTF-8, or if it cannot be
    /// opened as a cap-std directory capability.
    pub fn create(prefix: &str) -> TestSupportResult<Self> {
        let temp_dir = Utf8PathBuf::from_path_buf(std::env::temp_dir()).map_err(|path| {
            TestSupportError::expectation(format!(
                "temporary directory path is not valid UTF-8: {}",
                path.display()
            ))
        })?;
        let file_name = Utf8PathBuf::from(format!("{prefix}-{}.svg", Uuid::new_v4()));
        let path = temp_dir.join(&file_name);
        let dir = Dir::open_ambient_dir(&temp_dir, ambient_authority())
            .map_err(|error| TestSupportError::io("opening the temporary directory", error))?;
        let cleanup = TempFileGuard::new(dir, file_name.clone(), path);
        Ok(Self { file_name, cleanup })
    }

    /// Returns the full path of the temporary file, whether or not it has been written yet.
    pub fn path(&self) -> &Utf8Path {
        self.cleanup.path()
    }

    /// Consumes the temporary SVG and removes it through its directory capability.
    ///
    /// # Errors
    ///
    /// Returns `Err` if removing the temporary SVG fails. The inner guard's
    /// later drop-time cleanup is an idempotent best-effort fallback.
    pub fn cleanup(self) -> TestSupportResult<()> {
        self.cleanup.cleanup()
    }
}

#[cfg(test)]
mod tests {
    //! Tests that temporary SVG cleanup removes the wrapper's guarded file.

    use super::*;

    #[test]
    fn cleanup_removes_the_written_svg() -> TestSupportResult<()> {
        let temp_svg = TempSvgFile::create("gauss-test-temp-svg-cleanup")?;
        let file_name = temp_svg.file_name.clone();
        let dir = temp_svg
            .cleanup
            .dir
            .try_clone()
            .map_err(|error| TestSupportError::io("cloning the temporary directory", error))?;
        dir.write(file_name.as_path(), b"temporary")
            .map_err(|error| TestSupportError::io("writing the temporary SVG", error))?;

        temp_svg.cleanup()?;

        match dir.metadata(file_name.as_path()) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Ok(_) => Err(TestSupportError::expectation(
                "explicit cleanup left the temporary SVG on disc",
            )),
            Err(error) => Err(TestSupportError::io(
                "checking the explicitly cleaned temporary SVG",
                error,
            )),
        }
    }
}

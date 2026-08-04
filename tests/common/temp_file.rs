//! Temporary-file cleanup for GPUI integration tests.

use camino::Utf8PathBuf;
use cap_std::fs_utf8::Dir;
use test_support::{TestSupportError, TestSupportResult};

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

    /// Removes the guarded file through its directory capability.
    ///
    /// Calling this after the file has already been removed succeeds, allowing
    /// [`Drop`] to remain an idempotent best-effort fallback.
    ///
    /// # Errors
    ///
    /// Returns `Err` when removing the file fails for any reason other than the
    /// file already being absent.
    pub fn cleanup(&self) -> TestSupportResult<()> {
        match self.dir.remove_file(self.file_name.as_path()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(TestSupportError::io("removing the temporary file", error)),
        }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _cleanup = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    //! Tests explicit cleanup, error propagation, and the drop-time fallback.

    use cap_std::ambient_authority;
    use uuid::Uuid;

    use super::*;

    fn temp_guard(prefix: &str) -> TestSupportResult<TempFileGuard> {
        let temp_dir = Utf8PathBuf::from_path_buf(std::env::temp_dir()).map_err(|path| {
            TestSupportError::expectation(format!(
                "temporary directory path is not valid UTF-8: {}",
                path.display()
            ))
        })?;
        let dir = Dir::open_ambient_dir(&temp_dir, ambient_authority())
            .map_err(|error| TestSupportError::io("opening the temporary directory", error))?;
        let file_name = Utf8PathBuf::from(format!("{prefix}-{}", Uuid::new_v4()));
        Ok(TempFileGuard::new(dir, file_name))
    }

    #[test]
    fn cleanup_is_idempotent() -> TestSupportResult<()> {
        let guard = temp_guard("gauss-test-explicit-cleanup")?;
        guard
            .dir
            .write(guard.file_name.as_path(), b"temporary")
            .map_err(|error| TestSupportError::io("writing the temporary file", error))?;

        guard.cleanup()?;
        guard.cleanup()
    }

    #[test]
    fn cleanup_propagates_removal_errors() -> TestSupportResult<()> {
        let guard = temp_guard("gauss-test-cleanup-error")?;
        guard
            .dir
            .create_dir(guard.file_name.as_path())
            .map_err(|error| TestSupportError::io("creating the temporary directory", error))?;

        let cleanup_error = guard
            .cleanup()
            .expect_err("removing a directory as a file should fail");
        guard
            .dir
            .remove_dir(guard.file_name.as_path())
            .map_err(|error| TestSupportError::io("removing the temporary directory", error))?;

        if !matches!(cleanup_error, TestSupportError::Io { .. }) {
            return Err(TestSupportError::expectation(format!(
                "expected an I/O cleanup error, found {cleanup_error:?}"
            )));
        }
        Ok(())
    }

    #[test]
    fn drop_removes_the_guarded_file() -> TestSupportResult<()> {
        let guard = temp_guard("gauss-test-drop-cleanup")?;
        let file_name = guard.file_name.clone();
        let dir = guard
            .dir
            .try_clone()
            .map_err(|error| TestSupportError::io("cloning the temporary directory", error))?;
        guard
            .dir
            .write(file_name.as_path(), b"temporary")
            .map_err(|error| TestSupportError::io("writing the temporary file", error))?;
        drop(guard);

        match dir.metadata(file_name.as_path()) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Ok(_) => Err(TestSupportError::expectation(
                "drop left the guarded temporary file on disc",
            )),
            Err(error) => Err(TestSupportError::io(
                "checking the dropped temporary file",
                error,
            )),
        }
    }
}

//! Writing support for [`TempSvgFile`], compiled only where a scenario writes.

use test_support::{TestSupportError, TestSupportResult};

use crate::temp_svg::TempSvgFile;

impl TempSvgFile {
    /// Writes `contents` to the temporary file, creating or overwriting it, via the owning
    /// cap-std directory capability.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the underlying write to the capability-scoped directory fails.
    pub fn write(&self, contents: &str) -> TestSupportResult<()> {
        self.cleanup
            .dir()
            .write(self.file_name.as_path(), contents.as_bytes())
            .map_err(|error| TestSupportError::io("writing the temporary SVG", error))
    }
}

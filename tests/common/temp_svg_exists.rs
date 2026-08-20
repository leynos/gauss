//! Existence checking for [`TempSvgFile`], compiled only where a scenario asks.

use test_support::{TestSupportError, TestSupportResult};

use crate::temp_svg::TempSvgFile;

impl TempSvgFile {
    /// Reports whether the temporary file currently exists on disc.
    ///
    /// An absent file is a legitimate answer rather than a failure, so only that
    /// case yields `Ok(false)`. Every other metadata failure is reported, so a
    /// permission or capability fault cannot masquerade as "not written".
    ///
    /// # Errors
    ///
    /// Returns `Err` if the metadata lookup fails for any reason other than the
    /// file being absent.
    pub fn exists(&self) -> TestSupportResult<bool> {
        match self.cleanup.dir.metadata(self.file_name.as_path()) {
            Ok(_metadata) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(TestSupportError::io(
                "checking whether the temporary SVG exists",
                error,
            )),
        }
    }
}

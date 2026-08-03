//! Reading support for [`TempSvgFile`], compiled only where a scenario reads.

use test_support::{TestSupportError, TestSupportResult};

use crate::temp_svg::TempSvgFile;

impl TempSvgFile {
    /// Reads the temporary file's contents as a UTF-8 string via the owning cap-std directory
    /// capability.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the file cannot be read, for example because it has not been written
    /// yet or its contents are not valid UTF-8.
    pub fn read_to_string(&self) -> TestSupportResult<String> {
        self.cleanup
            .dir()
            .read_to_string(self.file_name.as_path())
            .map_err(|error| TestSupportError::io("reading the temporary SVG", error))
    }
}

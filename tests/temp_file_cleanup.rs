//! Property coverage for temporary-file cleanup state transitions.

#[path = "common/temp_file.rs"]
mod temp_file;

use std::fmt::Display;

use proptest::{prelude::*, test_runner::TestCaseError};
use temp_file::temp_guard;
use test_support::TestSupportError;

fn test_case<T, E: Display>(result: Result<T, E>) -> Result<T, TestCaseError> {
    result.map_err(|error| TestCaseError::fail(error.to_string()))
}

proptest! {
    #[test]
    fn cleanup_remains_successful_after_repeated_removal(repetitions in 1usize..8) {
        let guard = test_case(temp_guard("gauss-test-property-present"))?;
        prop_assert_eq!(guard.path().file_name(), guard.file_name.file_name());
        test_case(guard.dir.write(guard.file_name.as_path(), b"temporary"))?;

        for _ in 0..repetitions {
            test_case(guard.cleanup())?;
        }

        let is_absent = matches!(
            guard.dir.metadata(guard.file_name.as_path()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound
        );
        prop_assert!(is_absent);
    }

    #[test]
    fn cleanup_accepts_absent_files(repetitions in 1usize..8) {
        let guard = test_case(temp_guard("gauss-test-property-absent"))?;

        for _ in 0..repetitions {
            test_case(guard.cleanup())?;
        }
    }

    #[test]
    fn cleanup_reports_repeated_directory_removal_errors(repetitions in 1usize..8) {
        let guard = test_case(temp_guard("gauss-test-property-directory"))?;
        test_case(guard.dir.create_dir(guard.file_name.as_path()))?;

        let all_are_io_errors = (0..repetitions).all(|_| {
            matches!(guard.cleanup(), Err(TestSupportError::Io { .. }))
        });
        test_case(guard.dir.remove_dir(guard.file_name.as_path()))?;

        prop_assert!(all_are_io_errors);
    }
}

//! Shared durable-handle and temporary-file support for GPUI file I/O scenarios.
//!
//! Only the `gpui_file_io_*` scenario binaries include this module, via
//! `#[path = "common/file_io.rs"] mod file_io;`, so the helpers here stay out of
//! the general `common` surface that every GPUI integration test compiles.
#![expect(
    dead_code,
    reason = "each of the four scenario binaries drives a different subset; tracked by issue #150"
)]

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
use gauss::ui::Phase0Shell;
use gpui::{AnyWindowHandle, Entity, TestAppContext, VisualContext, VisualTestContext};
use test_support::{TestSupportError, TestSupportResult};
use uuid::Uuid;

use crate::common::TempFileGuard;

/// Assert that no file path prompt has been displayed.
///
/// `context` becomes the expectation failure message, so each scenario binary
/// keeps its own wording while sharing the check.
///
/// # Errors
///
/// Returns an expectation error when a prompt has in fact been displayed.
pub fn assert_no_path_prompt(cx: &mut TestAppContext, context: &str) -> TestSupportResult<()> {
    if cx.did_prompt_for_new_path() {
        return Err(TestSupportError::expectation(context.to_owned()));
    }
    Ok(())
}

/// Assert that a file path prompt has been displayed.
///
/// `context` becomes the expectation failure message, so each scenario binary
/// keeps its own wording while sharing the check.
///
/// # Errors
///
/// Returns an expectation error when no prompt has been displayed.
pub fn assert_path_prompt(cx: &mut TestAppContext, context: &str) -> TestSupportResult<()> {
    if !cx.did_prompt_for_new_path() {
        return Err(TestSupportError::expectation(context.to_owned()));
    }
    Ok(())
}

/// Durable GPUI handles that can safely survive between BDD steps.
#[derive(Clone)]
pub struct DurableShell {
    entity: Entity<Phase0Shell>,
    window: AnyWindowHandle,
}

impl DurableShell {
    /// Captures a durable shell handle from a live `VisualTestContext`.
    ///
    /// The entity and the context's window handle are stored so the pair can outlive the
    /// `VisualTestContext` itself, allowing scenario state to persist across BDD steps.
    pub fn new(entity: Entity<Phase0Shell>, visual_cx: &VisualTestContext) -> Self {
        Self {
            entity,
            window: visual_cx.window_handle(),
        }
    }

    /// Reconstructs a `VisualTestContext` from the stored window handle and runs `f` against it.
    ///
    /// The reconstructed context is scoped to the closure; it is not retained afterwards.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `f` returns `Err`.
    pub fn with_visual_cx<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<R>,
    ) -> TestSupportResult<R> {
        let mut visual_cx = VisualTestContext::from_window(self.window, cx);
        f(&mut visual_cx, &self.entity)
    }

    /// Returns a reference to the durably held shell entity.
    pub const fn entity(&self) -> &Entity<Phase0Shell> {
        &self.entity
    }
}

/// A uniquely named temporary SVG whose lifetime is owned by the scenario.
pub struct TempSvgFile {
    path: Utf8PathBuf,
    file_name: Utf8PathBuf,
    cleanup: TempFileGuard,
}

impl TempSvgFile {
    /// Creates a uniquely named, UUID-suffixed SVG file path under the system temporary
    /// directory and prepares it for cleanup on drop.
    ///
    /// The file itself is not created on disk until [`Self::write`] is called; only the
    /// owning directory capability and cleanup guard are established here.
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
        let cleanup = TempFileGuard::new(dir, file_name.clone());
        Ok(Self {
            path,
            file_name,
            cleanup,
        })
    }

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
        match self.cleanup.dir().metadata(self.file_name.as_path()) {
            Ok(_metadata) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(TestSupportError::io(
                "checking whether the temporary SVG exists",
                error,
            )),
        }
    }

    /// Returns the full path of the temporary file, whether or not it has been written yet.
    pub fn path(&self) -> &Utf8Path {
        self.path.as_path()
    }
}

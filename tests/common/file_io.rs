//! Shared durable-handle and temporary-file support for GPUI file I/O scenarios.

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
use gauss::ui::Phase0Shell;
use gpui::{AnyWindowHandle, Entity, TestAppContext, VisualContext, VisualTestContext};
use test_support::{TestSupportError, TestSupportResult};
use uuid::Uuid;

use super::TempFileGuard;

/// Durable GPUI handles that can safely survive between BDD steps.
#[derive(Clone)]
pub struct DurableShell {
    entity: Entity<Phase0Shell>,
    window: AnyWindowHandle,
}

impl DurableShell {
    pub fn new(entity: Entity<Phase0Shell>, visual_cx: &VisualTestContext) -> Self {
        Self {
            entity,
            window: visual_cx.window_handle(),
        }
    }

    pub fn with_visual_cx<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<R>,
    ) -> TestSupportResult<R> {
        let mut visual_cx = VisualTestContext::from_window(self.window, cx);
        f(&mut visual_cx, &self.entity)
    }

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

    pub fn write(&self, contents: &str) -> TestSupportResult<()> {
        self.cleanup
            .dir()
            .write(self.file_name.as_path(), contents.as_bytes())
            .map_err(|error| TestSupportError::io("writing the temporary SVG", error))
    }

    pub fn read_to_string(&self) -> TestSupportResult<String> {
        self.cleanup
            .dir()
            .read_to_string(self.file_name.as_path())
            .map_err(|error| TestSupportError::io("reading the temporary SVG", error))
    }

    pub fn exists(&self) -> bool {
        self.cleanup
            .dir()
            .metadata(self.file_name.as_path())
            .is_ok()
    }

    pub fn path(&self) -> &Utf8Path {
        self.path.as_path()
    }
}

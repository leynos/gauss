//! Installation state for one resolved runner row.
//!
//! A suite step may rely only on an installer that has already completed in the
//! same row, so the state that tracks it and the inspection that advances it
//! live together rather than as parallel locals.

use super::{Host, Route, Value, Workspace, installs_mold, require_suite_binding, step_route};

/// Installation state that belongs to one resolved runner row, never another job.
#[derive(Clone, Copy)]
pub(super) struct Provisioning {
    /// The statically resolved operating system; this does not prove architecture.
    pub(super) host: Host,
    /// Whether an unconditional current installer has already completed in source order.
    pub(super) installed: bool,
}

/// One runner row under inspection, together with what its earlier steps proved.
pub(super) struct RowInspection<'a, 'b> {
    /// The job the row belongs to.
    job: &'a Value,
    /// The workspace the row's steps are resolved against.
    workspace: &'a Workspace<'b>,
    /// What the row's earlier steps established.
    state: Provisioning,
}

impl<'a, 'b> RowInspection<'a, 'b> {
    /// Starts a row that has installed nothing yet.
    pub(super) const fn new(job: &'a Value, workspace: &'a Workspace<'b>, host: Host) -> Self {
        Self {
            job,
            workspace,
            state: Provisioning {
                host,
                installed: false,
            },
        }
    }

    /// Inspects one step in source order, recording any installer it runs.
    pub(super) fn step(&mut self, step: &Value) -> Result<Route, String> {
        self.state.installed |= installs_mold(step)?;
        let observed = step_route(step, self.workspace, self.state.host)?;
        if observed == Route::Suite {
            require_suite_binding(step, self.job, self.state)?;
        }
        Ok(observed)
    }
}

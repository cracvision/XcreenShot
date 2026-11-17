//! Starting with Windows in the Microsoft Store edition (APP-003): the package's startup
//! task (declared in its manifest), which the person also sees and can turn off in Task
//! Manager. The other editions use the login item.

/// The task's id in the package manifest (`scripts/make-msix.mjs`).
#[cfg(windows)]
const TASK: &str = "XcreenShotStartup";

/// Turns the startup task on or off; its state after (Windows may refuse: turned off
/// by the person in Task Manager or by policy).
#[cfg(windows)]
pub fn set(enabled: bool) -> Result<bool, String> {
    use windows::ApplicationModel::{StartupTask, StartupTaskState};
    let task = StartupTask::GetAsync(&windows::core::HSTRING::from(TASK))
        .and_then(|op| op.join())
        .map_err(|e| e.message().to_string())?;
    if enabled {
        let state = task
            .RequestEnableAsync()
            .and_then(|op| op.join())
            .map_err(|e| e.message().to_string())?;
        Ok(matches!(
            state,
            StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy
        ))
    } else {
        task.Disable().map_err(|e| e.message().to_string())?;
        Ok(false)
    }
}

/// Whether the startup task is on.
#[cfg(windows)]
pub fn enabled() -> Option<bool> {
    use windows::ApplicationModel::{StartupTask, StartupTaskState};
    let task = StartupTask::GetAsync(&windows::core::HSTRING::from(TASK))
        .and_then(|op| op.join())
        .ok()?;
    let state = task.State().ok()?;
    Some(matches!(
        state,
        StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy
    ))
}

#[cfg(not(windows))]
pub fn set(_enabled: bool) -> Result<bool, String> {
    Err("only in the Microsoft Store edition".into())
}

#[cfg(not(windows))]
pub fn enabled() -> Option<bool> {
    None
}

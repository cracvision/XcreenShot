//! XcreenShot core: the command registry and everything shared by the desktop
//! shell, the CLI and (from M2) the MCP server.
//!
//! Implements M0 foundations: APP-017–APP-021 (shortcuts), APP-027 (command
//! registry), APP-028 (CLI protocol), APP-032 (automation safety), APP-046 (logs).
//! M1 adds the editable document model and its `.xshot` container (EDT-106).

pub mod catalog;
pub mod command;
pub mod core;
pub mod document;
pub mod error;
pub mod events;
pub mod host;
pub mod i18n;
pub mod ipc;
pub mod logging;
pub mod menus;
pub mod paths;
pub mod platform;
pub mod schema;
pub mod settings;
pub mod settings_io;
pub mod shortcuts;
#[cfg(windows)]
pub mod winreg;

pub use crate::command::{CommandSpec, Context, Registry, Scope, Source};
pub use crate::core::Core;
pub use crate::error::{CommandError, CommandResult, ErrorCode};
pub use crate::i18n::Lang;

/// The system clipboard is used from several threads (paste, copy, captures, menus),
/// and its library is not safe for that: two uses at once corrupted the process's
/// memory. Every use takes this lock first.
pub fn clipboard_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

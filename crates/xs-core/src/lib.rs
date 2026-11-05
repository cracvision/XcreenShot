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
pub mod host;
pub mod i18n;
pub mod ipc;
pub mod logging;
pub mod menus;
pub mod paths;
pub mod platform;
pub mod schema;
pub mod settings;
pub mod shortcuts;
#[cfg(windows)]
pub mod winreg;

pub use crate::command::{CommandSpec, Context, Registry, Scope, Source};
pub use crate::core::Core;
pub use crate::error::{CommandError, CommandResult, ErrorCode};
pub use crate::i18n::Lang;

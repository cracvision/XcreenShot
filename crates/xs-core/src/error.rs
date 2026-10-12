//! Structured command errors (APP-027).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Stable, machine-readable error codes. Never translated (APP-013).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidParams,
    UnknownCommand,
    NotFound,
    Busy,
    Cancelled,
    PermissionDenied,
    DisabledByPolicy,
    NotSupportedOnPlatform,
    Timeout,
    NotRunning,
    Internal,
}

impl ErrorCode {
    /// CLI exit code for this error (APP-028).
    pub fn exit_code(self) -> i32 {
        match self {
            ErrorCode::Internal => 1,
            ErrorCode::InvalidParams | ErrorCode::UnknownCommand => 2,
            ErrorCode::NotRunning => 3,
            ErrorCode::Cancelled => 4,
            ErrorCode::Busy => 5,
            ErrorCode::PermissionDenied | ErrorCode::DisabledByPolicy | ErrorCode::NotSupportedOnPlatform => {
                6
            }
            ErrorCode::NotFound => 7,
            ErrorCode::Timeout => 8,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::InvalidParams => "invalid_params",
            ErrorCode::UnknownCommand => "unknown_command",
            ErrorCode::NotFound => "not_found",
            ErrorCode::Busy => "busy",
            ErrorCode::Cancelled => "cancelled",
            ErrorCode::PermissionDenied => "permission_denied",
            ErrorCode::DisabledByPolicy => "disabled_by_policy",
            ErrorCode::NotSupportedOnPlatform => "not_supported_on_platform",
            ErrorCode::Timeout => "timeout",
            ErrorCode::NotRunning => "not_running",
            ErrorCode::Internal => "internal",
        }
    }
}

/// Error returned by a command: `code` is stable English, `message` is in the UI language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, thiserror::Error)]
#[error("{code:?}: {message}")]
pub struct CommandError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl CommandError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }
}

pub type CommandResult = Result<Value, CommandError>;

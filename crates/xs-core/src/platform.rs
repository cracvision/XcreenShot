//! Platform identification.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Macos,
}

impl Platform {
    pub fn current() -> Platform {
        if cfg!(target_os = "macos") {
            Platform::Macos
        } else {
            Platform::Windows
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Platform::Windows => "windows",
            Platform::Macos => "macos",
        }
    }
}

pub fn arch() -> &'static str {
    std::env::consts::ARCH
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Short commit of this build, with "+" if it had uncommitted changes.
pub const BUILD: &str = env!("XCREENSHOT_BUILD");

/// "0.2.0 (670038c)": what the user sees in the app, diagnostics and `--version`.
pub const VERSION_FULL: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("XCREENSHOT_BUILD"), ")");

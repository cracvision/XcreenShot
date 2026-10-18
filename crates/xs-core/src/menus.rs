//! Menus as data. Every actionable entry names a registered command (APP-027), and
//! the shell builds the native menus from these definitions.

use serde_json::{json, Value};

use crate::platform::Platform;

#[derive(Debug, Clone)]
pub enum MenuEntry {
    /// Runs `command` with `params`. `checked` names a state the shell reads to tick it.
    Command {
        id: &'static str,
        label: &'static str,
        command: &'static str,
        params: Value,
        checked: Option<&'static str>,
    },
    Submenu {
        label: &'static str,
        items: Vec<MenuEntry>,
    },
    /// Disabled informational row (e.g. "No captures yet").
    Placeholder {
        label: &'static str,
    },
    /// OS-provided item (copy, paste…), not a XcreenShot command.
    Native(NativeItem),
    Separator,
}

#[derive(Debug, Clone, Copy)]
pub enum NativeItem {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Minimize,
    Hide,
    HideOthers,
    ShowAll,
}

fn cmd(id: &'static str, label: &'static str, command: &'static str) -> MenuEntry {
    MenuEntry::Command {
        id,
        label,
        command,
        params: json!({}),
        checked: None,
    }
}

/// Quick menu of the XcreenShot icon (APP-002) with its capture entries (CAP-005).
/// The status block, video, quick annotation, presets and Recent items arrive with
/// their features.
pub fn tray_menu() -> Vec<MenuEntry> {
    vec![
        cmd("capture_universal", "tray.capture_universal", "capture.universal"),
        cmd("capture_image", "tray.capture_image", "capture.image"),
        cmd(
            "capture_repeat_last",
            "tray.capture_repeat_last",
            "capture.repeat_last",
        ),
        MenuEntry::Command {
            id: "capture_panel",
            label: "tray.capture_panel",
            command: "capture.panel",
            params: json!({ "show": true }),
            checked: None,
        },
        MenuEntry::Separator,
        MenuEntry::Submenu {
            label: "tray.recent",
            items: vec![MenuEntry::Placeholder {
                label: "tray.recent_empty",
            }],
        },
        cmd("open_editor", "tray.open_editor", "app.show"),
        MenuEntry::Separator,
        MenuEntry::Command {
            id: "pause_shortcuts",
            label: "tray.pause_shortcuts",
            command: "shortcuts.pause",
            params: json!({}),
            checked: Some("shortcuts_paused"),
        },
        MenuEntry::Separator,
        cmd("preferences", "tray.preferences", "settings.open"),
        MenuEntry::Submenu {
            label: "tray.help",
            items: help_items(),
        },
        MenuEntry::Separator,
        cmd("quit", "tray.quit", "app.quit"),
    ]
}

fn help_items() -> Vec<MenuEntry> {
    vec![
        cmd("open_logs", "menu.open_logs", "app.open_logs"),
        MenuEntry::Command {
            id: "copy_diagnostics",
            label: "menu.copy_diagnostics",
            command: "app.diagnostics",
            params: json!({ "copy": true }),
            checked: None,
        },
    ]
}

/// Application menu: the macOS menu bar, or the main window menu on Windows.
pub fn app_menu(platform: Platform) -> Vec<MenuEntry> {
    match platform {
        Platform::Macos => vec![
            MenuEntry::Submenu {
                label: "app.name",
                items: vec![
                    cmd("app_settings", "menu.settings_mac", "settings.open"),
                    MenuEntry::Separator,
                    MenuEntry::Native(NativeItem::Hide),
                    MenuEntry::Native(NativeItem::HideOthers),
                    MenuEntry::Native(NativeItem::ShowAll),
                    MenuEntry::Separator,
                    cmd("app_quit", "menu.quit", "app.quit"),
                ],
            },
            MenuEntry::Submenu {
                label: "menu.edit",
                items: vec![
                    MenuEntry::Native(NativeItem::Undo),
                    MenuEntry::Native(NativeItem::Redo),
                    MenuEntry::Separator,
                    MenuEntry::Native(NativeItem::Cut),
                    MenuEntry::Native(NativeItem::Copy),
                    MenuEntry::Native(NativeItem::Paste),
                    MenuEntry::Native(NativeItem::SelectAll),
                ],
            },
            MenuEntry::Submenu {
                label: "menu.window",
                items: vec![
                    MenuEntry::Native(NativeItem::Minimize),
                    cmd("window_close", "menu.close_window", "app.close_window"),
                ],
            },
            MenuEntry::Submenu {
                label: "menu.help",
                items: help_items(),
            },
        ],
        Platform::Windows => vec![
            MenuEntry::Submenu {
                label: "menu.file",
                items: vec![
                    cmd("file_preferences", "menu.preferences", "settings.open"),
                    MenuEntry::Separator,
                    cmd("file_close", "menu.close_window", "app.close_window"),
                    cmd("file_exit", "menu.quit", "app.quit"),
                ],
            },
            MenuEntry::Submenu {
                label: "menu.help",
                items: help_items(),
            },
        ],
    }
}

/// Every command referenced by a menu, recursively.
pub fn commands_in(entries: &[MenuEntry]) -> Vec<&'static str> {
    let mut out = Vec::new();
    for e in entries {
        match e {
            MenuEntry::Command { command, .. } => out.push(*command),
            MenuEntry::Submenu { items, .. } => out.extend(commands_in(items)),
            _ => {}
        }
    }
    out
}

/// Every translation key used by a menu, recursively.
pub fn labels_in(entries: &[MenuEntry]) -> Vec<&'static str> {
    let mut out = Vec::new();
    for e in entries {
        match e {
            MenuEntry::Command { label, .. } | MenuEntry::Placeholder { label } => out.push(*label),
            MenuEntry::Submenu { label, items } => {
                out.push(*label);
                out.extend(labels_in(items));
            }
            _ => {}
        }
    }
    out
}

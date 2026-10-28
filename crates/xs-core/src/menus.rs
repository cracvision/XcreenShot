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

/// An entry of the editor menus. Its id is the row of Preferences > Shortcuts whose
/// shortcut it shows, when it has one (EDT-002, EDT-116).
fn action(id: &'static str, label: &'static str, action: &str) -> MenuEntry {
    MenuEntry::Command {
        id,
        label,
        command: "editor.action",
        params: json!({ "action": action }),
        checked: None,
    }
}

fn zoom(id: &'static str, label: &'static str, params: Value) -> MenuEntry {
    MenuEntry::Command {
        id,
        label,
        command: "editor.zoom",
        params,
        checked: None,
    }
}

/// The editor menus (EDT-002), with only what works in this build. Image and Object
/// arrive with their first entries (M2). The window keeps each entry's enabled and
/// ticked state up to date (`editor.menu_state`).
fn editor_menus() -> Vec<MenuEntry> {
    use MenuEntry::{Separator, Submenu};
    vec![
        Submenu {
            label: "menu.file",
            items: vec![
                action("editor.open", "menu.open", "open"),
                Separator,
                action("editor.save", "menu.save", "save"),
                action("editor.save_as", "menu.save_as", "save_as"),
                action("editor.export", "menu.export", "export"),
                Separator,
                cmd("file_preferences", "menu.preferences", "settings.open"),
                Separator,
                action("editor.close", "menu.close", "close"),
                action("editor.close_all", "menu.close_all", "close_all"),
                Separator,
                cmd("file_exit", "menu.quit", "app.quit"),
            ],
        },
        Submenu {
            label: "menu.edit",
            items: vec![
                action("editor.undo", "menu.undo", "undo"),
                action("editor.redo", "menu.redo", "redo"),
                Separator,
                action("editor.cut", "menu.cut", "cut"),
                action("editor.copy", "menu.copy", "copy"),
                action("editor.copy_all", "menu.copy_all", "copy_all"),
                action("editor.paste", "menu.paste", "paste"),
                action("editor.paste_in_place", "menu.paste_in_place", "paste_in_place"),
                Separator,
                action("editor.delete", "menu.delete", "delete"),
                action("editor.deselect", "menu.deselect", "deselect"),
            ],
        },
        Submenu {
            label: "menu.view",
            items: vec![
                zoom("editor.zoom_in", "menu.zoom_in", json!({ "mode": "in" })),
                zoom("editor.zoom_out", "menu.zoom_out", json!({ "mode": "out" })),
                zoom(
                    "editor.zoom_actual",
                    "menu.zoom_actual",
                    json!({ "mode": "percent", "value": 100 }),
                ),
                zoom("editor.zoom_fit", "menu.zoom_fit", json!({ "mode": "fit" })),
                zoom(
                    "editor.zoom_original",
                    "menu.zoom_original",
                    json!({ "mode": "original" }),
                ),
                Separator,
                MenuEntry::Command {
                    id: "editor.side_panel",
                    label: "menu.side_panel",
                    command: "editor.action",
                    params: json!({ "action": "side_panel" }),
                    checked: Some("editor.side_panel"),
                },
                Submenu {
                    label: "menu.toolbar",
                    items: vec![MenuEntry::Command {
                        id: "editor.toolbar_labels",
                        label: "menu.toolbar_labels",
                        command: "editor.action",
                        params: json!({ "action": "toolbar_labels" }),
                        checked: Some("editor.toolbar_labels"),
                    }],
                },
            ],
        },
        Submenu {
            label: "menu.help",
            items: {
                let mut items = vec![action("editor.recover", "menu.recover", "recover"), Separator];
                items.extend(help_items());
                items
            },
        },
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
        // The main window is the editor: its menu bar, under the title bar (EDT-002).
        Platform::Windows => editor_menus(),
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

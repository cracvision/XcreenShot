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
        cmd("capture_multi", "tray.capture_multi", "capture.multi_start"),
        MenuEntry::Separator,
        MenuEntry::Submenu {
            label: "tray.recent",
            items: vec![MenuEntry::Placeholder {
                label: "tray.recent_empty",
            }],
        },
        cmd("open_editor", "tray.open_editor", "app.show"),
        MenuEntry::Command {
            id: "open_library",
            label: "tray.open_library",
            command: "editor.show",
            params: json!({ "view": "library" }),
            checked: None,
        },
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
                // Create (EDT-095, EDT-103, EDT-104).
                action(
                    "editor.new_from_template",
                    "menu.new_from_template",
                    "new_from_template",
                ),
                action("editor.new_image", "menu.new_image", "new_image"),
                action(
                    "editor.new_from_clipboard",
                    "menu.new_from_clipboard",
                    "new_from_clipboard",
                ),
                action("editor.apply_template", "menu.apply_template", "apply_template"),
                Separator,
                action("editor.open", "menu.open", "open"),
                Separator,
                action("editor.save", "menu.save", "save"),
                action("editor.save_as", "menu.save_as", "save_as"),
                action(
                    "editor.save_as_template",
                    "menu.save_as_template",
                    "save_as_template",
                ),
                action("editor.export", "menu.export", "export"),
                action("editor.batch_export", "menu.batch_export", "batch_export"),
                action("editor.print", "menu.print", "print"),
                Separator,
                action("editor.rename_item", "menu.rename_item", "rename_item"),
                action("editor.show_in_folder", "menu.show_in_folder", "show_in_folder"),
                action(
                    "editor.show_in_library",
                    "menu.show_in_library",
                    "show_in_library",
                ),
                action("editor.delete_item", "menu.delete_item", "delete_item"),
                Separator,
                // A theme from a .xtheme file (EDT-018).
                Submenu {
                    label: "menu.import",
                    items: vec![
                        action("editor.import_theme", "menu.import_theme", "import_theme"),
                        action(
                            "editor.import_template",
                            "menu.import_template",
                            "import_template",
                        ),
                    ],
                },
                Separator,
                cmd("file_preferences", "menu.preferences", "settings.open"),
                Separator,
                action("editor.close", "menu.close", "close"),
                action("editor.close_others", "menu.close_others", "close_others"),
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
                action("editor.duplicate", "menu.duplicate", "duplicate"),
                action("editor.select_all", "menu.select_all", "select_all"),
                action("editor.deselect", "menu.deselect", "deselect"),
                Separator,
                // Pixel selection (EDT-044).
                action(
                    "editor.invert_selection",
                    "menu.invert_selection",
                    "invert_selection",
                ),
                action(
                    "editor.float_selection",
                    "menu.float_selection",
                    "float_selection",
                ),
                Separator,
                // EDT-076: undoable, after a confirmation.
                action("editor.revert", "menu.revert", "revert"),
            ],
        },
        // The image as a whole (EDT-069 to EDT-077).
        Submenu {
            label: "menu.image",
            items: vec![
                action("editor.resize_image", "menu.resize_image", "resize_image"),
                action("editor.canvas_size", "menu.canvas_size", "canvas_size"),
                action("editor.canvas_color", "menu.canvas_color", "canvas_color"),
                Separator,
                Submenu {
                    label: "menu.rotate",
                    items: vec![
                        action("editor.rotate_right", "menu.rotate_right", "rotate_right"),
                        action("editor.rotate_left", "menu.rotate_left", "rotate_left"),
                        action("editor.rotate_180", "menu.rotate_180", "rotate_180"),
                        Separator,
                        action("editor.rotate_custom", "menu.rotate_custom", "rotate_custom"),
                    ],
                },
                Submenu {
                    label: "menu.flip",
                    items: vec![
                        action(
                            "editor.flip_horizontal",
                            "menu.flip_horizontal",
                            "flip_horizontal",
                        ),
                        action("editor.flip_vertical", "menu.flip_vertical", "flip_vertical"),
                    ],
                },
                Separator,
                action("editor.crop_selection", "menu.crop_selection", "crop_selection"),
                // EDT-051.
                action("editor.crop_to_canvas", "menu.crop_to_canvas", "crop_to_canvas"),
                action("editor.fit_canvas", "menu.fit_canvas", "fit_canvas"),
                action("editor.trim", "menu.trim", "trim"),
                Separator,
                action("editor.corners", "menu.corners", "corners"),
                action("editor.resolution", "menu.resolution", "resolution"),
                Separator,
                Submenu {
                    label: "menu.effects",
                    items: vec![
                        action(
                            "editor.effect_border",
                            "ui.editor.fx.border.name",
                            "effect_border",
                        ),
                        action("editor.effect_edges", "ui.editor.fx.edges.name", "effect_edges"),
                        action(
                            "editor.effect_shadow",
                            "ui.editor.fx.shadow.name",
                            "effect_shadow",
                        ),
                        action(
                            "editor.effect_perspective",
                            "ui.editor.fx.perspective.name",
                            "effect_perspective",
                        ),
                        action(
                            "editor.effect_page_curl",
                            "ui.editor.fx.pageCurl.name",
                            "effect_page_curl",
                        ),
                        action(
                            "editor.effect_reflection",
                            "ui.editor.fx.reflection.name",
                            "effect_reflection",
                        ),
                        action("editor.effect_fade", "ui.editor.fx.fade.name", "effect_fade"),
                        action(
                            "editor.effect_filters",
                            "ui.editor.fx.filters.name",
                            "effect_filters",
                        ),
                        action("editor.effect_color", "ui.editor.fx.color.name", "effect_color"),
                        action(
                            "editor.effect_replace",
                            "ui.editor.fx.replace.name",
                            "effect_replace",
                        ),
                        action(
                            "editor.effect_watermark",
                            "ui.editor.fx.watermark.name",
                            "effect_watermark",
                        ),
                        action(
                            "editor.effect_details",
                            "ui.editor.fx.details.name",
                            "effect_details",
                        ),
                        action("editor.effect_trim", "ui.editor.fx.trim.name", "effect_trim"),
                        action(
                            "editor.effect_corners",
                            "ui.editor.fx.corners.name",
                            "effect_corners",
                        ),
                        action("editor.effect_scale", "ui.editor.fx.scale.name", "effect_scale"),
                        action(
                            "editor.effect_rotate",
                            "ui.editor.fx.rotate.name",
                            "effect_rotate",
                        ),
                    ],
                },
                Separator,
                // A tick: Image > Snapping (EDT-084).
                MenuEntry::Command {
                    id: "editor.snap",
                    label: "menu.snap",
                    command: "editor.action",
                    params: json!({ "action": "snap" }),
                    checked: Some("editor.snap"),
                },
            ],
        },
        Submenu {
            label: "menu.view",
            items: vec![
                action("editor.view_editor", "menu.view_editor", "view_editor"),
                action("editor.view_library", "menu.view_library", "view_library"),
                Separator,
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
                // A tick (EDT-008).
                MenuEntry::Command {
                    id: "editor.pixel_grid",
                    label: "menu.pixel_grid",
                    command: "editor.action",
                    params: json!({ "action": "pixel_grid" }),
                    checked: Some("editor.pixel_grid"),
                },
                Separator,
                MenuEntry::Command {
                    id: "editor.side_panel",
                    label: "menu.side_panel",
                    command: "editor.action",
                    params: json!({ "action": "side_panel" }),
                    checked: Some("editor.side_panel"),
                },
                MenuEntry::Command {
                    id: "editor.tray",
                    label: "menu.tray",
                    command: "editor.action",
                    params: json!({ "action": "tray" }),
                    checked: Some("editor.tray"),
                },
                action("editor.prev_item", "menu.prev_item", "prev_item"),
                action("editor.next_item", "menu.next_item", "next_item"),
                // EDT-009.
                MenuEntry::Command {
                    id: "editor.fullscreen",
                    label: "menu.fullscreen",
                    command: "editor.action",
                    params: json!({ "action": "fullscreen" }),
                    checked: Some("editor.fullscreen"),
                },
                Submenu {
                    label: "menu.toolbar",
                    items: vec![
                        action(
                            "editor.customize_toolbar",
                            "menu.customize_toolbar",
                            "customize_toolbar",
                        ),
                        MenuEntry::Command {
                            id: "editor.toolbar_labels",
                            label: "menu.toolbar_labels",
                            command: "editor.action",
                            params: json!({ "action": "toolbar_labels" }),
                            checked: Some("editor.toolbar_labels"),
                        },
                    ],
                },
            ],
        },
        // Objects (EDT-081): their stacking order; grouping and locking arrive with them.
        Submenu {
            label: "menu.object",
            items: vec![
                action("editor.bring_front", "menu.bring_front", "bring_front"),
                action("editor.bring_forward", "menu.bring_forward", "bring_forward"),
                action("editor.send_backward", "menu.send_backward", "send_backward"),
                action("editor.send_back", "menu.send_back", "send_back"),
                Separator,
                action("editor.flatten", "menu.flatten", "flatten"),
                action("editor.flatten_all", "menu.flatten_all", "flatten_all"),
                action("editor.unflatten_all", "menu.unflatten_all", "unflatten_all"),
                Separator,
                action("editor.group", "menu.group", "group"),
                action("editor.ungroup", "menu.ungroup", "ungroup"),
                Separator,
                action("editor.lock", "menu.lock", "lock"),
                action("editor.unlock_all", "menu.unlock_all", "unlock_all"),
                Separator,
                // Only the selected objects (EDT-073).
                Submenu {
                    label: "menu.flip",
                    items: vec![
                        action(
                            "editor.flip_objects_horizontal",
                            "menu.flip_horizontal",
                            "flip_objects_horizontal",
                        ),
                        action(
                            "editor.flip_objects_vertical",
                            "menu.flip_vertical",
                            "flip_objects_vertical",
                        ),
                    ],
                },
                Separator,
                Submenu {
                    label: "menu.align",
                    items: vec![
                        action("editor.align_left", "menu.align_left", "align_left"),
                        action("editor.align_hcenter", "menu.align_hcenter", "align_hcenter"),
                        action("editor.align_right", "menu.align_right", "align_right"),
                        Separator,
                        action("editor.align_top", "menu.align_top", "align_top"),
                        action("editor.align_vcenter", "menu.align_vcenter", "align_vcenter"),
                        action("editor.align_bottom", "menu.align_bottom", "align_bottom"),
                    ],
                },
                Submenu {
                    label: "menu.distribute",
                    items: vec![
                        action("editor.distribute_h", "menu.distribute_h", "distribute_h"),
                        action("editor.distribute_v", "menu.distribute_v", "distribute_v"),
                        Separator,
                        action("editor.equal_width", "menu.equal_width", "equal_width"),
                        action("editor.equal_height", "menu.equal_height", "equal_height"),
                    ],
                },
            ],
        },
        // Cloud AI (SMT-040, SMT-041): always listed; without it set up they explain
        // what is missing.
        Submenu {
            label: "menu.ai",
            items: vec![
                action("editor.ai_ask", "menu.ai_ask", "ai_ask"),
                action("editor.ai_name", "menu.ai_name", "ai_name"),
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

/// The Help menu (APP-015): the help pages, the logs, feedback and About.
fn help_items() -> Vec<MenuEntry> {
    vec![
        cmd("app.help", "menu.help_contents", "app.help"),
        MenuEntry::Separator,
        cmd("open_logs", "menu.open_logs", "app.open_logs"),
        MenuEntry::Command {
            id: "copy_diagnostics",
            label: "menu.copy_diagnostics",
            command: "app.diagnostics",
            params: json!({ "copy": true }),
            checked: None,
        },
        MenuEntry::Separator,
        cmd("welcome", "menu.welcome", "app.welcome"),
        cmd("feedback", "menu.feedback", "app.feedback"),
        MenuEntry::Command {
            id: "about",
            label: "menu.about",
            command: "app.about",
            params: json!({ "open": true }),
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

/// Menus as data for a window that draws them itself: the editor's menu bar on Windows
/// (EDT-002), which animates like the rest of the app (APP-012). Labels stay as keys;
/// items name their command and parameters, and the state that ticks them.
pub fn to_json(entries: &[MenuEntry]) -> Value {
    Value::Array(
        entries
            .iter()
            .filter_map(|e| match e {
                MenuEntry::Command {
                    id,
                    label,
                    command,
                    params,
                    checked,
                } => Some(json!({
                    "kind": "command",
                    "id": id,
                    "label": label,
                    "command": command,
                    "params": params,
                    "checked": checked,
                })),
                MenuEntry::Submenu { label, items } => {
                    Some(json!({ "kind": "submenu", "label": label, "items": to_json(items) }))
                }
                MenuEntry::Placeholder { label } => Some(json!({ "kind": "placeholder", "label": label })),
                MenuEntry::Separator => Some(json!({ "kind": "separator" })),
                // System items (macOS only) have no place in a drawn menu.
                MenuEntry::Native(_) => None,
            })
            .collect(),
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_windows_menu_bar_as_data() {
        let menus = to_json(&app_menu(Platform::Windows));
        let tops: Vec<&str> = menus
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["label"].as_str().unwrap())
            .collect();
        assert_eq!(
            tops,
            [
                "menu.file",
                "menu.edit",
                "menu.image",
                "menu.view",
                "menu.object",
                "menu.ai",
                "menu.help"
            ]
        );
        let file = &menus[0]["items"];
        assert_eq!(file[0]["kind"], "command");
        assert_eq!(file[0]["command"], "editor.action");
        assert_eq!(file[0]["params"], json!({ "action": "new_from_template" }));
        assert_eq!(file[1]["params"], json!({ "action": "new_image" }));
        assert_eq!(file[4]["kind"], "separator");
        let view = menus[3]["items"].as_array().unwrap();
        let toolbar = view.iter().find(|i| i["kind"] == "submenu").unwrap();
        assert_eq!(toolbar["items"][1]["checked"], "editor.toolbar_labels");
    }
}

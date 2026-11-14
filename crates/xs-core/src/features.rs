//! Smart features that can be switched off one by one (SMT-002): Preferences > Editor >
//! Smart features, `smart.<feature>.enabled`, also lockable by the managed configuration
//! (APP-035). A feature switched off loses its menu entries and shortcuts, and the
//! commands that start it fail with `feature_disabled`, naming the setting. What it
//! already left in documents (redactions, guides) still opens and edits as usual.
//!
//! A switch exists once its feature does (APP-008): extract text, auto redact, the
//! step recorder, edit text, separate elements, auto simplify, QR codes and "only the
//! text" for now; the others of SMT-002 join with their
//! milestone.

use crate::menus::MenuEntry;
use crate::settings::SettingsStore;

pub struct Feature {
    /// As the UI and the locales name it (`ui.features.<name>`).
    pub name: &'static str,
    pub setting: &'static str,
    /// The commands that start it: they fail while it is off.
    pub commands: &'static [&'static str],
    /// Menu entries (by id) and editor actions that show it.
    pub entries: &'static [&'static str],
}

pub const FEATURES: [Feature; 8] = [
    Feature {
        name: "extractText",
        setting: "smart.extractText.enabled",
        commands: &["smart.extract_text", "video.extract_text"],
        entries: &[
            "smart.extract_text",
            "editor.extract_text",
            "editor.video_extract_text",
            "extract_text",
            "video_extract_text",
        ],
    },
    Feature {
        name: "autoRedact",
        setting: "smart.autoRedact.enabled",
        commands: &[
            "smart.redact",
            "smart.redact_words",
            "smart.redact_batch",
            "smart.redact_batch_decide",
        ],
        entries: &["editor.auto_redact", "auto_redact"],
    },
    Feature {
        name: "steps",
        setting: "smart.steps.enabled",
        commands: &["smart.steps_start", "smart.steps_toggle"],
        entries: &["steps_record", "smart.steps_start", "smart.steps_toggle"],
    },
    Feature {
        name: "qr",
        setting: "smart.qr.enabled",
        commands: &["smart.qr_create", "smart.qr_read", "smart.qr_open"],
        entries: &["editor.insert_qr", "insert_qr", "read_qr"],
    },
    Feature {
        name: "separate",
        setting: "smart.separate.enabled",
        commands: &["smart.separate", "smart.separate_text"],
        entries: &["separate"],
    },
    Feature {
        name: "editText",
        setting: "smart.editText.enabled",
        commands: &["smart.edit_text"],
        entries: &["edit_text"],
    },
    Feature {
        name: "autoSimplify",
        setting: "smart.autoSimplify.enabled",
        commands: &["smart.simplify_auto"],
        entries: &[],
    },
    // A checkbox of three tools (SMT-021): no command or menu entry of its own.
    Feature {
        name: "textOnly",
        setting: "smart.textOnly.enabled",
        commands: &[],
        entries: &[],
    },
];

/// The feature a command starts.
pub fn of_command(command: &str) -> Option<&'static Feature> {
    FEATURES.iter().find(|f| f.commands.contains(&command))
}

pub fn is_setting(key: &str) -> bool {
    FEATURES.iter().any(|f| f.setting == key)
}

/// The features switched off.
pub fn off(settings: &SettingsStore) -> Vec<&'static Feature> {
    FEATURES
        .iter()
        .filter(|f| !settings.get_bool(f.setting))
        .collect()
}

/// A command that may not run now: its feature is off.
pub fn blocks(settings: &SettingsStore, command: &str) -> Option<&'static Feature> {
    of_command(command).filter(|f| !settings.get_bool(f.setting))
}

fn hidden(off: &[&Feature], id: &str, command: &str) -> bool {
    off.iter()
        .any(|f| f.entries.contains(&id) || f.commands.contains(&command))
}

/// A menu without the entries of the features switched off; no separator is left
/// first, last or doubled, and an empty submenu goes too.
pub fn menu_without(entries: Vec<MenuEntry>, off: &[&Feature]) -> Vec<MenuEntry> {
    if off.is_empty() {
        return entries;
    }
    let mut out: Vec<MenuEntry> = Vec::with_capacity(entries.len());
    for e in entries {
        match e {
            MenuEntry::Command { id, command, .. } if hidden(off, id, command) => {}
            MenuEntry::Named { ref id, command, .. } if hidden(off, id, command) => {}
            MenuEntry::Submenu { label, items } => {
                let items = menu_without(items, off);
                if !items.iter().all(|i| matches!(i, MenuEntry::Separator)) {
                    out.push(MenuEntry::Submenu { label, items });
                }
            }
            MenuEntry::Separator => {
                if !out.is_empty() && !matches!(out.last(), Some(MenuEntry::Separator)) {
                    out.push(MenuEntry::Separator);
                }
            }
            other => out.push(other),
        }
    }
    while matches!(out.last(), Some(MenuEntry::Separator)) {
        out.pop();
    }
    out
}

/// The menu with the features switched off left out.
pub fn menu_for(settings: &SettingsStore, entries: Vec<MenuEntry>) -> Vec<MenuEntry> {
    menu_without(entries, &off(settings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menus;
    use crate::platform::Platform;

    #[test]
    fn a_feature_off_leaves_the_menus() {
        let redact = &FEATURES[1];
        let menu = menu_without(menus::app_menu(Platform::Windows), &[redact]);
        let ids = menus::commands_in(&menu);
        assert!(!ids.contains(&"smart.redact"));
        let all = menus::to_json(&menu).to_string();
        assert!(!all.contains("editor.auto_redact"), "{all}");
        assert!(all.contains("editor.extract_text"));

        let steps = &FEATURES[2];
        let tray = menu_without(menus::tray_menu(&[]), &[steps, &FEATURES[0]]);
        let all = menus::to_json(&tray).to_string();
        assert!(
            !all.contains("steps_record") && !all.contains("smart.extract_text"),
            "{all}"
        );
        // No separator first, last or twice in a row.
        fn check(entries: &[MenuEntry]) {
            assert!(!matches!(entries.first(), Some(MenuEntry::Separator)));
            assert!(!matches!(entries.last(), Some(MenuEntry::Separator)));
            for w in entries.windows(2) {
                assert!(!(matches!(w[0], MenuEntry::Separator) && matches!(w[1], MenuEntry::Separator)));
            }
            for e in entries {
                if let MenuEntry::Submenu { items, .. } = e {
                    check(items);
                }
            }
        }
        check(&tray);
    }

    #[test]
    fn every_feature_names_real_commands() {
        let core = crate::Core::for_tests(Platform::Windows);
        let ids: Vec<&str> = core.registry().specs().iter().map(|s| s.id).collect();
        for f in &FEATURES {
            assert!(core.settings.def(f.setting).is_some(), "{}", f.setting);
            for c in f.commands {
                assert!(ids.contains(c), "{c}");
            }
        }
    }
}

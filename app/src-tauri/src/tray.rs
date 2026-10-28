//! XcreenShot icon in the notification area / menu bar and its quick menu (APP-002).

use std::collections::HashMap;

use serde_json::Value;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};
use xs_core::i18n::tr;
use xs_core::menus::{MenuEntry, NativeItem};
use xs_core::shortcuts::ShortcutScope;
use xs_core::{Core, Lang, Source};

use crate::AppState;

pub const TRAY_ID: &str = "xcreenshot";
const BASE_ICON: &[u8] = include_bytes!("../icons/tray.png");

/// Visual state of the icon. Every state differs by shape, not only color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Visual {
    pub paused: bool,
    pub attention: bool,
    pub monochrome: bool,
}

fn light_taskbar() -> bool {
    #[cfg(windows)]
    {
        xs_core::winreg::read_dword(
            xs_core::winreg::HKCU,
            r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "SystemUsesLightTheme",
        )
        .is_some_and(|v| v == 1)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Draws the icon for a state on top of the base glyph.
pub fn render(v: Visual) -> Image<'static> {
    let base = Image::from_bytes(BASE_ICON).expect("tray icon is a valid PNG");
    let (w, h) = (base.width(), base.height());
    let mut px = base.rgba().to_vec();
    let mono_color: [u8; 3] = if cfg!(target_os = "macos") || light_taskbar() {
        [0, 0, 0]
    } else {
        [255, 255, 255]
    };
    if v.monochrome {
        for p in px.as_chunks_mut::<4>().0.iter_mut() {
            p[..3].copy_from_slice(&mono_color);
        }
    }
    let glyph: [u8; 3] = if v.monochrome { mono_color } else { [14, 140, 127] };
    let size = w as f32;
    if v.paused {
        // Paused: the X goes away (the brackets stay) so the slash drawn below reads
        // as a different shape, not as one more stroke of the X.
        for y in 0..h {
            for x in 0..w {
                let (fx, fy) = (x as f32 / size, y as f32 / size);
                if (0.2..0.8).contains(&fx) && (0.2..0.8).contains(&fy) {
                    px[((y * w + x) * 4 + 3) as usize] = 0;
                }
            }
        }
    }
    let mut paint = |x: u32, y: u32, color: [u8; 3], alpha: f32| {
        let i = ((y * w + x) * 4) as usize;
        let a = alpha.clamp(0.0, 1.0);
        let old_a = px[i + 3] as f32 / 255.0;
        let out_a = a + old_a * (1.0 - a);
        for k in 0..3 {
            let c = (color[k] as f32 * a + px[i + k] as f32 * old_a * (1.0 - a)) / out_a.max(1e-6);
            px[i + k] = c.round() as u8;
        }
        px[i + 3] = (out_a * 255.0).round() as u8;
    };
    if v.paused {
        // One slash from bottom-left to top-right; the whole icon is dimmed below.
        let thickness = size * 0.11;
        for y in 0..h {
            for x in 0..w {
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                // Distance to the segment (0.3, 0.7)-(0.7, 0.3): a slash with round ends.
                let (ax, ay, bx, by) = (size * 0.3, size * 0.7, size * 0.7, size * 0.3);
                let t = (((fx - ax) * (bx - ax) + (fy - ay) * (by - ay))
                    / ((bx - ax).powi(2) + (by - ay).powi(2)))
                .clamp(0.0, 1.0);
                let d = ((fx - ax - t * (bx - ax)).powi(2) + (fy - ay - t * (by - ay)).powi(2)).sqrt();
                paint(x, y, glyph, thickness / 2.0 - d + 0.5);
            }
        }
    }
    if v.attention {
        // Small dot in the top-right corner.
        let (cx, cy, r) = (size * 0.80, size * 0.20, size * 0.17);
        let color = if v.monochrome { glyph } else { [232, 89, 12] };
        for y in 0..h {
            for x in 0..w {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                paint(x, y, color, r - d + 0.5);
            }
        }
    }
    if v.paused {
        for p in px.as_chunks_mut::<4>().0.iter_mut() {
            p[3] = (p[3] as f32 * 0.75) as u8;
        }
    }
    Image::new_owned(px, w, h)
}

/// Menu id -> (command, params).
pub type Actions = HashMap<String, (String, Value)>;

/// Shortcut of a menu entry: the editor row named like the entry (EDT-116), else the
/// global or app row of its command.
fn shortcut_of(core: &Core, id: &str, command: &str) -> Option<xs_core::shortcuts::Accelerator> {
    let profile = core.profile();
    core.shortcuts
        .find_slot(id, ShortcutScope::Editor)
        .into_iter()
        .chain(
            [ShortcutScope::Global, ShortcutScope::App]
                .into_iter()
                .filter_map(|scope| core.shortcuts.find_slot(command, scope)),
        )
        .find_map(|slot| core.shortcuts.binding(&slot, profile))
}

/// How a menu shows shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcuts {
    /// The macOS menu bar, until its key equivalents are set up.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    None,
    /// Real accelerators: the tray, where they never act as key bindings.
    Accelerators,
    /// As text after a tab, which Windows aligns to the right: in a window menu a real
    /// accelerator would also take the keys, and the editor handles them itself.
    Text,
}

fn native(app: &AppHandle, item: NativeItem) -> tauri::Result<PredefinedMenuItem<Wry>> {
    match item {
        NativeItem::Undo => PredefinedMenuItem::undo(app, None),
        NativeItem::Redo => PredefinedMenuItem::redo(app, None),
        NativeItem::Cut => PredefinedMenuItem::cut(app, None),
        NativeItem::Copy => PredefinedMenuItem::copy(app, None),
        NativeItem::Paste => PredefinedMenuItem::paste(app, None),
        NativeItem::SelectAll => PredefinedMenuItem::select_all(app, None),
        NativeItem::Minimize => PredefinedMenuItem::minimize(app, None),
        NativeItem::Hide => PredefinedMenuItem::hide(app, None),
        NativeItem::HideOthers => PredefinedMenuItem::hide_others(app, None),
        NativeItem::ShowAll => PredefinedMenuItem::show_all(app, None),
    }
}

/// Builds native menu items from the menu data.
pub fn build_items(
    app: &AppHandle,
    core: &Core,
    entries: &[MenuEntry],
    lang: Lang,
    shortcuts: Shortcuts,
    actions: &mut Actions,
) -> tauri::Result<Vec<Box<dyn IsMenuItem<Wry>>>> {
    let mut items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
    for entry in entries {
        match entry {
            MenuEntry::Command {
                id,
                label,
                command,
                params,
                checked,
            } => {
                actions.insert(id.to_string(), (command.to_string(), params.clone()));
                let mut text = tr(lang, label, &[]);
                let shortcut = shortcut_of(core, id, command);
                let accel = match shortcuts {
                    Shortcuts::Accelerators => shortcut.map(|a| a.to_string()),
                    Shortcuts::Text => {
                        if let Some(a) = shortcut {
                            text = format!("{text}\t{}", a.display(core.shortcuts.platform()));
                        }
                        None
                    }
                    Shortcuts::None => None,
                };
                if let Some(state) = checked {
                    let on = *state == "shortcuts_paused" && core.shortcuts.user_paused();
                    items.push(Box::new(CheckMenuItem::with_id(
                        app,
                        *id,
                        text,
                        true,
                        on,
                        accel.as_deref(),
                    )?));
                } else {
                    items.push(Box::new(MenuItem::with_id(
                        app,
                        *id,
                        text,
                        true,
                        accel.as_deref(),
                    )?));
                }
            }
            MenuEntry::Submenu { label, items: sub } => {
                let children = build_items(app, core, sub, lang, shortcuts, actions)?;
                let refs: Vec<&dyn IsMenuItem<Wry>> = children.iter().map(|b| b.as_ref()).collect();
                items.push(Box::new(Submenu::with_items(
                    app,
                    tr(lang, label, &[]),
                    true,
                    &refs,
                )?));
            }
            MenuEntry::Placeholder { label } => {
                items.push(Box::new(MenuItem::new(
                    app,
                    tr(lang, label, &[]),
                    false,
                    None::<&str>,
                )?));
            }
            MenuEntry::Native(n) => items.push(Box::new(native(app, *n)?)),
            MenuEntry::Separator => items.push(Box::new(PredefinedMenuItem::separator(app)?)),
        }
    }
    Ok(items)
}

pub fn build_menu(
    app: &AppHandle,
    core: &Core,
    entries: &[MenuEntry],
    shortcuts: Shortcuts,
    actions: &mut Actions,
) -> tauri::Result<Menu<Wry>> {
    let items = build_items(app, core, entries, core.lang(), shortcuts, actions)?;
    let refs: Vec<&dyn IsMenuItem<Wry>> = items.iter().map(|b| b.as_ref()).collect();
    Menu::with_items(app, &refs)
}

pub fn create(app: &AppHandle) -> tauri::Result<TrayIcon> {
    let state = app.state::<AppState>();
    let core = &state.core;
    let mut actions = state.actions.lock().unwrap();
    let menu = build_menu(
        app,
        core,
        &xs_core::menus::tray_menu(),
        Shortcuts::Accelerators,
        &mut actions,
    )?;
    let visual = current_visual(app);
    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(render(visual))
        .icon_as_template(cfg!(target_os = "macos") && visual.monochrome)
        .tooltip(tooltip(core, visual))
        .menu(&menu)
        // Windows: left click runs the configured action and right click opens the
        // menu. macOS: both open the menu, like every menu bar icon.
        .show_menu_on_left_click(cfg!(target_os = "macos"))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if cfg!(windows) {
                    let app = tray.app_handle().clone();
                    std::thread::spawn(move || left_click(&app));
                }
            }
        })
        .build(app)?;
    tray.set_visible(core.settings.get_bool("general.showIcon"))?;
    Ok(tray)
}

/// Windows left click: the action chosen in Preferences > General (APP-002).
fn left_click(app: &AppHandle) {
    let core = &app.state::<AppState>().core;
    let action = core.settings.get_str("general.iconClickAction");
    tracing::debug!(action, "tray left click");
    let (command, params) = match action.as_str() {
        "universalCapture" => ("capture.universal", serde_json::json!({})),
        "editor" => ("app.show", serde_json::json!({})),
        _ => ("capture.panel", serde_json::json!({ "show": true })),
    };
    if let Err(e) = core.call(Source::Ui, command, params, None) {
        tracing::warn!(command, code = e.code.as_str(), "icon click command failed");
    }
}

fn tooltip(core: &Core, v: Visual) -> String {
    let lang = core.lang();
    if v.paused {
        tr(lang, "tray.tooltip_paused", &[])
    } else if v.attention {
        tr(lang, "tray.tooltip_attention", &[])
    } else {
        tr(lang, "tray.tooltip", &[])
    }
}

pub fn current_visual(app: &AppHandle) -> Visual {
    let core = &app.state::<AppState>().core;
    let attention = !core.shortcuts.failed(core.profile()).is_empty()
        || crate::permissions::all().iter().any(|p| {
            p.permission == xs_core::host::Permission::ScreenRecording
                && p.state != xs_core::host::PermissionState::Granted
        });
    Visual {
        paused: core.shortcuts.user_paused(),
        attention,
        monochrome: core.settings.get_bool("general.monochromeIcon"),
    }
}

/// Re-renders icon, tooltip, visibility and menu (language, pause state, shortcuts).
pub fn refresh(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let state = app.state::<AppState>();
    let core = &state.core;
    let visual = current_visual(app);
    let _ = tray.set_icon(Some(render(visual)));
    let _ = tray.set_icon_as_template(cfg!(target_os = "macos") && visual.monochrome);
    let _ = tray.set_tooltip(Some(tooltip(core, visual)));
    let _ = tray.set_visible(core.settings.get_bool("general.showIcon"));
    let mut actions = state.actions.lock().unwrap();
    if let Ok(menu) = build_menu(
        app,
        core,
        &xs_core::menus::tray_menu(),
        Shortcuts::Accelerators,
        &mut actions,
    ) {
        let _ = tray.set_menu(Some(menu));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha_sum(img: &Image) -> u64 {
        img.rgba().as_chunks::<4>().0.iter().map(|p| p[3] as u64).sum()
    }

    /// APP-002: states differ by shape, so they differ in the alpha channel alone
    /// (what survives a grayscale screenshot of a monochrome icon).
    #[test]
    fn states_differ_by_shape() {
        let base = Visual {
            paused: false,
            attention: false,
            monochrome: true,
        };
        let normal = render(base);
        let paused = render(Visual { paused: true, ..base });
        let attention = render(Visual {
            attention: true,
            ..base
        });
        let shapes = |i: &Image| {
            i.rgba()
                .as_chunks::<4>()
                .0
                .iter()
                .map(|p| p[3] > 128)
                .collect::<Vec<_>>()
        };
        assert_ne!(shapes(&normal), shapes(&paused));
        assert_ne!(shapes(&normal), shapes(&attention));
        assert_ne!(shapes(&paused), shapes(&attention));
        assert!(alpha_sum(&attention) > alpha_sum(&normal));
    }
}

/// `XS_DUMP_ICONS=<dir> cargo test -p xcreenshot-app dump_icons` writes raw RGBA files
/// of every state, for visual review (`node scripts/raw-to-png.mjs <dir>`).
#[cfg(test)]
#[test]
fn dump_icons() {
    let Some(dir) = std::env::var_os("XS_DUMP_ICONS") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    for mono in [false, true] {
        for (name, paused, attention) in [
            ("normal", false, false),
            ("paused", true, false),
            ("attention", false, true),
        ] {
            let img = render(Visual {
                paused,
                attention,
                monochrome: mono,
            });
            let file = dir.join(format!(
                "{name}-{}-{}x{}.rgba",
                if mono { "mono" } else { "color" },
                img.width(),
                img.height()
            ));
            std::fs::write(file, img.rgba()).unwrap();
        }
    }
}

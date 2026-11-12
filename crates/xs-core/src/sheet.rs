//! The shortcut sheet (APP-023): every shortcut the user has now (profile, own
//! changes), grouped, with the inactive ones marked; as data, as a printable HTML page,
//! or as a PDF of one or two A4 pages (drawn by the editor, which has the text engine).

use serde_json::{json, Value};

use crate::i18n::tr;
use crate::{Core, Lang};

/// Draws the sheet as a PDF; the editor provides it (it has the text engine).
pub struct PdfSheet(pub Option<DrawPdf>);

/// Turns the sheet's data into the bytes of a PDF.
pub type DrawPdf = Box<dyn Fn(&Value) -> Result<Vec<u8>, String> + Send + Sync>;

/// The groups of the sheet, in order.
const GROUPS: &[&str] = &["capture", "app", "editor", "tools"];

/// The sheet's data: title, profile, platform, date and the groups with their rows
/// (only the actions that have a shortcut).
pub fn sheet(core: &Core, lang: Lang) -> Value {
    let rows = core.shortcut_rows(lang);
    let profile = rows["profile"].as_str().unwrap_or("coexistence").to_string();
    let mut groups = Vec::new();
    for g in GROUPS {
        let list: Vec<Value> = rows["rows"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|r| r["group"] == json!(g) && r["display"].is_string())
            .map(|r| {
                json!({
                    "title": r["title"],
                    "keys": r["display"],
                    "scope": r["scope"],
                    "inactive": r["conflict"] == json!(true),
                })
            })
            .collect();
        if !list.is_empty() {
            groups.push(json!({
                "id": g,
                "title": tr(lang, &format!("ui.shortcuts.sheet_group_{g}"), &[]),
                "rows": list,
            }));
        }
    }
    let platform = core.shortcuts.platform();
    json!({
        "title": tr(lang, "ui.shortcuts.sheet_title", &[]),
        "profile": tr(lang, &format!("ui.shortcuts.sheet_profile_{profile}"), &[]),
        "platform": if platform == crate::platform::Platform::Macos { "macOS" } else { "Windows" },
        "date": chrono::Local::now().format("%Y-%m-%d").to_string(),
        "inactive": tr(lang, "ui.shortcuts.sheet_inactive", &[]),
        "groups": groups,
    })
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// A standalone page that prints on one or two A4 sheets.
pub fn html(sheet: &Value) -> String {
    let text = |k: &str| esc(sheet[k].as_str().unwrap_or_default());
    let mut body = String::new();
    for g in sheet["groups"].as_array().into_iter().flatten() {
        body.push_str(&format!(
            "<section><h2>{}</h2><table>",
            esc(g["title"].as_str().unwrap_or_default())
        ));
        for r in g["rows"].as_array().into_iter().flatten() {
            let inactive = r["inactive"] == json!(true);
            body.push_str(&format!(
                "<tr{}><td>{}</td><td class=\"keys\">{}{}</td></tr>",
                if inactive { " class=\"inactive\"" } else { "" },
                esc(r["title"].as_str().unwrap_or_default()),
                esc(r["keys"].as_str().unwrap_or_default()),
                if inactive {
                    format!(" <small>({})</small>", text("inactive"))
                } else {
                    String::new()
                },
            ));
        }
        body.push_str("</table></section>");
    }
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title><style>\
@page{{size:A4;margin:12mm}}body{{font:9pt system-ui,sans-serif;color:#111;margin:0}}\
h1{{font-size:15pt;margin:0}}p.meta{{color:#555;margin:2pt 0 8pt}}\
main{{columns:2;column-gap:10mm}}section{{break-inside:avoid;margin-bottom:6pt}}\
h2{{font-size:10.5pt;margin:6pt 0 2pt;border-bottom:1px solid #999}}table{{width:100%;border-collapse:collapse}}\
td{{padding:1pt 0;vertical-align:top}}td.keys{{text-align:right;white-space:nowrap;font-weight:600;padding-left:6pt}}\
tr.inactive td{{color:#888}}</style></head><body><h1>{title}</h1>\
<p class=\"meta\">{profile} · {platform} · {date}</p><main>{body}</main></body></html>",
        title = text("title"),
        profile = text("profile"),
        platform = text("platform"),
        date = text("date"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Platform;

    #[test]
    fn the_sheet_lists_the_shortcuts_the_user_has_now() {
        let core = Core::for_tests(Platform::Windows);
        let before = sheet(&core, Lang::En);
        assert!(before["groups"]
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g["id"] == "tools"));
        core.call(
            crate::Source::Cli,
            "shortcuts.set",
            json!({"command": "capture.image", "scope": "global", "shortcut": "Ctrl+Alt+Shift+KeyJ"}),
            Some(Lang::En),
        )
        .unwrap();
        let after = sheet(&core, Lang::En);
        let text = after.to_string();
        assert!(text.contains("Ctrl+Alt+Shift+J"), "the new value: {text}");
        let page = html(&after);
        assert!(page.starts_with("<!doctype html>") && page.contains("@page"));
        assert!(page.contains("Ctrl+Alt+Shift+J"));
    }
}

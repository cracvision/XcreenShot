//! The standard sharing destinations (SHR-020 to SHR-026; decided by Carlos on
//! 2026-11-15): email, another app, the printer, Word, PowerPoint and Excel. Nothing
//! here goes to the internet: each one hands files to something on this computer.
//!
//! What is shared is first prepared as files (SHR-007): a document or a `.xshot` as a
//! PNG with its annotations; images, videos and GIF as they are; a guide as Word or
//! PowerPoint for those apps. They go to `<local data>/share/<time>/`, kept 24 hours so
//! the other app has time to open them.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use xs_core::{CommandError, Core, ErrorCode, Lang, Source};

/// The destinations of this module.
pub const IDS: &[&str] = &["email", "app", "printer", "word", "powerpoint", "excel"];
/// Attachments over this ask before the message opens (SHR-020).
pub const EMAIL_LIMIT: u64 = 20 << 20;

fn err(lang: Lang, key: &str, args: &[(&str, &str)]) -> CommandError {
    CommandError::new(ErrorCode::Internal, xs_core::i18n::tr(lang, key, args))
}

/// Whether a destination can be used on this computer, and why not.
pub fn available(id: &str) -> Result<(), &'static str> {
    match id {
        "word" => office("Word.Application"),
        "powerpoint" => office("PowerPoint.Application"),
        "excel" => office("Excel.Application"),
        "email" | "app" | "printer" => Ok(()),
        _ => Err("unknown"),
    }
}

#[cfg(windows)]
fn office(prog_id: &str) -> Result<(), &'static str> {
    use xs_core::winreg::{read_string, HKCU, HKLM};
    let key = format!("SOFTWARE\\Classes\\{prog_id}\\CLSID");
    if read_string(HKLM, &key, "").is_some() || read_string(HKCU, &key, "").is_some() {
        Ok(())
    } else {
        Err("not_installed")
    }
}

#[cfg(not(windows))]
fn office(_prog_id: &str) -> Result<(), &'static str> {
    Err("not_supported")
}

/// What is being shared.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub files: Vec<PathBuf>,
    /// Guides among them, as their own `.xshot` (Word and PowerPoint take them as a
    /// document of their own, SMT-030).
    pub guides: Vec<PathBuf>,
}

/// Prepares documents (`documentIds`) and files (`paths`) as files to hand over.
pub fn prepare(core: &Core, lang: Lang, ids: &[String], paths: &[String]) -> Result<Prepared, CommandError> {
    let dir = core
        .paths
        .local_dir
        .join("share")
        .join(chrono::Utc::now().format("%Y%m%d-%H%M%S%3f").to_string());
    clean_old(&core.paths.local_dir.join("share"));
    std::fs::create_dir_all(&dir)
        .map_err(|e| err(lang, "errors.share_failed", &[("reason", &e.to_string())]))?;
    let mut out = Prepared {
        files: Vec::new(),
        guides: Vec::new(),
    };
    let export = |doc: &str, name: &str| -> Result<PathBuf, CommandError> {
        let path = unique(&dir, &format!("{}.png", clean_name(name)));
        core.call(
            Source::Ui,
            "document.export",
            json!({"documentId": doc, "format": "png", "path": path, "onConflict": "replace"}),
            Some(lang),
        )?;
        Ok(path)
    };
    for id in ids {
        let info = core.call(Source::Ui, "document.info", json!({"documentId": id}), Some(lang))?;
        let name = info["name"].as_str().unwrap_or("capture").to_string();
        if let Some(p) = info["path"].as_str().filter(|_| info["video"].is_object()) {
            out.files.push(PathBuf::from(p));
            continue;
        }
        // A guide of steps (SMT-035): its file, for Word and PowerPoint.
        if let Some(p) = info["path"].as_str().filter(|p| p.ends_with(".xshot")) {
            if xs_core::document::Xshot::read_summary(Path::new(p)).is_ok_and(|s| s.guide_steps.is_some()) {
                out.guides.push(PathBuf::from(p));
            }
        }
        out.files.push(export(id, &name)?);
    }
    for p in paths {
        let path = PathBuf::from(p);
        if !path.is_file() {
            return Err(CommandError::new(
                ErrorCode::NotFound,
                xs_core::i18n::tr(lang, "errors.file_not_found", &[]),
            ));
        }
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if ext != "xshot" {
            out.files.push(path);
            continue;
        }
        if xs_core::document::Xshot::read_summary(&path).is_ok_and(|s| s.guide_steps.is_some()) {
            out.guides.push(path.clone());
        }
        // Opened apart, exported with its annotations, and closed if it was not open.
        let before = core.call(Source::Ui, "document.list", json!({}), Some(lang))?;
        let opened = core.call(
            Source::Ui,
            "document.open",
            json!({"path": path, "show": false}),
            Some(lang),
        )?;
        let doc = opened["documentId"].as_str().unwrap_or_default().to_string();
        let was_open = before["documents"]
            .as_array()
            .is_some_and(|a| a.iter().any(|d| d["documentId"] == doc.as_str()));
        let name = path
            .file_stem()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let r = export(&doc, &name);
        if !was_open {
            let _ = core.call(
                Source::Ui,
                "document.close",
                json!({"documentId": doc}),
                Some(lang),
            );
        }
        out.files.push(r?);
    }
    Ok(out)
}

/// Prepared folders older than a day go (SHR-021).
fn clean_old(root: &Path) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for e in entries.flatten() {
        let old = e
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age.as_secs() > 24 * 3600);
        if old {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}

fn clean_name(name: &str) -> String {
    let n: String = name
        .chars()
        .filter(|c| !c.is_control() && !"<>:\"/\\|?*".contains(*c))
        .collect();
    let n = n.trim().trim_end_matches('.').to_string();
    if n.is_empty() {
        "capture".into()
    } else {
        n.chars().take(100).collect()
    }
}

fn unique(dir: &Path, name: &str) -> PathBuf {
    let mut p = dir.join(name);
    let (stem, ext) = name.rsplit_once('.').unwrap_or((name, ""));
    let mut n = 2;
    while p.exists() {
        p = dir.join(format!("{stem} ({n}).{ext}"));
        n += 1;
    }
    p
}

// ---------------------------------------------------------------------------
// Email (SHR-020)

/// A message ready to send, as an `.eml` the mail app opens as a draft.
pub fn eml(to: &str, subject: &str, body: &str, files: &[PathBuf]) -> Result<Vec<u8>, String> {
    use base64::Engine as _;
    let b64 = |b: &[u8]| base64::engine::general_purpose::STANDARD.encode(b);
    let wrap = |s: String| -> String {
        s.as_bytes()
            .chunks(76)
            .map(|c| String::from_utf8_lossy(c).into_owned())
            .collect::<Vec<_>>()
            .join("\r\n")
    };
    let boundary = "xcreenshot-mime-boundary";
    let mut m = String::new();
    m.push_str("X-Unsent: 1\r\n");
    if !to.trim().is_empty() {
        m.push_str(&format!("To: {}\r\n", to.trim()));
    }
    m.push_str(&format!("Subject: =?utf-8?B?{}?=\r\n", b64(subject.as_bytes())));
    m.push_str("MIME-Version: 1.0\r\n");
    m.push_str(&format!(
        "Content-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\r\n"
    ));
    m.push_str(&format!(
        "--{boundary}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n",
        wrap(b64(body.as_bytes()))
    ));
    for f in files {
        let bytes = std::fs::read(f).map_err(|e| e.to_string())?;
        let name = f
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mime = match f
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .as_deref()
        {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("mp4") | Some("m4v") => "video/mp4",
            Some("mov") => "video/quicktime",
            Some("pdf") => "application/pdf",
            _ => "application/octet-stream",
        };
        let encoded_name = format!("=?utf-8?B?{}?=", b64(name.as_bytes()));
        m.push_str(&format!(
            "--{boundary}\r\nContent-Type: {mime}; name=\"{encoded_name}\"\r\nContent-Disposition: attachment; filename=\"{encoded_name}\"\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n",
            wrap(b64(&bytes))
        ));
    }
    m.push_str(&format!("--{boundary}--\r\n"));
    Ok(m.into_bytes())
}

/// Opens a new message in the default mail app (SHR-020): the system's mail interface
/// (MAPI), or an `.eml` draft when the app does not have it (the new Outlook).
#[cfg(windows)]
pub fn email(
    dir: &Path,
    to: &str,
    subject: &str,
    body: &str,
    files: &[PathBuf],
) -> Result<&'static str, String> {
    if mapi::send(to, subject, body, files).is_ok() {
        return Ok("mail_app");
    }
    let path = dir.join("message.eml");
    std::fs::write(&path, eml(to, subject, body, files)?).map_err(|e| e.to_string())?;
    open_with_default(&path)?;
    Ok("draft")
}

#[cfg(not(windows))]
pub fn email(
    _dir: &Path,
    _to: &str,
    _subject: &str,
    _body: &str,
    _files: &[PathBuf],
) -> Result<&'static str, String> {
    Err("not yet on this system".into())
}

#[cfg(windows)]
mod mapi {
    use std::path::PathBuf;

    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    use windows_sys::Win32::System::Mapi::{
        MapiFileDescW, MapiMessageW, MapiRecipDescW, MAPI_DIALOG, MAPI_LOGON_UI, MAPI_TO,
    };

    type SendMailW = unsafe extern "system" fn(usize, usize, *const MapiMessageW, u32, u32) -> u32;

    /// MAPISendMailW of the system's mail interface, which the mail app provides.
    fn send_mail() -> Option<SendMailW> {
        let dll: Vec<u16> = "MAPI32.dll".encode_utf16().chain(Some(0)).collect();
        // SAFETY: loading a system library and looking up a function by name.
        unsafe {
            let h = LoadLibraryW(dll.as_ptr());
            if h.is_null() {
                return None;
            }
            let f = GetProcAddress(h, c"MAPISendMailW".as_ptr().cast())?;
            Some(std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                SendMailW,
            >(f))
        }
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// The new message, with the mail app's own window (it waits until it is sent or
    /// closed, so this runs apart from the command).
    pub fn send(to: &str, subject: &str, body: &str, files: &[PathBuf]) -> Result<(), u32> {
        let subject = wide(subject);
        let body = wide(body);
        let paths: Vec<Vec<u16>> = files.iter().map(|f| wide(&f.to_string_lossy())).collect();
        let names: Vec<Vec<u16>> = files
            .iter()
            .map(|f| {
                wide(
                    &f.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                )
            })
            .collect();
        let mut descs: Vec<MapiFileDescW> = paths
            .iter()
            .zip(&names)
            .map(|(p, n)| MapiFileDescW {
                ulReserved: 0,
                flFlags: 0,
                nPosition: u32::MAX,
                lpszPathName: p.as_ptr() as *mut u16,
                lpszFileName: n.as_ptr() as *mut u16,
                lpFileType: std::ptr::null_mut(),
            })
            .collect();
        let to_w = wide(to.trim());
        let addr = wide(&format!("SMTP:{}", to.trim()));
        let mut recips: Vec<MapiRecipDescW> = if to.trim().is_empty() {
            Vec::new()
        } else {
            vec![MapiRecipDescW {
                ulReserved: 0,
                ulRecipClass: MAPI_TO,
                lpszName: to_w.as_ptr() as *mut u16,
                lpszAddress: addr.as_ptr() as *mut u16,
                ulEIDSize: 0,
                lpEntryID: std::ptr::null_mut(),
            }]
        };
        let msg = MapiMessageW {
            ulReserved: 0,
            lpszSubject: subject.as_ptr() as *mut u16,
            lpszNoteText: body.as_ptr() as *mut u16,
            lpszMessageType: std::ptr::null_mut(),
            lpszDateReceived: std::ptr::null_mut(),
            lpszConversationID: std::ptr::null_mut(),
            flFlags: 0,
            lpOriginator: std::ptr::null_mut(),
            nRecipCount: recips.len() as u32,
            lpRecips: if recips.is_empty() {
                std::ptr::null_mut()
            } else {
                recips.as_mut_ptr()
            },
            nFileCount: descs.len() as u32,
            lpFiles: descs.as_mut_ptr(),
        };
        let Some(send) = send_mail() else {
            return Err(u32::MAX);
        };
        // SAFETY: every pointer refers to a buffer that lives until the call returns.
        let rc = unsafe { send(0, 0, &msg, MAPI_DIALOG | MAPI_LOGON_UI, 0) };
        if rc == 0 {
            Ok(())
        } else {
            Err(rc)
        }
    }
}

#[cfg(windows)]
fn open_with_default(path: &Path) -> Result<(), String> {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let file: Vec<u16> = path
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let verb: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
    // SAFETY: valid NUL-terminated strings.
    let r = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    if r as isize > 32 {
        Ok(())
    } else {
        Err("no app opens it".into())
    }
}

// ---------------------------------------------------------------------------
// Another app (SHR-021)

/// The arguments for a program: `{file}` (one) and `{files}` (all) are replaced; with
/// neither, the files go at the end. Words are split on spaces; "quoted" ones stay whole.
pub fn app_args(params: &str, files: &[PathBuf]) -> Vec<Vec<String>> {
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for c in params.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !cur.is_empty() {
                    words.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    let all: Vec<String> = files.iter().map(|f| f.to_string_lossy().into_owned()).collect();
    if words.iter().any(|w| w == "{files}") {
        let mut a = Vec::new();
        for w in &words {
            if w == "{files}" {
                a.extend(all.iter().cloned());
            } else {
                a.push(w.clone());
            }
        }
        return vec![a];
    }
    if words.iter().any(|w| w.contains("{file}")) {
        return all
            .iter()
            .map(|f| words.iter().map(|w| w.replace("{file}", f)).collect())
            .collect();
    }
    let mut a = words;
    a.extend(all);
    vec![a]
}

pub fn run_app(program: &str, params: &str, files: &[PathBuf]) -> Result<(), String> {
    if !Path::new(program).is_file() {
        return Err("app_missing".into());
    }
    for args in app_args(params, files) {
        std::process::Command::new(program)
            .args(args)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Word, PowerPoint and Excel (SHR-024 to SHR-026)

const WORD: &str = r#"param([string]$Where)
$ErrorActionPreference = 'Stop'
try { $w = [Runtime.InteropServices.Marshal]::GetActiveObject('Word.Application') } catch { $w = New-Object -ComObject Word.Application }
if ($env:XS_OFFICE_HIDDEN -ne '1') { $w.Visible = $true }
if ($Where -eq 'new' -or $w.Documents.Count -eq 0) { $doc = $w.Documents.Add() } else { $doc = $w.ActiveDocument }
$sel = $w.Selection
$max = $doc.PageSetup.PageWidth - $doc.PageSetup.LeftMargin - $doc.PageSetup.RightMargin
foreach ($f in $args) {
  if ($f -match '\.docx$') { $sel.InsertFile($f); continue }
  $s = $sel.InlineShapes.AddPicture($f, $false, $true)
  $s.LockAspectRatio = -1
  if ($s.Width -gt $max) { $s.Width = $max }
  $s.AlternativeText = [IO.Path]::GetFileNameWithoutExtension($f)
  $sel.TypeParagraph()
}
if ($env:XS_OFFICE_HIDDEN -eq '1') { Write-Output $doc.InlineShapes.Count; $doc.Close(0); if ($w.Documents.Count -eq 0) { $w.Quit() } } else { $w.Activate() }
"#;

const POWERPOINT: &str = r#"param([string]$Where)
$ErrorActionPreference = 'Stop'
try { $p = [Runtime.InteropServices.Marshal]::GetActiveObject('PowerPoint.Application') } catch { $p = New-Object -ComObject PowerPoint.Application }
$hidden = $env:XS_OFFICE_HIDDEN -eq '1'
if ($Where -eq 'new' -or $p.Presentations.Count -eq 0) { $pres = $p.Presentations.Add([int](-not $hidden) * -1) } else { $pres = $p.ActivePresentation }
$sw = $pres.PageSetup.SlideWidth; $sh = $pres.PageSetup.SlideHeight
$idx = $pres.Slides.Count
try { $idx = $p.ActiveWindow.View.Slide.SlideIndex } catch { }
foreach ($f in $args) {
  if ($f -match '\.pptx$') { $null = $pres.Slides.InsertFromFile($f, $idx); $idx = $pres.Slides.Count; continue }
  if ($Where -eq 'current' -and $pres.Slides.Count -gt 0) { $slide = $pres.Slides.Item([Math]::Max(1, $idx)) } else { $idx = $idx + 1; $slide = $pres.Slides.Add($idx, 12) }
  if ($f -match '\.(mp4|m4v|mov)$') { $s = $slide.Shapes.AddMediaObject2($f, 0, -1, 0, 0) } else { $s = $slide.Shapes.AddPicture($f, 0, -1, 0, 0) }
  $s.LockAspectRatio = -1
  $k = [Math]::Min(1, [Math]::Min($sw / $s.Width, $sh / $s.Height))
  $s.Width = $s.Width * $k
  $s.Left = ($sw - $s.Width) / 2; $s.Top = ($sh - $s.Height) / 2
  $s.AlternativeText = [IO.Path]::GetFileNameWithoutExtension($f)
}
if ($hidden) { Write-Output $pres.Slides.Count; $pres.Close(); if ($p.Presentations.Count -eq 0) { $p.Quit() } } else { $p.Activate() }
"#;

const EXCEL: &str = r#"param([string]$Where)
$ErrorActionPreference = 'Stop'
try { $x = [Runtime.InteropServices.Marshal]::GetActiveObject('Excel.Application') } catch { $x = New-Object -ComObject Excel.Application }
$hidden = $env:XS_OFFICE_HIDDEN -eq '1'
if (-not $hidden) { $x.Visible = $true }
if ($Where -eq 'new' -or $x.Workbooks.Count -eq 0) { $wb = $x.Workbooks.Add() } else { $wb = $x.ActiveWorkbook }
$sheet = $x.ActiveSheet
$cell = $x.ActiveCell
if ($null -eq $cell) { $cell = $sheet.Range('A1') }
$top = $cell.Top; $left = $cell.Left
foreach ($f in $args) {
  $s = $sheet.Shapes.AddPicture($f, 0, -1, $left, $top, -1, -1)
  $s.AlternativeText = [IO.Path]::GetFileNameWithoutExtension($f)
  $top = $top + $s.Height + $sheet.StandardHeight
}
if ($hidden) { Write-Output $sheet.Shapes.Count; $wb.Close($false); if ($x.Workbooks.Count -eq 0) { $x.Quit() } }
"#;

/// Runs the Office script of `app` with `where` and the files; its output (tests: how
/// many pictures went in).
pub fn office_insert(
    app: &str,
    dir: &Path,
    place: &str,
    files: &[PathBuf],
    hidden: bool,
) -> Result<String, String> {
    let script = match app {
        "word" => WORD,
        "powerpoint" => POWERPOINT,
        "excel" => EXCEL,
        _ => return Err("unknown".into()),
    };
    let ps1 = dir.join(format!("{app}.ps1"));
    std::fs::write(&ps1, script).map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
    ])
    .arg(&ps1)
    .arg(place)
    .args(files);
    if hidden {
        cmd.env("XS_OFFICE_HIDDEN", "1");
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // No console window over the user's work.
        cmd.creation_flags(0x0800_0000);
    }
    let out = cmd.output().map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if out.status.success() {
        Ok(text)
    } else {
        Err(String::from_utf8_lossy(&out.stderr)
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("the app did not answer")
            .to_string())
    }
}

// ---------------------------------------------------------------------------
// The command

/// `share.send` (SHR-020 to SHR-026).
pub fn send(core: &Core, lang: Lang, p: &Value) -> Result<Value, CommandError> {
    let dest = p["destination"].as_str().unwrap_or_default();
    if let Err(why) = available(dest) {
        return Err(CommandError::new(
            if why == "unknown" {
                ErrorCode::InvalidParams
            } else {
                ErrorCode::NotSupportedOnPlatform
            },
            xs_core::i18n::tr(lang, &format!("errors.share_{why}"), &[("destination", dest)]),
        ));
    }
    // APP-035: a destination the administrator turned off.
    let off = core
        .managed()
        .policy
        .get("disabledDestinations")
        .and_then(Value::as_array)
        .is_some_and(|a| a.iter().any(|d| d == dest));
    if off {
        return Err(CommandError::new(
            ErrorCode::DisabledByPolicy,
            xs_core::i18n::tr(lang, "errors.share_disabled", &[("destination", dest)]),
        ));
    }
    let strings = |v: &Value| -> Vec<String> {
        match v {
            Value::String(s) => vec![s.clone()],
            Value::Array(a) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
            _ => Vec::new(),
        }
    };
    let ids = strings(&p["documentIds"]);
    let mut ids = ids;
    if let Some(one) = p["documentId"].as_str() {
        ids.push(one.to_string());
    }
    let paths = strings(&p["paths"]);
    if dest == "printer" {
        // The print preview and dialog of EDT-114 (SHR-022).
        let mut q = json!({});
        if let Some(id) = ids.first() {
            q["documentId"] = json!(id);
        }
        if !paths.is_empty() {
            q["paths"] = json!(paths);
        }
        return core.call(Source::Ui, "document.print", q, Some(lang));
    }
    if ids.is_empty() && paths.is_empty() {
        let active = core.call(Source::Ui, "document.list", json!({}), Some(lang))?;
        match active["active"].as_str() {
            Some(a) => ids.push(a.to_string()),
            None => {
                return Err(CommandError::new(
                    ErrorCode::InvalidParams,
                    xs_core::i18n::tr(lang, "errors.share_nothing", &[]),
                ))
            }
        }
    }
    let prepared = prepare(core, lang, &ids, &paths)?;
    let dir = prepared
        .files
        .first()
        .and_then(|f| f.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| core.paths.local_dir.join("share"));
    let o = &p["options"];
    let fail = |reason: String| err(lang, "errors.share_failed", &[("reason", &reason)]);
    match dest {
        "email" => {
            let total: u64 = prepared
                .files
                .iter()
                .filter_map(|f| std::fs::metadata(f).ok())
                .map(|m| m.len())
                .sum();
            if total > EMAIL_LIMIT && o["force"] != true {
                return Ok(json!({"status": "too_large", "size": total, "limit": EMAIL_LIMIT}));
            }
            let first = prepared
                .files
                .first()
                .and_then(|f| f.file_stem())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let subject = o["subject"].as_str().map(str::to_string).unwrap_or(first);
            let to = o["to"].as_str().unwrap_or_default().to_string();
            let body = o["body"].as_str().unwrap_or_default().to_string();
            let files = prepared.files.clone();
            // The mail app's window waits for the person: apart from the command.
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(email(&dir, &to, &subject, &body, &files));
            });
            match rx.recv_timeout(std::time::Duration::from_secs(3)) {
                Ok(Ok(how)) => Ok(json!({"status": "opened", "how": how, "files": prepared.files})),
                Ok(Err(e)) => Err(fail(e)),
                // Still open in the mail app: it is there for the person.
                Err(_) => Ok(json!({"status": "opened", "how": "mail_app", "files": prepared.files})),
            }
        }
        "app" => {
            let program = o["program"].as_str().unwrap_or_default();
            run_app(program, o["params"].as_str().unwrap_or_default(), &prepared.files).map_err(|e| {
                if e == "app_missing" {
                    CommandError::new(
                        ErrorCode::NotFound,
                        xs_core::i18n::tr(lang, "errors.share_app_missing", &[]),
                    )
                } else {
                    fail(e)
                }
            })?;
            Ok(json!({"status": "opened", "files": prepared.files}))
        }
        "word" | "powerpoint" | "excel" => {
            let place = o["where"].as_str().unwrap_or("current");
            // A guide goes as its Word or PowerPoint document (SMT-030).
            let mut files = Vec::new();
            for f in &prepared.files {
                files.push(f.clone());
            }
            if dest != "excel" && !prepared.guides.is_empty() {
                files.clear();
                for g in &prepared.guides {
                    let ext = if dest == "word" { "docx" } else { "pptx" };
                    let out = unique(
                        &dir,
                        &format!(
                            "{}.{ext}",
                            clean_name(
                                &g.file_stem()
                                    .map(|n| n.to_string_lossy().into_owned())
                                    .unwrap_or_default()
                            )
                        ),
                    );
                    core.call(
                        Source::Ui,
                        "smart.guide_export",
                        json!({"path": g, "format": ext, "out": out}),
                        Some(lang),
                    )?;
                    files.push(out);
                }
            }
            let hidden = o["hidden"] == true;
            let r = office_insert(dest, &dir, place, &files, hidden).map_err(fail)?;
            Ok(json!({"status": "inserted", "files": files, "result": r}))
        }
        _ => Err(fail("unknown".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_arguments() {
        let files = vec![PathBuf::from(r"C:\a b.png"), PathBuf::from(r"C:\c.png")];
        assert_eq!(
            app_args("", &files),
            vec![vec![r"C:\a b.png".to_string(), r"C:\c.png".into()]]
        );
        assert_eq!(
            app_args("--new-window {file}", &files),
            vec![
                vec!["--new-window".to_string(), r"C:\a b.png".into()],
                vec!["--new-window".to_string(), r"C:\c.png".into()]
            ]
        );
        assert_eq!(
            app_args("\"-x y\" {files} -z", &files),
            vec![vec![
                "-x y".to_string(),
                r"C:\a b.png".into(),
                r"C:\c.png".into(),
                "-z".into()
            ]]
        );
    }

    #[test]
    fn an_eml_draft_with_its_attachment() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("captura.png");
        std::fs::write(&f, b"PNGDATA").unwrap();
        let m = String::from_utf8(eml("ana@example.com", "Hola ñ", "Mira", &[f]).unwrap()).unwrap();
        assert!(m.starts_with("X-Unsent: 1\r\nTo: ana@example.com\r\n"));
        assert!(m.contains("Content-Type: image/png"));
        assert!(m.contains("UE5HREFUQQ=="), "the attachment, in base64");
        assert!(m.trim_end().ends_with("--xcreenshot-mime-boundary--"));
    }
}

/// Office for real (SHR-025, SHR-026), hidden and closed without saving: run by hand on
/// a computer with Office and the app closed (`cargo test -p xs-share -- --ignored`).
#[cfg(all(test, windows))]
mod office_tests {
    use super::*;

    fn png(dir: &Path) -> PathBuf {
        let p = dir.join("captura.png");
        // 2×1 red PNG.
        let bytes: [u8; 70] = [
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
            0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00,
            0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xdd, 0x8d, 0xb0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e,
            0x44, 0xae, 0x42, 0x60, 0x82, 0x00,
        ];
        std::fs::write(&p, &bytes[..69]).unwrap();
        p
    }

    #[test]
    #[ignore]
    fn powerpoint_gets_one_slide_per_picture() {
        let dir = tempfile::tempdir().unwrap();
        let f = png(dir.path());
        let out = office_insert("powerpoint", dir.path(), "new", &[f.clone(), f], true).unwrap();
        assert_eq!(out, "2");
    }

    #[test]
    #[ignore]
    fn excel_gets_the_picture() {
        let dir = tempfile::tempdir().unwrap();
        let out = office_insert("excel", dir.path(), "new", &[png(dir.path())], true).unwrap();
        assert_eq!(out, "1");
    }
}

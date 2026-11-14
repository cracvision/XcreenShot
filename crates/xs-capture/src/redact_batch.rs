//! Auto redact over several captures (SMT-016): each capture is opened aside and
//! analyzed in the background with the same categories, terms and style as the panel,
//! plus the texts redacted by hand in the document it started from. The review then
//! accepts, skips or opens each one in the editor; until it closes, everything accepted
//! can be undone. Videos are left out.
//!
//! What an accepted capture keeps: a `.xshot` is saved with its redactions as objects
//! (still editable); a PNG or JPG gets a redacted copy beside it, "<name> (redacted)",
//! and stays as it was (provisional, `spec/QUESTIONS.md`). A document that was already
//! open keeps its redactions in the editor.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode};

use crate::redact;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Not analyzed yet.
    Waiting,
    Working,
    /// Redactions proposed: waiting for a decision.
    Ready,
    /// Nothing to redact.
    Nothing,
    Accepted,
    Skipped,
    /// Opened in the editor: the user decides there.
    Review,
    Video,
    Failed,
    Cancelled,
    /// Accepted, then undone with Undo all.
    Undone,
}

impl State {
    fn name(self) -> &'static str {
        match self {
            State::Waiting => "waiting",
            State::Working => "working",
            State::Ready => "ready",
            State::Nothing => "nothing",
            State::Accepted => "accepted",
            State::Skipped => "skipped",
            State::Review => "review",
            State::Video => "video",
            State::Failed => "failed",
            State::Cancelled => "cancelled",
            State::Undone => "undone",
        }
    }
}

#[derive(Debug, Clone)]
struct Item {
    path: Option<PathBuf>,
    /// The document while the job holds it.
    document_id: Option<String>,
    name: String,
    /// It was open before: it stays open, and the job's changes are undone in it.
    was_open: bool,
    /// Undo steps the job made in the document.
    steps: usize,
    state: State,
    redactions: usize,
    counts: Map<String, Value>,
    error: Option<String>,
    /// The `.xshot` as it was, to put back with Undo all.
    backup: Option<PathBuf>,
    /// The redacted copy written beside a picture.
    output: Option<PathBuf>,
}

struct Job {
    id: u64,
    items: Vec<Item>,
    cancel: Arc<AtomicBool>,
    running: bool,
    /// The texts redacted by hand in the document the job started from.
    texts: Vec<String>,
    /// Backups of the saved `.xshot` files.
    dir: PathBuf,
}

static JOBS: Mutex<Vec<Job>> = Mutex::new(Vec::new());
static NEXT: AtomicU64 = AtomicU64::new(1);

fn is_xshot(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case("xshot"))
}

fn is_video(path: &Path) -> bool {
    path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .is_some_and(|e| redact::VIDEO_EXTENSIONS.contains(&e.as_str()))
}

fn not_found(lang: xs_core::Lang, id: u64) -> CommandError {
    CommandError::new(
        ErrorCode::NotFound,
        tr(lang, "errors.redact_batch_not_found", &[("id", &id.to_string())]),
    )
}

fn job_id(p: &Value) -> u64 {
    p.get("jobId").and_then(Value::as_u64).unwrap_or(0)
}

fn status_of(job: &Job) -> Value {
    let items: Vec<Value> = job
        .items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            let mut v = json!({
                "index": i,
                "name": it.name,
                "path": it.path,
                "documentId": it.document_id,
                "state": it.state.name(),
                "redactions": it.redactions,
                "counts": it.counts,
            });
            if let Some(e) = &it.error {
                v["error"] = json!(e);
            }
            if let Some(o) = &it.output {
                v["output"] = json!(o);
            }
            v
        })
        .collect();
    let done = job
        .items
        .iter()
        .filter(|i| !matches!(i.state, State::Waiting | State::Working))
        .count();
    json!({
        "jobId": job.id,
        "running": job.running,
        "done": done,
        "total": job.items.len(),
        "texts": job.texts.len(),
        "items": items,
    })
}

fn with_job<R>(lang: xs_core::Lang, id: u64, f: impl FnOnce(&mut Job) -> R) -> Result<R, CommandError> {
    let mut jobs = JOBS.lock().unwrap();
    let job = jobs
        .iter_mut()
        .find(|j| j.id == id)
        .ok_or_else(|| not_found(lang, id))?;
    Ok(f(job))
}

/// The steps a call made: none when nothing changed.
fn steps_of(r: &Value) -> usize {
    let n = |k: &str| r.get(k).and_then(Value::as_u64).unwrap_or(0);
    usize::from(n("added") + n("removed") + n("restyled") > 0)
}

/// Analyzes one capture: the panel's categories and terms, then the texts by hand.
fn analyze(
    core: &Arc<Core>,
    ctx: &Context,
    item: &mut Item,
    params: &Value,
    texts: &[String],
) -> Result<(), String> {
    let lang = ctx.lang;
    let ed = xs_editor::editor(core);
    let id = match (&item.document_id, &item.path) {
        (Some(id), _) => id.clone(),
        (None, Some(path)) => match ed.store.find_by_path(path) {
            Some(id) => {
                item.was_open = true;
                id
            }
            None => xs_editor::open(core, lang, path, false).map_err(|e| e.message)?,
        },
        (None, None) => return Err("no capture".into()),
    };
    item.document_id = Some(id.clone());
    if xs_editor::video::refuse_for_video(core, lang, &json!({"documentId": id})).is_err() {
        item.state = State::Video;
        return Ok(());
    }
    let r = redact::redact(core, ctx, params, &id).map_err(|e| e.message)?;
    item.steps += steps_of(&r);
    let r = if texts.is_empty() {
        r
    } else {
        let matches: Vec<Value> = texts.iter().map(|t| json!({"text": t})).collect();
        let r = redact::redact(core, ctx, &json!({"matches": matches}), &id).map_err(|e| e.message)?;
        item.steps += steps_of(&r);
        r
    };
    item.redactions = r["redactions"].as_u64().unwrap_or(0) as usize;
    item.counts = r["counts"].as_object().cloned().unwrap_or_default();
    // Counted but not covered (a category off) is not proposed.
    let covered: Vec<&str> = r["items"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|i| i["category"].as_str())
        .collect();
    item.counts.retain(|c, _| covered.contains(&c.as_str()));
    item.state = if item.redactions == 0 {
        State::Nothing
    } else {
        State::Ready
    };
    Ok(())
}

/// Takes the job's changes away: undone in a document that was open, else closed.
fn revert(core: &Core, ctx: &Context, item: &mut Item) {
    let Some(id) = item.document_id.clone() else {
        return;
    };
    if item.was_open {
        for _ in 0..item.steps {
            let _ = core.call(
                ctx.source,
                "document.undo",
                json!({"documentId": id}),
                Some(ctx.lang),
            );
        }
        item.steps = 0;
    } else {
        close(core, ctx, &id);
        item.document_id = None;
    }
}

fn close(core: &Core, ctx: &Context, id: &str) {
    let _ = core.call(
        ctx.source,
        "document.close",
        json!({"documentId": id, "discard": [id]}),
        Some(ctx.lang),
    );
}

/// Keeps the redactions: a `.xshot` saved (its original kept to undo), a picture's
/// redacted copy written beside it.
fn accept(core: &Core, ctx: &Context, item: &mut Item, dir: &Path, index: usize) -> Result<(), String> {
    let id = item.document_id.clone().ok_or("no document")?;
    let call = |cmd: &str, p: Value| {
        core.call(ctx.source, cmd, p, Some(ctx.lang))
            .map_err(|e| e.message)
    };
    match item.path.clone() {
        Some(path) if is_xshot(&path) => {
            if !item.was_open {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
                let backup = dir.join(format!("{index}.xshot"));
                std::fs::copy(&path, &backup).map_err(|e| e.to_string())?;
                item.backup = Some(backup);
            }
            call("document.save", json!({"documentId": id}))?;
            if !item.was_open {
                close(core, ctx, &id);
                item.document_id = None;
            }
        }
        Some(path) => {
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .filter(|e| ["png", "jpg", "jpeg"].contains(&e.as_str()))
                .unwrap_or_else(|| "png".into());
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let out = path.with_file_name(format!("{stem} (redacted).{ext}"));
            let format = if ext == "png" { "png" } else { "jpg" };
            let written = call(
                "document.export",
                json!({"documentId": id, "format": format, "path": out, "onConflict": "keepBoth"}),
            )?;
            item.output = written["path"].as_str().map(PathBuf::from);
            // The picture itself stays as it was.
            revert(core, ctx, item);
        }
        // A capture never saved: it keeps the redactions in the editor.
        None => {}
    }
    item.state = State::Accepted;
    Ok(())
}

/// Puts an accepted capture back as it was.
fn undo(core: &Core, ctx: &Context, item: &mut Item) {
    match (&item.path, item.was_open) {
        (Some(path), false) if is_xshot(path) => {
            if let Some(backup) = &item.backup {
                let _ = std::fs::copy(backup, path);
            }
        }
        (Some(path), true) if is_xshot(path) => {
            revert(core, ctx, item);
            if let Some(id) = &item.document_id {
                let _ = core.call(
                    ctx.source,
                    "document.save",
                    json!({"documentId": id}),
                    Some(ctx.lang),
                );
            }
        }
        (Some(_), _) => {
            if let Some(out) = item.output.take() {
                let _ = std::fs::remove_file(out);
            }
        }
        (None, _) => revert(core, ctx, item),
    }
    item.state = State::Undone;
}

/// The background analysis, one capture after the other.
fn work(core: Arc<Core>, ctx: Context, id: u64, params: Value, cancel: Arc<AtomicBool>) {
    let texts = with_job(ctx.lang, id, |j| j.texts.clone()).unwrap_or_default();
    loop {
        // The next capture waiting, taken out of the list while it is analyzed.
        let next = with_job(ctx.lang, id, |j| {
            let i = j.items.iter().position(|i| i.state == State::Waiting)?;
            j.items[i].state = State::Working;
            Some((i, j.items[i].clone()))
        })
        .ok()
        .flatten();
        let Some((index, mut item)) = next else {
            break;
        };
        if cancel.load(Ordering::SeqCst) {
            item.state = State::Cancelled;
        } else if let Err(e) = analyze(&core, &ctx, &mut item, &params, &texts) {
            item.state = State::Failed;
            item.error = Some(e);
        }
        // Cancelled while it was analyzed: it is left as it was.
        if cancel.load(Ordering::SeqCst) && item.state == State::Ready {
            revert(&core, &ctx, &mut item);
            item.state = State::Cancelled;
        }
        if matches!(item.state, State::Nothing | State::Video | State::Failed) {
            revert(&core, &ctx, &mut item);
        }
        let kept = with_job(ctx.lang, id, |j| j.items[index] = item.clone());
        if kept.is_err() {
            // The job was closed meanwhile.
            revert(&core, &ctx, &mut item);
            return;
        }
    }
    let _ = with_job(ctx.lang, id, |j| {
        for i in j.items.iter_mut().filter(|i| i.state == State::Waiting) {
            i.state = State::Cancelled;
        }
        j.running = false;
    });
}

fn name_of(core: &Core, ctx: &Context, id: &str, path: Option<&Path>) -> String {
    core.call(
        ctx.source,
        "document.info",
        json!({"documentId": id}),
        Some(ctx.lang),
    )
    .ok()
    .and_then(|r| r["name"].as_str().map(str::to_string))
    .or_else(|| {
        path.and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
    })
    .unwrap_or_default()
}

/// `smart.redact_batch`: starts the analysis of several captures in the background.
pub fn start(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let mut items: Vec<Item> = Vec::new();
    let blank = |path: Option<PathBuf>, document_id: Option<String>, name: String, was_open: bool| Item {
        path,
        document_id,
        name,
        was_open,
        steps: 0,
        state: State::Waiting,
        redactions: 0,
        counts: Map::new(),
        error: None,
        backup: None,
        output: None,
    };
    let ed = xs_editor::editor(core);
    for d in p
        .get("documentIds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(id) = d.as_str() else { continue };
        let path = ed.store.with(id, |doc| doc.path.clone()).ok_or_else(|| {
            CommandError::new(
                ErrorCode::NotFound,
                tr(lang, "errors.document_not_found", &[("id", id)]),
            )
        })?;
        if items.iter().any(|i| i.document_id.as_deref() == Some(id)) {
            continue;
        }
        let name = name_of(core, ctx, id, path.as_deref());
        items.push(blank(path, Some(id.to_string()), name, true));
    }
    for v in p.get("paths").and_then(Value::as_array).into_iter().flatten() {
        let Some(s) = v.as_str() else { continue };
        let path = PathBuf::from(s);
        if items.iter().any(|i| i.path.as_deref() == Some(path.as_path())) {
            continue;
        }
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let video = is_video(&path);
        let mut item = blank(Some(path), None, name, false);
        if video {
            item.state = State::Video;
        }
        items.push(item);
    }
    // The document the job starts from is not one of its captures.
    let origin = p.get("documentId").and_then(Value::as_str);
    if let Some(origin) = origin {
        let origin_path = ed.store.with(origin, |doc| doc.path.clone()).flatten();
        items.retain(|i| {
            i.document_id.as_deref() != Some(origin) && (origin_path.is_none() || i.path != origin_path)
        });
    }
    if items.is_empty() {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.redact_batch_empty", &[]),
        ));
    }
    // The texts redacted by hand where it started, and the texts asked.
    let mut texts: Vec<String> = match origin {
        Some(o) => {
            let o = xs_editor::resolve_document(core, lang, &json!({"documentId": o}))?;
            redact::manual_texts(core, ctx, &o)?
        }
        None => Vec::new(),
    };
    for t in p.get("texts").and_then(Value::as_array).into_iter().flatten() {
        if let Some(t) = t.as_str().map(str::trim).filter(|t| !t.is_empty()) {
            if !texts.iter().any(|x| x == t) {
                texts.push(t.to_string());
            }
        }
    }
    let mut params = Map::new();
    for k in ["categories", "terms", "style"] {
        if let Some(v) = p.get(k) {
            params.insert(k.into(), v.clone());
        }
    }
    let id = NEXT.fetch_add(1, Ordering::SeqCst);
    let cancel = Arc::new(AtomicBool::new(false));
    let dir = std::env::temp_dir().join(format!("xcreenshot-redact-{}-{id}", std::process::id()));
    let job = Job {
        id,
        items,
        cancel: cancel.clone(),
        running: true,
        texts,
        dir,
    };
    let status = status_of(&job);
    {
        let mut jobs = JOBS.lock().unwrap();
        // Old jobs nobody closed: their documents were left to them.
        while jobs.len() >= 8 {
            jobs.remove(0);
        }
        jobs.push(job);
    }
    let core = core.clone();
    let ctx = *ctx;
    let params = Value::Object(params);
    std::thread::Builder::new()
        .name("redact-batch".into())
        .spawn(move || work(core, ctx, id, params, cancel))
        .map_err(|e| CommandError::internal(e.to_string()))?;
    tracing::info!(
        job = id,
        captures = status["total"].as_u64().unwrap_or(0),
        "auto redact of several captures"
    );
    Ok(status)
}

/// `smart.redact_batch_status`.
pub fn status(ctx: &Context, p: &Value) -> CommandResult {
    with_job(ctx.lang, job_id(p), |j| status_of(j))
}

/// The captures a decision is about: those asked, else all those ready.
fn chosen(job: &Job, p: &Value, wanted: &[State]) -> Vec<usize> {
    match p.get("items").and_then(Value::as_array) {
        Some(list) => list
            .iter()
            .filter_map(Value::as_u64)
            .map(|i| i as usize)
            .filter(|i| job.items.get(*i).is_some_and(|it| wanted.contains(&it.state)))
            .collect(),
        None => (0..job.items.len())
            .filter(|i| wanted.contains(&job.items[*i].state))
            .collect(),
    }
}

/// `smart.redact_batch_decide`: accept, skip or review captures.
pub fn decide(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = job_id(p);
    let decision = p.get("decision").and_then(Value::as_str).unwrap_or("accept");
    if !["accept", "skip", "review"].contains(&decision) {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                lang,
                "errors.invalid_value",
                &[("key", "decision"), ("reason", "accept | skip | review")],
            ),
        ));
    }
    // The work is done outside the lock: the captures are taken, then put back.
    let (picked, dir) = with_job(lang, id, |j| {
        let picked: Vec<(usize, Item)> = chosen(j, p, &[State::Ready])
            .into_iter()
            .map(|i| (i, j.items[i].clone()))
            .collect();
        (picked, j.dir.clone())
    })?;
    let mut failed: Option<String> = None;
    let mut done: Vec<(usize, Item)> = Vec::new();
    let mut shown: Option<String> = None;
    for (i, mut item) in picked {
        match decision {
            "accept" => {
                if let Err(e) = accept(core, ctx, &mut item, &dir, i) {
                    item.error = Some(e.clone());
                    failed.get_or_insert(e);
                }
            }
            "skip" => {
                revert(core, ctx, &mut item);
                item.state = State::Skipped;
            }
            _ => {
                // In the editor, with its redactions: the user decides there.
                if shown.is_none() {
                    shown = item.document_id.clone();
                }
                item.state = State::Review;
                item.document_id = None;
            }
        }
        done.push((i, item));
    }
    if let Some(doc) = &shown {
        let _ = core.call(ctx.source, "editor.show", json!({"documentId": doc}), Some(lang));
    }
    let status = with_job(lang, id, |j| {
        for (i, item) in done {
            j.items[i] = item;
        }
        status_of(j)
    })?;
    match failed {
        Some(e) if decision == "accept" => Err(CommandError::new(ErrorCode::Internal, e)),
        _ => Ok(match shown {
            Some(doc) => {
                let mut s = status;
                s["shown"] = json!(doc);
                s
            }
            None => status,
        }),
    }
}

/// `smart.redact_batch_undo`: puts every accepted capture back as it was.
pub fn undo_all(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = job_id(p);
    let picked = with_job(lang, id, |j| {
        chosen(j, p, &[State::Accepted])
            .into_iter()
            .map(|i| (i, j.items[i].clone()))
            .collect::<Vec<_>>()
    })?;
    let mut done = Vec::new();
    for (i, mut item) in picked {
        undo(core, ctx, &mut item);
        done.push((i, item));
    }
    with_job(lang, id, |j| {
        for (i, item) in done {
            j.items[i] = item;
        }
        status_of(j)
    })
}

/// `smart.redact_batch_cancel`: stops the analysis; the captures not accepted are left
/// as they were, those accepted keep their redactions.
pub fn cancel(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = job_id(p);
    let picked = with_job(lang, id, |j| {
        j.cancel.store(true, Ordering::SeqCst);
        for i in j.items.iter_mut().filter(|i| i.state == State::Waiting) {
            i.state = State::Cancelled;
        }
        chosen(j, &json!({}), &[State::Ready])
            .into_iter()
            .map(|i| (i, j.items[i].clone()))
            .collect::<Vec<_>>()
    })?;
    let mut done = Vec::new();
    for (i, mut item) in picked {
        revert(core, ctx, &mut item);
        item.state = State::Cancelled;
        done.push((i, item));
    }
    with_job(lang, id, |j| {
        for (i, item) in done {
            j.items[i] = item;
        }
        status_of(j)
    })
}

/// `smart.redact_batch_close`: ends the review. The captures still waiting for a
/// decision are left as they were; Undo all is no longer possible.
pub fn close_job(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let status = cancel(core, ctx, p)?;
    let id = job_id(p);
    let job = {
        let mut jobs = JOBS.lock().unwrap();
        let at = jobs.iter().position(|j| j.id == id);
        at.map(|i| jobs.remove(i))
    };
    if let Some(job) = job {
        let _ = std::fs::remove_dir_all(&job.dir);
    }
    let mut status = status;
    status["closed"] = json!(true);
    Ok(status)
}

pub fn register(reg: &mut xs_core::command::Registry, weak: &std::sync::Weak<Core>) {
    type Run = fn(&Arc<Core>, &Context, &Value) -> CommandResult;
    let handlers: [(&str, Run); 5] = [
        ("smart.redact_batch", start),
        ("smart.redact_batch_decide", decide),
        ("smart.redact_batch_undo", undo_all),
        ("smart.redact_batch_cancel", cancel),
        ("smart.redact_batch_close", close_job),
    ];
    for (name, run) in handlers {
        let w = weak.clone();
        reg.handle(name, move |ctx, p| {
            let core = w
                .upgrade()
                .ok_or_else(|| CommandError::internal("shutting down"))?;
            run(&core, ctx, &p)
        });
    }
    reg.handle("smart.redact_batch_status", |ctx, p| status(ctx, &p));
}

#[cfg(test)]
mod tests {
    use super::*;
    use xs_core::Source;

    fn core(dir: &Path) -> Arc<Core> {
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![crate::extension(), xs_editor::extension()],
        );
        xs_editor::editor(&core).set_folders(dir.join("lib"), dir.join("rec"));
        core
    }

    /// A `.xshot` capture in the library with these lines, closed.
    fn capture(core: &Arc<Core>, lines: &[&str]) -> PathBuf {
        let (id, _) =
            xs_editor::add_capture(core, &redact::tests::form(lines), json!({}), 1.0, None).unwrap();
        let path = xs_editor::editor(core)
            .store
            .with(&id, |d| d.path.clone())
            .flatten()
            .unwrap();
        close(
            core,
            &Context {
                source: Source::Cli,
                lang: xs_core::Lang::Es,
            },
            &id,
        );
        path
    }

    fn wait(core: &Core, job: u64) -> Value {
        for _ in 0..600 {
            let s = core
                .call(
                    Source::Cli,
                    "smart.redact_batch_status",
                    json!({"jobId": job}),
                    None,
                )
                .unwrap();
            if s["running"] == false {
                return s;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        panic!("the analysis did not end");
    }

    fn managed_in(core: &Arc<Core>, path: &Path) -> Vec<String> {
        let ctx = Context {
            source: Source::Cli,
            lang: xs_core::Lang::Es,
        };
        let id = xs_editor::open(core, ctx.lang, path, false).unwrap();
        let cats = xs_editor::auto_redact::managed(core, &id)
            .unwrap()
            .into_iter()
            .map(|m| m.category)
            .collect();
        close(core, &ctx, &id);
        cats
    }

    /// SMT-016: the same email in several captures is proposed in each and Accept all
    /// redacts it in all; a name redacted by hand where it started is proposed where it
    /// appears; Undo all puts them back; a video is left out.
    #[test]
    fn several_captures_redacted() {
        if xs_ocr::languages().is_empty() {
            eprintln!("no recognition language on this computer: skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let core = core(dir.path());
        let call = |cmd: &str, p: Value| core.call(Source::Cli, cmd, p, Some(xs_core::Lang::Es)).unwrap();
        let a = capture(&core, &["Correo: ana.ruiz@example.com", "Gracias"]);
        let b = capture(&core, &["Escribir a ana.ruiz@example.com", "Firma: Marta Ruiz"]);
        let c = capture(&core, &["Sin datos que tapar"]);
        let video = dir.path().join("clip.mp4");
        std::fs::write(&video, b"not a video").unwrap();

        // Where it starts: "Marta Ruiz" redacted by hand.
        let origin = xs_editor::add_capture(
            &core,
            &redact::tests::form(&["Cliente: Marta Ruiz"]),
            json!({}),
            1.0,
            None,
        )
        .unwrap()
        .0;
        let words = call("smart.redact_words", json!({"documentId": origin}));
        let rects: Vec<Value> = words["lines"][0]["words"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|w| w["text"] == "Marta" || w["text"] == "Ruiz")
            .map(|w| w["rect"].clone())
            .collect();
        let x0 = rects
            .iter()
            .map(|r| r[0].as_f64().unwrap())
            .fold(f64::MAX, f64::min);
        let x1 = rects
            .iter()
            .map(|r| r[0].as_f64().unwrap() + r[2].as_f64().unwrap())
            .fold(0.0, f64::max);
        let r0 = &rects[0];
        call(
            "smart.redact",
            json!({"documentId": origin, "categories": ["email"], "add": [[x0, r0[1], x1 - x0, r0[3]]]}),
        );

        let modified = |p: &Path| std::fs::metadata(p).unwrap().modified().unwrap();
        let before = modified(&a);
        let s = call(
            "smart.redact_batch",
            json!({"documentId": origin, "paths": [a, b, c, video]}),
        );
        assert_eq!(
            (s["total"].as_u64(), s["texts"].as_u64()),
            (Some(4), Some(1)),
            "{s}"
        );
        let s = wait(&core, s["jobId"].as_u64().unwrap());
        let job = s["jobId"].as_u64().unwrap();
        let states: Vec<&str> = s["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["state"].as_str().unwrap())
            .collect();
        assert_eq!(states, ["ready", "ready", "nothing", "video"], "{s}");
        assert_eq!(s["items"][0]["counts"]["email"], 1, "{s}");
        assert_eq!(s["items"][1]["counts"]["email"], 1, "{s}");
        assert_eq!(
            s["items"][1]["counts"]["manual"], 1,
            "the name redacted by hand: {s}"
        );
        // The files are not touched before a decision.
        assert_eq!(modified(&a), before);

        let s = call(
            "smart.redact_batch_decide",
            json!({"jobId": job, "decision": "accept"}),
        );
        assert_eq!(s["items"][0]["state"], "accepted", "{s}");
        assert_eq!(managed_in(&core, &a), ["email"]);
        let mut cats = managed_in(&core, &b);
        cats.sort();
        assert_eq!(cats, ["email", "manual"]);

        call("smart.redact_batch_undo", json!({"jobId": job}));
        assert!(managed_in(&core, &a).is_empty());
        assert!(managed_in(&core, &b).is_empty());
        let s = call("smart.redact_batch_close", json!({"jobId": job}));
        assert_eq!(s["closed"], true);
        assert!(core
            .call(
                Source::Cli,
                "smart.redact_batch_status",
                json!({"jobId": job}),
                None
            )
            .is_err());
    }

    /// SMT-016: cancelling leaves the captures not accepted as they were; skipping one
    /// leaves it too; an open capture keeps being open, without the job's changes.
    #[test]
    fn cancel_and_skip_leave_captures_alone() {
        if xs_ocr::languages().is_empty() {
            eprintln!("no recognition language on this computer: skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let core = core(dir.path());
        let call = |cmd: &str, p: Value| core.call(Source::Cli, cmd, p, Some(xs_core::Lang::Es)).unwrap();
        let paths: Vec<PathBuf> = (0..3)
            .map(|_| capture(&core, &["Correo: ana.ruiz@example.com"]))
            .collect();
        // One of them open in the editor.
        let open = xs_editor::open(&core, xs_core::Lang::Es, &paths[1], true).unwrap();
        let s = call(
            "smart.redact_batch",
            json!({"paths": paths, "categories": ["email"]}),
        );
        let job = s["jobId"].as_u64().unwrap();
        let s = wait(&core, job);
        assert!(
            s["items"]
                .as_array()
                .unwrap()
                .iter()
                .all(|i| i["state"] == "ready"),
            "{s}"
        );
        call(
            "smart.redact_batch_decide",
            json!({"jobId": job, "decision": "accept", "items": [0]}),
        );
        call(
            "smart.redact_batch_decide",
            json!({"jobId": job, "decision": "skip", "items": [2]}),
        );
        let s = call("smart.redact_batch_cancel", json!({"jobId": job}));
        let states: Vec<&str> = s["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["state"].as_str().unwrap())
            .collect();
        assert_eq!(states, ["accepted", "cancelled", "skipped"], "{s}");
        assert_eq!(managed_in(&core, &paths[0]), ["email"]);
        assert!(managed_in(&core, &paths[2]).is_empty());
        // Still open, without the redaction the job proposed.
        assert!(xs_editor::auto_redact::managed(&core, &open).unwrap().is_empty());
        call("smart.redact_batch_close", json!({"jobId": job}));
        assert_eq!(
            managed_in(&core, &paths[0]),
            ["email"],
            "closing keeps what was accepted"
        );
    }
}

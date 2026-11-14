//! The step recorder's workers (SMT-024 to SMT-027, ADR 0009), on Windows.
//!
//! - The worker turns what the hook heard into steps: at the press it takes the picture
//!   of the monitor and asks UI Automation what is there; at the release it writes the
//!   step's sentence and counts it (the bar shows it at once).
//! - The writer, on a thread of its own, writes each picture to the recording's folder
//!   (scaled with the Scale image effect) and, when UI Automation said nothing useful,
//!   reads the text near the click to name it (SMT-025). Slow work never delays the
//!   next click's picture.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Weak};
use std::thread::JoinHandle;

use image::RgbaImage;
use xs_core::{Core, Source};

use crate::steps::{self, Classifier, Kind, Out, Step, StepPointer, Target};
use crate::steps_rec::{meta_of, save, tell, Named, Session, Status};
use crate::steps_win::{double_click, window_rect, Heard, Probe, Probed};

enum Msg {
    Heard(Heard),
    Finish,
}

pub struct Worker {
    tx: Sender<Msg>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn start(core: Weak<Core>, s: Arc<Session>) -> Result<Worker, String> {
        let (tx, rx) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("steps-worker".into())
            .spawn(move || run(core, s, rx))
            .map_err(|e| e.to_string())?;
        Ok(Worker {
            tx,
            thread: Some(thread),
        })
    }

    /// Where the hook sends what it hears.
    pub fn sink(&self) -> Box<dyn Fn(Heard) + Send> {
        let tx = self.tx.clone();
        Box::new(move |h| {
            let _ = tx.send(Msg::Heard(h));
        })
    }

    /// Ends the worker once the steps heard so far are written.
    pub fn finish(mut self) {
        let _ = self.tx.send(Msg::Finish);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// A press waiting for its release.
struct Begun {
    image: RgbaImage,
    /// The monitor, desktop pixels.
    monitor: [f64; 4],
    probed: Probed,
    pointer: Option<crate::pointer::ScreenPointer>,
}

/// A picture to write, and maybe a name to read near the click.
struct Job {
    n: u32,
    path: PathBuf,
    image: RgbaImage,
    pointer: Option<(PathBuf, RgbaImage)>,
    /// Read the text near this point of the picture (unscaled) to name the step.
    read_at: Option<(f64, f64)>,
    app: Option<String>,
    drop: Option<Target>,
}

fn monitor_at(x: i32, y: i32) -> Option<(xcap::Monitor, [f64; 4])> {
    crate::monitors().ok()?.into_iter().find_map(|(m, i)| {
        i.rect.contains(x, y).then_some((
            m,
            [
                i.rect.x as f64,
                i.rect.y as f64,
                i.rect.width as f64,
                i.rect.height as f64,
            ],
        ))
    })
}

fn app_name(pid: u32) -> String {
    if pid == 0 {
        return String::new();
    }
    crate::platform::app_info(pid, "").name
}

fn run(core: Weak<Core>, s: Arc<Session>, rx: Receiver<Msg>) {
    let probe = Probe::new();
    let (double_ms, double_px) = double_click();
    let mut c = Classifier::new(double_ms, double_px);
    let mut pending: HashMap<u64, Begun> = HashMap::new();
    let (jobs, inbox) = mpsc::channel::<Job>();
    let writer = {
        let (core, s) = (core.clone(), s.clone());
        std::thread::Builder::new()
            .name("steps-writer".into())
            .spawn(move || write_loop(core, s, inbox))
            .ok()
    };
    for msg in rx {
        let heard = match msg {
            Msg::Finish => break,
            Msg::Heard(h) => h,
        };
        let recording = s.shared.lock().unwrap().status == Status::Recording;
        match heard {
            Heard::Cancel => {
                // From a thread of its own: cancelling ends this worker.
                let core = core.clone();
                std::thread::spawn(move || {
                    if let Some(core) = core.upgrade() {
                        let _ = core.call(Source::Ui, "smart.steps_cancel", serde_json::json!({}), None);
                    }
                });
            }
            Heard::Mouse(e) => {
                // While paused (or under the notice) presses do not count; releases still
                // reach the classifier, so it is never left waiting for one.
                if !recording && matches!(e, steps::Event::Down { .. }) {
                    continue;
                }
                match c.feed(e) {
                    Some(Out::Begin { id, button, x, y }) => {
                        let (xi, yi) = (x as i32, y as i32);
                        let Some((m, monitor)) = monitor_at(xi, yi) else {
                            continue;
                        };
                        // The picture first: the screen as it was at the press (SMT-024).
                        let Ok(image) = m.capture_image() else { continue };
                        let pointer = if s.cursor { crate::pointer::grab() } else { None };
                        let probed = probe.at(xi, yi);
                        let _ = button;
                        pending.insert(
                            id,
                            Begun {
                                image,
                                monitor,
                                probed,
                                pointer,
                            },
                        );
                    }
                    Some(Out::Finish { id, kind, to }) => {
                        let Some(b) = pending.remove(&id) else { continue };
                        if !recording {
                            continue;
                        }
                        let drop = to.and_then(|(tx, ty)| probe.at(tx as i32, ty as i32).target);
                        make_step(&s, id, b, kind, to, drop, &jobs);
                        if let Some(core) = core.upgrade() {
                            tell(&core, &s, "steps.added");
                        }
                    }
                    Some(Out::Double { id }) => {
                        let mut sh = s.shared.lock().unwrap();
                        if sh.ids.last() == Some(&id) {
                            let n = sh.steps.last().map(|st| st.n).unwrap_or(0);
                            let text = sh.named.iter().find(|w| w.n == n).map(|w| {
                                steps::phrase(s.lang, Kind::Double, &w.target, None, w.app.as_deref())
                            });
                            if let Some(step) = sh.steps.last_mut() {
                                step.kind = Kind::Double;
                                if let Some(text) = text {
                                    step.text = text;
                                }
                            }
                            let _ = save(&s.folder, &sh.steps, &meta_of(&s));
                        }
                        drop(sh);
                        if let Some(core) = core.upgrade() {
                            tell(&core, &s, "steps.changed");
                        }
                    }
                    None => {}
                }
            }
            Heard::Shortcut(keys) if recording && s.keyboard => {
                keys_step(&s, &keys, &jobs);
                if let Some(core) = core.upgrade() {
                    tell(&core, &s, "steps.added");
                }
            }
            Heard::Typed if recording && s.keyboard => {
                let mut sh = s.shared.lock().unwrap();
                if sh.typed || sh.steps.is_empty() {
                    continue;
                }
                sh.typed = true;
                let field = probe.focused_field();
                if let Some(step) = sh.steps.last_mut() {
                    step.text.push_str(&steps::typed_phrase(s.lang, field.as_deref()));
                }
                let _ = save(&s.folder, &sh.steps, &meta_of(&s));
            }
            _ => {}
        }
    }
    // The pictures still being written are finished before the guide is made.
    drop(jobs);
    if let Some(w) = writer {
        let _ = w.join();
    }
}

/// The application of a step, when it changed from the step before (SMT-025).
fn app_change(s: &Session, app: &str) -> Option<String> {
    let sh = s.shared.lock().unwrap();
    let before = sh.steps.last().map(|st| st.app.clone());
    (before.as_deref() != Some(app) && !app.is_empty()).then(|| app.to_string())
}

/// Counts a step and hands its picture to the writer.
fn make_step(
    s: &Session,
    id: u64,
    b: Begun,
    kind: Kind,
    to: Option<(f64, f64)>,
    dropped: Option<Target>,
    jobs: &Sender<Job>,
) {
    let k = s.scale;
    let m = b.monitor;
    let local = |x: f64, y: f64| [(x - m[0]) * k, (y - m[1]) * k];
    let app = app_name(b.probed.pid);
    let prefix = app_change(s, &app);
    let target = b.probed.target.clone().unwrap_or(Target::Nothing);
    let text = steps::phrase(s.lang, kind, &target, dropped.as_ref(), prefix.as_deref());
    let frame = steps::frame(s.framing, &b.probed.containers, b.probed.window, m).map(|v| (v * k).round());
    let at = b.probed.point;
    let mut sh = s.shared.lock().unwrap();
    let n = sh.next_n;
    sh.next_n += 1;
    let image = format!("step-{n:03}.png");
    let pointer = b.pointer.as_ref().map(|p| {
        let file = format!("pointer-{n:03}.png");
        let at = local(p.x as f64, p.y as f64);
        (
            StepPointer {
                file: file.clone(),
                at,
                size: [
                    ((p.image.width() as f64) * k).round().max(1.0) as u32,
                    ((p.image.height() as f64) * k).round().max(1.0) as u32,
                ],
            },
            file,
            p.image.clone(),
        )
    });
    let step = Step {
        n,
        kind,
        image: image.clone(),
        at: local(at.0, at.1),
        to: to.map(|(x, y)| local(x, y)),
        frame,
        text,
        app: app.clone(),
        pointer: pointer.as_ref().map(|(p, _, _)| p.clone()),
        time: chrono::Local::now().to_rfc3339(),
    };
    sh.steps.push(step);
    sh.ids.push(id);
    sh.typed = false;
    sh.named.push(Named {
        n,
        target: target.clone(),
        app: prefix.clone(),
    });
    if sh.named.len() > 16 {
        sh.named.remove(0);
    }
    let _ = save(&s.folder, &sh.steps, &meta_of(s));
    drop(sh);
    let read_at = matches!(target, Target::Nothing).then(|| (at.0 - m[0], at.1 - m[1]));
    let _ = jobs.send(Job {
        n,
        path: s.folder.join(&image),
        image: b.image,
        pointer: pointer.map(|(_, file, img)| (s.folder.join(file), img)),
        read_at,
        app: prefix,
        drop: dropped,
    });
}

/// A keyboard shortcut's step (SMT-024): the picture of the monitor under the
/// pointer, framed on the window in front.
fn keys_step(s: &Session, keys: &str, jobs: &Sender<Job>) {
    let (x, y) = crate::platform::cursor_position().unwrap_or((0, 0));
    let Some((m, monitor)) = monitor_at(x, y) else {
        return;
    };
    let Ok(image) = m.capture_image() else { return };
    let fg = crate::steps_win::foreground();
    let window = fg.and_then(|(w, _)| window_rect(w));
    let pid = fg.map(|(_, p)| p).unwrap_or(0);
    let app = app_name(pid);
    let prefix = app_change(s, &app);
    let k = s.scale;
    let frame = steps::frame(s.framing, &[], window, monitor).map(|v| (v * k).round());
    let mut sh = s.shared.lock().unwrap();
    let n = sh.next_n;
    sh.next_n += 1;
    let name = format!("step-{n:03}.png");
    sh.steps.push(Step {
        n,
        kind: Kind::Keys,
        image: name.clone(),
        at: [(x as f64 - monitor[0]) * k, (y as f64 - monitor[1]) * k],
        to: None,
        frame,
        text: steps::keys_phrase(s.lang, keys, prefix.as_deref()),
        app,
        pointer: None,
        time: chrono::Local::now().to_rfc3339(),
    });
    sh.ids.push(0);
    sh.typed = false;
    let _ = save(&s.folder, &sh.steps, &meta_of(s));
    drop(sh);
    let _ = jobs.send(Job {
        n,
        path: s.folder.join(name),
        image,
        pointer: None,
        read_at: None,
        app: None,
        drop: None,
    });
}

/// Writes the pictures (SMT-027) and names the steps UI Automation could not
/// (SMT-025).
fn write_loop(core: Weak<Core>, s: Arc<Session>, inbox: Receiver<Job>) {
    let languages = core
        .upgrade()
        .map(|c| crate::extract::wanted(&c, None))
        .unwrap_or_default();
    for job in inbox {
        let text = job.read_at.and_then(|p| text_near(&job.image, p, &languages));
        let image = scaled(&job.image, s.scale);
        let written = xs_image::encode::png(&image, 0.0, true)
            .map_err(std::io::Error::other)
            .and_then(|png| std::fs::write(&job.path, png));
        if let Some((path, img)) = &job.pointer {
            if let Ok(png) = xs_image::encode::png(&scaled(img, s.scale), 0.0, true) {
                let _ = std::fs::write(path, png);
            }
        }
        if let Err(e) = written {
            // A full disk pauses the recording and keeps what is written (SMT-027).
            tracing::warn!(error = %e, "a step's picture could not be written");
            {
                let mut sh = s.shared.lock().unwrap();
                sh.status = Status::Paused;
                sh.disk_full = true;
            }
            if let Some(core) = core.upgrade() {
                tell(&core, &s, "steps.paused");
                let lang = s.lang;
                core.host().notify(
                    "steps",
                    &xs_core::i18n::tr(lang, "notify.steps_disk_full", &[]),
                    "",
                );
            }
            continue;
        }
        if let Some(found) = text {
            let mut sh = s.shared.lock().unwrap();
            if let Some(w) = sh.named.iter_mut().find(|w| w.n == job.n) {
                w.target = Target::Text(found.clone());
            }
            if let Some(step) = sh.steps.iter_mut().find(|st| st.n == job.n) {
                step.text = steps::phrase(
                    s.lang,
                    step.kind,
                    &Target::Text(found),
                    job.drop.as_ref(),
                    job.app.as_deref(),
                );
            }
            let _ = save(&s.folder, &sh.steps, &meta_of(&s));
        }
    }
}

fn scaled(img: &RgbaImage, k: f64) -> RgbaImage {
    if (k - 1.0).abs() < 1e-6 {
        return img.clone();
    }
    let (w, h) = (
        ((img.width() as f64) * k).round().max(1.0) as u32,
        ((img.height() as f64) * k).round().max(1.0) as u32,
    );
    image::imageops::resize(img, w, h, image::imageops::FilterType::Triangle)
}

/// The text under a point of a picture, or the nearest (SMT-025): the line there,
/// or its nearest word when the line is long.
fn text_near(img: &RgbaImage, p: (f64, f64), languages: &[String]) -> Option<String> {
    let (w, h) = (360.0f64, 140.0f64);
    let x0 = (p.0 - w / 2.0).clamp(0.0, (img.width() as f64 - w).max(0.0));
    let y0 = (p.1 - h / 2.0).clamp(0.0, (img.height() as f64 - h).max(0.0));
    let cw = w.min(img.width() as f64) as u32;
    let ch = h.min(img.height() as f64) as u32;
    let crop = image::imageops::crop_imm(img, x0 as u32, y0 as u32, cw, ch).to_image();
    let rec = xs_ocr::recognize(&crop, languages, &AtomicBool::new(false)).ok()?;
    let (px, py) = (p.0 - x0, p.1 - y0);
    let dist = |r: &[f64; 4]| {
        let dx = (r[0] - px).max(px - (r[0] + r[2])).max(0.0);
        let dy = (r[1] - py).max(py - (r[1] + r[3])).max(0.0);
        dx.hypot(dy)
    };
    let line = rec
        .blocks
        .iter()
        .flat_map(|b| &b.lines)
        .min_by(|a, b| dist(&a.rect).total_cmp(&dist(&b.rect)))?;
    if dist(&line.rect) > 60.0 {
        return None;
    }
    if line.words.len() <= 5 {
        return Some(line.text.clone());
    }
    line.words
        .iter()
        .min_by(|a, b| dist(&a.rect).total_cmp(&dist(&b.rect)))
        .map(|w| w.text.clone())
}

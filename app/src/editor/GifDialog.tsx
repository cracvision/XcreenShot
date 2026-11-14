// Create animated GIF (VID-068 to VID-070): the selection or the whole video, Optimize
// for with its presets or Custom, an estimate of size, length and dimensions with a
// preview of the first seconds, and the size guide. Creating runs in the background,
// with its progress and Cancel on the playback bar.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import { useEditor } from "./state";
import { createGif, useVideo } from "./videoState";
import { wholeSelected } from "./videoTimeline";

type Optimize = "screen" | "motion" | "small" | "best" | "custom";

interface GifOptions {
  optimize: Optimize;
  fps: number;
  unit: "px" | "percent";
  width: number;
  height: number;
  percent: number;
  lock: boolean;
  dither: boolean;
  dynamic: boolean;
  loops: "forever" | "once" | "times";
  times: number;
  fade: boolean;
  fadeSeconds: number;
}

interface Estimate {
  width: number;
  height: number;
  fps: number;
  duration: number;
  bytes: number;
  dither: boolean;
  dynamic: boolean;
  long: boolean;
  heavy: boolean;
  preview: string;
}

const KEY = "xs.gifOptions";
const DEFAULTS: GifOptions = {
  optimize: "screen",
  fps: 15,
  unit: "px",
  width: 0,
  height: 0,
  percent: 100,
  lock: true,
  dither: false,
  dynamic: true,
  loops: "forever",
  times: 3,
  fade: false,
  fadeSeconds: 1,
};

/** The last choices (VID-069, VID-070): a convenience of this computer. */
function remembered(): GifOptions {
  try {
    const saved = localStorage.getItem(KEY);
    return saved ? { ...DEFAULTS, ...(JSON.parse(saved) as Partial<GifOptions>) } : DEFAULTS;
  } catch {
    return DEFAULTS;
  }
}

function remember(o: GifOptions): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(o));
  } catch {
    // Without storage the dialog starts from the defaults next time.
  }
}

/** The parameters of `video.gif` and `video.gif_estimate`. */
function gifParams(
  o: GifOptions,
  part: { start: number; end: number } | null,
): Record<string, unknown> {
  const p: Record<string, unknown> = { optimize: o.optimize };
  if (part) Object.assign(p, part);
  if (o.optimize === "custom") {
    p.fps = o.fps;
    if (o.unit === "percent") p.percent = o.percent;
    else if (o.width > 0 && o.height > 0) Object.assign(p, { width: o.width, height: o.height });
    p.dither = o.dither;
    p.dynamic = o.dynamic;
  }
  p.loops = o.loops === "times" ? o.times : o.loops;
  if (o.fade) p.fade = o.fadeSeconds;
  return p;
}

function megabytes(bytes: number, lang: string): string {
  return bytes < 1_048_576
    ? `${Math.max(1, Math.round(bytes / 1024)).toLocaleString(lang)} KB`
    : `${(bytes / 1_048_576).toLocaleString(lang, { maximumFractionDigits: 1 })} MB`;
}

export function GifDialog() {
  const open = useVideo((s) => s.gifDialog);
  return open ? <GifForm /> : null;
}

function GifForm() {
  const { t, i18n } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const v = doc?.video;
  const start = useVideo((s) => s.start);
  const end = useVideo((s) => s.end);
  const whole = !v || wholeSelected(start, end, v.length);
  const [part, setPart] = useState<"selection" | "all">(whole ? "all" : "selection");
  const [o, setO] = useState<GifOptions>(() => {
    const r = remembered();
    // Pixels start at the video's size.
    return r.width > 0 ? r : { ...r, width: v?.width ?? 0, height: v?.height ?? 0 };
  });
  const [estimate, setEstimate] = useState<{ key: string; e: Estimate | null }>({
    key: "",
    e: null,
  });
  const close = () => useVideo.setState({ gifDialog: false });
  const range = part === "selection" ? { start, end } : null;
  const params = gifParams(o, range);
  const key = JSON.stringify(params);

  // The estimate follows every change (VID-068), a moment after the last one.
  const docId = doc?.documentId;
  useEffect(() => {
    if (!docId) return;
    let live = true;
    const timer = window.setTimeout(() => {
      void run<Estimate>("video.gif_estimate", {
        documentId: docId,
        ...(JSON.parse(key) as Record<string, unknown>),
      })
        .then((e) => {
          if (!live) return;
          setEstimate({ key, e });
          // A preset fills the Custom fields, as a starting point (VID-069).
          setO((cur) =>
            cur.optimize === "custom"
              ? cur
              : {
                  ...cur,
                  fps: e.fps,
                  width: e.width,
                  height: e.height,
                  dither: e.dither,
                  dynamic: e.dynamic,
                },
          );
        })
        .catch(() => live && setEstimate({ key, e: null }));
    }, 350);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [docId, key]);

  if (!doc || !v) return null;
  const set = (patch: Partial<GifOptions>) => setO((cur) => ({ ...cur, ...patch }));
  const ratio = v.height / Math.max(1, v.width);
  const e = estimate.key === key ? estimate.e : null;
  const custom = o.optimize === "custom";
  const create = () => {
    remember(o);
    close();
    void createGif(params);
  };

  return (
    <Dialog
      title={t("ui.video.gif.title")}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn primary" onClick={create}>
            {t("ui.video.gif.create")}
          </button>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <div className="image-form gif-form">
        <fieldset className="gif-part">
          <legend>{t("ui.video.gif.what")}</legend>
          <label className="panel-check">
            <input
              type="radio"
              name="gif-part"
              checked={part === "selection"}
              disabled={whole}
              onChange={() => setPart("selection")}
            />
            {t("ui.video.gif.selection", { seconds: (end - start).toFixed(1) })}
          </label>
          <label className="panel-check">
            <input
              type="radio"
              name="gif-part"
              checked={part === "all"}
              onChange={() => setPart("all")}
            />
            {t("ui.video.gif.all", { seconds: v.length.toFixed(1) })}
          </label>
        </fieldset>
        <div className="prop-field">
          <label htmlFor="gif-optimize">{t("ui.video.gif.optimize")}</label>
          <select
            id="gif-optimize"
            data-autofocus
            value={o.optimize}
            onChange={(ev) => set({ optimize: ev.target.value as Optimize })}
          >
            {(["screen", "motion", "small", "best", "custom"] as const).map((k) => (
              <option key={k} value={k}>
                {t(`ui.video.gif.optimize_${k}`)}
              </option>
            ))}
          </select>
        </div>
        {custom && (
          <>
            <div className="prop-field">
              <label htmlFor="gif-fps">{t("ui.video.gif.fps")}</label>
              <input
                id="gif-fps"
                type="number"
                min={1}
                max={Math.min(30, v.fps || 30)}
                value={o.fps}
                onChange={(ev) => set({ fps: Number(ev.target.value) || 1 })}
              />
            </div>
            <div className="prop-field">
              <label htmlFor="gif-unit">{t("ui.video.gif.unit")}</label>
              <select
                id="gif-unit"
                value={o.unit}
                onChange={(ev) => set({ unit: ev.target.value as GifOptions["unit"] })}
              >
                <option value="px">{t("ui.video.gif.pixels")}</option>
                <option value="percent">{t("ui.video.gif.percent")}</option>
              </select>
            </div>
            {o.unit === "percent" ? (
              <div className="prop-field">
                <label htmlFor="gif-percent">{t("ui.video.gif.percent")}</label>
                <input
                  id="gif-percent"
                  type="number"
                  min={1}
                  max={100}
                  value={o.percent}
                  onChange={(ev) => set({ percent: Number(ev.target.value) || 1 })}
                />
              </div>
            ) : (
              <div className="image-sides">
                <div className="prop-field">
                  <label htmlFor="gif-w">{t("ui.editor.image.width")}</label>
                  <input
                    id="gif-w"
                    type="number"
                    min={16}
                    max={v.width}
                    value={o.width}
                    onChange={(ev) => {
                      const w = Number(ev.target.value) || 16;
                      set(o.lock ? { width: w, height: Math.round(w * ratio) } : { width: w });
                    }}
                  />
                </div>
                <div className="prop-field">
                  <label htmlFor="gif-h">{t("ui.editor.image.height")}</label>
                  <input
                    id="gif-h"
                    type="number"
                    min={16}
                    max={v.height}
                    value={o.height}
                    onChange={(ev) => {
                      const h = Number(ev.target.value) || 16;
                      set(o.lock ? { height: h, width: Math.round(h / ratio) } : { height: h });
                    }}
                  />
                </div>
              </div>
            )}
            {o.unit === "px" && (
              <label className="panel-check">
                <input
                  type="checkbox"
                  checked={o.lock}
                  onChange={(ev) => set({ lock: ev.target.checked })}
                />
                {t("ui.editor.image.lock")}
              </label>
            )}
            <label className="panel-check" title={t("ui.video.gif.dither_hint")}>
              <input
                type="checkbox"
                checked={o.dither}
                onChange={(ev) => set({ dither: ev.target.checked })}
              />
              {t("ui.video.gif.dither")}
            </label>
            <label className="panel-check" title={t("ui.video.gif.dynamic_hint")}>
              <input
                type="checkbox"
                checked={o.dynamic}
                onChange={(ev) => set({ dynamic: ev.target.checked })}
              />
              {t("ui.video.gif.dynamic")}
            </label>
          </>
        )}
        <div className="prop-field">
          <label htmlFor="gif-loops">{t("ui.video.gif.loops")}</label>
          <select
            id="gif-loops"
            value={o.loops}
            onChange={(ev) => set({ loops: ev.target.value as GifOptions["loops"] })}
          >
            <option value="forever">{t("ui.video.gif.loops_forever")}</option>
            <option value="once">{t("ui.video.gif.loops_once")}</option>
            <option value="times">{t("ui.video.gif.loops_times")}</option>
          </select>
        </div>
        {o.loops === "times" && (
          <div className="prop-field">
            <label htmlFor="gif-times">{t("ui.video.gif.times")}</label>
            <input
              id="gif-times"
              type="number"
              min={2}
              max={100}
              value={o.times}
              onChange={(ev) =>
                set({ times: Math.min(100, Math.max(2, Number(ev.target.value) || 2)) })
              }
            />
          </div>
        )}
        <label className="panel-check">
          <input
            type="checkbox"
            checked={o.fade}
            onChange={(ev) => set({ fade: ev.target.checked })}
          />
          {t("ui.video.gif.fade")}
        </label>
        {o.fade && (
          <div className="prop-field">
            <label htmlFor="gif-fade">{t("ui.video.gif.fade_seconds")}</label>
            <input
              id="gif-fade"
              type="number"
              min={0.5}
              max={3}
              step={0.5}
              value={o.fadeSeconds}
              onChange={(ev) =>
                set({ fadeSeconds: Math.min(3, Math.max(0.5, Number(ev.target.value) || 1)) })
              }
            />
          </div>
        )}
      </div>
      <div className="gif-estimate" aria-live="polite">
        {e ? (
          <>
            <img className="gif-preview" src={e.preview} alt={t("ui.video.gif.preview")} />
            <p>
              {t("ui.video.gif.estimate", {
                size: megabytes(e.bytes, i18n.language),
                seconds: e.duration.toFixed(1),
                width: e.width,
                height: e.height,
                fps: e.fps,
              })}
            </p>
          </>
        ) : (
          <p className="muted">{t("ui.video.gif.estimating")}</p>
        )}
        {/* The size guide (VID-068): a warning, never a block. */}
        {e && (e.long || e.heavy) && <p className="warning-text">{t("ui.video.gif.guide")}</p>}
      </div>
    </Dialog>
  );
}

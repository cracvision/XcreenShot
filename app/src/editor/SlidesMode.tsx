// Narrated slides (VID-071 to VID-074): the editor shows the slide as it will be in
// the video, a strip with all of them below, and the slides bar instead of the
// toolbar. What is recorded is the slide, not the screen.
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { clock } from "../windows/recClock";
import { CameraControl } from "../windows/RecordCamera";
import { MicControl } from "../windows/RecordSound";
import { useLevels } from "../windows/recSound";
import {
  place,
  pollRecording,
  SLIDE_SIZES,
  slideKey,
  useSlides,
  type SlidesMode as Mode,
} from "./slidesState";

const images = new Map<string, Promise<string>>();

/** A document's final image as a data URL, at most `max` pixels across (cached). */
function useImage(documentId: string, max: number): string | null {
  const [url, setUrl] = useState<string | null>(null);
  useEffect(() => {
    let alive = true;
    const key = `${documentId}:${max}`;
    let p = images.get(key);
    if (!p) {
      p = run<{ image: { mimeType: string; data: string } }>("document.image", {
        documentId,
        maxSize: max,
      }).then((r) => `data:${r.image.mimeType};base64,${r.image.data}`);
      images.set(key, p);
      p.catch(() => images.delete(key));
    }
    void p
      .then((u) => {
        if (alive) setUrl(u);
      })
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [documentId, max]);
  return url;
}

const go = (to: string | number) => void run("slides.go", { to }).catch(() => undefined);
const act = (command: string, params: Record<string, unknown> = {}) =>
  void run(command, params).catch(() => undefined);

export function SlidesMode({ mode }: { mode: Mode }) {
  const { t } = useTranslation();
  const rec = useSlides((s) => s.rec);
  const since = useSlides((s) => s.since);
  const [now, setNow] = useState(() => Date.now());
  const [asking, setAsking] = useState(false);
  const recording = rec.status === "recording";
  const paused = rec.status === "paused";
  const busy = recording || paused || rec.status === "countdown" || rec.status === "finalizing";
  const levels = useLevels(true);

  // The recording's state: from its events, and asked now and then.
  useEffect(() => {
    pollRecording();
    const poll = window.setInterval(pollRecording, 500);
    const tick = window.setInterval(() => setNow(Date.now()), 250);
    return () => {
      window.clearInterval(poll);
      window.clearInterval(tick);
    };
  }, []);
  const elapsed = rec.elapsed + (recording ? Math.max(0, now - since) / 1000 : 0);

  // The slide keys (VID-072): with the editor focused, before its own shortcuts.
  const space = useRef<{ down: boolean; panned: boolean }>({ down: false, panned: false });
  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      if (target?.closest("input, select, textarea, [role=menu]")) return;
      if (e.key === " ") {
        e.preventDefault();
        e.stopPropagation();
        if (!e.repeat) space.current = { down: true, panned: false };
        return;
      }
      if (
        (e.ctrlKey || e.metaKey) &&
        (e.key === "ArrowLeft" || e.key === "ArrowRight") &&
        !recording
      ) {
        e.preventDefault();
        e.stopPropagation();
        const to = mode.index + (e.key === "ArrowLeft" ? -1 : 1);
        if (to >= 0 && to < mode.count) act("slides.move", { from: mode.index, to });
        return;
      }
      const where = slideKey(e);
      if (!where) return;
      e.preventDefault();
      e.stopPropagation();
      if (where === "reset") act("slides.view", { reset: true });
      else go(where);
    };
    const up = (e: KeyboardEvent) => {
      if (e.key !== " " || !space.current.down) return;
      e.preventDefault();
      // Space let go without moving the slide: the next one.
      if (!space.current.panned) go("next");
      space.current = { down: false, panned: false };
    };
    window.addEventListener("keydown", down, true);
    window.addEventListener("keyup", up, true);
    return () => {
      window.removeEventListener("keydown", down, true);
      window.removeEventListener("keyup", up, true);
    };
  }, [mode.index, mode.count, recording]);

  const current = mode.slides[mode.index];
  const [w, h] = mode.size;
  const sizeKey = `${w}x${h}`;
  const listed = SLIDE_SIZES.some(([a, b]) => a === w && b === h);
  const [custom, setCustom] = useState(!listed);

  const stop = () => act("record.stop");
  const cancel = () => {
    if (elapsed > 10) setAsking(true);
    else act("record.cancel");
  };

  return (
    <div className="slides-mode">
      <div className="slides-bar" role="toolbar" aria-label={t("ui.slides.title")}>
        <label className="slides-field">
          <span>{t("ui.slides.background")}</span>
          <input
            type="color"
            value={mode.background.toLowerCase()}
            onChange={(e) => act("slides.set", { background: e.target.value.toUpperCase() })}
          />
        </label>
        <label className="slides-field">
          <span>{t("ui.slides.size")}</span>
          <select
            value={custom ? "custom" : sizeKey}
            disabled={busy}
            title={busy ? t("ui.slides.size_locked") : undefined}
            onChange={(e) => {
              if (e.target.value === "custom") return setCustom(true);
              setCustom(false);
              act("slides.set", { size: e.target.value });
            }}
          >
            {SLIDE_SIZES.map(([a, b]) => (
              <option key={`${a}x${b}`} value={`${a}x${b}`}>
                {a} × {b}
              </option>
            ))}
            <option value="custom">{t("ui.slides.custom")}</option>
          </select>
        </label>
        {custom && (
          <span className="slides-custom">
            {([0, 1] as const).map((i) => (
              <input
                key={`${i}-${sizeKey}`}
                type="number"
                min={320}
                max={3840}
                step={2}
                disabled={busy}
                aria-label={t(i === 0 ? "ui.slides.width" : "ui.slides.height")}
                defaultValue={mode.size[i]}
                onBlur={(e) => {
                  const v = Number(e.target.value);
                  if (!Number.isFinite(v) || v === mode.size[i]) return;
                  act("slides.set", { size: i === 0 ? [v, h] : [w, v] });
                }}
              />
            ))}
          </span>
        )}
        <label className="panel-check slides-check">
          <input
            type="checkbox"
            checked={mode.enlarge}
            onChange={(e) => act("slides.set", { enlarge: e.target.checked })}
          />
          {t("ui.slides.enlarge")}
        </label>
        <span className="slides-gap" />
        <MicControl levels={levels} up={false} onMenu={() => undefined} />
        <CameraControl levels={levels} up={false} onMenu={() => undefined} />
        <span className="slides-gap" />
        <button
          type="button"
          className="btn small icon"
          aria-label={t("ui.slides.prev")}
          title={`${t("ui.slides.prev")} (←)`}
          disabled={mode.index === 0}
          onClick={() => go("prev")}
        >
          ‹
        </button>
        <span className="slides-count" aria-live="polite">
          {t("ui.slides.count", { n: mode.index + 1, total: mode.count })}
        </span>
        <button
          type="button"
          className="btn small icon"
          aria-label={t("ui.slides.next")}
          title={`${t("ui.slides.next")} (→)`}
          disabled={mode.index >= mode.count - 1}
          onClick={() => go("next")}
        >
          ›
        </button>
        <span className="slides-gap" />
        <span className={`recbar-dot${recording ? " on" : ""}`} aria-hidden="true" />
        <span className="recbar-time">{clock(elapsed)}</span>
        {asking ? (
          <>
            <span className="slides-ask">
              {t("ui.recbar.cancel_lost", { time: clock(elapsed) })}
            </span>
            <button
              type="button"
              className="btn small danger"
              onClick={() => {
                setAsking(false);
                act("record.cancel");
              }}
            >
              {t("ui.recbar.cancel")}
            </button>
            <button type="button" className="btn small" autoFocus onClick={() => setAsking(false)}>
              {t("ui.recbar.keep")}
            </button>
          </>
        ) : busy ? (
          <>
            <button
              type="button"
              className="btn small"
              disabled={!recording && !paused}
              onClick={() => act(paused ? "record.resume" : "record.pause")}
            >
              {t(paused ? "ui.recbar.resume" : "ui.recbar.pause")}
            </button>
            <button type="button" className="btn small primary" onClick={stop}>
              {t("ui.recbar.stop")}
            </button>
            <button
              type="button"
              className="btn icon small"
              aria-label={t("ui.recbar.cancel")}
              title={t("ui.recbar.cancel")}
              onClick={cancel}
            >
              ×
            </button>
          </>
        ) : (
          <>
            <button
              type="button"
              className="btn small primary rec"
              onClick={() => act("record.start", { source: "slides" })}
            >
              {t("ui.recbar.record")}
            </button>
            <button type="button" className="btn small" onClick={() => act("slides.exit")}>
              {t("ui.slides.exit")}
            </button>
          </>
        )}
      </div>
      {current && (
        <Stage
          mode={mode}
          slide={current}
          spaceHeld={() => space.current.down}
          onPan={() => {
            space.current.panned = true;
          }}
        />
      )}
      <Strip mode={mode} locked={recording} />
    </div>
  );
}

/** The slide as it will be in the video (VID-074): its background, the image where it
 * goes, a line at the video's edge. Ctrl+wheel zooms at the pointer; dragging with the
 * middle button, or with Space held, moves it. */
function Stage({
  mode,
  slide,
  spaceHeld,
  onPan,
}: {
  mode: Mode;
  slide: Mode["slides"][number];
  /** Space is held: a drag moves the slide. */
  spaceHeld: () => boolean;
  /** The slide was moved while Space was held. */
  onPan: () => void;
}) {
  const { t } = useTranslation();
  const host = useRef<HTMLDivElement>(null);
  const [box, setBox] = useState({ w: 0, h: 0 });
  const url = useImage(slide.documentId, 2576);
  useEffect(() => {
    const el = host.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setBox({ w: el.clientWidth, h: el.clientHeight }));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);
  const [w, h] = mode.size;
  const k = Math.min((box.w - 32) / w, (box.h - 32) / h);
  const fw = Math.max(0, w * k);
  const fh = Math.max(0, h * k);
  const r = place(w, h, slide.width, slide.height, mode.enlarge, slide.zoom, slide.center);
  const drag = useRef<{ x: number; y: number; center: [number, number] } | null>(null);
  return (
    <div
      ref={host}
      className="slides-stage"
      onWheel={(e) => {
        if (!(e.ctrlKey || e.metaKey) || k <= 0) return;
        e.preventDefault();
        // The image's point under the pointer comes to the middle as it zooms.
        const frame = e.currentTarget.querySelector(".slides-frame")?.getBoundingClientRect();
        if (!frame) return;
        const px = (e.clientX - frame.left) / k;
        const py = (e.clientY - frame.top) / k;
        const center: [number, number] = [
          Math.min(1, Math.max(0, (px - r.x) / r.w)),
          Math.min(1, Math.max(0, (py - r.y) / r.h)),
        ];
        const zoom = Math.min(8, Math.max(1, slide.zoom * (e.deltaY < 0 ? 1.25 : 0.8)));
        act("slides.view", zoom === 1 ? { reset: true } : { zoom, center });
      }}
      onPointerDown={(e) => {
        if (e.button !== 1 && !(e.button === 0 && spaceHeld())) return;
        e.preventDefault();
        e.currentTarget.setPointerCapture(e.pointerId);
        drag.current = { x: e.clientX, y: e.clientY, center: slide.center };
      }}
      onPointerMove={(e) => {
        const d = drag.current;
        if (!d || k <= 0) return;
        onPan();
        const dx = (e.clientX - d.x) / k / r.w;
        const dy = (e.clientY - d.y) / k / r.h;
        act("slides.view", {
          center: [
            Math.min(1, Math.max(0, d.center[0] - dx)),
            Math.min(1, Math.max(0, d.center[1] - dy)),
          ],
        });
      }}
      onPointerUp={() => {
        drag.current = null;
      }}
    >
      <div
        className="slides-frame"
        role="img"
        aria-label={t("ui.slides.showing", { name: slide.name })}
        style={{ width: fw, height: fh, background: mode.background }}
      >
        {url && (
          <img
            src={url}
            alt=""
            draggable={false}
            style={{ left: r.x * k, top: r.y * k, width: r.w * k, height: r.h * k }}
          />
        )}
      </div>
    </div>
  );
}

/** The slides in order (VID-071): a click shows one; dragging reorders; × takes one
 * out (not while recording). */
function Strip({ mode, locked }: { mode: Mode; locked: boolean }) {
  const { t } = useTranslation();
  const [dragged, setDragged] = useState<number | null>(null);
  return (
    <ol className="slides-strip" aria-label={t("ui.slides.strip")}>
      {mode.slides.map((s, i) => (
        <li
          key={`${s.documentId}-${i}`}
          className={i === mode.index ? "current" : undefined}
          draggable={!locked}
          onDragStart={() => setDragged(i)}
          onDragOver={(e) => e.preventDefault()}
          onDrop={() => {
            if (dragged !== null && dragged !== i) act("slides.move", { from: dragged, to: i });
            setDragged(null);
          }}
        >
          <button
            type="button"
            className="slides-thumb"
            aria-current={i === mode.index}
            aria-label={t("ui.slides.go_to", { n: i + 1, name: s.name })}
            onClick={() => go(i)}
          >
            <Thumb documentId={s.documentId} />
            <span className="slides-num">{i + 1}</span>
          </button>
          {!locked && mode.count > 1 && (
            <button
              type="button"
              className="btn icon small slides-remove"
              aria-label={t("ui.slides.remove", { n: i + 1 })}
              title={t("ui.slides.remove", { n: i + 1 })}
              onClick={() => act("slides.remove", { index: i })}
            >
              ×
            </button>
          )}
        </li>
      ))}
    </ol>
  );
}

function Thumb({ documentId }: { documentId: string }) {
  const url = useImage(documentId, 160);
  return url ? <img src={url} alt="" draggable={false} /> : <span className="slides-thumb-empty" />;
}

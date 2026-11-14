// The recording bar (VID-011): Ready with Record and Cancel; while recording, the
// time, Pause / Resume, Stop and Cancel (VID-012 to VID-014). It never shows in the
// video, and it can be moved by its edge.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { useSetting } from "../store";
import { clock, longHint } from "./recClock";
import { CameraControl } from "./RecordCamera";
import { MicControl, SoundStrip, SystemAudioControl } from "./RecordSound";
import { barWarning, DISMISS_MS, useLevels } from "./recSound";

/** The bar's height, a warning's under it and a menu entry's, in CSS px. */
const BAR_H = 52;
const STRIP_H = 46;
const ROW_H = 34;

interface RecState {
  status: "ready" | "countdown" | "recording" | "paused" | "finalizing" | "idle";
  elapsed?: number;
  width?: number;
  height?: number;
  /** The pointer is in the video (VID-044). */
  cursor?: boolean;
  /** A wave at each click (VID-046). */
  clicks?: boolean;
  /** Live drawing is on (VID-048). */
  drawing?: boolean;
  /** The system's sound, chosen while Ready (VID-040). */
  systemAudio?: boolean;
  /** The Ready summary (VID-006): frames, codec and minutes that fit (VID-023). */
  fps?: number;
  codec?: string;
  minutesFree?: number | null;
  /** The video is smaller than the area (VID-028), whose size this is. */
  reduced?: boolean;
  areaWidth?: number;
  areaHeight?: number;
  /** Open in (VID-052): the editor, or only the library. */
  open?: "editor" | "none";
}

export function RecordBar() {
  const { t } = useTranslation();
  const [state, setState] = useState<RecState>({ status: "ready" });
  // Restart or Cancel waiting for a yes, after 10 s recorded (VID-014).
  const [asking, setAsking] = useState<"cancel" | "restart" | "image" | null>(null);
  // Live drawing (VID-048): its button, when the Video tab allows it (VID-051).
  const liveDraw = useSetting("video.liveDraw", true);
  const [drawing, setDrawing] = useState(false);
  useEffect(() => {
    const off = listen<{ on: boolean }>("xs://livedraw", (e) => setDrawing(e.payload.on));
    return () => void off.then((f) => f());
  }, []);
  // When the last state arrived, and now: the time moves on between updates.
  const [since, setSince] = useState(0);
  const [now, setNow] = useState(0);
  // The sound's meters and warnings (VID-011, VID-019); the microphone's menu open.
  const active = ["ready", "recording", "paused"].includes(state.status);
  const levels = useLevels(active);
  const alerts = useSetting("video.mutedMicAlert", true);
  const [dismissed, setDismissed] = useState<Partial<Record<string, number>>>({});
  const [menuRows, setMenuRows] = useState(0);
  // Low on its monitor, the bar grows upwards (for the menu and the warning).
  const [up] = useState(() => {
    const s = window.screen as Screen & { availTop?: number };
    return window.screenY - (s.availTop ?? 0) + BAR_H / 2 > s.availHeight / 2;
  });
  const raw = barWarning(levels, alerts);
  const closedAt = raw ? dismissed[raw] : undefined;
  const warning = raw && !(closedAt !== undefined && now - closedAt < DISMISS_MS) ? raw : null;
  const height = BAR_H + (warning ? STRIP_H : 0) + (menuRows ? menuRows * ROW_H + 24 : 0);
  useEffect(() => {
    void invoke("xs_recbar_fit", { height, up }).catch(() => undefined);
  }, [height, up]);

  useEffect(() => {
    document.documentElement.classList.add("launcher-root");
    const take = (s: RecState) => {
      const t = Date.now();
      setSince(t);
      setNow(t);
      setState(s);
    };
    void run<RecState>("record.status")
      .then(take)
      .catch(() => undefined);
    const offs = [
      listen<RecState>("xs://recbar", (e) => take(e.payload)),
      listen<RecState>("xs://record", (e) => take(e.payload)),
    ];
    const timer = window.setInterval(() => setNow(Date.now()), 250);
    return () => {
      window.clearInterval(timer);
      offs.forEach((o) => void o.then((f) => f()));
    };
  }, []);

  const elapsed =
    (state.elapsed ?? 0) + (state.status === "recording" ? Math.max(0, now - since) / 1000 : 0);
  const act = (command: string) => void run(command).catch(() => undefined);
  const recording = state.status === "recording";
  const paused = state.status === "paused";
  const ready = state.status === "ready";
  // Losing up to 10 s needs no question (VID-014).
  const drop = (what: "cancel" | "restart" | "image") => {
    if (elapsed > 10) setAsking(what);
    else act(`record.${what}`);
  };
  const fits = (m: number) => (m >= 120 ? `${Math.floor(m / 60)} h` : `${m} min`);
  // The Ready summary (VID-006): it gives way first when the bar is short of room.
  const summary =
    state.minutesFree != null && state.minutesFree < 10
      ? t("ui.recbar.little_space", { minutes: state.minutesFree })
      : t(state.open === "none" ? "ui.recbar.summary_library" : "ui.recbar.summary_editor", {
          fps: state.fps ?? 0,
          codec: state.codec ?? "H.264",
        });

  const strip = warning && (
    <SoundStrip
      warning={warning}
      onDismiss={() => setDismissed((d) => ({ ...d, [warning]: now }))}
    />
  );
  const stack = (bar: React.ReactNode) => (
    <div className={`recbar-stack${up ? " up" : ""}`}>
      {bar}
      {strip}
    </div>
  );

  if (asking) {
    const question = t(
      asking === "cancel"
        ? "ui.recbar.cancel_lost"
        : asking === "image"
          ? "ui.recbar.image_confirm"
          : "ui.recbar.restart_confirm",
      { time: clock(elapsed) },
    );
    return stack(
      <div
        className="recbar glass"
        role="alertdialog"
        aria-label={question}
        onKeyDown={(e) => e.key === "Escape" && setAsking(null)}
      >
        <span className="recbar-ask">{question}</span>
        <button
          type="button"
          className="btn small danger"
          onClick={() => {
            setAsking(null);
            act(`record.${asking}`);
          }}
        >
          {t(`ui.recbar.${asking}`)}
        </button>
        <button type="button" className="btn small" autoFocus onClick={() => setAsking(null)}>
          {t("ui.recbar.keep")}
        </button>
      </div>,
    );
  }

  return stack(
    <div
      className="recbar glass"
      role="toolbar"
      aria-label={t("ui.recbar.title")}
      onKeyDown={(e) => {
        // Esc on the bar: cancels when Ready, asks while recording (VID-014). Esc in
        // the app being recorded never reaches here.
        // I: an image of the area instead (VID-010).
        if ((e.key === "i" || e.key === "I") && (ready || recording || paused)) {
          e.preventDefault();
          if (ready) act("record.image");
          else drop("image");
          return;
        }
        if (e.key !== "Escape") return;
        e.preventDefault();
        if (ready) act("record.cancel");
        else if (recording || paused) drop("cancel");
      }}
    >
      <span className="recbar-grip" data-tauri-drag-region aria-hidden="true">
        ⋮⋮
      </span>
      <span
        className={`recbar-dot${recording ? " on" : ""}`}
        role="img"
        aria-label={t(`ui.recbar.${ready ? "ready" : paused ? "paused" : "recording"}`)}
      />
      <span className="recbar-time" aria-live="off">
        {clock(elapsed)}
      </span>
      {longHint(elapsed, recording || paused) ? (
        // An hour: stop and go on in another clip, they combine later (VID-023).
        <span className="recbar-size recbar-hint" role="status">
          {t("ui.recbar.long")}
        </span>
      ) : (
        state.width &&
        state.height && (
          <span
            className="recbar-size keep"
            title={
              ready && state.reduced
                ? t("ui.recbar.reduced_help", {
                    width: state.areaWidth ?? 0,
                    height: state.areaHeight ?? 0,
                  })
                : undefined
            }
          >
            {t(ready && state.reduced ? "ui.recbar.size_reduced" : "ui.recbar.size", {
              width: state.width,
              height: state.height,
            })}
          </span>
        )
      )}
      {ready && state.fps && (
        <span
          className={`recbar-size${state.minutesFree != null && state.minutesFree < 10 ? " low" : ""}`}
          title={[
            summary,
            state.minutesFree != null ? t("ui.recbar.fits", { time: fits(state.minutesFree) }) : "",
          ]
            .filter(Boolean)
            .join(" · ")}
        >
          {summary}
        </span>
      )}
      <span className="recbar-fill" data-tauri-drag-region />
      <button
        type="button"
        className="btn small recbar-toggle"
        aria-pressed={state.cursor !== false}
        title={t("ui.recbar.cursor")}
        onClick={() =>
          void run<{ cursor: boolean }>("record.cursor")
            .then((r) => setState((s) => ({ ...s, cursor: r.cursor })))
            .catch(() => undefined)
        }
      >
        {t("ui.recbar.cursor")}
      </button>
      <button
        type="button"
        className="btn small recbar-toggle"
        aria-pressed={state.clicks === true && state.cursor !== false}
        title={t("ui.recbar.clicks")}
        onClick={() => {
          const on = !(state.clicks === true && state.cursor !== false);
          void run<{ clicks: boolean; cursor: boolean }>("record.effects", { clicks: on })
            .then((r) => setState((s) => ({ ...s, clicks: r.clicks, cursor: r.cursor })))
            .catch(() => undefined);
        }}
      >
        {t("ui.recbar.clicks")}
      </button>
      {liveDraw && (recording || paused) && (
        <button
          type="button"
          className="btn small recbar-toggle"
          aria-pressed={drawing}
          title={t("ui.livedraw.title")}
          onClick={() =>
            void run<{ on: boolean }>("record.draw", { on: !drawing })
              .then((r) => setDrawing(r.on))
              .catch(() => undefined)
          }
        >
          {t("ui.livedraw.button")}
        </button>
      )}
      {active && <CameraControl levels={levels} up={up} onMenu={setMenuRows} />}
      {active && <SystemAudioControl levels={levels} />}
      {active && <MicControl levels={levels} up={up} onMenu={setMenuRows} />}
      {state.status === "countdown" && (
        <span className="recbar-size" role="status">
          {t("ui.recbar.countdown")}
        </span>
      )}
      {ready && (
        <button
          type="button"
          className="btn small primary rec"
          autoFocus
          onClick={() => act("record.start")}
        >
          {t("ui.recbar.record")}
        </button>
      )}
      {(recording || paused) && (
        <>
          <button
            type="button"
            className="btn small"
            title="Ctrl+Shift+F9"
            onClick={() => act(paused ? "record.resume" : "record.pause")}
          >
            {t(paused ? "ui.recbar.resume" : "ui.recbar.pause")}
          </button>
          <button
            type="button"
            className="btn small primary"
            title="Ctrl+Shift+F10"
            autoFocus
            onClick={() => act("record.stop")}
          >
            {t("ui.recbar.stop")}
          </button>
          <button
            type="button"
            className="btn icon small"
            aria-label={t("ui.recbar.restart")}
            title={t("ui.recbar.restart")}
            onClick={() => drop("restart")}
          >
            ↺
          </button>
        </>
      )}
      {(ready || recording || paused) && (
        <button
          type="button"
          className="btn small"
          title={`${t("ui.recbar.image")} (I)`}
          onClick={() => (ready ? act("record.image") : drop("image"))}
        >
          {t("ui.recbar.image")}
        </button>
      )}
      <button
        type="button"
        className="btn icon small"
        aria-label={t("ui.recbar.cancel")}
        title={t("ui.recbar.cancel")}
        onClick={() => (ready ? act("record.cancel") : drop("cancel"))}
      >
        ×
      </button>
    </div>,
  );
}

/** The border around the recorded area (VID-020): it lets clicks through. */
export function RecordFrame() {
  const [status, setStatus] = useState<RecState["status"]>("ready");
  useEffect(() => {
    document.documentElement.classList.add("launcher-root");
    void run<RecState>("record.status")
      .then((s) => setStatus(s.status))
      .catch(() => undefined);
    const offs = [
      listen<RecState>("xs://recbar", (e) => setStatus(e.payload.status)),
      listen<RecState>("xs://record", (e) => setStatus(e.payload.status)),
    ];
    return () => offs.forEach((o) => void o.then((f) => f()));
  }, []);
  return <div className={`rec-frame ${status}`} aria-hidden="true" />;
}

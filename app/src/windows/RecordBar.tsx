// The recording bar (VID-011): Ready with Record and Cancel; while recording, the
// time, Pause / Resume, Stop and Cancel (VID-012 to VID-014). It never shows in the
// video, and it can be moved by its edge.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { clock } from "./recClock";

interface RecState {
  status: "ready" | "countdown" | "recording" | "paused" | "finalizing" | "idle";
  elapsed?: number;
  width?: number;
  height?: number;
  /** The pointer is in the video (VID-044). */
  cursor?: boolean;
  /** The system's sound, chosen while Ready (VID-040). */
  systemAudio?: boolean;
  /** The Ready summary (VID-006): frames, codec and minutes that fit (VID-023). */
  fps?: number;
  codec?: string;
  minutesFree?: number | null;
}

export function RecordBar() {
  const { t } = useTranslation();
  const [state, setState] = useState<RecState>({ status: "ready" });
  // Restart or Cancel waiting for a yes, after 10 s recorded (VID-014).
  const [asking, setAsking] = useState<"cancel" | "restart" | "image" | null>(null);
  // When the last state arrived, and now: the time moves on between updates.
  const [since, setSince] = useState(0);
  const [now, setNow] = useState(0);

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

  if (asking) {
    const question = t(
      asking === "cancel"
        ? "ui.recbar.cancel_lost"
        : asking === "image"
          ? "ui.recbar.image_confirm"
          : "ui.recbar.restart_confirm",
      { time: clock(elapsed) },
    );
    return (
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
      </div>
    );
  }

  return (
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
      {state.width && state.height && (
        <span className="recbar-size">
          {t("ui.recbar.size", { width: state.width, height: state.height })}
        </span>
      )}
      {ready && state.fps && (
        <span
          className={`recbar-size${state.minutesFree != null && state.minutesFree < 10 ? " low" : ""}`}
          title={
            state.minutesFree != null
              ? t("ui.recbar.fits", { time: fits(state.minutesFree) })
              : undefined
          }
        >
          {state.minutesFree != null && state.minutesFree < 10
            ? t("ui.recbar.little_space", { minutes: state.minutesFree })
            : t("ui.recbar.summary", { fps: state.fps, codec: state.codec ?? "H.264" })}
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
      {ready && (
        <button
          type="button"
          className="btn small recbar-toggle"
          aria-pressed={state.systemAudio === true}
          title={t("ui.recbar.sound")}
          onClick={() => {
            const on = state.systemAudio !== true;
            void run("settings.set", { key: "video.systemAudio", value: on })
              .then(() => setState((s) => ({ ...s, systemAudio: on })))
              .catch(() => undefined);
          }}
        >
          {t("ui.recbar.sound")}
        </button>
      )}
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
    </div>
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

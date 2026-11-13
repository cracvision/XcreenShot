// The recording bar (VID-011): Ready with Record and Cancel; while recording, the
// time, Pause / Resume, Stop and Cancel (VID-012 to VID-014). It never shows in the
// video, and it can be moved by its edge.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { clock } from "./recClock";

interface RecState {
  status: "ready" | "recording" | "paused" | "finalizing" | "idle";
  elapsed?: number;
  width?: number;
  height?: number;
  /** The pointer is in the video (VID-044). */
  cursor?: boolean;
}

export function RecordBar() {
  const { t } = useTranslation();
  const [state, setState] = useState<RecState>({ status: "ready" });
  const [asking, setAsking] = useState(false);
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

  if (asking) {
    return (
      <div className="recbar glass" role="alertdialog" aria-label={t("ui.recbar.cancel_confirm")}>
        <span className="recbar-ask">{t("ui.recbar.cancel_confirm")}</span>
        <button type="button" className="btn small danger" onClick={() => act("record.cancel")}>
          {t("ui.recbar.cancel")}
        </button>
        <button type="button" className="btn small" autoFocus onClick={() => setAsking(false)}>
          {t("ui.recbar.keep")}
        </button>
      </div>
    );
  }

  return (
    <div className="recbar glass" role="toolbar" aria-label={t("ui.recbar.title")}>
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
        </>
      )}
      <button
        type="button"
        className="btn icon small"
        aria-label={t("ui.recbar.cancel")}
        title={t("ui.recbar.cancel")}
        onClick={() => (ready ? act("record.cancel") : setAsking(true))}
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

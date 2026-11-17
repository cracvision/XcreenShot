// The recording bar (VID-011): Ready with Record and Cancel; while recording, the
// time, Pause / Resume, Stop and Cancel (VID-012 to VID-014). It never shows in the
// video, and it can be moved by its edge.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { DurationField } from "../components/DurationField";
import { Menu, type MenuItem } from "../components/Menu";
import { isCommandError, run } from "../lib/commands";
import { timerProblem } from "../lib/duration";
import { useSetting } from "../store";
import { clock, longHint, timeLeft, TIMER_CHOICES } from "./recClock";
import { CameraControl } from "./RecordCamera";
import { MicControl, SoundStrip, SystemAudioControl } from "./RecordSound";
import { ReadySummary, SUMMARY_H } from "./ReadySummary";
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
  /** The timer (VID-015): seconds of recording, or none. */
  limit?: number | null;
  /** What is recorded: `region`, `window`, `screen`, `slides` (VID-003). */
  source?: { type?: string };
  /** A whole monitor and only one: a pill while recording (VID-016). */
  compact?: boolean;
}

/** The pill's width and the whole bar's, in CSS px (VID-016). */
const PILL_W = 380;
const BAR_W = 960;

/** While Ready (VID-003): the area's width and height, written, and Adjust area to move
 * or resize it on screen. The sizes shown are the even ones the video will have. */
function AreaFields({ width, height }: { width: number; height: number }) {
  const { t } = useTranslation();
  const [w, setW] = useState(String(width));
  const [h, setH] = useState(String(height));
  const commit = () => {
    const nw = Math.round(Number(w));
    const nh = Math.round(Number(h));
    if (!Number.isFinite(nw) || !Number.isFinite(nh) || (nw === width && nh === height)) return;
    void run("record.area", { width: nw, height: nh }).catch(() => {
      setW(String(width));
      setH(String(height));
    });
  };
  const field = (value: string, set: (v: string) => void, label: string) => (
    <input
      className="recbar-number"
      type="number"
      min={64}
      step={2}
      value={value}
      aria-label={label}
      title={label}
      onChange={(e) => set(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          commit();
        }
      }}
    />
  );
  return (
    <span className="recbar-area">
      {field(w, setW, t("ui.recbar.width"))}
      <span aria-hidden="true">×</span>
      {field(h, setH, t("ui.recbar.height"))}
      <button
        type="button"
        className="btn small"
        title={t("ui.recbar.adjust_area_help")}
        onClick={() => void run("record.area", { adjust: true }).catch(() => undefined)}
      >
        {t("ui.recbar.adjust_area")}
      </button>
    </span>
  );
}

export function RecordBar() {
  const { t } = useTranslation();
  const [state, setState] = useState<RecState>({ status: "ready" });
  // Restart or Cancel waiting for a yes, after 10 s recorded (VID-014).
  const [asking, setAsking] = useState<"cancel" | "restart" | "image" | null>(null);
  // The pill opened into the whole bar (VID-016).
  const [expanded, setExpanded] = useState(false);
  // A timer of one's own being written (VID-015): its seconds, and why it cannot be.
  const [timing, setTiming] = useState<number | null>(null);
  const [timingError, setTimingError] = useState<string | null>(null);
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
  // The microphone and the system's sound in tracks of their own (VID-042).
  const combineTracks = useSetting("video.combineAudioTracks", true);
  const micOn = useSetting("video.microphone", true);
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
  // The Ready summary beside the bar (VID-006), unless the preference hides it; a menu,
  // a question or the timer's fields take its place.
  const readySummary = useSetting("video.readySummary", true);
  const summaryOn =
    state.status === "ready" && readySummary && menuRows === 0 && !asking && timing === null;
  const height =
    BAR_H +
    (warning ? STRIP_H : 0) +
    (menuRows ? menuRows * ROW_H + 24 : 0) +
    (summaryOn ? SUMMARY_H + 6 : 0);
  // The pill (VID-016): a whole monitor and only one, while recording.
  const pill =
    !!state.compact && !expanded && (state.status === "recording" || state.status === "paused");
  const width = pill ? PILL_W : BAR_W;
  useEffect(() => {
    void invoke("xs_recbar_fit", { height, up, width }).catch(() => undefined);
  }, [height, up, width]);

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
  const act = (command: string, params: Record<string, unknown> = {}) =>
    void run(command, params).catch(() => undefined);
  const recording = state.status === "recording";
  const paused = state.status === "paused";
  const ready = state.status === "ready";
  // Restart and Cancel from the icon's menu come here, to be asked the same way
  // (VID-020, VID-014).
  useEffect(() => {
    const off = listen<"cancel" | "restart">("xs://recbar-ask", (e) => {
      const what = e.payload;
      void run<RecState>("record.status")
        .then((s) => {
          if ((s.elapsed ?? 0) > 10) {
            setMenuRows(0);
            setAsking(what);
          } else void run(`record.${what}`).catch(() => undefined);
        })
        .catch(() => undefined);
    });
    return () => void off.then((f) => f());
  }, []);
  // The shortcut Recording bar gives it the keyboard (VID-011, VID-075): Tab and the
  // arrows go through its controls, and Esc gives the keyboard back to the app that had
  // it (provisional: spec/QUESTIONS.md, VID-011). Clicked on, Esc cancels (VID-014).
  const byKeys = useRef(false);
  useEffect(() => {
    const off = listen("xs://recbar-focus", () => {
      byKeys.current = true;
      document.querySelector<HTMLElement>(".recbar button:not([disabled])")?.focus();
    });
    const lost = () => {
      byKeys.current = false;
    };
    window.addEventListener("blur", lost);
    return () => {
      void off.then((f) => f());
      window.removeEventListener("blur", lost);
    };
  }, []);
  const keys = (e: KeyboardEvent<HTMLElement>): boolean => {
    if (e.key === "Escape" && byKeys.current) {
      e.preventDefault();
      byKeys.current = false;
      void run("record.focus_bar", { back: true }).catch(() => undefined);
      return true;
    }
    if (
      (e.key === "ArrowLeft" || e.key === "ArrowRight") &&
      e.target instanceof HTMLButtonElement
    ) {
      const all = [...e.currentTarget.querySelectorAll<HTMLElement>("button:not([disabled])")];
      const at = all.indexOf(e.target);
      const next = all[(at + (e.key === "ArrowRight" ? 1 : -1) + all.length) % all.length];
      if (next) {
        e.preventDefault();
        next.focus();
      }
      return true;
    }
    return false;
  };
  // Losing up to 10 s needs no question (VID-014).
  const drop = (what: "cancel" | "restart" | "image") => {
    if (elapsed > 10) {
      // The menu gives way to the question without saying it closed.
      setMenuRows(0);
      setAsking(what);
    } else act(`record.${what}`);
  };
  const fits = (m: number) => (m >= 120 ? `${Math.floor(m / 60)} h` : `${m} min`);
  // The Ready summary (VID-006): it gives way first when the bar is short of room.
  const apartTracks = !combineTracks && micOn && state.systemAudio === true;
  const summary =
    state.minutesFree != null && state.minutesFree < 10
      ? t("ui.recbar.little_space", { minutes: state.minutesFree })
      : t(state.open === "none" ? "ui.recbar.summary_library" : "ui.recbar.summary_editor", {
          fps: state.fps ?? 0,
          codec: state.codec ?? "H.264",
        }) + (apartTracks ? ` · ${t("ui.recbar.tracks_apart")}` : "");

  // The timer (VID-015): no limit or a quick choice; during the recording, not less
  // than what is recorded already.
  const left = timeLeft(state.limit, elapsed);
  const timerItems: MenuItem[] = [
    {
      id: "none",
      label: t("ui.recbar.timer_none"),
      checked: !state.limit,
      onSelect: () => act("record.timer", { seconds: null }),
    },
    ...TIMER_CHOICES.map((s) => {
      const short = (recording || paused) && s <= elapsed;
      return {
        id: `t${s}`,
        label: clock(s),
        checked: state.limit === s,
        disabled: short,
        ...(short ? { title: t("ui.recbar.timer_short") } : {}),
        onSelect: () => act("record.timer", { seconds: s }),
      };
    }),
    {
      id: "own",
      label: t("ui.recbar.timer_own"),
      separator: true,
      checked: !!state.limit && !TIMER_CHOICES.includes(state.limit),
      onSelect: () => {
        // The menu gives way to the fields without saying it closed.
        setMenuRows(0);
        setTimingError(null);
        setTiming(state.limit || 300);
      },
    },
  ];
  const applyTiming = () => {
    if (timing === null) return;
    const problem = timerProblem(timing, recording || paused ? elapsed : 0);
    if (problem) {
      setTimingError(
        problem === "range"
          ? t("errors.timer_range")
          : t("errors.timer_too_short", { recorded: clock(elapsed) }),
      );
      return;
    }
    void run("record.timer", { seconds: timing })
      .then(() => setTiming(null))
      .catch((e: unknown) => setTimingError(isCommandError(e) ? e.message : String(e)));
  };

  // The bar's More menu (VID-011).
  const cleanDesktop = useSetting("video.hideDesktopIcons", false);
  const moreItems: MenuItem[] = [
    ...(recording || paused
      ? [{ id: "restart", label: t("ui.recbar.restart"), onSelect: () => drop("restart") }]
      : []),
    {
      id: "cancel",
      label: t("ui.recbar.cancel"),
      onSelect: () => (ready ? act("record.cancel") : drop("cancel")),
    },
    ...(ready || recording || paused
      ? [
          {
            id: "image",
            label: t("ui.recbar.capture_image"),
            shortcut: "I",
            separator: true,
            onSelect: () => (ready ? act("record.image") : drop("image")),
          },
        ]
      : []),
    {
      id: "clean",
      label: t("ui.recbar.clean_desktop"),
      checked: cleanDesktop,
      separator: true,
      onSelect: () => act("record.clean_desktop", { on: !cleanDesktop }),
    },
    {
      id: "settings",
      label: t("ui.recbar.video_settings"),
      onSelect: () => act("settings.open", { section: "video" }),
    },
  ];

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
      {summaryOn && (
        <ReadySummary
          state={state}
          onRecord={() => act("record.start")}
          onImage={() => act("record.image")}
          onCancel={() => act("record.cancel")}
        />
      )}
    </div>
  );

  if (timing !== null) {
    return stack(
      <div
        className="recbar glass"
        role="dialog"
        aria-label={t("ui.recbar.timer")}
        onKeyDown={(e) => {
          if (e.key !== "Escape") return;
          e.preventDefault();
          setTiming(null);
        }}
      >
        <span className="recbar-grip" data-tauri-drag-region aria-hidden="true">
          ⋮⋮
        </span>
        {(recording || paused) && (
          <span className="recbar-time" aria-live="off">
            {clock(elapsed)}
          </span>
        )}
        <span className="recbar-ask recbar-timing">
          <span>{t("ui.recbar.timer")}</span>
          <DurationField
            value={timing}
            label={t("ui.recbar.timer")}
            autoFocus
            onEnter={applyTiming}
            onChange={(v) => {
              setTiming(v);
              setTimingError(null);
            }}
          />
          {timingError && (
            <span className="recbar-timing-error" role="alert">
              {timingError}
            </span>
          )}
        </span>
        <button type="button" className="btn small primary" onClick={applyTiming}>
          {t("ui.recbar.timer_apply")}
        </button>
        <button type="button" className="btn small" onClick={() => setTiming(null)}>
          {t("ui.common.cancel")}
        </button>
      </div>,
    );
  }

  if (pill && !asking && timing === null)
    return stack(
      <div
        className="recbar glass recbar-pill"
        role="toolbar"
        aria-label={t("ui.recbar.title")}
        onKeyDown={(e) => {
          if (keys(e) || e.key !== "Escape") return;
          e.preventDefault();
          drop("cancel");
        }}
      >
        <span className="recbar-grip" data-tauri-drag-region aria-hidden="true">
          ⋮⋮
        </span>
        <span
          className={`recbar-dot${recording ? " on" : ""}`}
          role="img"
          aria-label={t(`ui.recbar.${paused ? "paused" : "recording"}`)}
        />
        <button
          type="button"
          className="btn small recbar-time"
          title={t("ui.recbar.expand")}
          aria-expanded={false}
          onClick={() => setExpanded(true)}
        >
          {clock(elapsed)}
        </button>
        <span className="recbar-fill" data-tauri-drag-region />
        <button
          type="button"
          className="btn small"
          title={`${t(paused ? "ui.recbar.resume" : "ui.recbar.pause")} (Ctrl+Shift+F9)`}
          onClick={() => act(paused ? "record.resume" : "record.pause")}
        >
          {t(paused ? "ui.recbar.resume" : "ui.recbar.pause")}
        </button>
        <button
          type="button"
          className="btn small primary"
          title={`${t("ui.recbar.stop")} (Ctrl+Shift+F10)`}
          onClick={() => act("record.stop")}
        >
          {t("ui.recbar.stop")}
        </button>
        {liveDraw && (
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
      </div>,
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
        if (keys(e)) return;
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
      {state.compact && (recording || paused) ? (
        // The whole bar opened from the pill: a click makes it small again (VID-016).
        <button
          type="button"
          className="btn small recbar-time"
          title={t("ui.recbar.collapse")}
          aria-expanded={true}
          onClick={() => setExpanded(false)}
        >
          {clock(elapsed)}
        </button>
      ) : (
        <span className="recbar-time" aria-live="off">
          {clock(elapsed)}
        </span>
      )}
      {left !== null && (recording || paused) && (
        // The time left, the last 10 s marked (VID-015).
        <span className={`recbar-left${left <= 10 ? " last" : ""}`} aria-live="off">
          {t("ui.recbar.timer_left", { time: clock(left) })}
        </span>
      )}
      {longHint(elapsed, recording || paused) ? (
        // An hour: stop and go on in another clip, they combine later (VID-023).
        <span className="recbar-size recbar-hint" role="status">
          {t("ui.recbar.long")}
        </span>
      ) : ready && state.source?.type === "region" && state.areaWidth && state.areaHeight ? (
        // Ready with a region: its size can be written, or the area adjusted (VID-003).
        <AreaFields
          key={`${state.areaWidth}x${state.areaHeight}`}
          width={state.areaWidth - (state.areaWidth % 2)}
          height={state.areaHeight - (state.areaHeight % 2)}
        />
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
            apartTracks ? t("ui.recbar.tracks_apart_hint") : "",
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
        <>
          <span className="recbar-size" role="status">
            {t("ui.recbar.countdown")}
          </span>
          {/* Record again: the countdown ends now (VID-008). */}
          <button
            type="button"
            className="btn small primary rec"
            title={t("ui.recbar.record_now_help")}
            onClick={() => act("record.toggle")}
          >
            {t("ui.recbar.record_now")}
          </button>
        </>
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
        </>
      )}
      {(ready || recording || paused) && (
        <Menu
          label={state.limit ? `⏱ ${clock(state.limit)}` : t("ui.recbar.timer")}
          ariaLabel={t("ui.recbar.timer")}
          title={t("ui.recbar.timer")}
          triggerClass={`btn small recbar-toggle${state.limit ? " on" : ""}`}
          align="end"
          side={up ? "top" : "bottom"}
          onShown={(shown) => setMenuRows(shown ? timerItems.length : 0)}
          items={timerItems}
        />
      )}
      {/* More (VID-011): Restart and Cancel (VID-014), Capture image (VID-010), the
          clean desktop (VID-018) and the video settings. */}
      <Menu
        label={<span aria-hidden="true">⋯</span>}
        ariaLabel={t("ui.recbar.more")}
        title={t("ui.recbar.more")}
        triggerClass="btn small icon"
        chevron={false}
        align="end"
        side={up ? "top" : "bottom"}
        onShown={(shown) => setMenuRows(shown ? moreItems.length : 0)}
        items={moreItems}
      />
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

// Capture panel (CAP-001, CAP-002, CAP-003, CAP-010): tabs, options, the Capture button,
// its global shortcut and the More menu. Every change is a command; the panel only
// shows what `capture.settings` returns.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { Menu } from "../components/Menu";
import { useIndicator } from "../components/useIndicator";
import { display } from "../lib/accelerator";
import { useAppShortcuts } from "../lib/appShortcuts";
import {
  isCommandError,
  run,
  type CommandEvent,
  type IntervalSeries,
  type ScheduledCapture,
  type PanelSettings,
  type ShortcutRow,
} from "../lib/commands";
import { ShortcutRecorder } from "../prefs/ShortcutRecorder";
import { useApp } from "../store";
import { ImageOptions, UniversalOptions } from "./CaptureOptions";
import { askImport, exportPresets, restoreBuiltins } from "./presetActions";
import { PresetsSection } from "./PresetsSection";
import { useFileDrop } from "./fileDrop";

type Tab = PanelSettings["tab"];
const TABS: Tab[] = ["universal", "image"];

/** Deep partial for `capture.configure` patches. */
type Patch = {
  tab?: Tab;
  expanded?: boolean;
  universal?: Partial<PanelSettings["universal"]>;
  image?: Partial<PanelSettings["image"]>;
};

export function CapturePanel() {
  const { t } = useTranslation();
  useAppShortcuts();
  const [settings, setSettings] = useState<PanelSettings | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [confirmReset, setConfirmReset] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  // Files dropped on the panel (CAP-005).
  const dropping = useFileDrop(setStatus);
  // The panel's interval series (CAP-049) and schedule (CAP-050), if any.
  const [series, setSeries] = useState<IntervalSeries | null>(null);
  const [scheduled, setScheduled] = useState<ScheduledCapture | null>(null);

  useEffect(() => {
    type Timed = { series: IntervalSeries[]; schedules: ScheduledCapture[] };
    const pick = (state: Timed) => {
      setSeries(state.series.find((x) => x.preset === "@panel") ?? null);
      setScheduled(state.schedules.find((x) => x.preset === "@panel") ?? null);
    };
    void run<Timed>("capture.interval_state").then(pick);
    const un = listen<Timed>("xs://interval", (e) => pick(e.payload));
    return () => void un.then((f) => f());
  }, []);

  const load = useCallback(() => {
    void run<PanelSettings>("capture.settings").then(setSettings);
  }, []);

  useEffect(() => {
    load();
    // Settings may also change from the CLI or an agent meanwhile.
    const un = listen<CommandEvent>("xs://command", (e) => {
      const { id, ok, source } = e.payload;
      if (ok && source !== "ui" && (id === "capture.configure" || id === "capture.reset_settings"))
        load();
    });
    window.addEventListener("focus", load);
    return () => {
      window.removeEventListener("focus", load);
      void un.then((f) => f());
    };
  }, [load]);

  // The window follows its content: compact or expanded (CAP-001).
  useLayoutEffect(() => {
    const el = root.current;
    if (!el) return;
    // Rounding at fractional scales must never leave a scroll bar.
    document.documentElement.style.overflow = "hidden";
    const fit = () =>
      void invoke("xs_panel_fit", { height: Math.ceil(el.getBoundingClientRect().height) + 2 });
    const observer = new ResizeObserver(fit);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const change = useCallback(
    async (patch: Patch) => {
      // Shown at once; the core keeps the last word.
      setSettings((s) => (s ? merge(s, patch) : s));
      try {
        setSettings(await run<PanelSettings>("capture.configure", patch));
      } catch (e) {
        setStatus(isCommandError(e) ? e.message : String(e));
        load();
      }
    },
    [load],
  );

  const capture = async () => {
    if (!settings) return;
    setStatus(null);
    try {
      await run(settings.tab === "universal" ? "capture.universal" : "capture.image");
    } catch (e) {
      // Cancelling is not an error worth showing.
      if (isCommandError(e) && e.code === "cancelled") return;
      setStatus(isCommandError(e) ? e.message : String(e));
    }
  };

  const reset = async () => {
    setConfirmReset(false);
    setSettings(await run<PanelSettings>("capture.reset_settings"));
  };

  if (!settings) return <div className="panel" ref={root} />;
  const tab = settings.tab;
  // Timing = Interval: Start, and Stop while its series runs (CAP-049).
  const interval = tab === "image" && settings.image.timing === "interval";
  // Timing = Scheduled: Schedule, and Cancel schedule while it waits (CAP-050).
  const schedule = tab === "image" && settings.image.timing === "scheduled";
  const label = series
    ? t("ui.panel.stop")
    : scheduled
      ? t("ui.panel.unschedule")
      : interval
        ? t("ui.panel.start")
        : schedule
          ? t("ui.panel.schedule")
          : t("ui.panel.capture");

  return (
    <div className={`panel${dropping ? " dropping" : ""}`} ref={root}>
      <div className="panel-top">
        <Tabs value={tab} onChange={(next) => void change({ tab: next })} />
        <button
          type="button"
          className="btn icon"
          aria-expanded={settings.expanded}
          aria-controls="panel-options"
          title={settings.expanded ? t("ui.panel.hide_options") : t("ui.panel.show_options")}
          aria-label={settings.expanded ? t("ui.panel.hide_options") : t("ui.panel.show_options")}
          onClick={() => void change({ expanded: !settings.expanded })}
        >
          <span aria-hidden="true">{settings.expanded ? "▴" : "▾"}</span>
        </button>
        <MoreMenu onReset={() => setConfirmReset(true)} />
      </div>

      {settings.expanded && (
        <div
          id="panel-options"
          className="panel-options"
          role="tabpanel"
          aria-labelledby={`tab-${tab}`}
        >
          {tab === "universal" ? (
            <UniversalOptions
              value={settings.universal}
              onChange={(u) => void change({ universal: u })}
            />
          ) : (
            <ImageOptions value={settings.image} onChange={(i) => void change({ image: i })} />
          )}
        </div>
      )}

      <div className="panel-bottom">
        <button
          type="button"
          className="btn primary panel-capture"
          onClick={() =>
            void (series
              ? run("capture.interval_stop", { id: series.id })
              : scheduled
                ? run("capture.schedule_cancel", { id: scheduled.id })
                : capture())
          }
        >
          {label}
        </button>
        <CaptureShortcut />
      </div>
      {scheduled && (
        <p className="panel-status info" role="status">
          {t("ui.panel.schedule_status", {
            when: new Date(scheduled.at).toLocaleString([], {
              weekday: "short",
              day: "numeric",
              month: "short",
              hour: "2-digit",
              minute: "2-digit",
            }),
          })}
        </p>
      )}
      {series && (
        <p className="panel-status info" role="status">
          {t("ui.panel.series_status", {
            count: series.count,
            time: new Date(series.nextAt).toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
              second: "2-digit",
            }),
          })}
        </p>
      )}
      {status && (
        <p className="panel-status" role="alert">
          {status}
        </p>
      )}

      <PresetsSection tab={tab} />

      {confirmReset && (
        <Dialog
          title={t("ui.panel.reset_title")}
          onCancel={() => setConfirmReset(false)}
          actions={
            <>
              <button type="button" className="btn" onClick={() => setConfirmReset(false)}>
                {t("ui.panel.cancel")}
              </button>
              <button type="button" className="btn primary" onClick={() => void reset()}>
                {t("ui.panel.reset_confirm")}
              </button>
            </>
          }
        >
          <p>{t("ui.panel.reset_body")}</p>
        </Dialog>
      )}
    </div>
  );
}

function merge(s: PanelSettings, p: Patch): PanelSettings {
  return {
    ...s,
    ...(p.tab ? { tab: p.tab } : {}),
    ...(p.expanded !== undefined ? { expanded: p.expanded } : {}),
    universal: { ...s.universal, ...p.universal },
    image: { ...s.image, ...p.image },
  };
}

/** Tabs with arrow keys, Home and End (WAI-ARIA tabs, automatic activation). */
function Tabs({ value, onChange }: { value: Tab; onChange: (t: Tab) => void }) {
  const { t } = useTranslation();
  const refs = useRef<Record<string, HTMLButtonElement | null>>({});
  const list = useRef<HTMLDivElement>(null);
  useIndicator(list, value);
  const onKey = (e: React.KeyboardEvent) => {
    const i = TABS.indexOf(value);
    const next =
      e.key === "ArrowRight"
        ? TABS[(i + 1) % TABS.length]
        : e.key === "ArrowLeft"
          ? TABS[(i + TABS.length - 1) % TABS.length]
          : e.key === "Home"
            ? TABS[0]
            : e.key === "End"
              ? TABS[TABS.length - 1]
              : null;
    if (!next) return;
    e.preventDefault();
    onChange(next);
    refs.current[next]?.focus();
  };
  return (
    <div
      className="panel-tabs seg"
      role="tablist"
      aria-label={t("ui.panel.tabs")}
      onKeyDown={onKey}
      ref={list}
    >
      <span className="seg-indicator" aria-hidden="true" />
      {TABS.map((tab) => (
        <button
          key={tab}
          id={`tab-${tab}`}
          ref={(el) => {
            refs.current[tab] = el;
          }}
          type="button"
          role="tab"
          aria-selected={value === tab}
          aria-controls="panel-options"
          tabIndex={value === tab ? 0 : -1}
          onClick={() => onChange(tab)}
        >
          {t(`ui.panel.tab_${tab}`)}
        </button>
      ))}
    </div>
  );
}

/** The global shortcut of "Capture" (CAP-006); red with the reason when it fails. */
function CaptureShortcut() {
  const { t } = useTranslation();
  const { shortcuts, layout, ping } = useApp();
  const [error, setError] = useState<string | null>(null);
  const [reassign, setReassign] = useState<{ shortcut: string; other: string } | null>(null);
  const row = shortcuts?.rows.find((r) => r.command === "capture.capture" && r.scope === "global");
  const platform = shortcuts?.platform ?? ping?.platform ?? "windows";
  if (!row) return null;

  const titleOf = (id: string) => shortcuts?.rows.find((r) => r.command === id)?.title ?? id;
  const assign = async (r: ShortcutRow, shortcut: string | null, reassignOther = false) => {
    setError(null);
    try {
      await run("shortcuts.set", {
        command: r.command,
        scope: r.scope,
        shortcut,
        reassign: reassignOther,
      });
    } catch (e) {
      if (isCommandError(e) && e.details?.["conflict"] === "duplicate" && shortcut) {
        setReassign({ shortcut, other: titleOf(String(e.details["command"])) });
      } else {
        setError(isCommandError(e) ? e.message : String(e));
      }
    }
  };
  const problem = error ?? (row.conflict ? row.statusText : null);

  return (
    <div className="panel-shortcut">
      <span className="muted" id="panel-shortcut-label">
        {t("ui.panel.shortcut")}
      </span>
      <ShortcutRecorder
        value={row.shortcut}
        platform={platform}
        layout={layout}
        conflict={row.conflict || error !== null}
        label={t("ui.panel.shortcut")}
        onCommit={(s) => void assign(row, s)}
      />
      {problem && (
        <span className="panel-shortcut-error" role="alert">
          {problem}
        </span>
      )}
      {reassign && (
        <Dialog
          title={t("ui.shortcuts.reassign_title")}
          onCancel={() => setReassign(null)}
          actions={
            <>
              <button type="button" className="btn" onClick={() => setReassign(null)}>
                {t("ui.shortcuts.cancel")}
              </button>
              <button
                type="button"
                className="btn primary"
                onClick={() => {
                  const r = reassign;
                  setReassign(null);
                  void assign(row, r.shortcut, true);
                }}
              >
                {t("ui.shortcuts.reassign")}
              </button>
            </>
          }
        >
          <p>
            {t("ui.shortcuts.reassign_body", {
              shortcut: display(reassign.shortcut, platform, layout),
              command: reassign.other,
            })}
          </p>
        </Dialog>
      )}
    </div>
  );
}

/** "More" menu (CAP-001): only the entries whose features exist. */
function MoreMenu({ onReset }: { onReset: () => void }) {
  const { t } = useTranslation();
  // The edge launcher (CAP-004): shown with the panel, or not.
  const iface = useApp((s) => s.settings.find((x) => x.key === "capture.interface")?.value);
  const launcherOn = iface === "launcher" || iface === "both";
  return (
    <div className="panel-more">
      <Menu
        triggerClass="btn icon"
        align="end"
        chevron={false}
        ariaLabel={t("ui.panel.more")}
        title={t("ui.panel.more")}
        label={<span aria-hidden="true">⋯</span>}
        items={[
          {
            id: "prefs",
            label: t("ui.panel.capture_prefs"),
            onSelect: () => void run("settings.open", { section: "capture" }),
          },
          { id: "reset", label: t("ui.panel.reset"), onSelect: onReset },
          {
            id: "launcher",
            label: launcherOn ? t("ui.panel.hide_launcher") : t("ui.panel.show_launcher"),
            onSelect: () =>
              void run("settings.set", {
                key: "capture.interface",
                value: launcherOn ? "panel" : "both",
              }),
          },
          // CAP-063, CAP-064.
          {
            id: "import_presets",
            label: t("ui.panel.presets.import"),
            separator: true,
            onSelect: () => void askImport(),
          },
          {
            id: "export_presets",
            label: t("ui.panel.presets.export_all"),
            onSelect: () => void exportPresets(null),
          },
          {
            id: "restore_presets",
            label: t("ui.panel.presets.restore_builtins"),
            onSelect: () => void restoreBuiltins(),
          },
        ]}
      />
    </div>
  );
}

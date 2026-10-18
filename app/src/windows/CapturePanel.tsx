// Capture panel (CAP-001, CAP-002, CAP-003, CAP-010): tabs, options, the Capture button,
// its global shortcut and the More menu. Every change is a command; the panel only
// shows what `capture.settings` returns.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { display } from "../lib/accelerator";
import { useAppShortcuts } from "../lib/appShortcuts";
import {
  isCommandError,
  run,
  type CommandEvent,
  type PanelSettings,
  type ShortcutRow,
} from "../lib/commands";
import { ShortcutRecorder } from "../prefs/ShortcutRecorder";
import { useApp } from "../store";

type Tab = PanelSettings["tab"];
const TABS: Tab[] = ["universal", "image"];
const SELECTIONS = ["region", "window", "fullscreen"] as const;

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

  return (
    <div className="panel" ref={root}>
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
        <button type="button" className="btn primary panel-capture" onClick={() => void capture()}>
          {t("ui.panel.capture")}
        </button>
        <CaptureShortcut />
      </div>
      {status && (
        <p className="panel-status" role="alert">
          {status}
        </p>
      )}

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
    <div className="panel-tabs" role="tablist" aria-label={t("ui.panel.tabs")} onKeyDown={onKey}>
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

interface Common {
  selection: (typeof SELECTIONS)[number];
  copy: boolean;
  cursor: boolean;
  open: "editor" | "none";
}

/** Controls shared by both tabs, in the CAP-002 order (Selection … Capture cursor). */
function CommonOptions<T extends Common>({
  value,
  onChange,
}: {
  value: T;
  onChange: (patch: Partial<Common>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  return (
    <>
      <label className="panel-field" htmlFor={`${id}-sel`}>
        <span>{t("ui.panel.selection")}</span>
        <select
          id={`${id}-sel`}
          value={value.selection}
          onChange={(e) => onChange({ selection: e.target.value as Common["selection"] })}
        >
          {SELECTIONS.map((s) => (
            <option key={s} value={s}>
              {t(`ui.panel.sel_${s}`)}
            </option>
          ))}
        </select>
      </label>
      <label className="panel-field" htmlFor={`${id}-open`}>
        <span>{t("ui.panel.open_in")}</span>
        <select
          id={`${id}-open`}
          value={value.open}
          onChange={(e) => onChange({ open: e.target.value as Common["open"] })}
        >
          <option value="editor">{t("ui.panel.open_editor")}</option>
          <option value="none">{t("ui.panel.open_none")}</option>
        </select>
      </label>
      <label className="panel-check">
        <input
          type="checkbox"
          checked={value.copy}
          onChange={(e) => onChange({ copy: e.target.checked })}
        />
        {t("ui.panel.copy")}
      </label>
      {/* CAP-047 arrives with the editor; the reason is in the tooltip (CAP-002). */}
      <label className="panel-check disabled" title={t("ui.panel.cursor_later")}>
        <input type="checkbox" checked={false} disabled aria-describedby={`${id}-cursor`} />
        {t("ui.panel.cursor")}
        <span id={`${id}-cursor`} className="sr-only">
          {t("ui.panel.cursor_later")}
        </span>
      </label>
    </>
  );
}

function UniversalOptions({
  value,
  onChange,
}: {
  value: PanelSettings["universal"];
  onChange: (patch: Partial<PanelSettings["universal"]>) => void;
}) {
  const { t } = useTranslation();
  return (
    <>
      <CommonOptions value={value} onChange={onChange} />
      <label className="panel-check">
        <input
          type="checkbox"
          checked={value.delay5}
          onChange={(e) => onChange({ delay5: e.target.checked })}
        />
        {t("ui.panel.delay5")}
      </label>
    </>
  );
}

function ImageOptions({
  value,
  onChange,
}: {
  value: PanelSettings["image"];
  onChange: (patch: Partial<PanelSettings["image"]>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  // What is being typed; the saved value shows otherwise.
  const [draft, setDraft] = useState<string | null>(null);
  const commitSeconds = () => {
    const n = Math.round(Number(draft));
    if (draft !== null && Number.isFinite(n) && n >= 1 && n <= 60) onChange({ delay: n });
    setDraft(null);
  };
  return (
    <>
      <CommonOptions value={value} onChange={onChange} />
      <div className="panel-row">
        <label className="panel-field" htmlFor={`${id}-timing`}>
          <span>{t("ui.panel.timing")}</span>
          <select
            id={`${id}-timing`}
            value={value.timing}
            onChange={(e) => onChange({ timing: e.target.value as "none" | "delay" })}
          >
            <option value="none">{t("ui.panel.timing_none")}</option>
            <option value="delay">{t("ui.panel.timing_delay")}</option>
          </select>
        </label>
        {value.timing === "delay" && (
          <label className="panel-field narrow" htmlFor={`${id}-sec`}>
            <span>{t("ui.panel.seconds")}</span>
            <input
              id={`${id}-sec`}
              type="number"
              min={1}
              max={60}
              step={1}
              value={draft ?? String(value.delay)}
              title={t("ui.panel.seconds_range")}
              onChange={(e) => setDraft(e.target.value)}
              onBlur={commitSeconds}
              onKeyDown={(e) => e.key === "Enter" && commitSeconds()}
            />
          </label>
        )}
      </div>
    </>
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
  const [open, setOpen] = useState(false);
  const button = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    menu.current?.querySelector<HTMLElement>("[role=menuitem]")?.focus();
    const close = (e: MouseEvent) => {
      const target = e.target as Node;
      if (!menu.current?.contains(target) && !button.current?.contains(target)) setOpen(false);
    };
    window.addEventListener("mousedown", close);
    return () => window.removeEventListener("mousedown", close);
  }, [open]);

  const onKey = (e: React.KeyboardEvent) => {
    const items = [...(menu.current?.querySelectorAll<HTMLElement>("[role=menuitem]") ?? [])];
    const i = items.indexOf(document.activeElement as HTMLElement);
    if (e.key === "Escape") {
      e.stopPropagation();
      setOpen(false);
      button.current?.focus();
    } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const d = e.key === "ArrowDown" ? 1 : -1;
      items[(i + d + items.length) % items.length]?.focus();
    }
  };
  const choose = (action: () => void) => {
    setOpen(false);
    button.current?.focus();
    action();
  };

  return (
    <div className="panel-more">
      <button
        ref={button}
        type="button"
        className="btn icon"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={t("ui.panel.more")}
        title={t("ui.panel.more")}
        onClick={() => setOpen((o) => !o)}
      >
        <span aria-hidden="true">⋯</span>
      </button>
      {open && (
        <div className="menu" role="menu" ref={menu} onKeyDown={onKey}>
          <button
            type="button"
            role="menuitem"
            tabIndex={-1}
            onClick={() => choose(() => void run("settings.open", { section: "capture" }))}
          >
            {t("ui.panel.capture_prefs")}
          </button>
          <button type="button" role="menuitem" tabIndex={-1} onClick={() => choose(onReset)}>
            {t("ui.panel.reset")}
          </button>
        </div>
      )}
    </div>
  );
}

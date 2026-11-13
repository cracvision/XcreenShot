// The edge launcher (CAP-004): a thin tab on a screen edge. The pointer over it
// unfolds Capture (always Universal into the editor, with the pointer) and Presets;
// the grip drags it to another edge or monitor. A new preset shows its name for a
// moment. The window's size follows what it shows; the core keeps where it is.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { run, type CommandEvent } from "../lib/commands";

type Mode = "tab" | "open" | "list";

interface PresetRow {
  id: string;
  name: string;
}

/** How long the pointer may leave before it folds back, and a new preset's name stays. */
const LEAVE_MS = 500;
const BALLOON_MS = 4000;

export function Launcher() {
  const { t } = useTranslation();
  const [mode, setMode] = useState<Mode>("tab");
  const [edge, setEdge] = useState("top");
  const [presets, setPresets] = useState<PresetRow[]>([]);
  const [balloon, setBalloon] = useState<string | null>(null);
  const dragging = useRef(false);
  const leave = useRef<number | undefined>(undefined);
  const known = useRef<string[] | null>(null);

  const go = useCallback((next: Mode, rows?: number) => {
    setMode(next);
    void invoke<{ edge: string | null }>("xs_launcher", { mode: next, rows }).then((r) => {
      if (r.edge) setEdge(r.edge);
    });
  }, []);

  const loadPresets = useCallback(() => {
    void run<{ presets: PresetRow[] }>("preset.list").then((r) => {
      const rows = r.presets.map((p) => ({ id: p.id, name: p.name }));
      // A new preset: its name in a balloon for a few seconds (CAP-004).
      const before = known.current;
      const added = before ? rows.find((p) => !before.includes(p.id)) : undefined;
      known.current = rows.map((p) => p.id);
      setPresets(rows);
      if (added) {
        setBalloon(added.name);
        go("list", 1);
        window.setTimeout(() => {
          setBalloon(null);
          if (!dragging.current) go("tab");
        }, BALLOON_MS);
      }
    });
  }, [go]);

  useEffect(() => {
    document.documentElement.classList.add("launcher-root");
    void invoke<{ edge: string | null }>("xs_launcher", { mode: "tab" }).then((r) => {
      if (r.edge) setEdge(r.edge);
    });
    loadPresets();
    const un = listen<CommandEvent>("xs://command", (e) => {
      if (e.payload.ok && e.payload.id.startsWith("preset.")) loadPresets();
    });
    return () => void un.then((f) => f());
  }, [loadPresets]);

  const enter = () => {
    window.clearTimeout(leave.current);
    if (mode === "tab") go("open");
  };
  const exit = () => {
    if (dragging.current) return;
    window.clearTimeout(leave.current);
    leave.current = window.setTimeout(() => {
      if (!balloon) go("tab");
    }, LEAVE_MS);
  };

  const grab = (e: React.PointerEvent) => {
    e.currentTarget.setPointerCapture(e.pointerId);
    dragging.current = true;
    void invoke("xs_launcher_grab");
  };
  const drag = () => {
    if (dragging.current) void invoke("xs_launcher_drag");
  };
  const drop = (e: React.PointerEvent) => {
    if (!dragging.current) return;
    dragging.current = false;
    e.currentTarget.releasePointerCapture(e.pointerId);
    void invoke<{ edge: string | null }>("xs_launcher_drop").then((r) => {
      if (r.edge) setEdge(r.edge);
      setMode("open");
    });
  };

  if (mode === "tab") {
    return (
      <div
        className={`launcher-tab ${edge}`}
        role="button"
        tabIndex={0}
        aria-label={t("ui.launcher.tab")}
        onMouseEnter={enter}
        onFocus={enter}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") go("open");
        }}
      />
    );
  }

  return (
    <div className="launcher-box" onMouseEnter={enter} onMouseLeave={exit}>
      <div className="launcher-row">
        <span
          className="launcher-grip"
          role="button"
          tabIndex={-1}
          aria-label={t("ui.launcher.move")}
          title={t("ui.launcher.move")}
          onPointerDown={grab}
          onPointerMove={drag}
          onPointerUp={drop}
        >
          ⋮⋮
        </span>
        <button
          type="button"
          className="btn primary small"
          onClick={() => {
            go("tab");
            void run("capture.universal", { fixed: true }).catch(() => undefined);
          }}
        >
          {t("ui.launcher.capture")}
        </button>
        <button
          type="button"
          className="btn small"
          aria-expanded={mode === "list"}
          onClick={() => (mode === "list" ? go("open") : go("list", Math.max(1, presets.length)))}
        >
          {t("ui.launcher.presets")} ▾
        </button>
      </div>
      {balloon && (
        <p className="launcher-balloon" role="status" title={balloon}>
          {t("ui.launcher.new", { name: balloon })}
        </p>
      )}
      {mode === "list" && !balloon && (
        <ul className="launcher-list">
          {presets.length === 0 && <li className="muted">{t("ui.launcher.none")}</li>}
          {presets.slice(0, 10).map((p) => (
            <li key={p.id}>
              <button
                type="button"
                title={p.name}
                onClick={() => {
                  go("tab");
                  void run("preset.run", { id: p.id }).catch(() => undefined);
                }}
              >
                {p.name}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

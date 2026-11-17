// Window picker (CAP-030): every window of the desktop, covered and minimized ones too,
// with its app and title. A click, or Enter on the highlighted one, captures it at its
// real size; typing filters by app or title; the arrows move; Esc cancels. The picker
// never shows in a capture.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { xsdocUrl } from "../editor/state";

interface Entry {
  id: number;
  app: string;
  title: string;
  minimized: boolean;
  thumb: boolean;
}

interface PickerState {
  open: boolean;
  windows: Entry[];
}

const fold = (s: string) => s.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

export function WindowPicker() {
  const { t } = useTranslation();
  const [state, setState] = useState<PickerState>({ open: true, windows: [] });
  const [filter, setFilter] = useState("");
  const [current, setCurrent] = useState(0);
  const grid = useRef<HTMLUListElement>(null);

  useEffect(() => {
    void run<PickerState>("capture.picker_state").then(setState);
    const un = listen<PickerState>("xs://picker", (e) => setState(e.payload));
    return () => void un.then((f) => f());
  }, []);

  const shown = useMemo(() => {
    const q = fold(filter.trim());
    return q
      ? state.windows.filter((w) => fold(w.app).includes(q) || fold(w.title).includes(q))
      : state.windows;
  }, [state.windows, filter]);
  const at = Math.min(current, Math.max(0, shown.length - 1));

  const pick = (id: number) => void run("capture.picker_choose", { id });
  const cancel = () => void run("capture.picker_choose", {});

  // How many cards fit across, for the up and down arrows.
  const across = () => {
    const el = grid.current;
    const first = el?.firstElementChild as HTMLElement | null;
    if (!el || !first) return 1;
    return Math.max(1, Math.round(el.clientWidth / (first.offsetWidth + 16)));
  };

  return (
    <div
      className="picker"
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          cancel();
        } else if (e.key === "Enter") {
          e.preventDefault();
          const w = shown[at];
          if (w) pick(w.id);
        } else if (e.key.startsWith("Arrow")) {
          e.preventDefault();
          const step =
            e.key === "ArrowLeft"
              ? -1
              : e.key === "ArrowRight"
                ? 1
                : e.key === "ArrowUp"
                  ? -across()
                  : across();
          setCurrent(Math.min(Math.max(0, at + step), Math.max(0, shown.length - 1)));
        }
      }}
    >
      <header className="picker-head">
        <h1>{t("ui.picker.title")}</h1>
        <input
          type="search"
          autoFocus
          value={filter}
          placeholder={t("ui.picker.filter")}
          aria-label={t("ui.picker.filter")}
          onChange={(e) => {
            setFilter(e.target.value);
            setCurrent(0);
          }}
        />
        <span className="muted">{t("ui.picker.hint")}</span>
        <button type="button" className="btn" onClick={cancel}>
          {t("ui.picker.cancel")} <kbd>Esc</kbd>
        </button>
      </header>
      {shown.length === 0 ? (
        <p className="picker-none muted">{t("ui.picker.none")}</p>
      ) : (
        <ul ref={grid} className="picker-grid" role="listbox" aria-label={t("ui.picker.title")}>
          {shown.map((w, i) => (
            <li key={w.id} role="option" aria-selected={i === at}>
              <button
                type="button"
                className={`picker-card${i === at ? " current" : ""}`}
                tabIndex={-1}
                onMouseEnter={() => setCurrent(i)}
                onClick={() => pick(w.id)}
              >
                <span className="picker-thumb">
                  {w.thumb ? (
                    <img src={xsdocUrl(`picker/${w.id}`)} alt="" draggable={false} />
                  ) : (
                    <span className="muted">
                      {w.minimized ? t("ui.picker.minimized") : t("ui.picker.loading")}
                    </span>
                  )}
                </span>
                <span className="picker-app">{w.app}</span>
                <span className="picker-title">{w.title}</span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

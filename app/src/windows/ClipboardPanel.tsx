// XcreenShot's clipboard (TRY-018): the recent tray's captures, to pick one or more and
// paste them into the app that was in front. A click or Space picks; Shift+click picks a
// range; Enter or Insert pastes the original images, in the tray's order; Ctrl+C only
// copies the first; Esc or leaving the panel closes it. Never shows in a capture.
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { docUrl } from "../editor/state";
import { pickRange, pickToggle, pickedInOrder } from "./clipboardPick";

interface Item {
  documentId: string;
  name: string;
  format: string;
  version: number;
  missing: boolean;
}

const fold = (s: string) => s.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
const pastable = (i: Item) => i.format !== "mp4" && i.format !== "gif" && !i.missing;

export function ClipboardPanel() {
  const { t } = useTranslation();
  const [items, setItems] = useState<Item[] | null>(null);
  const [filter, setFilter] = useState("");
  const [current, setCurrent] = useState(0);
  const [picked, setPicked] = useState<Set<string>>(new Set());
  const [anchor, setAnchor] = useState<string | null>(null);
  const [error, setError] = useState("");
  const busy = useRef(false);
  const grid = useRef<HTMLUListElement>(null);

  const close = () => void run("tray.clipboard", { open: false }).catch(() => undefined);

  useEffect(() => {
    void run<{ items: Item[] }>("tray.list").then((r) => setItems(r.items));
    // Clicking elsewhere closes it, like the system's clipboard.
    const un = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (!focused && !busy.current) close();
    });
    return () => void un.then((f) => f());
  }, []);

  const shown = useMemo(() => {
    const q = fold(filter.trim());
    const all = items ?? [];
    return q ? all.filter((i) => fold(i.name).includes(q)) : all;
  }, [items, filter]);
  const at = Math.min(current, Math.max(0, shown.length - 1));
  const order = (items ?? []).map((i) => i.documentId);

  const toggle = (i: Item, range: boolean) => {
    if (!pastable(i)) return;
    const ok = (id: string) => (items ?? []).some((x) => x.documentId === id && pastable(x));
    setPicked((p) =>
      range && anchor
        ? pickRange(p, shown.map((x) => x.documentId).filter(ok), anchor, i.documentId)
        : pickToggle(p, i.documentId),
    );
    setAnchor(i.documentId);
  };

  const chosen = (): string[] => {
    const list = pickedInOrder(picked, order);
    if (list.length > 0) return list;
    const cur = shown[at];
    return cur && pastable(cur) ? [cur.documentId] : [];
  };

  const insert = async (paste: boolean, ids = chosen()) => {
    if (ids.length === 0 || busy.current) return;
    busy.current = true;
    setError("");
    try {
      await run("tray.insert", { documentIds: ids, paste });
      if (!paste) close();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      busy.current = false;
    }
  };

  // How many cards fit across, for the up and down arrows.
  const across = () => {
    const el = grid.current;
    const first = el?.firstElementChild as HTMLElement | null;
    if (!el || !first) return 1;
    return Math.max(1, Math.round(el.clientWidth / (first.offsetWidth + 12)));
  };

  const n = chosen().length;

  return (
    <div
      className="picker clip"
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          close();
        } else if (e.key === "Enter") {
          e.preventDefault();
          void insert(true);
        } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "c") {
          e.preventDefault();
          void insert(false);
        } else if (e.key === " " && (e.target as HTMLElement).tagName !== "INPUT") {
          e.preventDefault();
          const cur = shown[at];
          if (cur) toggle(cur, e.shiftKey);
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
        <h1>{t("ui.clipboard.title")}</h1>
        <input
          type="search"
          autoFocus
          value={filter}
          placeholder={t("ui.clipboard.filter")}
          aria-label={t("ui.clipboard.filter")}
          onChange={(e) => {
            setFilter(e.target.value);
            setCurrent(0);
          }}
        />
      </header>
      {items !== null && items.length === 0 ? (
        <div className="picker-none clip-empty">
          <p className="muted">{t("ui.clipboard.none")}</p>
          <button
            type="button"
            className="btn primary"
            onClick={() => {
              close();
              void run("capture.capture").catch(() => undefined);
            }}
          >
            {t("ui.clipboard.capture")}
          </button>
        </div>
      ) : shown.length === 0 && items !== null ? (
        <p className="picker-none muted">{t("ui.clipboard.no_match")}</p>
      ) : (
        <ul
          ref={grid}
          className="picker-grid clip-grid"
          role="listbox"
          aria-multiselectable="true"
          aria-label={t("ui.clipboard.title")}
        >
          {shown.map((i, k) => {
            const ok = pastable(i);
            const on = picked.has(i.documentId);
            return (
              <li key={i.documentId} role="option" aria-selected={on} aria-disabled={!ok}>
                <button
                  type="button"
                  className={`picker-card clip-card${k === at ? " current" : ""}${on ? " picked" : ""}`}
                  tabIndex={-1}
                  disabled={!ok}
                  title={ok ? i.name : t("ui.clipboard.video")}
                  onMouseEnter={() => setCurrent(k)}
                  onClick={(e) => toggle(i, e.shiftKey)}
                  onDoubleClick={() => ok && void insert(true, [i.documentId])}
                >
                  <span className="picker-thumb clip-thumb">
                    <img src={docUrl(i.documentId, "thumb", i.version)} alt="" draggable={false} />
                    {on && (
                      <span className="clip-check" aria-label={t("ui.clipboard.picked")}>
                        ✓
                      </span>
                    )}
                  </span>
                  <span className="picker-title">{i.name}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
      <footer className="clip-foot">
        <span className="muted">{error || t("ui.clipboard.hint")}</span>
        <button type="button" className="btn" onClick={close}>
          {t("ui.clipboard.close")} <kbd>Esc</kbd>
        </button>
        <button type="button" className="btn" disabled={n === 0} onClick={() => void insert(false)}>
          {t("ui.clipboard.copy")}
        </button>
        <button
          type="button"
          className="btn primary"
          disabled={n === 0}
          onClick={() => void insert(true)}
        >
          {n > 1 ? t("ui.clipboard.insert_n", { n }) : t("ui.clipboard.insert")}
        </button>
      </footer>
    </div>
  );
}

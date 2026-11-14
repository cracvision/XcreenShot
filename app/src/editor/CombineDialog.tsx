// Combine videos (VID-067): the library's videos above (filter and order), the videos
// chosen below in the order they will play (the open one first), the total length and
// the result's size at once; Combine runs with its progress and Cancel, and the result
// opens in the editor. The originals do not change.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { isCommandError, run } from "../lib/commands";
import { libraryThumb } from "../library/quickViewState";
import type { LibraryItem } from "../library/state";
import { useEditor } from "./state";
import { useVideo } from "./videoState";
import { formatTime } from "./videoTimeline";

type Sort = "modified" | "captured" | "name" | "size";

interface Quick {
  width: number;
  height: number;
  duration: number;
}

interface Chosen {
  path: string;
  name: string;
}

export function CombineDialog() {
  const open = useVideo((s) => s.combineDialog);
  return open ? <CombineForm /> : null;
}

function CombineForm() {
  const { t } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const [items, setItems] = useState<LibraryItem[]>([]);
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<Sort>("modified");
  // The videos chosen in the library, or the video shown in the editor, go first
  // (VID-067).
  const [chosen, setChosen] = useState<Chosen[]>(() => {
    const given = useVideo.getState().combineWith;
    if (given) {
      useVideo.setState({ combineWith: null });
      return given;
    }
    return doc?.video && doc.video.codec !== "GIF" && doc.path
      ? [{ path: doc.path, name: doc.name }]
      : [];
  });
  const [quick, setQuick] = useState<Record<string, Quick>>({});
  const [progress, setProgress] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const close = () => useVideo.setState({ combineDialog: false, combineWith: null });

  // The library's videos, in the order and with the filter chosen.
  useEffect(() => {
    let live = true;
    const timer = window.setTimeout(() => {
      void run<{ items: LibraryItem[] }>("library.list", {
        sort,
        ...(sort === "name" ? { order: "asc" } : {}),
        ...(query.trim() ? { query: query.trim() } : {}),
        filters: { types: ["video"] },
      })
        .then((r) => live && setItems(r.items))
        .catch(() => undefined);
    }, 200);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [query, sort]);

  // Each chosen video's length and size, from its header.
  const missing = chosen.map((c) => c.path).filter((p) => !(p in quick));
  const missingKey = missing.join("|");
  useEffect(() => {
    if (!missingKey) return;
    let live = true;
    for (const path of missingKey.split("|"))
      void run<Quick>("video.info", { path, quick: true })
        .then((q) => live && setQuick((cur) => ({ ...cur, [path]: q })))
        .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [missingKey]);

  // The combination's progress.
  useEffect(() => {
    const off = listen<{ kind: string; payload: { documentId?: string; progress?: number } }>(
      "xs://editor",
      (e) => {
        if (e.payload.kind === "video_progress" && e.payload.payload.documentId === "combine")
          setProgress(e.payload.payload.progress ?? null);
      },
    );
    return () => void off.then((f) => f());
  }, []);

  const add = (item: LibraryItem) =>
    setChosen((cur) => [...cur, { path: item.path, name: item.name }]);
  const move = (i: number, by: -1 | 1) =>
    setChosen((cur) => {
      const j = i + by;
      if (j < 0 || j >= cur.length) return cur;
      const next = [...cur];
      [next[i], next[j]] = [next[j] as Chosen, next[i] as Chosen];
      return next;
    });
  const remove = (i: number) => setChosen((cur) => cur.filter((_, k) => k !== i));
  const known = chosen.map((c) => quick[c.path]).filter((q): q is Quick => !!q);
  const total = known.reduce((s, q) => s + q.duration, 0);
  const width = Math.max(0, ...known.map((q) => q.width));
  const height = Math.max(0, ...known.map((q) => q.height));
  const busy = progress !== null;

  const combine = async () => {
    setError(null);
    setProgress(0);
    try {
      await run("video.combine", { paths: chosen.map((c) => c.path) });
      close();
    } catch (e) {
      if (!(isCommandError(e) && e.code === "cancelled"))
        setError(isCommandError(e) ? e.message : String(e));
    } finally {
      setProgress(null);
    }
  };

  return (
    <Dialog
      title={t("ui.video.combine.title")}
      onCancel={busy ? () => undefined : close}
      actions={
        busy ? (
          <>
            <span className="vb-progress" role="status">
              {t("ui.video.combine.busy", { percent: Math.round((progress ?? 0) * 100) })}
              <progress max={1} value={progress ?? 0} />
            </span>
            <button
              type="button"
              className="btn"
              onClick={() => void run("video.cancel", { job: "combine" }).catch(() => undefined)}
            >
              {t("ui.editor.cancel")}
            </button>
          </>
        ) : (
          <>
            <button
              type="button"
              className="btn primary"
              disabled={chosen.length < 2}
              title={chosen.length < 2 ? t("errors.combine_two") : undefined}
              onClick={() => void combine()}
            >
              {t("ui.video.combine.button")}
            </button>
            <button type="button" className="btn" onClick={close}>
              {t("ui.editor.cancel")}
            </button>
          </>
        )
      }
    >
      <div className="combine">
        <div className="combine-tools">
          <input
            type="search"
            data-autofocus
            value={query}
            placeholder={t("ui.video.combine.filter")}
            aria-label={t("ui.video.combine.filter")}
            onChange={(e) => setQuery(e.target.value)}
          />
          <select
            value={sort}
            aria-label={t("ui.video.combine.sort")}
            onChange={(e) => setSort(e.target.value as Sort)}
          >
            {(["modified", "captured", "name", "size"] as const).map((s) => (
              <option key={s} value={s}>
                {t(`ui.video.combine.sort_${s}`)}
              </option>
            ))}
          </select>
        </div>
        <ul className="combine-list" aria-label={t("ui.video.combine.library")}>
          {items.map((item) => (
            <li key={item.id}>
              <button
                type="button"
                className="combine-item"
                title={t("ui.video.combine.add")}
                disabled={busy}
                onClick={() => add(item)}
              >
                <img src={libraryThumb(item)} alt="" draggable={false} />
                <span className="combine-name">{item.name}</span>
                <span className="muted">
                  {item.width && item.height ? `${item.width} × ${item.height}` : ""}
                </span>
              </button>
            </li>
          ))}
          {items.length === 0 && <li className="muted">{t("ui.video.combine.none")}</li>}
        </ul>
        <h3 className="side-subtitle">{t("ui.video.combine.order")}</h3>
        <ol className="combine-strip">
          {chosen.map((c, i) => (
            <li
              key={`${c.path}-${i}`}
              tabIndex={0}
              onKeyDown={(e) => {
                // Ctrl+arrows move it (VID-067).
                if (e.ctrlKey && e.key === "ArrowLeft") move(i, -1);
                else if (e.ctrlKey && e.key === "ArrowRight") move(i, 1);
                else if (e.key === "Delete") remove(i);
              }}
            >
              <span className="combine-name">
                {i + 1}. {c.name}
              </span>
              <span className="muted">
                {quick[c.path] ? formatTime(quick[c.path]?.duration ?? 0) : "…"}
              </span>
              <button
                type="button"
                className="btn small"
                aria-label={t("ui.video.combine.earlier")}
                disabled={i === 0 || busy}
                onClick={() => move(i, -1)}
              >
                ←
              </button>
              <button
                type="button"
                className="btn small"
                aria-label={t("ui.video.combine.later")}
                disabled={i === chosen.length - 1 || busy}
                onClick={() => move(i, 1)}
              >
                →
              </button>
              <button
                type="button"
                className="btn small"
                aria-label={t("ui.video.combine.remove")}
                disabled={busy}
                onClick={() => remove(i)}
              >
                ✕
              </button>
            </li>
          ))}
        </ol>
        <p className="muted" aria-live="polite">
          {chosen.length >= 2
            ? t("ui.video.combine.result", {
                count: chosen.length,
                time: formatTime(total),
                width,
                height,
              })
            : t("errors.combine_two")}
        </p>
        {error && <p className="error-text">{error}</p>}
      </div>
    </Dialog>
  );
}

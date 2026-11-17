// Auto redact over several captures (SMT-016): the review window, the bar that brings
// it back while a capture is reviewed in the editor, and the choice of captures for
// "Search other captures…" from auto redact's panel.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import { libraryThumb } from "../library/quickViewState";
import type { LibraryItem } from "../library/state";
import { docUrl, useEditor } from "./state";
import { trayOrder } from "./tray";
import {
  acceptItems,
  cancelBatch,
  closeBatch,
  countsOf,
  pending,
  restoreBatch,
  reviewItem,
  skipItems,
  startBatch,
  stopPicking,
  undoAll,
  useRedactBatch,
  type BatchItem,
} from "./redactBatchState";

const VIDEO_FORMATS = ["mp4", "m4v", "mov"];

/** A category's name: those of the panel, the user's terms by their name. */
function categoryLabel(c: string, t: (k: string) => string): string {
  if (c.startsWith("term:")) return c.slice(5);
  return t(`ui.redact.cat.${c}`);
}

function ItemLine({ item }: { item: BatchItem }) {
  const { t } = useTranslation();
  if (item.state === "ready") {
    return (
      <span>
        {countsOf(item)
          .map(([c, n]) => `${categoryLabel(c, t)}: ${n}`)
          .join(" · ")}
      </span>
    );
  }
  if (item.state === "failed") {
    return (
      <span className="redact-bad">
        {t("ui.redact_batch.state.failed", { error: item.error ?? "" })}
      </span>
    );
  }
  if (item.state === "accepted" && item.output) {
    const name = item.output.split(/[\\/]/).pop() ?? item.output;
    return <span>{t("ui.redact_batch.state.copy", { name })}</span>;
  }
  return <span className="muted">{t(`ui.redact_batch.state.${item.state}`)}</span>;
}

function Review() {
  const { t } = useTranslation();
  const status = useRedactBatch((s) => s.status);
  const thumbs = useRedactBatch((s) => s.thumbs);
  const busy = useRedactBatch((s) => s.busy);
  if (!status) return null;
  const ready = pending(status);
  const accepted = status.items.filter((i) => i.state === "accepted").length;
  return (
    <Dialog
      title={t("ui.redact_batch.title")}
      onCancel={() => void closeBatch()}
      actions={
        <>
          {status.running && (
            <button
              type="button"
              className="btn"
              disabled={busy}
              onClick={() => void cancelBatch()}
            >
              {t("ui.common.cancel")}
            </button>
          )}
          {accepted > 0 && (
            <button type="button" className="btn" disabled={busy} onClick={() => void undoAll()}>
              {t("ui.redact_batch.undo_all")}
            </button>
          )}
          <button type="button" className="btn" onClick={() => void closeBatch()}>
            {t("ui.common.close")}
          </button>
          <button
            type="button"
            className="btn primary"
            disabled={busy || ready.length === 0}
            onClick={() => void acceptItems()}
          >
            {t("ui.redact_batch.accept_all")}
          </button>
        </>
      }
    >
      <div className="redact-batch">
        <div className="redact-batch-progress" role="status" aria-live="polite">
          <span>
            {status.running
              ? t("ui.redact_batch.progress", { done: status.done, total: status.total })
              : t("ui.redact_batch.finished", { count: status.total })}
          </span>
          {status.running && (
            <progress
              value={status.done}
              max={status.total}
              aria-label={t("ui.redact_batch.progress", { done: status.done, total: status.total })}
            />
          )}
        </div>
        {status.texts > 0 && (
          <p className="muted hint">{t("ui.redact_batch.texts", { count: status.texts })}</p>
        )}
        <ul className="redact-batch-list">
          {status.items.map((item) => (
            <li key={item.index} className={`state-${item.state}`}>
              {thumbs[item.index] ? (
                <img src={thumbs[item.index]} alt="" draggable={false} />
              ) : (
                <span className="redact-batch-thumb" aria-hidden="true" />
              )}
              <div className="redact-batch-info">
                <strong>{item.name}</strong>
                <ItemLine item={item} />
              </div>
              {item.state === "ready" && (
                <div className="redact-batch-buttons" role="group" aria-label={item.name}>
                  <button
                    type="button"
                    className="btn small"
                    disabled={busy}
                    onClick={() => void acceptItems([item.index])}
                  >
                    {t("ui.redact_batch.accept")}
                  </button>
                  <button
                    type="button"
                    className="btn small"
                    disabled={busy}
                    onClick={() => void reviewItem(item.index)}
                  >
                    {t("ui.redact_batch.review")}
                  </button>
                  <button
                    type="button"
                    className="btn small"
                    disabled={busy}
                    onClick={() => void skipItems([item.index])}
                  >
                    {t("ui.redact_batch.skip")}
                  </button>
                </div>
              )}
            </li>
          ))}
        </ul>
      </div>
    </Dialog>
  );
}

/** While a capture is reviewed in the editor: how many wait, and the way back. */
function Bar() {
  const { t } = useTranslation();
  const status = useRedactBatch((s) => s.status);
  if (!status) return null;
  const count = pending(status).length;
  return (
    <div className="redact-batch-bar" role="status">
      <span>{t("ui.redact_batch.waiting", { count })}</span>
      <button type="button" className="btn small primary" onClick={restoreBatch}>
        {t("ui.redact_batch.back")}
      </button>
    </div>
  );
}

interface Choice {
  key: string;
  name: string;
  thumb: string;
  documentId?: string;
  path?: string;
}

/** The captures to search, from the recent tray and the library. */
function Picker({ origin }: { origin: string }) {
  const { t } = useTranslation();
  const documents = useEditor((s) => s.documents);
  const tray = useEditor((s) => s.tray);
  const [query, setQuery] = useState("");
  const [library, setLibrary] = useState<LibraryItem[]>([]);
  const [chosen, setChosen] = useState<string[]>([]);
  const originPath = documents.find((d) => d.documentId === origin)?.path ?? null;

  useEffect(() => {
    let stale = false;
    const timer = setTimeout(() => {
      run<{ items: LibraryItem[] }>("library.list", {
        sort: "modified",
        limit: 60,
        query: query.trim() || undefined,
        filters: { types: ["image", "guide"] },
      })
        .then((r) => {
          if (!stale) setLibrary(r.items);
        })
        .catch(() => undefined);
    }, 200);
    return () => {
      stale = true;
      clearTimeout(timer);
    };
  }, [query]);

  const recent: Choice[] = trayOrder(documents, tray)
    .filter((d) => d.documentId !== origin && !d.missing && !VIDEO_FORMATS.includes(d.format))
    .map((d) => ({
      key: `doc:${d.documentId}`,
      name: d.name,
      thumb: docUrl(d.documentId, "thumb", d.version),
      documentId: d.documentId,
    }));
  const inTray = new Set(documents.map((d) => d.path).filter(Boolean));
  const fromLibrary: Choice[] = library
    .filter((i) => i.path !== originPath && !inTray.has(i.path))
    .map((i) => ({ key: `lib:${i.path}`, name: i.name, thumb: libraryThumb(i), path: i.path }));
  const all = [...recent, ...fromLibrary];
  const toggle = (key: string) =>
    setChosen((c) => (c.includes(key) ? c.filter((k) => k !== key) : [...c, key]));
  const search = () => {
    const picked = all.filter((c) => chosen.includes(c.key));
    void startBatch({
      documentId: origin,
      documentIds: picked.flatMap((c) => (c.documentId ? [c.documentId] : [])),
      paths: picked.flatMap((c) => (c.path ? [c.path] : [])),
    });
  };
  const list = (title: string, choices: Choice[]) => (
    <section aria-label={title}>
      <h3 className="side-subtitle">{title}</h3>
      {choices.length === 0 ? (
        <p className="muted">{t("ui.redact_batch.none_here")}</p>
      ) : (
        <ul className="redact-pick">
          {choices.map((c) => (
            <li key={c.key}>
              <label className={chosen.includes(c.key) ? "chosen" : undefined}>
                <input
                  type="checkbox"
                  checked={chosen.includes(c.key)}
                  onChange={() => toggle(c.key)}
                />
                <img src={c.thumb} alt="" loading="lazy" draggable={false} />
                <span>{c.name}</span>
              </label>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
  return (
    <Dialog
      title={t("ui.redact_batch.pick_title")}
      onCancel={stopPicking}
      actions={
        <>
          <button type="button" className="btn" onClick={stopPicking}>
            {t("ui.common.cancel")}
          </button>
          <button
            type="button"
            className="btn primary"
            disabled={chosen.length === 0}
            onClick={search}
          >
            {t("ui.redact_batch.search", { count: chosen.length })}
          </button>
        </>
      }
    >
      <div className="redact-batch">
        <p className="muted hint">{t("ui.redact_batch.pick_hint")}</p>
        {list(t("ui.redact_batch.recent"), recent)}
        <input
          type="search"
          className="redact-pick-search"
          value={query}
          placeholder={t("ui.redact_batch.library_search")}
          aria-label={t("ui.redact_batch.library_search")}
          onChange={(e) => setQuery(e.target.value)}
        />
        {list(t("ui.redact_batch.library"), fromLibrary)}
      </div>
    </Dialog>
  );
}

export function RedactBatch() {
  const status = useRedactBatch((s) => s.status);
  const minimized = useRedactBatch((s) => s.minimized);
  const picking = useRedactBatch((s) => s.picking);
  if (picking) return <Picker origin={picking} />;
  if (!status) return null;
  return minimized ? <Bar /> : <Review />;
}

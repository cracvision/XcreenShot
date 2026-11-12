// Preferences > Library: where the library is (LIB-027), Show, and Change… (LIB-028):
// move the library to another folder, start an empty one there or use the one already
// there; the move runs in the background, with its progress and Cancel. A note says
// when the folder syncs to the cloud (LIB-029).
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { isCommandError, run } from "../lib/commands";

interface Service {
  service: string;
  account: string | null;
  path: string;
  library: boolean;
  current: boolean;
}

interface Where {
  path: string;
  synced: boolean;
  moving: boolean;
  /** The sync services' folders of this computer (LIB-029). */
  services: Service[];
}

interface Inspect {
  path: string;
  exists: boolean;
  items: number;
  library: boolean;
  writable: boolean;
  synced: boolean;
  same: boolean;
}

interface MoveEvent {
  done?: number;
  total?: number;
  finished?: boolean;
  cancelled?: boolean;
  moved?: number;
  failed?: string[];
}

export function LibraryLocation() {
  const { t } = useTranslation();
  const [where, setWhere] = useState<Where | null>(null);
  const [choice, setChoice] = useState<Inspect | null>(null);
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [total, setTotal] = useState(0);

  const load = useCallback(() => {
    run<Where>("library.location")
      .then(setWhere)
      .catch(() => undefined);
    run<{ total: number }>("library.list", { limit: 1 })
      .then((r) => setTotal(r.total))
      .catch(() => undefined);
  }, []);
  useEffect(load, [load]);

  useEffect(() => {
    const off = listen<{ kind: string; payload: MoveEvent }>("xs://editor", (e) => {
      if (e.payload.kind !== "library_move") return;
      const m = e.payload.payload;
      if (m.finished) {
        setProgress(null);
        const failed = m.failed?.length ?? 0;
        setNote(
          m.cancelled
            ? t("ui.library.move_cancelled")
            : [
                t("ui.library.moved", { count: m.moved ?? 0 }),
                failed > 0 ? t("ui.library.move_failed", { count: failed }) : "",
              ]
                .filter(Boolean)
                .join(" "),
        );
        load();
      } else if (m.total !== undefined) {
        setProgress({ done: m.done ?? 0, total: m.total });
      }
    });
    return () => void off.then((f) => f());
  }, [t, load]);

  const change = async (chosen?: string) => {
    setNote(null);
    setError(null);
    const dir = chosen ?? (await open({ directory: true, title: t("ui.library.choose_folder") }));
    if (typeof dir !== "string") return;
    try {
      const i = await run<Inspect>("library.location", { action: "inspect", path: dir });
      if (!i.same) setChoice(i);
    } catch (e) {
      setError(isCommandError(e) ? e.message : String(e));
    }
  };

  const apply = async (mode: "move" | "empty" | "use") => {
    if (!choice || !where) return;
    setChoice(null);
    try {
      const r = await run<{ started: boolean; total?: number }>("library.relocate", {
        path: choice.path,
        mode,
      });
      if (r.started) setProgress({ done: 0, total: r.total ?? 0 });
      else load();
    } catch (e) {
      setError(isCommandError(e) ? e.message : String(e));
    }
  };

  if (!where) return null;
  const busy = progress !== null || where.moving;
  return (
    <div className="setting library-location">
      <span className="label">{t("ui.library.location")}</span>
      <div className="control row">
        <button
          type="button"
          className="btn"
          onClick={() => void run("library.show_in_folder").catch(() => undefined)}
        >
          {t("ui.library.show")}
        </button>
        <button type="button" className="btn" disabled={busy} onClick={() => void change()}>
          {t("ui.library.change")}
        </button>
      </div>
      <div className="description">
        <code className="path" title={where.path}>
          {where.path}
        </code>
        <div>{t("ui.library.location_help")}</div>
        {where.synced && <div>{t("ui.library.synced_help", { path: where.path })}</div>}
        {where.services.some((s) => !s.current) && !busy && (
          <div className="lib-services">
            <span className="muted">{t("ui.library.services")}</span>
            {where.services
              .filter((s) => !s.current)
              .map((s) => (
                <button
                  key={s.path}
                  type="button"
                  className="btn small"
                  title={s.path}
                  onClick={() => void change(s.path)}
                >
                  {t(`ui.library.service.${s.service}`)}
                  {s.account ? ` (${s.account})` : ""}
                  {s.library ? ` · ${t("ui.library.service_has")}` : ""}
                </button>
              ))}
          </div>
        )}
        {progress && (
          <div className="lib-move" role="status">
            <span>{t("ui.library.moving", { done: progress.done, total: progress.total })}</span>
            <progress max={Math.max(1, progress.total)} value={progress.done} />
            <button
              type="button"
              className="btn small"
              onClick={() => void run("library.relocate", { cancel: true }).catch(() => undefined)}
            >
              {t("ui.editor.cancel")}
            </button>
          </div>
        )}
        {note && <div role="status">{note}</div>}
        {error && (
          <div className="tpl-warning" role="alert">
            {error}
          </div>
        )}
      </div>
      {choice && (
        <Dialog
          title={t("ui.library.move_title", { path: choice.path })}
          onCancel={() => setChoice(null)}
          actions={
            <button type="button" className="btn" onClick={() => setChoice(null)}>
              {t("ui.editor.cancel")}
            </button>
          }
        >
          {!choice.writable && <p className="tpl-warning">{t("ui.library.not_writable")}</p>}
          {choice.synced && <p className="muted">{t("ui.library.synced_folder")}</p>}
          <div className="move-options">
            {choice.library && choice.items > 0 ? (
              <button
                type="button"
                className="btn move-option"
                data-autofocus
                disabled={!choice.writable}
                onClick={() => void apply("use")}
              >
                <strong>{t("ui.library.use_there")}</strong>
                <span className="muted">
                  {t("ui.library.use_there_help", { count: choice.items })}
                </span>
              </button>
            ) : (
              <>
                <button
                  type="button"
                  className="btn primary move-option"
                  data-autofocus
                  disabled={!choice.writable}
                  onClick={() => void apply("move")}
                >
                  <strong>{t("ui.library.move_here")}</strong>
                  <span>{t("ui.library.move_here_help", { count: total })}</span>
                </button>
                <button
                  type="button"
                  className="btn move-option"
                  disabled={!choice.writable}
                  onClick={() => void apply("empty")}
                >
                  <strong>{t("ui.library.start_empty")}</strong>
                  <span className="muted">{t("ui.library.start_empty_help")}</span>
                </button>
              </>
            )}
          </div>
        </Dialog>
      )}
    </div>
  );
}

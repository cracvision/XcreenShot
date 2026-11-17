// Library > Export the whole library… (LIB-033): a copy of the library, or of what the
// filters show, as plain files in a folder, with a CSV of the details. Progress with
// Cancel, then a summary with Show folder.
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { useIndicator } from "../components/useIndicator";
import { run } from "../lib/commands";
import {
  exportLibrary,
  insideFolder,
  libraryFiltered,
  useLibraryExport,
  type LibraryExportResult,
} from "./exportState";
import { fail } from "./maintenanceState";
import { useLibrary } from "./state";

export function LibraryExportDialog() {
  const open = useLibraryExport((s) => s.open);
  return open ? <ExportForm /> : null;
}

function ExportForm() {
  const { t } = useTranslation();
  const tt = (k: string, o?: Record<string, unknown>) => t(`ui.library.export.${k}`, o ?? {});
  const root = useLibrary((s) => s.root);
  const total = useLibrary((s) => s.total);
  const shown = useLibrary((s) => s.items.length);
  const [filtered, setFiltered] = useState(false);
  const [format, setFormat] = useState<"png" | "jpg">("png");
  const [folder, setFolder] = useState<string | null>(null);
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<{ index: number; total: number } | null>(null);
  const [result, setResult] = useState<LibraryExportResult | null>(null);
  const seg = useRef<HTMLDivElement>(null);
  useIndicator(seg, format);
  const close = () => useLibraryExport.setState({ open: false });

  useEffect(() => {
    const off = listen<{ kind: string; payload: Record<string, unknown> }>("xs://editor", (e) => {
      const p = e.payload.payload;
      if (e.payload.kind !== "progress" || p.task !== "library_export") return;
      setProgress(
        p.done === null ? null : { index: Number(p.index ?? 0), total: Number(p.total ?? 0) },
      );
    });
    return () => void off.then((f) => f());
  }, []);

  const choose = async () => {
    const picked = await open({ directory: true });
    if (typeof picked === "string") setFolder(picked);
  };
  const inLibrary = folder !== null && insideFolder(root, folder);
  const go = async () => {
    if (!folder) return;
    setRunning(true);
    try {
      setResult(await exportLibrary(folder, format, filtered));
    } catch (e) {
      fail(e);
    } finally {
      setRunning(false);
      setProgress(null);
    }
  };

  if (result) {
    const shownPath = result.csv ?? null;
    return (
      <Dialog
        title={tt("done_title")}
        onCancel={close}
        actions={
          <>
            {shownPath && (
              <button
                type="button"
                className="btn"
                onClick={() => void run("library.show_in_folder", { path: shownPath }).catch(fail)}
              >
                {t("menu.show_in_folder")}
              </button>
            )}
            <button type="button" className="btn primary" data-autofocus onClick={close}>
              {t("ui.editor.image.done")}
            </button>
          </>
        }
      >
        <p>{tt("done_exported", { count: result.exported })}</p>
        {result.copied > 0 && <p>{tt("done_copied", { count: result.copied })}</p>}
        {result.csv && <p className="muted">{tt("done_csv")}</p>}
        {result.cancelled && <p className="warning-text">{tt("done_cancelled")}</p>}
        {result.failed.length > 0 && (
          <>
            <p className="warning-text">{tt("done_failed", { count: result.failed.length })}</p>
            <ul className="batch-failed">
              {result.failed.map((f) => (
                <li key={f.source}>
                  {f.name}: {f.reason}
                </li>
              ))}
            </ul>
          </>
        )}
      </Dialog>
    );
  }

  return (
    <Dialog
      title={tt("title")}
      onCancel={() => (running ? void run("editor.cancel").catch(() => undefined) : close())}
      actions={
        running ? (
          <button type="button" className="btn" onClick={() => void run("editor.cancel")}>
            {t("ui.editor.cancel")}
          </button>
        ) : (
          <>
            <button
              type="button"
              className="btn primary"
              disabled={!folder || inLibrary}
              onClick={() => void go()}
            >
              {tt("go")}
            </button>
            <button type="button" className="btn" onClick={close}>
              {t("ui.editor.cancel")}
            </button>
          </>
        )
      }
    >
      <p className="muted">{tt("help")}</p>
      {running ? (
        <div className="export-progress" role="status">
          <span>
            {progress
              ? tt("progress", { done: progress.index, total: progress.total })
              : tt("starting")}
          </span>
          <progress
            max={Math.max(1, progress?.total ?? 1)}
            value={progress?.index ?? 0}
            aria-label={tt("title")}
          />
        </div>
      ) : (
        <div className="image-form">
          {libraryFiltered() && (
            <fieldset className="export-scope">
              <legend className="sr-only">{tt("scope")}</legend>
              <label className="panel-check">
                <input
                  type="radio"
                  name="lib-export-scope"
                  checked={!filtered}
                  onChange={() => setFiltered(false)}
                />
                {tt("scope_all", { count: total })}
              </label>
              <label className="panel-check">
                <input
                  type="radio"
                  name="lib-export-scope"
                  checked={filtered}
                  onChange={() => setFiltered(true)}
                />
                {tt("scope_filtered", { count: shown })}
              </label>
            </fieldset>
          )}
          <div className="prop-field export-format">
            <span id="lib-export-format">{tt("format")}</span>
            <div
              ref={seg}
              className="seg small"
              role="radiogroup"
              aria-labelledby="lib-export-format"
            >
              <span className="seg-indicator" aria-hidden="true" />
              {(["png", "jpg"] as const).map((f) => (
                <button
                  key={f}
                  type="button"
                  role="radio"
                  aria-checked={format === f}
                  onClick={() => setFormat(f)}
                >
                  {t(`ui.editor.export.format_${f}`)}
                </button>
              ))}
            </div>
          </div>
          <div className="batch-row">
            <span className="batch-name muted" title={folder ?? ""}>
              {folder ?? tt("no_folder")}
            </span>
            <button type="button" className="btn small" onClick={() => void choose()}>
              {tt("choose")}
            </button>
          </div>
          {inLibrary && <p className="warning-text">{tt("in_library")}</p>}
        </div>
      )}
    </Dialog>
  );
}

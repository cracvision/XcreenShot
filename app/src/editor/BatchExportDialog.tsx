// File > Batch export (EDT-113): files, effects, format, destination and names in one
// dialog; then the progress with Cancel, and a summary.
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import {
  batchName,
  rememberBatch,
  rememberedBatch,
  stemOf,
  useBatchDialog,
  type BatchSettings,
} from "./batchExport";
import { FormatOptions } from "./ExportDialog";
import { FORMATS, rememberedOptions, type ExportFormat } from "./exportOptions";
import { useEditor } from "./state";

interface Summary {
  exported: { source: string; path: string }[];
  failed: { source: string; reason: string }[];
  skipped: { source: string; path: string }[];
  cancelled: boolean;
}

interface SavedEffect {
  id: string;
  name: string;
}

export function BatchExportDialog() {
  const files = useBatchDialog((s) => s.files);
  if (!files) return null;
  return <BatchForm initial={files} />;
}

function BatchForm({ initial }: { initial: string[] }) {
  const { t } = useTranslation();
  const [files, setFiles] = useState<string[]>(initial);
  const [s, setS] = useState<BatchSettings>(rememberedBatch);
  const [effects, setEffects] = useState<SavedEffect[]>([]);
  const [progress, setProgress] = useState<{ index: number; total: number } | null>(null);
  const [running, setRunning] = useState(false);
  const [summary, setSummary] = useState<Summary | null>(null);
  const set = (changes: Partial<BatchSettings>) => setS((prev) => ({ ...prev, ...changes }));
  const close = () => useBatchDialog.setState({ files: null });
  const tt = (k: string, o?: Record<string, unknown>) => t(`ui.editor.batch.${k}`, o ?? {});

  useEffect(() => {
    void run<{ effects: SavedEffect[] }>("editor.saved_effects")
      .then((r) => setEffects(r.effects))
      .catch(() => undefined);
  }, []);
  // The core's progress while exporting (after 300 ms).
  useEffect(() => {
    const off = listen<{ kind: string; payload: Record<string, unknown> }>("xs://editor", (e) => {
      const p = e.payload.payload;
      if (e.payload.kind !== "progress" || p.task !== "batch") return;
      setProgress(
        p.done === null ? null : { index: Number(p.index ?? 0), total: Number(p.total ?? 0) },
      );
    });
    return () => void off.then((f) => f());
  }, []);

  const addFiles = async () => {
    const picked = await open({
      multiple: true,
      filters: [{ name: tt("images"), extensions: ["xshot", "png", "jpg", "jpeg"] }],
    });
    if (Array.isArray(picked)) setFiles((f) => [...f, ...picked.filter((p) => !f.includes(p))]);
  };
  const addFolder = async () => {
    const picked = await open({ directory: true });
    if (typeof picked === "string" && !files.includes(picked)) setFiles((f) => [...f, picked]);
  };
  const chooseFolder = async () => {
    const picked = await open({ directory: true });
    if (typeof picked === "string") set({ folder: picked, nextToOriginal: false });
  };
  const move = (i: number, by: number) =>
    setFiles((f) => {
      const next = [...f];
      const [item] = next.splice(i, 1);
      if (item !== undefined) next.splice(Math.max(0, Math.min(next.length, i + by)), 0, item);
      return next;
    });

  const ready = files.length > 0 && (s.nextToOriginal || Boolean(s.folder)) && !running;
  const go = async () => {
    rememberBatch(s);
    setRunning(true);
    try {
      const r = await run<Summary>("document.batch_export", {
        paths: files,
        format: s.format,
        options: s.options,
        effects: s.effect,
        ...(s.nextToOriginal ? { nextToOriginal: true } : { folder: s.folder }),
        naming: s.naming,
        onConflict: s.onConflict,
      });
      setSummary(r);
    } catch (e) {
      useEditor.getState().showError(e);
    } finally {
      setRunning(false);
      setProgress(null);
    }
  };

  if (summary) {
    const first = summary.exported[0]?.path;
    return (
      <Dialog
        title={tt("done_title")}
        onCancel={close}
        actions={
          <>
            {first && (
              <button
                type="button"
                className="btn"
                onClick={() =>
                  void run("library.show_in_folder", { path: first }).catch(() => undefined)
                }
              >
                {t("menu.show_in_folder")}
              </button>
            )}
            <button type="button" className="btn primary" onClick={close}>
              {t("ui.editor.image.done")}
            </button>
          </>
        }
      >
        <p>{tt("done_exported", { count: summary.exported.length })}</p>
        {summary.skipped.length > 0 && (
          <p>{tt("done_skipped", { count: summary.skipped.length })}</p>
        )}
        {summary.cancelled && <p className="warning-text">{tt("done_cancelled")}</p>}
        {summary.failed.length > 0 && (
          <>
            <p className="warning-text">{tt("done_failed", { count: summary.failed.length })}</p>
            <ul className="batch-failed">
              {summary.failed.map((f) => (
                <li key={f.source}>
                  {stemOf(f.source)}: {f.reason}
                </li>
              ))}
            </ul>
          </>
        )}
      </Dialog>
    );
  }

  const names = files.slice(0, 3).map((f, i) => batchName(s.naming, stemOf(f), i));
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
              disabled={!ready}
              onClick={() => void go()}
            >
              {tt("export", { count: files.length })}
            </button>
            <button type="button" className="btn" onClick={close}>
              {t("ui.editor.cancel")}
            </button>
          </>
        )
      }
    >
      <div className="batch">
        <section aria-labelledby="batch-files">
          <h3 id="batch-files">{tt("files")}</h3>
          <ul className="batch-files">
            {files.map((f, i) => (
              <li key={f}>
                <span className="batch-name" title={f}>
                  {stemOf(f)}
                </span>
                <button
                  type="button"
                  className="btn icon small"
                  aria-label={tt("up")}
                  disabled={i === 0}
                  onClick={() => move(i, -1)}
                >
                  ↑
                </button>
                <button
                  type="button"
                  className="btn icon small"
                  aria-label={tt("down")}
                  disabled={i === files.length - 1}
                  onClick={() => move(i, 1)}
                >
                  ↓
                </button>
                <button
                  type="button"
                  className="btn icon small"
                  aria-label={tt("remove")}
                  onClick={() => setFiles((all) => all.filter((x) => x !== f))}
                >
                  ×
                </button>
              </li>
            ))}
          </ul>
          <div className="batch-row">
            <button type="button" className="btn small" onClick={() => void addFiles()}>
              {tt("add_files")}
            </button>
            <button type="button" className="btn small" onClick={() => void addFolder()}>
              {tt("add_folder")}
            </button>
          </div>
        </section>
        <section aria-labelledby="batch-effects">
          <h3 id="batch-effects">{tt("effects")}</h3>
          <select
            aria-labelledby="batch-effects"
            value={s.effect ?? ""}
            onChange={(e) => set({ effect: e.target.value || null })}
          >
            <option value="">{tt("no_effects")}</option>
            {effects.map((e) => (
              <option key={e.id} value={e.id}>
                {e.name}
              </option>
            ))}
          </select>
        </section>
        <section aria-labelledby="batch-format" className="image-form">
          <h3 id="batch-format">{tt("format")}</h3>
          <select
            aria-labelledby="batch-format"
            value={s.format}
            onChange={(e) => {
              const format = e.target.value as ExportFormat;
              set({ format, options: rememberedOptions(format) });
            }}
          >
            {FORMATS.map((f) => (
              <option key={f} value={f}>
                {t(`ui.editor.export.format_${f}`)}
              </option>
            ))}
          </select>
          <FormatOptions
            format={s.format}
            o={s.options}
            set={(changes) => set({ options: { ...s.options, ...changes } })}
          />
        </section>
        <section aria-labelledby="batch-dest">
          <h3 id="batch-dest">{tt("destination")}</h3>
          <label className="panel-check">
            <input
              type="radio"
              name="batch-dest"
              checked={!s.nextToOriginal}
              onChange={() => set({ nextToOriginal: false })}
            />
            {tt("to_folder")}
          </label>
          <div className="batch-row">
            <span className="batch-name muted" title={s.folder ?? ""}>
              {s.folder ?? tt("no_folder")}
            </span>
            <button type="button" className="btn small" onClick={() => void chooseFolder()}>
              {tt("choose")}
            </button>
          </div>
          <label className="panel-check">
            <input
              type="radio"
              name="batch-dest"
              checked={s.nextToOriginal}
              onChange={() => set({ nextToOriginal: true })}
            />
            {tt("next_to")}
          </label>
        </section>
        <section aria-labelledby="batch-names" className="image-form">
          <h3 id="batch-names">{tt("names")}</h3>
          <label className="panel-check">
            <input
              type="radio"
              name="batch-naming"
              checked={s.naming.mode === "keep"}
              onChange={() => set({ naming: { ...s.naming, mode: "keep" } })}
            />
            {tt("keep_names")}
          </label>
          <label className="panel-check">
            <input
              type="radio"
              name="batch-naming"
              checked={s.naming.mode === "base"}
              onChange={() => set({ naming: { ...s.naming, mode: "base" } })}
            />
            {tt("base_name")}
          </label>
          {s.naming.mode === "base" && (
            <div className="batch-grid">
              {(["base", "prefix", "suffix"] as const).map((k) => (
                <label key={k} className="prop-field">
                  <span>{tt(`naming_${k}`)}</span>
                  <input
                    type="text"
                    value={s.naming[k]}
                    onChange={(e) => set({ naming: { ...s.naming, [k]: e.target.value } })}
                  />
                </label>
              ))}
              {(["start", "digits"] as const).map((k) => (
                <label key={k} className="prop-field">
                  <span>{tt(`naming_${k}`)}</span>
                  <input
                    type="number"
                    min={k === "start" ? 0 : 1}
                    max={k === "start" ? 99999 : 9}
                    value={s.naming[k]}
                    onChange={(e) =>
                      set({
                        naming: {
                          ...s.naming,
                          [k]: Math.max(0, Math.round(Number(e.target.value) || 0)),
                        },
                      })
                    }
                  />
                </label>
              ))}
            </div>
          )}
          <p className="muted">
            {tt("preview")}{" "}
            {names.map((n) => `${n}.${s.format === "tiff" ? "tif" : s.format}`).join(", ")}
          </p>
          <div className="prop-field">
            <label htmlFor="batch-conflict">{tt("if_exists")}</label>
            <select
              id="batch-conflict"
              value={s.onConflict}
              onChange={(e) => set({ onConflict: e.target.value as BatchSettings["onConflict"] })}
            >
              {(["keepBoth", "replace", "skip"] as const).map((c) => (
                <option key={c} value={c}>
                  {tt(`conflict_${c}`)}
                </option>
              ))}
            </select>
          </div>
        </section>
      </div>
      {running && (
        <p aria-live="polite">
          {progress
            ? tt("progress", { index: progress.index + 1, total: progress.total })
            : tt("working")}
        </p>
      )}
    </Dialog>
  );
}

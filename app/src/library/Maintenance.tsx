// Rebuild the index and Check the library (LIB-023), from the Library's ⋯ menu.
import { taskDone, taskProgress } from "./tasks";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import { useEditor } from "../editor/state";
import { fail, resolveConflict, useMaintenance, useResolve, type Report } from "./maintenanceState";

/** Resolve a sync conflict (LIB-029): keep this copy, the original or both. */
function ResolveDialog() {
  const { t } = useTranslation();
  const item = useResolve((s) => s.item);
  if (!item) return null;
  const option = (keep: "this" | "original" | "both", primary = false) => (
    <button
      type="button"
      className={`btn move-option${primary ? " primary" : ""}`}
      {...(primary ? { "data-autofocus": true } : {})}
      onClick={() => void resolveConflict(item.id, keep)}
    >
      <strong>{t(`ui.library.resolve_${keep}`)}</strong>
      <span>{t(`ui.library.resolve_${keep}_help`)}</span>
    </button>
  );
  return (
    <Dialog
      title={t("ui.library.resolve_title", { name: item.name })}
      onCancel={() => useResolve.setState({ item: null })}
      actions={
        <button type="button" className="btn" onClick={() => useResolve.setState({ item: null })}>
          {t("ui.editor.cancel")}
        </button>
      }
    >
      <p className="muted">{t("ui.library.conflict_help")}</p>
      <div className="move-options">
        {option("both", true)}
        {option("this")}
        {option("original")}
      </div>
    </Dialog>
  );
}

export function MaintenanceDialogs() {
  const { t } = useTranslation();
  const resolving = useResolve((s) => s.item !== null);
  const dialog = useMaintenance((s) => s.dialog);
  const report = useMaintenance((s) => s.report);
  const [busy, setBusy] = useState(false);
  const close = () => useMaintenance.setState({ dialog: null });
  if (resolving) return <ResolveDialog />;
  if (dialog === "rebuild")
    return (
      <Dialog
        title={t("ui.library.rebuild_title")}
        onCancel={() => !busy && close()}
        actions={
          <>
            <button type="button" className="btn" disabled={busy} onClick={close}>
              {t("ui.editor.cancel")}
            </button>
            <button
              type="button"
              className="btn primary"
              disabled={busy}
              onClick={() => {
                setBusy(true);
                taskProgress({ id: "rebuild", label: "ui.library.rebuilding" });
                run<{ items: number }>("library.rebuild")
                  .then((r) => {
                    const done = t("ui.library.rebuilt", { count: r.items });
                    useEditor.getState().showToast(done);
                    taskDone("rebuild", done);
                  })
                  .catch((e: unknown) => {
                    taskDone("rebuild");
                    fail(e);
                  })
                  .finally(() => {
                    setBusy(false);
                    close();
                  });
              }}
            >
              {t("ui.library.rebuild_go")}
            </button>
          </>
        }
      >
        <p>{busy ? t("ui.library.rebuilding") : t("ui.library.rebuild_body")}</p>
      </Dialog>
    );
  if (dialog === "check" && report) {
    const ok = report.damaged.length === 0 && report.orphans === 0 && report.writable;
    return (
      <Dialog
        title={t("ui.library.check_title")}
        onCancel={close}
        actions={
          <button type="button" className="btn primary" onClick={close}>
            {t("ui.library.tags.done")}
          </button>
        }
      >
        {ok && <p>{t("ui.library.check_ok")}</p>}
        {!report.writable && <p className="tpl-warning">{t("ui.library.check_readonly")}</p>}
        {report.damaged.length > 0 && (
          <>
            <p>{t("ui.library.check_damaged", { count: report.damaged.length })}</p>
            <ul className="check-list">
              {report.damaged.map((path) => (
                <li key={path}>
                  <span className="check-path" title={path}>
                    {path}
                  </span>
                  <button
                    type="button"
                    className="btn small"
                    onClick={() => void run("library.show_in_folder", { path }).catch(fail)}
                  >
                    {t("ui.library.check_show")}
                  </button>
                </li>
              ))}
            </ul>
          </>
        )}
        {report.orphans > 0 && (
          <div className="row">
            <span>{t("ui.library.check_orphans", { count: report.orphans })}</span>
            <button
              type="button"
              className="btn small"
              onClick={() => {
                run<Report>("library.check", { action: "clean" })
                  .then((r) => useMaintenance.setState({ report: r }))
                  .catch(fail);
              }}
            >
              {t("ui.library.check_clean")}
            </button>
          </div>
        )}
      </Dialog>
    );
  }
  return null;
}

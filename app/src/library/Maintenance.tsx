// Rebuild the index and Check the library (LIB-023), from the Library's ⋯ menu.
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import { useEditor } from "../editor/state";
import { fail, useMaintenance, type Report } from "./maintenanceState";

export function MaintenanceDialogs() {
  const { t } = useTranslation();
  const dialog = useMaintenance((s) => s.dialog);
  const report = useMaintenance((s) => s.report);
  const [busy, setBusy] = useState(false);
  const close = () => useMaintenance.setState({ dialog: null });
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
                run<{ items: number }>("library.rebuild")
                  .then((r) =>
                    useEditor.getState().showToast(t("ui.library.rebuilt", { count: r.items })),
                  )
                  .catch(fail)
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

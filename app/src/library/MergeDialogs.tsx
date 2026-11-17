// Merge another library (LIB-031): the confirmation and the final report.
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { baseName, mergeSummary, startMerge, useMerge, type MergeReport } from "./mergeState";

const SHOWN = 8;

export function MergeDialogs() {
  const { t } = useTranslation();
  const asking = useMerge((s) => s.asking);
  const report = useMerge((s) => s.report);

  useEffect(() => {
    const off = listen<{ kind: string; payload: MergeReport & { finished?: boolean } }>(
      "xs://editor",
      (e) => {
        if (e.payload.kind !== "library_merge" || !e.payload.payload.finished) return;
        // Only the window that started it shows the report.
        if (!useMerge.getState().running) return;
        useMerge.setState({ running: false, report: e.payload.payload });
      },
    );
    return () => void off.then((f) => f());
  }, []);

  if (asking) {
    const close = () => useMerge.setState({ asking: null });
    return (
      <Dialog
        title={t("ui.library.merge.title")}
        onCancel={close}
        actions={
          <>
            <button type="button" className="btn" onClick={close}>
              {t("ui.editor.cancel")}
            </button>
            <button
              type="button"
              className="btn primary"
              data-autofocus
              disabled={asking.items === 0}
              onClick={() => void startMerge(asking.path)}
            >
              {t("ui.library.merge.go")}
            </button>
          </>
        }
      >
        {asking.items === 0 ? (
          <p className="tpl-warning">{t("ui.library.merge.empty")}</p>
        ) : (
          <p className="merge-body">
            {t("ui.library.merge.body", { count: asking.items, path: asking.path })}
          </p>
        )}
        {asking.items > 0 && !asking.library && (
          <p className="muted">{t("ui.library.merge.not_library")}</p>
        )}
        <p className="muted">{t("ui.library.merge.note")}</p>
      </Dialog>
    );
  }

  if (report) {
    const close = () => useMerge.setState({ report: null });
    return (
      <Dialog
        title={t(
          report.cancelled ? "ui.library.merge.cancelled_title" : "ui.library.merge.done_title",
        )}
        onCancel={close}
        actions={
          <button type="button" className="btn primary" data-autofocus onClick={close}>
            {t("ui.editor.ok")}
          </button>
        }
      >
        <div role="status">
          {mergeSummary(report, t).map((line) => (
            <p key={line}>{line}</p>
          ))}
        </div>
        {!report.cancelled && report.renamed.length > 0 && (
          <ul className="merge-list">
            {report.renamed.slice(0, SHOWN).map((r) => (
              <li key={r.path}>
                {baseName(r.from)} → {baseName(r.path)}
              </li>
            ))}
            {report.renamed.length > SHOWN && (
              <li className="muted">
                {t("ui.library.import.more", { count: report.renamed.length - SHOWN })}
              </li>
            )}
          </ul>
        )}
      </Dialog>
    );
  }
  return null;
}

// Separate elements (SMT-018): the side panel while the mode is on, with Detail, Reset
// and Done, and the question before separating again over changes.
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { NumberField } from "./Fields";
import {
  answerConfirm,
  finishSeparate,
  resetSeparate,
  setDetail,
  useSeparate,
} from "./separateState";

export function SeparatePanel() {
  const { t } = useTranslation();
  const detail = useSeparate((s) => s.detail);
  const busy = useSeparate((s) => s.busy);
  const confirm = useSeparate((s) => s.confirm);
  return (
    <section className="separate-panel" aria-label={t("ui.separate.title")} aria-busy={busy}>
      <h2 className="side-title">{t("ui.separate.title")}</h2>
      <p className="muted hint">{t("ui.separate.hint")}</p>
      <NumberField
        label={t("ui.separate.detail")}
        value={detail}
        min={1}
        max={10}
        step={1}
        unit=""
        onCommit={(d) => void setDetail(d)}
      />
      <p className="muted hint">{t("ui.separate.detail_help")}</p>
      <div className="redact-actions">
        <button type="button" className="btn" disabled={busy} onClick={() => void resetSeparate()}>
          {t("ui.separate.reset")}
        </button>
        <button
          type="button"
          className="btn primary"
          disabled={busy}
          onClick={() => void finishSeparate()}
        >
          {t("ui.separate.done")}
        </button>
      </div>
      {confirm && (
        <Dialog
          title={t("ui.separate.confirm_title")}
          onCancel={() => void answerConfirm(false)}
          actions={
            <>
              <button type="button" className="btn" onClick={() => void answerConfirm(false)}>
                {t("ui.common.cancel")}
              </button>
              <button
                type="button"
                className="btn primary"
                onClick={() => void answerConfirm(true)}
              >
                {t("ui.separate.again")}
              </button>
            </>
          }
        >
          <p>{t("ui.separate.confirm_body", { count: confirm.edited })}</p>
        </Dialog>
      )}
    </section>
  );
}

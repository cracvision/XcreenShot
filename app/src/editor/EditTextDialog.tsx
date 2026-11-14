// "The selection cuts the text" (SMT-009): fit the selection to the whole lines, edit
// as it is, or cancel.
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { answerCut, useEditText } from "./editText";

export function EditTextDialog() {
  const { t } = useTranslation();
  const ask = useEditText((s) => s.ask);
  if (!ask) return null;
  return (
    <Dialog
      title={t("ui.edit_text.cut_title")}
      onCancel={() => void answerCut(null)}
      actions={
        <>
          <button type="button" className="btn" onClick={() => void answerCut(null)}>
            {t("ui.common.cancel")}
          </button>
          <button type="button" className="btn" onClick={() => void answerCut("anyway")}>
            {t("ui.edit_text.anyway")}
          </button>
          <button type="button" className="btn primary" onClick={() => void answerCut("fit")}>
            {t("ui.edit_text.fit")}
          </button>
        </>
      }
    >
      <p>{t("ui.edit_text.cut_body")}</p>
    </Dialog>
  );
}

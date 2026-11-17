// A guide exported to a folder or page that already exists (SMT-032): Replace it, or
// Create another beside it (-2, -3…).
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { useGuideConflict } from "./guideExport";

export function GuideConflictDialog() {
  const { t } = useTranslation();
  const ask = useGuideConflict((s) => s.ask);
  if (!ask) return null;
  return (
    <Dialog
      title={t("ui.guide.exists_title")}
      onCancel={() => ask.choose(null)}
      actions={
        <>
          <button type="button" className="btn" onClick={() => ask.choose("replace")}>
            {t("ui.guide.exists_replace")}
          </button>
          <button type="button" className="btn primary" onClick={() => ask.choose("new")}>
            {t("ui.guide.exists_new")}
          </button>
        </>
      }
    >
      <p>{t("ui.guide.exists_body", { name: ask.name })}</p>
    </Dialog>
  );
}

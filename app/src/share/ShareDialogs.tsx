// Email over 20 MB (SHR-020): many mail servers refuse it; open the message anyway?
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { fileSize } from "./lanState";
import { loadDestinations, shareTo, useShare } from "./destinations";

export function ShareDialogs() {
  const { t, i18n } = useTranslation();
  useEffect(() => loadDestinations(), []);
  const large = useShare((s) => s.large);
  if (!large) return null;
  const close = () => useShare.setState({ large: null });
  return (
    <Dialog
      title={t("ui.share.too_large_title")}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
          <button
            type="button"
            className="btn primary"
            onClick={() => {
              close();
              void shareTo("email", large.target, { force: true });
            }}
          >
            {t("ui.share.send_anyway")}
          </button>
        </>
      }
    >
      <p>{t("ui.share.too_large", { size: fileSize(large.size, i18n.language) })}</p>
    </Dialog>
  );
}

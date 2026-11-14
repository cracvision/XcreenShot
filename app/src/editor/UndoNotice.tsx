// "3 items moved to the trash · Undo" for 10 s (LIB-015, TRY-008, TRY-009), and the
// other notices at the foot of the window (TRY-015).
import { useTranslation } from "react-i18next";
import { dismissNotice, useFiles } from "../library/actions";
import { Icon } from "./icons";

export function UndoNotice() {
  const { t } = useTranslation();
  const notice = useFiles((s) => s.notice);
  if (!notice) return null;
  return (
    <div className="undo-notice" role="status" key={notice.seq}>
      <span>{notice.text}</span>
      {notice.action && (
        <button type="button" className="btn small" onClick={() => void notice.action?.run()}>
          {notice.action.label}
        </button>
      )}
      <button
        type="button"
        className="btn icon small"
        aria-label={t("ui.common.close")}
        title={t("ui.common.close")}
        onClick={() => dismissNotice()}
      >
        <Icon name="close" />
      </button>
    </div>
  );
}

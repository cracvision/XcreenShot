// The QR codes of the selection, while capturing (SMT-038). Never part of the
// capture: the capture is cut from the frozen screens.

import { useTranslation } from "react-i18next";
import { QrCodeList } from "../components/QrCodeList";
import type { QrCode } from "../lib/qr";

interface Props {
  codes: QrCode[];
  /** Where the panel goes, in CSS px of this window. */
  style: React.CSSProperties;
  onClose: () => void;
}

export function QrPanel({ codes, style, onClose }: Props) {
  const { t } = useTranslation();
  return (
    <div className="overlay-qr" role="dialog" aria-label={t("ui.qr.codes_title")} style={style}>
      <strong>{t("ui.qr.codes_title")}</strong>
      <QrCodeList codes={codes} button="overlay-btn" primary="overlay-btn primary" />
      <div className="qr-list-actions">
        <button type="button" className="overlay-btn primary" onClick={onClose} autoFocus>
          {t("ui.common.close")} <kbd>Esc</kbd>
        </button>
      </div>
    </div>
  );
}

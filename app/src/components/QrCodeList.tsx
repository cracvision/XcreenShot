// The QR codes read (SMT-038): what each says, Copy and Open link (always asked
// first, with the domain apart). Used while capturing and in Extract text.

import { useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { splitUrl, type QrCode } from "../lib/qr";

interface Props {
  codes: QrCode[];
  /** Classes of its buttons, and of the one that answers yes. */
  button: string;
  primary: string;
}

function Code({
  code,
  button,
  onOpen,
}: {
  code: QrCode;
  button: string;
  onOpen: (c: QrCode) => void;
}) {
  const { t } = useTranslation();
  const [shown, setShown] = useState(false);
  const [copied, setCopied] = useState(false);
  const copy = () =>
    void navigator.clipboard
      .writeText(code.text)
      .then(() => {
        setCopied(true);
        window.setTimeout(() => setCopied(false), 1200);
      })
      .catch(() => undefined);
  return (
    <li className="qr-list-code">
      {code.kind === "wifi" ? (
        <div>
          <strong>{t("ui.qr.network", { ssid: code.ssid ?? "" })}</strong>
          <div>
            {t("ui.qr.password")}:{" "}
            {shown ? (
              <code>{code.password ?? ""}</code>
            ) : (
              <button type="button" className={button} onClick={() => setShown(true)}>
                {t("ui.qr.show")}
              </button>
            )}
          </div>
        </div>
      ) : (
        <code className="qr-list-text">{code.text}</code>
      )}
      <div className="qr-list-actions">
        <button type="button" className={button} onClick={copy}>
          {copied ? t("ui.qr.copied") : t("ui.qr.copy")}
        </button>
        {code.kind === "url" && (
          <button type="button" className={button} onClick={() => onOpen(code)}>
            {t("ui.qr.open")}
          </button>
        )}
      </div>
    </li>
  );
}

export function QrCodeList({ codes, button, primary }: Props) {
  const { t } = useTranslation();
  const [asking, setAsking] = useState<QrCode | null>(null);
  if (asking) {
    const [scheme, host, rest] = splitUrl(asking.text);
    const open = (yes: boolean) => {
      setAsking(null);
      if (yes) void run("smart.qr_open", { url: asking.text }).catch(() => undefined);
    };
    return (
      <div className="qr-list-ask" role="alertdialog" aria-label={t("ui.qr.open_title")}>
        <strong>{t("ui.qr.open_title")}</strong>
        <p>{t("ui.qr.open_body")}</p>
        <p className="qr-list-url">
          {scheme}
          <strong>{host}</strong>
          {rest}
        </p>
        <div className="qr-list-actions">
          <button type="button" className={button} onClick={() => open(false)} autoFocus>
            {t("ui.common.cancel")}
          </button>
          <button type="button" className={primary} onClick={() => open(true)}>
            {t("ui.qr.open_yes")}
          </button>
        </div>
      </div>
    );
  }
  return (
    <ul className="qr-list">
      {codes.map((c, i) => (
        <Code key={`${i}-${c.text}`} code={c} button={button} onOpen={setAsking} />
      ))}
    </ul>
  );
}

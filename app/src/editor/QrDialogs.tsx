// QR codes (SMT-036, SMT-038): the panel that creates or changes a code, the codes read
// from the image with their actions, the confirmation before a link opens, and the
// "QR code detected" pill on a capture just opened.
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { Dialog } from "../components/Dialog";
import { isOn } from "../features";
import { settingValue, useApp } from "../store";
import { useIndicator } from "../components/useIndicator";
import { ColorPicker } from "./ColorPicker";
import { NumberField, SelectField } from "./Fields";
import {
  askOpen,
  closeCodes,
  detectOnOpen,
  closeQr,
  insertQr,
  openConfirmed,
  previewQr,
  redactCode,
  splitUrl,
  useQr,
  type QrCode,
} from "./qrState";
import { useEditor, type QrEcc, type QrStyle } from "./state";

const ECCS: QrEcc[] = ["L", "M", "Q", "H"];

function Create() {
  const { t } = useTranslation();
  const editing = useQr((s) => s.editing);
  const check = useQr((s) => s.check);
  const tooLong = useQr((s) => s.tooLong);
  const [content, setContent] = useState(editing?.content ?? "");
  const [style, setStyle] = useState<QrStyle>(
    editing?.style ?? {
      kind: "link",
      ecc: "M",
      color: "#000000",
      background: "#FFFFFF",
      margin: 4,
    },
  );
  const seg = useRef<HTMLDivElement>(null);
  useIndicator(seg, style.kind);
  // The preview follows what is typed.
  useEffect(() => {
    const timer = setTimeout(() => void previewQr(content, style), 150);
    return () => clearTimeout(timer);
  }, [content, style]);
  if (!editing) return null;
  const set = (change: Partial<QrStyle>) => setStyle((s) => ({ ...s, ...change }));
  const ready = content.trim() !== "" && !tooLong && check !== null;
  return (
    <Dialog
      title={t(editing.objectId ? "ui.qr.edit_title" : "ui.qr.title")}
      onCancel={closeQr}
      actions={
        <>
          <button type="button" className="btn" onClick={closeQr}>
            {t("ui.common.cancel")}
          </button>
          <button
            type="button"
            className="btn primary"
            disabled={!ready}
            onClick={() => void insertQr(content, style)}
          >
            {t(editing.objectId ? "ui.qr.save" : "ui.qr.insert")}
          </button>
        </>
      }
    >
      <div className="qr-panel">
        <div className="qr-fields">
          <div className="seg small" role="radiogroup" aria-label={t("ui.qr.content")} ref={seg}>
            <span className="seg-indicator" aria-hidden="true" />
            {(["link", "text"] as const).map((k) => (
              <button
                key={k}
                type="button"
                role="radio"
                aria-checked={style.kind === k}
                onClick={() => set({ kind: k })}
              >
                {t(`ui.qr.${k}`)}
              </button>
            ))}
          </div>
          <label className="field">
            <span>{t("ui.qr.content")}</span>
            {style.kind === "link" ? (
              <input
                type="text"
                data-autofocus
                value={content}
                placeholder={t("ui.qr.link_placeholder")}
                onChange={(e) => setContent(e.target.value)}
                aria-invalid={tooLong !== null}
              />
            ) : (
              <textarea
                data-autofocus
                rows={3}
                value={content}
                placeholder={t("ui.qr.text_placeholder")}
                onChange={(e) => setContent(e.target.value)}
                aria-invalid={tooLong !== null}
              />
            )}
          </label>
          {tooLong && (
            <p className="redact-bad" role="alert">
              {t("ui.qr.too_long", tooLong)}
            </p>
          )}
          <SelectField
            label={t("ui.qr.ecc")}
            value={style.ecc}
            options={ECCS.map((e) => ({ value: e, label: t(`ui.qr.ecc_${e}`) }))}
            onCommit={(ecc) => set({ ecc })}
          />
          <p className="muted hint">{t("ui.qr.ecc_help")}</p>
          <ColorPicker
            label={t("ui.qr.color")}
            value={style.color}
            onCommit={(color) => color && set({ color })}
          />
          <ColorPicker
            label={t("ui.qr.background")}
            value={style.background}
            allowTransparent
            onCommit={(background) => set({ background })}
          />
          <NumberField
            label={t("ui.qr.margin")}
            value={style.margin}
            min={0}
            max={8}
            step={1}
            unit=""
            onCommit={(margin) => set({ margin })}
          />
        </div>
        <figure className="qr-preview" aria-label={t("ui.qr.preview")}>
          {check?.svg ? (
            <img src={`data:image/svg+xml;utf8,${encodeURIComponent(check.svg)}`} alt="" />
          ) : (
            <span className="qr-empty" aria-hidden="true" />
          )}
          {check && check.hardToRead && (
            <figcaption className="qr-warn" role="status">
              {t("ui.qr.hard")}
            </figcaption>
          )}
          {check && !check.readable && (
            <figcaption className="qr-warn" role="status">
              {t("ui.qr.unreadable")}
            </figcaption>
          )}
        </figure>
      </div>
    </Dialog>
  );
}

function Code({ code }: { code: QrCode }) {
  const { t } = useTranslation();
  const [shown, setShown] = useState(false);
  const [copied, setCopied] = useState(false);
  const copy = (text: string) =>
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    });
  return (
    <li className="qr-code">
      {code.kind === "wifi" ? (
        <div>
          <strong>{t("ui.qr.network", { ssid: code.ssid ?? "" })}</strong>
          <div>
            {t("ui.qr.password")}:{" "}
            {shown ? (
              <code>{code.password ?? ""}</code>
            ) : (
              <button type="button" className="btn small" onClick={() => setShown(true)}>
                {t("ui.qr.show")}
              </button>
            )}
          </div>
        </div>
      ) : (
        <code className="qr-text">{code.text}</code>
      )}
      <div className="qr-actions">
        <button type="button" className="btn small" onClick={() => copy(code.text)}>
          {copied ? t("ui.qr.copied") : t("ui.qr.copy")}
        </button>
        {code.kind === "url" && (
          <button type="button" className="btn small" onClick={() => askOpen(code)}>
            {t("ui.qr.open")}
          </button>
        )}
        <button type="button" className="btn small" onClick={() => void redactCode(code)}>
          {t("ui.qr.redact")}
        </button>
      </div>
    </li>
  );
}

function Codes() {
  const { t } = useTranslation();
  const codes = useQr((s) => s.codes);
  const confirm = useQr((s) => s.confirmOpen);
  if (!codes) return null;
  if (confirm) {
    const [scheme, host, rest] = splitUrl(confirm.text);
    return (
      <Dialog
        title={t("ui.qr.open_title")}
        onCancel={() => void openConfirmed(false)}
        actions={
          <>
            <button type="button" className="btn" onClick={() => void openConfirmed(false)}>
              {t("ui.common.cancel")}
            </button>
            <button type="button" className="btn primary" onClick={() => void openConfirmed(true)}>
              {t("ui.qr.open_yes")}
            </button>
          </>
        }
      >
        <p>{t("ui.qr.open_body")}</p>
        <p className="qr-url">
          {scheme}
          <strong>{host}</strong>
          {rest}
        </p>
      </Dialog>
    );
  }
  return (
    <Dialog
      title={t("ui.qr.codes_title")}
      onCancel={closeCodes}
      actions={
        <button type="button" className="btn primary" onClick={closeCodes}>
          {t("ui.common.close")}
        </button>
      }
    >
      {codes.length === 0 ? (
        <p>{t("ui.qr.none")}</p>
      ) : (
        <ul className="qr-codes">
          {codes.map((c, i) => (
            <Code key={`${i}-${c.text}`} code={c} />
          ))}
        </ul>
      )}
    </Dialog>
  );
}

/** "QR code detected" over the canvas of a capture just opened (SMT-038). */
function Pill() {
  const { t } = useTranslation();
  const pill = useQr((s) => s.pill);
  const doc = useEditor((s) => s.doc?.documentId);
  if (!pill || pill.documentId !== doc) return null;
  return (
    <button
      type="button"
      className="qr-pill"
      onClick={() => useQr.setState({ codes: pill.codes, pill: null })}
    >
      {t("ui.qr.detected", { count: pill.codes.length })}
    </button>
  );
}

export function QrDialogs() {
  const editing = useQr((s) => s.editing);
  // A capture that opens in the editor with QR codes says so (SMT-038).
  useEffect(() => {
    let off: (() => void) | undefined;
    void listen<{ result: { documentId?: string } }>("xs://capture", (e) => {
      const id = e.payload.result.documentId;
      const wanted = settingValue(useApp.getState().settings, "smart.qr.detectOnCapture") !== false;
      if (id && wanted && isOn("qr")) {
        // Once the editor shows it.
        setTimeout(() => void detectOnOpen(id), 600);
      }
    }).then((u) => (off = u));
    return () => off?.();
  }, []);
  return (
    <>
      {editing && <Create key={editing.objectId ?? "new"} />}
      <Codes />
      <Pill />
    </>
  );
}

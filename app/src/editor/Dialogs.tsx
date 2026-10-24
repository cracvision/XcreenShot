// Editor dialogs: unsaved changes (EDT-108, APP-004), flattening a PNG or JPG
// (EDT-107), Recover work (APP-047) and errors.
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { decide, recover, saveActive } from "./actions";
import { useEditor } from "./state";

function Decision() {
  const { t } = useTranslation();
  const decision = useEditor((s) => s.decision);
  if (!decision) return null;
  const one = decision.pending.length === 1 ? decision.pending[0] : undefined;
  return (
    <Dialog
      title={
        one
          ? t("ui.editor.unsaved_title_one", { name: one.name })
          : t("ui.editor.unsaved_title_many")
      }
      onCancel={() => void decide("cancel")}
      actions={
        <>
          <button type="button" className="btn primary" onClick={() => void decide("save")}>
            {one ? t("ui.editor.save") : t("ui.editor.save_all")}
          </button>
          <button type="button" className="btn" onClick={() => void decide("discard")}>
            {t("ui.editor.dont_save")}
          </button>
          <button type="button" className="btn" onClick={() => void decide("cancel")}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <p>{t(decision.kind === "quit" ? "ui.editor.unsaved_quit" : "ui.editor.unsaved_close")}</p>
      {!one && (
        <ul>
          {decision.pending.map((p) => (
            <li key={p.documentId}>{p.name}</li>
          ))}
        </ul>
      )}
    </Dialog>
  );
}

function Flatten() {
  const { t } = useTranslation();
  const flatten = useEditor((s) => s.flatten);
  const name = useEditor((s) => s.documents.find((d) => d.documentId === s.flatten)?.name);
  const [dontAsk, setDontAsk] = useState(false);
  if (!flatten) return null;
  const close = () => useEditor.setState({ flatten: null });
  return (
    <Dialog
      title={t("ui.editor.flatten_title")}
      onCancel={close}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            onClick={() => {
              close();
              void saveActive(true, dontAsk);
            }}
          >
            {t("ui.editor.save")}
          </button>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <p>{t("ui.editor.flatten_body", { name: name ?? "" })}</p>
      <label className="panel-check">
        <input type="checkbox" checked={dontAsk} onChange={(e) => setDontAsk(e.target.checked)} />
        {t("ui.editor.dont_ask_again")}
      </label>
    </Dialog>
  );
}

function Recovery() {
  const { t, i18n } = useTranslation();
  const items = useEditor((s) => s.recovery);
  const [later, setLater] = useState(false);
  if (items.length === 0 || later) return null;
  return (
    <Dialog
      title={t("ui.editor.recover_title")}
      onCancel={() => setLater(true)}
      actions={
        <button type="button" className="btn" onClick={() => setLater(true)}>
          {t("ui.editor.later")}
        </button>
      }
    >
      <p>{t("ui.editor.recover_body")}</p>
      <ul className="recover-list">
        {items.map((item) => (
          <li key={item.id}>
            <div>
              <strong>{item.name}</strong>
              <div className="muted">
                {new Date(item.savedAt).toLocaleString(i18n.language)}
                {item.originalPath ? ` · ${item.originalPath}` : ""}
              </div>
            </div>
            <button
              type="button"
              className="btn primary small"
              onClick={() => void recover([item.id], true)}
            >
              {t("ui.editor.recover")}
            </button>
            <button
              type="button"
              className="btn small"
              onClick={() => void recover([item.id], false)}
            >
              {t("ui.editor.discard")}
            </button>
          </li>
        ))}
      </ul>
    </Dialog>
  );
}

function ErrorMessage() {
  const { t } = useTranslation();
  const error = useEditor((s) => s.error);
  if (!error) return null;
  const close = () => useEditor.setState({ error: null });
  return (
    <Dialog
      title={t("ui.editor.error_title")}
      onCancel={close}
      actions={
        <button type="button" className="btn primary" onClick={close}>
          {t("ui.common.close")}
        </button>
      }
    >
      <p>{error}</p>
    </Dialog>
  );
}

export function Dialogs() {
  return (
    <>
      <Decision />
      <Flatten />
      <Recovery />
      <ErrorMessage />
    </>
  );
}

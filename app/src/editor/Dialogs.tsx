// Editor dialogs: unsaved changes (EDT-108, APP-004), flattening a PNG or JPG
// (EDT-107), Recover work (APP-047) and errors.
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { AiDialogs } from "./AiDialogs";
import { FileDialogs } from "../library/FileDialogs";
import { decide, flattenNow, recover, saveActive, setStepValue } from "./actions";
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
  const shown = useEditor((s) => s.recoveryShown);
  // "Later" keeps the items: Help > Recover work shows them again (APP-047).
  const later = () => useEditor.setState({ recoveryShown: false });
  if (items.length === 0 || !shown) return null;
  return (
    <Dialog
      title={t("ui.editor.recover_title")}
      onCancel={later}
      actions={
        <button type="button" className="btn" onClick={later}>
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

/** Edit value of a step (EDT-035): up to six characters. */
function StepValue() {
  const { t } = useTranslation();
  const edit = useEditor((s) => s.stepEdit);
  const [value, setValue] = useState<string | null>(null);
  if (!edit) return null;
  const close = () => {
    setValue(null);
    useEditor.setState({ stepEdit: null });
  };
  const current = value ?? edit.value;
  const ok = () => {
    close();
    if (current.trim() !== edit.value) void setStepValue(edit.id, current);
  };
  return (
    <Dialog
      title={t("ui.editor.step_edit_title")}
      onCancel={close}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            disabled={current.trim() === ""}
            onClick={ok}
          >
            {t("ui.editor.ok")}
          </button>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <label className="prop-field">
        <span>{t("ui.editor.step_edit_label")}</span>
        <input
          type="text"
          maxLength={6}
          data-autofocus
          value={current}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && current.trim() !== "") {
              e.preventDefault();
              ok();
            }
          }}
        />
      </label>
    </Dialog>
  );
}

/** Flattening a blur cannot be undone with Unflatten (EDT-091). */
function FlattenBlur() {
  const { t } = useTranslation();
  const ask = useEditor((s) => s.flattenAsk);
  if (!ask) return null;
  const close = () => useEditor.setState({ flattenAsk: null });
  return (
    <Dialog
      title={t("ui.editor.flatten_blur_title")}
      onCancel={close}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            onClick={() => {
              close();
              void flattenNow(ask.all);
            }}
          >
            {t("ui.editor.flatten_blur_ok")}
          </button>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <p>{t("ui.editor.flatten_blur_body")}</p>
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
      <FlattenBlur />
      <StepValue />
      <ErrorMessage />
      <AiDialogs />
      <FileDialogs />
    </>
  );
}

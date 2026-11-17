// Delete (LIB-015, TRY-009) and Rename (TRY-006) dialogs, shared by the Library view
// and the recent tray.
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { deleteFiles, renameFile, useFiles, type DeleteRequest, type FileRef } from "./actions";

/** At most this many names are listed (LIB-015). */
const NAMES = 5;

function DeleteDialog({ request }: { request: DeleteRequest }) {
  const { t } = useTranslation();
  const [dontAsk, setDontAsk] = useState(false);
  const { files, unsaved, forever } = request;
  const cancel = () => useFiles.setState({ deleting: null });
  const first = files[0]?.name ?? "";
  return (
    <Dialog
      title={
        forever
          ? t("ui.files.forever_title")
          : t("ui.files.delete_title", { count: files.length, name: first })
      }
      onCancel={cancel}
      actions={
        <>
          <button
            type="button"
            className={`btn ${forever ? "danger" : "primary"}`}
            onClick={() => void deleteFiles(files, forever, dontAsk)}
          >
            {forever ? t("ui.files.forever") : t("ui.files.delete")}
          </button>
          <button type="button" className="btn" onClick={cancel}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <p>
        {forever ? t("ui.files.forever_body") : t("ui.files.delete_body", { count: files.length })}
      </p>
      {files.length > 1 && (
        <ul className="delete-names">
          {files.slice(0, NAMES).map((f, i) => (
            <li key={`${f.path ?? f.name}-${i}`}>{f.name}</li>
          ))}
          {files.length > NAMES && (
            <li className="muted">{t("ui.files.and_more", { count: files.length - NAMES })}</li>
          )}
        </ul>
      )}
      {unsaved > 0 && (
        <p className="warning-text">{t("ui.files.delete_unsaved", { count: unsaved })}</p>
      )}
      {/* Deleting for good always asks, whatever the preference (LIB-015). */}
      {!forever && (
        <label className="panel-check">
          <input type="checkbox" checked={dontAsk} onChange={(e) => setDontAsk(e.target.checked)} />
          {t("ui.files.dont_ask_again")}
        </label>
      )}
    </Dialog>
  );
}

function RenameDialog({ file }: { file: FileRef }) {
  const { t } = useTranslation();
  const [name, setName] = useState(file.name);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  // The dialog focuses its first button first (its effect runs before this one); the
  // name is what the user came for.
  useEffect(() => {
    input.current?.focus();
    input.current?.select();
  }, []);
  const cancel = () => useFiles.setState({ renaming: null });
  const submit = async () => {
    if (!name.trim() || busy) return;
    setBusy(true);
    setError(await renameFile(file, name.trim()));
    setBusy(false);
  };
  return (
    <Dialog
      title={t("ui.files.rename_title")}
      onCancel={cancel}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            disabled={!name.trim() || busy}
            onClick={() => void submit()}
          >
            {t("ui.files.rename")}
          </button>
          <button type="button" className="btn" onClick={cancel}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <label className="field">
        <span>{t("ui.files.rename_label")}</span>
        <input
          type="text"
          value={name}
          ref={input}
          aria-invalid={error ? true : undefined}
          aria-describedby={error ? "rename-error" : undefined}
          onChange={(e) => {
            setName(e.target.value);
            setError(null);
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter") void submit();
          }}
        />
      </label>
      {error && (
        <p id="rename-error" className="warning-text" role="alert">
          {error}
        </p>
      )}
    </Dialog>
  );
}

export function FileDialogs() {
  const deleting = useFiles((s) => s.deleting);
  const renaming = useFiles((s) => s.renaming);
  return (
    <>
      {deleting && <DeleteDialog request={deleting} />}
      {renaming && <RenameDialog file={renaming} />}
    </>
  );
}

// Import to the library (LIB-024): what is about to be copied and whether each
// folder's name becomes a tag of its images (on by default).
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { importPaths, looksLikeFolder, useImport } from "./importState";

export function ImportDialog() {
  const paths = useImport((s) => s.paths);
  return paths ? <ImportForm paths={paths} /> : null;
}

function ImportForm({ paths }: { paths: string[] }) {
  const { t } = useTranslation();
  const [tags, setTags] = useState(true);
  const close = () => useImport.setState({ paths: null });
  const names = paths.map((p) => p.split(/[\\/]/).pop() ?? p);
  const folders = paths.filter(looksLikeFolder).map((p) => p.split(/[\\/]/).pop() ?? p);
  return (
    <Dialog
      title={t("ui.library.import.title")}
      onCancel={close}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            data-autofocus
            onClick={() => void importPaths(paths, tags)}
          >
            {t("ui.library.import.go")}
          </button>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <p className="muted">{t("ui.library.import.help")}</p>
      <ul className="batch-files">
        {names.slice(0, 8).map((n, i) => (
          <li key={`${n}-${i}`}>
            <span className="batch-name">{n}</span>
          </li>
        ))}
        {names.length > 8 && (
          <li className="muted">{t("ui.library.import.more", { count: names.length - 8 })}</li>
        )}
      </ul>
      {folders.length > 0 && (
        <label className="panel-check">
          <input type="checkbox" checked={tags} onChange={(e) => setTags(e.target.checked)} />
          {folders.length === 1
            ? t("ui.library.import.tag_one", { name: folders[0] })
            : t("ui.library.import.tag_many")}
        </label>
      )}
    </Dialog>
  );
}

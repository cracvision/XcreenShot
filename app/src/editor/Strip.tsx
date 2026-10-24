// Strip of open documents at the foot of the editor: the basic Recent tray of build 4
// (TRY-002, TRY-011). Pinning, persistence and the rest arrive with build 5.
import { useTranslation } from "react-i18next";
import { closeDocument, showDocument } from "./actions";
import { Icon } from "./icons";
import { docUrl, useEditor } from "./state";

export function Strip() {
  const { t } = useTranslation();
  const all = useEditor((s) => s.documents);
  const documents = all.filter((d) => d.shown);
  const active = useEditor((s) => s.active);
  if (documents.length === 0) return null;
  return (
    <nav className="strip" aria-label={t("ui.editor.strip")}>
      <ul>
        {documents.map((d) => {
          const current = d.documentId === active;
          const name = d.dirty ? `${d.name}, ${t("ui.editor.unsaved")}` : d.name;
          return (
            <li key={d.documentId} className={current ? "current" : undefined}>
              <button
                type="button"
                className="strip-item"
                aria-current={current ? "true" : undefined}
                aria-label={current ? t("ui.editor.strip_open", { name }) : name}
                title={`${d.name}\n${d.width} × ${d.height} px`}
                onClick={() => showDocument(d.documentId)}
              >
                <img src={docUrl(d.documentId, "thumb", d.version)} alt="" draggable={false} />
                {d.dirty && (
                  <span className="dirty-dot" title={t("ui.editor.unsaved")} aria-hidden="true" />
                )}
              </button>
              <button
                type="button"
                className="strip-close"
                aria-label={t("ui.editor.close_named", { name: d.name })}
                title={t("ui.editor.close")}
                onClick={() => void closeDocument(d.documentId)}
              >
                <Icon name="close" />
              </button>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}

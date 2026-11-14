// Help > Keyboard shortcuts (APP-023, Ctrl+/): every current shortcut, grouped, with
// the keys that only work in a context (a selection on screen, the canvas, lists), a
// search field, Print and Save as PDF.
import { save } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import { CONTEXTUAL, matches, useShortcutSheet, type Sheet } from "./sheetData";
import { useEditor } from "./state";

export function ShortcutSheet() {
  const open = useShortcutSheet((s) => s.open);
  return open ? <SheetDialog /> : null;
}

function SheetDialog() {
  const { t } = useTranslation();
  const [sheet, setSheet] = useState<Sheet | null>(null);
  const [query, setQuery] = useState("");
  const close = () => useShortcutSheet.setState({ open: false });
  useEffect(() => {
    void run<{ sheet: Sheet }>("shortcuts.export")
      .then((r) => setSheet(r.sheet))
      .catch((e: unknown) => useEditor.getState().showError(e));
  }, []);
  const savePdf = async () => {
    const out = await save({
      defaultPath: `${t("ui.shortcuts.sheet_title")}.pdf`,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!out) return;
    try {
      await run("shortcuts.export", { format: "pdf", out });
      useEditor.getState().showToast(t("ui.shortcuts.sheet_saved"));
    } catch (e) {
      useEditor.getState().showError(e);
    }
  };
  const groups = [
    ...(sheet?.groups ?? []),
    ...CONTEXTUAL.map((g) => ({
      id: g.id,
      title: t(`ui.shortcuts.sheet_group_${g.id}`),
      rows: g.rows.map((r) => ({
        title: t(r.label),
        keys: r.keys.startsWith("@") ? t(r.keys.slice(1)) : r.keys,
        inactive: false,
      })),
    })),
  ]
    .map((g) => ({ ...g, rows: g.rows.filter((r) => matches(r, query)) }))
    .filter((g) => g.rows.length > 0);
  return (
    <Dialog
      title={t("ui.shortcuts.sheet_title")}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={() => window.print()}>
            {t("ui.shortcuts.sheet_print")}
          </button>
          <button type="button" className="btn" onClick={() => void savePdf()}>
            {t("ui.shortcuts.sheet_pdf")}
          </button>
          <button type="button" className="btn primary" onClick={close}>
            {t("ui.editor.image.done")}
          </button>
        </>
      }
    >
      <div className="shortcut-sheet">
        {sheet && (
          <p className="muted sheet-meta">
            {sheet.profile} · {sheet.platform} · {sheet.date}
          </p>
        )}
        <input
          type="search"
          className="lib-search sheet-search"
          data-autofocus
          placeholder={t("ui.shortcuts.sheet_search")}
          aria-label={t("ui.shortcuts.sheet_search")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <div className="sheet-groups">
          <div className="sheet-cols">
            {groups.length === 0 && sheet && (
              <p className="muted">{t("ui.shortcuts.sheet_none")}</p>
            )}
            {groups.map((g) => (
              <section key={g.id} aria-labelledby={`sheet-${g.id}`}>
                <h3 id={`sheet-${g.id}`}>{g.title}</h3>
                <table>
                  <tbody>
                    {g.rows.map((r, i) => (
                      <tr key={`${r.title}-${i}`} className={r.inactive ? "inactive" : undefined}>
                        <td>{r.title}</td>
                        <td className="sheet-keys">
                          <kbd>{r.keys}</kbd>
                          {r.inactive && (
                            <span className="muted"> ({t("ui.shortcuts.sheet_inactive")})</span>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </section>
            ))}
          </div>
        </div>
      </div>
    </Dialog>
  );
}

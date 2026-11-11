// Recent tray at the foot of the Editor view (TRY-001, TRY-002, TRY-008, TRY-009,
// TRY-011, TRY-013, TRY-015): the documents open in the editor, the most recent
// first. Files moved or deleted outside XcreenShot show "Not found" with Find… and
// Remove. An item dragged out hands a file to another app.
import { useLayoutEffect, useRef, useState, type KeyboardEvent, type MouseEvent } from "react";
import { useTranslation } from "react-i18next";
import { removeFromTray } from "../library/actions";
import { watchDrag } from "../library/dragOut";
import { useTray } from "../library/state";
import { closeDocument, showDocument } from "./actions";
import { Icon } from "./icons";
import { menuPoint, useContextMenu } from "./menus";
import { docUrl, useEditor, type DocSummary } from "./state";
import { trayOrder } from "./tray";
import { trayMenu } from "./trayMenu";

export function Strip() {
  const { t } = useTranslation();
  const all = useEditor((s) => s.documents);
  const active = useEditor((s) => s.active);
  const visible = useTray((s) => s.visible);
  const [focus, setFocus] = useState<string | null>(null);
  const list = useRef<HTMLUListElement>(null);
  const keyed = useRef(false);
  const documents = trayOrder(all);
  const order = documents.map((d) => d.documentId);

  useLayoutEffect(() => {
    if (!keyed.current || !focus) return;
    keyed.current = false;
    const el = list.current?.querySelector<HTMLElement>(`[data-doc="${focus}"]`);
    el?.focus();
    el?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [focus]);

  if (!visible || documents.length === 0) return null;

  const current =
    focus && order.includes(focus) ? focus : active && order.includes(active) ? active : order[0];

  const onKey = (e: KeyboardEvent) => {
    const at = order.indexOf(current ?? "");
    const target =
      e.key === "ArrowLeft"
        ? order[Math.max(0, at - 1)]
        : e.key === "ArrowRight"
          ? order[Math.min(order.length - 1, at + 1)]
          : e.key === "Home"
            ? order[0]
            : e.key === "End"
              ? order[order.length - 1]
              : undefined;
    if (!target || e.ctrlKey || e.metaKey || e.altKey) return;
    e.preventDefault();
    keyed.current = true;
    setFocus(target);
  };

  const menu = (doc: DocSummary, e: MouseEvent) => {
    e.preventDefault();
    setFocus(doc.documentId);
    useContextMenu.setState({
      open: true,
      ...menuPoint(e),
      label: t("ui.editor.strip_menu"),
      items: trayMenu(doc, documents.length, t),
    });
  };

  return (
    <nav
      className="strip"
      aria-label={t("menu.tray")}
      // The mouse wheel scrolls the tray sideways, also on Windows (TRY-013).
      onWheel={(e) => {
        if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) e.currentTarget.scrollLeft += e.deltaY;
      }}
    >
      <ul ref={list} onKeyDown={onKey}>
        {documents.map((d) => {
          const shown = d.documentId === active;
          let name = d.dirty ? `${d.name}, ${t("ui.editor.unsaved")}` : d.name;
          if (d.missing) name = `${name}, ${t("ui.editor.not_found")}`;
          const dirtyHint =
            d.kind === "external"
              ? `${t("ui.editor.unsaved")}\n${t("ui.editor.not_in_library")}`
              : t("ui.editor.unsaved");
          return (
            <li
              key={d.documentId}
              className={
                `${shown ? "current" : ""}${d.missing ? " missing" : ""}`.trim() || undefined
              }
            >
              <button
                type="button"
                className="strip-item"
                data-doc={d.documentId}
                tabIndex={d.documentId === current ? 0 : -1}
                aria-current={shown ? "true" : undefined}
                aria-label={shown ? t("ui.editor.strip_open", { name }) : name}
                title={
                  d.missing
                    ? `${d.name}\n${t("ui.editor.not_found_help")}`
                    : `${d.name}\n${d.width} × ${d.height} px`
                }
                onClick={() => {
                  setFocus(d.documentId);
                  showDocument(d.documentId);
                }}
                onContextMenu={(e) => menu(d, e)}
                // Dragged out, it hands a file to another app (TRY-015).
                onPointerDown={(e) =>
                  watchDrag(e, () => (d.missing ? null : { docs: [d.documentId], paths: [] }))
                }
              >
                <img src={docUrl(d.documentId, "thumb", d.version)} alt="" draggable={false} />
                {d.missing && (
                  <span className="strip-missing" aria-hidden="true">
                    {t("ui.editor.not_found")}
                  </span>
                )}
                {d.dirty && <span className="dirty-dot" title={dirtyHint} aria-hidden="true" />}
              </button>
              <button
                type="button"
                className="strip-close"
                tabIndex={-1}
                aria-label={t("ui.editor.close_named", { name: d.name })}
                title={t("ui.editor.close")}
                onClick={() => (d.missing ? removeFromTray(d) : void closeDocument(d.documentId))}
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

// Recent tray at the foot of the Editor view (TRY-001 to TRY-009, TRY-011, TRY-013,
// TRY-015): the documents open in the editor, pinned ones first, then the newest.
// Files moved or deleted outside XcreenShot show "Not found" with Find… and Remove.
// Dragged inside the tray an item moves (TRY-005); dragged out, it hands a file to
// another app.
import {
  useLayoutEffect,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent,
  type MouseEvent,
} from "react";
import { useTranslation } from "react-i18next";
import { removeFromTray } from "../library/actions";
import { watchDrag } from "../library/dragOut";
import { useTray } from "../library/state";
import { showQuickView } from "../library/quickViewState";
import { run } from "../lib/commands";
import { settingValue, useApp } from "../store";
import { clickSelection } from "../library/selection";
import { closeDocument, showDocument, togglePin } from "./actions";
import { Icon } from "./icons";
import { menuPoint, useContextMenu } from "./menus";
import { docUrl, useEditor, xsdocUrl, type DocSummary } from "./state";
import { dropIndex, trayOrder, TRAY_SIZES, type TraySize } from "./tray";
import { trayMenu } from "./trayMenu";

const IS_MAC = navigator.userAgent.includes("Mac");

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

export function Strip() {
  const { t } = useTranslation();
  const all = useEditor((s) => s.documents);
  const tray = useEditor((s) => s.tray);
  const active = useEditor((s) => s.active);
  const visible = useTray((s) => s.visible);
  const picked = useTray((s) => s.selected);
  const size = useApp((s) => settingValue(s.settings, "tray.thumbnailSize")) as TraySize;
  const capacity = useApp((s) => settingValue(s.settings, "tray.capacity"));
  const [focus, setFocus] = useState<string | null>(null);
  const [drop, setDrop] = useState<{ id: string; index: number } | null>(null);
  const [said, setSaid] = useState("");
  const list = useRef<HTMLUListElement>(null);
  const keyed = useRef(false);
  const documents = trayOrder(all, tray);
  const order = documents.map((d) => d.documentId);
  const pinnedCount = documents.filter((d) => d.pinned).length;
  // TRY-012: the selection, without items that left the tray.
  const selected = picked.filter((id) => order.includes(id));

  useLayoutEffect(() => {
    if (!keyed.current || !focus) return;
    keyed.current = false;
    const el = list.current?.querySelector<HTMLElement>(`[data-doc="${focus}"]`);
    el?.focus();
    el?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [focus, tray]);

  if (!visible || documents.length === 0) return null;

  const current =
    focus && order.includes(focus) ? focus : active && order.includes(active) ? active : order[0];

  /** Moves `id` to `index` (TRY-005) and says where it is now. */
  const moveTo = (id: string, index: number) => {
    void run<{ index: number; pinned: boolean }>("tray.move", { documentId: id, index })
      .then((r) => {
        keyed.current = true;
        setFocus(id);
        const where = t("ui.editor.tray.moved", { at: r.index + 1, count: order.length });
        setSaid(r.pinned ? `${where}, ${t("ui.editor.tray.pinned")}` : where);
      })
      .catch(fail);
  };

  const onKey = (e: KeyboardEvent) => {
    // The item with the focus (Tab lands on the current one).
    const subject = (e.target as HTMLElement).dataset.doc ?? current;
    const at = order.indexOf(subject ?? "");
    const primary = IS_MAC ? e.metaKey : e.ctrlKey;
    // TRY-005: Ctrl+Shift+← / → (Cmd on macOS) move the item one place.
    if (primary && e.shiftKey && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
      e.preventDefault();
      const to = at + (e.key === "ArrowLeft" ? -1 : 1);
      if (subject && to >= 0 && to < order.length) moveTo(subject, to);
      return;
    }
    // TRY-012: Ctrl+A (Cmd+A) selects the whole tray.
    if (primary && !e.shiftKey && e.code === "KeyA") {
      e.preventDefault();
      useTray.getState().select(order, subject ?? null);
      return;
    }
    if (e.key === " " && !e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey) {
      // LIB-013: the quick view, also in the tray.
      e.preventDefault();
      showQuickView({
        entries: documents.map((d) => ({
          id: d.documentId,
          name: d.name,
          src: xsdocUrl(`${d.documentId}/final?v=${d.version}`),
          thumb: docUrl(d.documentId, "thumb", d.version),
          width: d.width,
          height: d.height,
          date: null,
        })),
        // From the item with the focus.
        at,
        open: (id) => showDocument(id),
        done: (id) => {
          setFocus(id);
          requestAnimationFrame(() =>
            list.current?.querySelector<HTMLElement>(`[data-doc="${id}"]`)?.focus(),
          );
        },
      });
      return;
    }
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
    e.stopPropagation();
    setFocus(doc.documentId);
    useContextMenu.setState({
      open: true,
      ...menuPoint(e),
      label: t("ui.editor.strip_menu"),
      items: trayMenu(doc, documents, t, selected),
    });
  };

  // An empty place of the tray: the size of the thumbnails (TRY-007).
  const emptyMenu = (e: MouseEvent) => {
    if ((e.target as HTMLElement).closest("[data-doc]")) return;
    e.preventDefault();
    useContextMenu.setState({
      open: true,
      ...menuPoint(e),
      label: t("ui.editor.tray.menu"),
      items: TRAY_SIZES.map((s) => ({
        id: `size_${s}`,
        label: `${t("menu.tray_size")}: ${t(`settings.tray_thumbnailSize.options.${s}`)}`,
        checked: (size ?? "medium") === s,
        onSelect: () =>
          void run("settings.set", { key: "tray.thumbnailSize", value: s }).catch(fail),
      })),
    });
  };

  /** Where an item dragged to `x` would go among the others (TRY-005). */
  const indexAt = (id: string, x: number) => {
    const mids = Array.from(list.current?.querySelectorAll<HTMLElement>("[data-doc]") ?? [])
      .filter((el) => el.dataset.doc !== id)
      .map((el) => {
        const r = el.getBoundingClientRect();
        return r.left + r.width / 2;
      });
    return dropIndex(mids, x);
  };

  const others = drop ? order.filter((id) => id !== drop.id) : [];
  const markBefore = drop ? others[drop.index] : undefined;
  const markAfter = drop && drop.index >= others.length ? others[others.length - 1] : undefined;
  const height = size === "small" ? 48 : size === "large" ? 112 : 72;
  const over =
    typeof capacity === "number" && documents.length > capacity
      ? { count: documents.length, capacity }
      : null;

  return (
    <nav
      className="strip"
      aria-label={t("menu.tray")}
      style={{ "--tray-h": `${height}px` } as CSSProperties}
      // The mouse wheel scrolls the tray sideways, also on Windows (TRY-013).
      onWheel={(e) => {
        const el = list.current;
        if (el && Math.abs(e.deltaY) > Math.abs(e.deltaX)) el.scrollLeft += e.deltaY;
      }}
      onContextMenu={emptyMenu}
    >
      <ul ref={list} onKeyDown={onKey}>
        {documents.map((d, i) => {
          const shown = d.documentId === active;
          let name = d.dirty ? `${d.name}, ${t("ui.editor.unsaved")}` : d.name;
          if (d.pinned) name = `${name}, ${t("ui.editor.tray.pinned")}`;
          if (d.missing) name = `${name}, ${t("ui.editor.not_found")}`;
          const dirtyHint =
            d.kind === "external"
              ? `${t("ui.editor.unsaved")}\n${t("ui.editor.not_in_library")}`
              : t("ui.editor.unsaved");
          const classes = [
            shown ? "current" : "",
            d.missing ? "missing" : "",
            d.pinned ? "pinned" : "",
            markBefore === d.documentId ? "drop-before" : "",
            markAfter === d.documentId ? "drop-after" : "",
            drop?.id === d.documentId ? "dragging" : "",
            selected.length > 1 && selected.includes(d.documentId) ? "selected" : "",
          ]
            .filter(Boolean)
            .join(" ");
          return [
            // A thin line between the pinned items and the rest (TRY-004).
            i === pinnedCount && pinnedCount > 0 ? (
              <li key="sep" className="strip-sep" role="presentation" aria-hidden="true" />
            ) : null,
            <li key={d.documentId} className={classes || undefined}>
              <button
                type="button"
                className="strip-item"
                data-doc={d.documentId}
                tabIndex={d.documentId === current ? 0 : -1}
                aria-current={shown ? "true" : undefined}
                aria-pressed={selected.length > 1 ? selected.includes(d.documentId) : undefined}
                aria-label={shown ? t("ui.editor.strip_open", { name }) : name}
                title={
                  d.missing
                    ? `${d.name}\n${t("ui.editor.not_found_help")}`
                    : `${d.name}\n${d.width} × ${d.height} px`
                }
                onClick={(e) => {
                  setFocus(d.documentId);
                  const toggle = IS_MAC ? e.metaKey : e.ctrlKey;
                  if (toggle || e.shiftKey) {
                    // TRY-012: Ctrl+click adds or removes, Shift+click a range; the
                    // canvas keeps showing the active item.
                    const { anchor } = useTray.getState();
                    const base = selected.length > 0 ? selected : active ? [active] : [];
                    const r = clickSelection(order, base, anchor ?? active, d.documentId, {
                      toggle,
                      range: e.shiftKey,
                    });
                    useTray.getState().select(r.selection, r.anchor);
                    return;
                  }
                  useTray.getState().select([d.documentId], d.documentId);
                  showDocument(d.documentId);
                }}
                onContextMenu={(e) => menu(d, e)}
                // Inside the tray it moves the item (TRY-005); out of it, it hands a
                // file to another app (TRY-015).
                onPointerDown={(e) =>
                  watchDrag(e, () => (d.missing ? null : { docs: [d.documentId], paths: [] }), {
                    area: () => list.current?.getBoundingClientRect() ?? null,
                    over: (x) => setDrop({ id: d.documentId, index: indexAt(d.documentId, x) }),
                    drop: (x) => {
                      setDrop(null);
                      const index = indexAt(d.documentId, x);
                      if (index !== order.indexOf(d.documentId)) moveTo(d.documentId, index);
                    },
                    leave: () => setDrop(null),
                  })
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
              {!d.missing && (
                <button
                  type="button"
                  className="strip-pin"
                  tabIndex={-1}
                  aria-pressed={d.pinned}
                  aria-label={t(d.pinned ? "ui.editor.tray.unpin" : "ui.editor.tray.pin", {
                    name: d.name,
                  })}
                  title={t(d.pinned ? "menu.unpin_item" : "menu.pin_item")}
                  onClick={() => void togglePin([d.documentId])}
                >
                  <Icon name="pin" />
                </button>
              )}
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
            </li>,
          ];
        })}
        {over && (
          <li className="strip-over" title={t("ui.editor.tray.over_help")}>
            {t("ui.editor.tray.over", over)}
          </li>
        )}
      </ul>
      <div className="sr-only" role="status" aria-live="polite">
        {said}
      </div>
    </nav>
  );
}

// The Library view of the main window (LIB-001, LIB-002, LIB-026): every capture of
// the library folder, newest first, with selection by mouse and keyboard, a rubber band,
// the item menu and a status bar. Filters, search, sorting and thumbnail sizes arrive
// with M2 (LIB-003 to LIB-007).
import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type KeyboardEvent,
  type MouseEvent,
  type PointerEvent,
} from "react";
import { useTranslation } from "react-i18next";
import { LanguageToggle } from "../components/LanguageToggle";
import { run } from "../lib/commands";
import { openCapturePanel, openFiles } from "../editor/actions";
import { Icon } from "../editor/icons";
import { menuPoint, useContextMenu } from "../editor/menus";
import { xsdocUrl } from "../editor/state";
import { hint, useShortcut } from "../editor/tools";
import { askDelete, itemRef, openItems } from "./actions";
import { watchDrag } from "./dragOut";
import { itemMenu } from "./itemMenu";
import { boxOf, clickSelection, columnsOf, moveFocus, overlaps, range } from "./selection";
import { selectedItems, useLibrary, type LibraryItem } from "./state";
import { ViewSwitch } from "./ViewSwitch";

const IS_MAC = navigator.userAgent.includes("Mac");

function thumbUrl(item: LibraryItem): string {
  return xsdocUrl(`library/${item.id}/thumb?v=${encodeURIComponent(item.modified)}`);
}

function typeName(item: LibraryItem, t: (key: string) => string): string {
  const format = item.format === "jpeg" ? "jpg" : item.format;
  return t(`ui.library.type_${format}`);
}

function LibraryToolbar() {
  const { t } = useTranslation();
  const openKey = useShortcut("editor.open");
  return (
    <div
      className="editor-toolbar library-toolbar"
      role="toolbar"
      aria-label={t("ui.library.title")}
    >
      <ViewSwitch />
      <button type="button" className="btn primary tb-btn tb-capture" onClick={openCapturePanel}>
        <Icon name="capture" />
        <span className="tb-label">{t("ui.editor.capture")}</span>
      </button>
      <div className="tb-spacer" />
      <div className="tb-group">
        <button
          type="button"
          className="btn tb-btn"
          title={t("ui.library.open_folder")}
          onClick={() => void run("library.show_in_folder").catch(() => undefined)}
        >
          <Icon name="library" />
          <span className="tb-label tb-wide">{t("ui.library.open_folder")}</span>
        </button>
        <button
          type="button"
          className="btn tb-btn"
          title={hint(t("ui.editor.open"), openKey)}
          onClick={() => void openFiles()}
        >
          <Icon name="open" />
          <span className="tb-label tb-wide">{t("ui.editor.open")}</span>
        </button>
      </div>
      <LanguageToggle />
      <button
        type="button"
        className="btn icon"
        title={t("ui.editor.preferences")}
        aria-label={t("ui.editor.preferences")}
        onClick={() => void run("settings.open")}
      >
        <Icon name="settings" />
      </button>
    </div>
  );
}

function Item({
  item,
  selected,
  focusable,
  onPick,
  onOpen,
  onMenu,
  onPress,
}: {
  item: LibraryItem;
  selected: boolean;
  focusable: boolean;
  onPick: (item: LibraryItem, e: MouseEvent) => void;
  onOpen: (item: LibraryItem) => void;
  onMenu: (item: LibraryItem, e: MouseEvent) => void;
  onPress: (item: LibraryItem, e: PointerEvent) => void;
}) {
  const { t, i18n } = useTranslation();
  const date = new Date(item.captured).toLocaleString(i18n.language, {
    dateStyle: "medium",
    timeStyle: "short",
  });
  const size =
    item.width && item.height
      ? t("ui.library.size", { width: item.width, height: item.height })
      : "";
  return (
    <div
      role="option"
      aria-selected={selected}
      tabIndex={focusable ? 0 : -1}
      data-item={item.id}
      className={`lib-item${selected ? " selected" : ""}`}
      aria-label={t("ui.library.item_label", { name: item.name, type: typeName(item, t), date })}
      title={[item.name, size, date].filter(Boolean).join("\n")}
      onClick={(e) => onPick(item, e)}
      onDoubleClick={() => onOpen(item)}
      onContextMenu={(e) => onMenu(item, e)}
      onPointerDown={(e) => onPress(item, e)}
    >
      <div className="lib-thumb">
        <img src={thumbUrl(item)} alt="" loading="lazy" draggable={false} />
        {item.format !== "xshot" && (
          <span className="lib-badge" aria-hidden="true">
            {item.format === "jpeg" ? "JPG" : item.format.toUpperCase()}
          </span>
        )}
      </div>
      <div className="lib-name">{item.name}</div>
      <div className="lib-date">{date}</div>
    </div>
  );
}

interface Band {
  x: number;
  y: number;
  x2: number;
  y2: number;
  base: string[];
}

function Grid() {
  const { t } = useTranslation();
  const items = useLibrary((s) => s.items);
  const selection = useLibrary((s) => s.selection);
  const focus = useLibrary((s) => s.focus);
  const select = useLibrary((s) => s.select);
  const grid = useRef<HTMLDivElement>(null);
  const [band, setBand] = useState<Band | null>(null);
  /** The focus moves to the item only after a key, not after a click or a reload. */
  const keyed = useRef(false);
  const order = items.map((i) => i.id);
  const chosen = new Set(selection);

  useLayoutEffect(() => {
    if (!keyed.current || !focus) return;
    keyed.current = false;
    const el = grid.current?.querySelector<HTMLElement>(`[data-item="${focus}"]`);
    el?.focus();
    el?.scrollIntoView({ block: "nearest" });
  }, [focus]);

  const pick = (item: LibraryItem, e: MouseEvent) => {
    const { selection: current, anchor } = useLibrary.getState();
    const r = clickSelection(order, current, anchor, item.id, {
      toggle: IS_MAC ? e.metaKey : e.ctrlKey,
      range: e.shiftKey,
    });
    select(r.selection, item.id, r.anchor);
  };

  const menu = (item: LibraryItem, e: MouseEvent) => {
    e.preventDefault();
    if (!useLibrary.getState().selection.includes(item.id)) select([item.id], item.id, item.id);
    useContextMenu.setState({
      open: true,
      ...menuPoint(e),
      label: t("ui.library.item_menu"),
      items: itemMenu(selectedItems(), t),
    });
  };

  // Dragging an item out drags the selection if it is in it, or else only that item,
  // which becomes the selection (TRY-015).
  const press = (item: LibraryItem, e: PointerEvent) =>
    watchDrag(e, () => {
      if (!useLibrary.getState().selection.includes(item.id)) select([item.id], item.id, item.id);
      return { docs: [], paths: selectedItems().map((i) => i.path) };
    });

  const layout = () => {
    const els = Array.from(grid.current?.querySelectorAll<HTMLElement>("[data-item]") ?? []);
    const columns = columnsOf(els.map((el) => el.offsetTop));
    const height = els[0]?.offsetHeight ?? 1;
    const rows = Math.max(1, Math.floor((grid.current?.clientHeight ?? 0) / height));
    return { columns, page: rows * columns };
  };

  const onKey = (e: KeyboardEvent) => {
    const primary = IS_MAC ? e.metaKey : e.ctrlKey;
    const { selection: current, anchor, focus: at } = useLibrary.getState();
    if (primary && e.code === "KeyA") {
      e.preventDefault();
      select(order, at, at);
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      void openItems(selectedItems());
      return;
    }
    if (
      (!IS_MAC && e.key === "Delete" && !primary) ||
      (IS_MAC && e.metaKey && e.key === "Backspace")
    ) {
      e.preventDefault();
      askDelete(selectedItems().map(itemRef));
      return;
    }
    if (e.key === "Escape" && current.length > 0) {
      e.preventDefault();
      select([]);
      return;
    }
    const { columns, page } = layout();
    const next = moveFocus(order, at, e.key, columns, page, primary);
    if (!next) return;
    e.preventDefault();
    keyed.current = true;
    if (e.shiftKey) select(range(order, anchor ?? at ?? next, next), next, anchor ?? at ?? next);
    else select([next], next, next);
  };

  // Rubber band from an empty place of the grid (LIB-002).
  const down = (e: PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0 || e.target !== e.currentTarget) return;
    const box = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - box.left;
    const y = e.clientY - box.top + e.currentTarget.scrollTop;
    const additive = IS_MAC ? e.metaKey : e.ctrlKey;
    const base = additive ? useLibrary.getState().selection : [];
    e.currentTarget.setPointerCapture(e.pointerId);
    setBand({ x, y, x2: x, y2: y, base });
    if (!additive) select([], null, null);
  };

  const moveBand = (e: PointerEvent<HTMLDivElement>) => {
    if (!band) return;
    const box = e.currentTarget.getBoundingClientRect();
    const scroll = e.currentTarget.scrollTop;
    const x2 = e.clientX - box.left;
    const y2 = e.clientY - box.top + scroll;
    setBand({ ...band, x2, y2 });
    const area = boxOf(band.x + box.left, band.y + box.top - scroll, e.clientX, e.clientY);
    const hits = Array.from(e.currentTarget.querySelectorAll<HTMLElement>("[data-item]"))
      .filter((el) => overlaps(area, el.getBoundingClientRect()))
      .map((el) => el.dataset.item ?? "");
    select(order.filter((id) => band.base.includes(id) || hits.includes(id)));
  };

  const upBand = () => setBand(null);

  const firstFocusable = focus && order.includes(focus) ? focus : order[0];
  const shown = band ? boxOf(band.x, band.y, band.x2, band.y2) : null;
  return (
    <div
      ref={grid}
      className="library-grid"
      role="listbox"
      aria-multiselectable="true"
      aria-label={t("ui.library.grid")}
      onKeyDown={onKey}
      onPointerDown={down}
      onPointerMove={moveBand}
      onPointerUp={upBand}
      onPointerCancel={upBand}
    >
      {items.map((item) => (
        <Item
          key={item.id}
          item={item}
          selected={chosen.has(item.id)}
          focusable={item.id === firstFocusable}
          onPick={pick}
          onOpen={(i) => void openItems([i])}
          onMenu={menu}
          onPress={press}
        />
      ))}
      {shown && (
        <div
          className="rubber-band"
          aria-hidden="true"
          style={{
            left: shown.left,
            top: shown.top,
            width: shown.right - shown.left,
            height: shown.bottom - shown.top,
          }}
        />
      )}
    </div>
  );
}

function LibraryStatus() {
  const { t } = useTranslation();
  const total = useLibrary((s) => s.items.length);
  const selected = useLibrary((s) => s.selection.length);
  const root = useLibrary((s) => s.root);
  const synced = useLibrary((s) => s.synced);
  return (
    <footer className="status-bar library-status">
      <span className="status-item">{t("ui.library.count", { count: total })}</span>
      {selected > 0 && (
        <span className="status-item" role="status">
          {t("ui.library.selected", { count: selected })}
        </span>
      )}
      <span className="grow" />
      {synced && (
        <span className="lib-synced" title={t("ui.library.synced_help", { path: root })}>
          {t("ui.library.synced")}
        </span>
      )}
      <span className="lib-root muted" title={root}>
        {root}
      </span>
    </footer>
  );
}

function EmptyLibrary() {
  const { t } = useTranslation();
  return (
    <div className="editor-empty">
      <div className="empty-card">
        <h1>{t("ui.library.empty_title")}</h1>
        <p className="muted">{t("ui.library.empty_help")}</p>
        <div className="row">
          <button type="button" className="btn primary" onClick={openCapturePanel}>
            {t("ui.editor.capture")}
          </button>
        </div>
      </div>
    </div>
  );
}

export function LibraryView() {
  const { t } = useTranslation();
  const loaded = useLibrary((s) => s.loaded);
  const empty = useLibrary((s) => s.items.length === 0);

  useEffect(() => {
    void useLibrary
      .getState()
      .load()
      .catch(() => undefined);
  }, []);

  return (
    <>
      <LibraryToolbar />
      <div className="library-main">
        {!loaded ? (
          <p className="library-loading muted" role="status">
            {t("ui.library.loading")}
          </p>
        ) : empty ? (
          <EmptyLibrary />
        ) : (
          <Grid />
        )}
      </div>
      <LibraryStatus />
    </>
  );
}

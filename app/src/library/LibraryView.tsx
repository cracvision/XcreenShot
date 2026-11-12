// The Library view of the main window (LIB-001, LIB-002, LIB-026): the captures of the
// library folder as a grid or a list, sorted (LIB-004), filtered from the left pane
// (LIB-005, LIB-006) and searched (LIB-007); selection by mouse and keyboard, a rubber
// band, favorites (LIB-011), tags (LIB-009), the item menu and a status bar.
import { createItems } from "./create";
import { COLUMNS, MIN_WIDTH, shownColumns, template } from "./columns";
import { listen } from "@tauri-apps/api/event";
import {
  keepStatusExpanded,
  statusExpanded,
  taskDone,
  taskProgress,
  totalSize,
  useLibraryTasks,
} from "./tasks";
import { setZoom, zoomOf, ZOOM_MAX, ZOOM_MIN, ZOOM_STEP } from "./zoom";
import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent,
  type MouseEvent,
  type PointerEvent,
} from "react";
import { useTranslation } from "react-i18next";
import { LanguageToggle } from "../components/LanguageToggle";
import { useIndicator } from "../components/useIndicator";
import { run } from "../lib/commands";
import { openCapturePanel, openFiles } from "../editor/actions";
import { Icon } from "../editor/icons";
import { menuPoint, useContextMenu } from "../editor/menus";
import { hint, useShortcut } from "../editor/tools";
import { askDelete, itemRef, openItems } from "./actions";
import { DetailsPane } from "./DetailsPane";
import { fileSize } from "./format";
import { libraryThumb, openLibraryQuickView } from "./quickViewState";
import { FilterPane } from "./FilterPane";
import { editTags, toggleFavorite } from "./marks";
import { NO_FILTERS, SORT_KEYS, chips, dateLabel, hasFilters, toggle, type SortKey } from "./query";
import { MaintenanceDialogs } from "./Maintenance";
import { MergeDialogs } from "./MergeDialogs";
import { askMerge, cancelMerge } from "./mergeState";
import { askRebuild, checkLibrary } from "./maintenanceState";
import { askExportLibrary } from "./exportState";
import { LibraryExportDialog } from "./LibraryExport";
import { ImportDialog } from "./ImportDialog";
import { importFiles, importFolder } from "./importState";
import { Menu } from "../components/Menu";
import { watchDrag } from "./dragOut";
import { itemMenu } from "./itemMenu";
import { boxOf, clickSelection, moveFocus, overlaps, range } from "./selection";
import { selectedItems, useLibrary, type LibraryItem } from "./state";
import { ViewSwitch } from "./ViewSwitch";
import { columnsFor, scrollToRow, visibleSpan } from "./virtual";

/** Smallest tile of the grid, in CSS px; and a list row's height. */
/** A tile at 100 %: a 160 px thumbnail and its padding (LIB-003). */
const THUMB = 160;
const TILE_PAD = 16;
const ROW = 34;

const IS_MAC = navigator.userAgent.includes("Mac");

function typeName(item: LibraryItem, t: (key: string) => string): string {
  const format = item.format === "jpeg" ? "jpg" : item.format;
  return t(`ui.library.type_${format}`);
}

/** The search field (LIB-007): Ctrl+F (Cmd+F) gives it the focus, Esc empties it. */
function SearchField() {
  const { t } = useTranslation();
  const query = useLibrary((s) => s.query);
  const setQuery = useLibrary((s) => s.setQuery);
  const field = useRef<HTMLInputElement>(null);
  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      const primary = IS_MAC ? e.metaKey : e.ctrlKey;
      if (primary && !e.shiftKey && !e.altKey && e.code === "KeyF") {
        e.preventDefault();
        field.current?.focus();
        field.current?.select();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  return (
    <input
      ref={field}
      type="search"
      className="lib-search"
      value={query}
      placeholder={t("ui.library.search")}
      aria-label={t("ui.library.search")}
      onChange={(e) => setQuery(e.target.value)}
      onKeyDown={(e) => {
        if (e.key === "Escape" && query) {
          e.preventDefault();
          e.stopPropagation();
          setQuery("");
        }
      }}
    />
  );
}

function SortControls() {
  const { t } = useTranslation();
  const shape = useLibrary((s) => s.shape);
  const setShape = useLibrary((s) => s.setShape);
  const seg = useRef<HTMLDivElement>(null);
  useIndicator(seg, shape.layout);
  return (
    <div className="lib-sort">
      <select
        value={shape.sort}
        aria-label={t("ui.library.sort.label")}
        onChange={(e) => setShape({ sort: e.target.value as SortKey })}
      >
        {SORT_KEYS.map((k) => (
          <option key={k} value={k}>
            {t(`ui.library.sort.${k}`)}
          </option>
        ))}
      </select>
      <button
        type="button"
        className="btn icon small"
        title={t(shape.order === "asc" ? "ui.library.sort.asc" : "ui.library.sort.desc")}
        aria-label={t(shape.order === "asc" ? "ui.library.sort.asc" : "ui.library.sort.desc")}
        onClick={() => setShape({ order: shape.order === "asc" ? "desc" : "asc" })}
      >
        <span className={`lib-dir ${shape.order}`} aria-hidden="true">
          <Icon name="up" />
        </span>
      </button>
      <div className="seg small" role="radiogroup" aria-label={t("ui.library.layout")} ref={seg}>
        <span className="seg-indicator" aria-hidden="true" />
        {(["grid", "list"] as const).map((l) => (
          <button
            key={l}
            type="button"
            role="radio"
            aria-checked={shape.layout === l}
            onClick={() => setShape({ layout: l })}
          >
            {t(`ui.library.layout_${l}`)}
          </button>
        ))}
      </div>
    </div>
  );
}

/** Create ▾ (LIB-016): new things from the selection. */
function CreateMenu() {
  const { t } = useTranslation();
  const items = useLibrary((s) => s.items);
  const selection = useLibrary((s) => s.selection);
  const chosen = items.filter((i) => selection.includes(i.id));
  return (
    <Menu
      label={t("ui.library.create.button")}
      triggerClass="btn tb-btn"
      disabled={chosen.length === 0}
      {...(chosen.length === 0 ? { title: t("ui.library.create.need_selection") } : {})}
      items={createItems(chosen, t)}
    />
  );
}

/** The thumbnail size of the grid (LIB-003): 50 to 300 %. */
function ZoomControl() {
  const { t } = useTranslation();
  const zoom = useLibrary((s) => zoomOf(s.shape.zoom));
  const list = useLibrary((s) => s.shape.layout === "list");
  if (list) return null;
  return (
    <label className="lib-zoom" title={t("ui.library.zoom")}>
      <span className="sr-only">{t("ui.library.zoom")}</span>
      <input
        type="range"
        min={ZOOM_MIN}
        max={ZOOM_MAX}
        step={ZOOM_STEP}
        value={zoom}
        aria-valuetext={`${zoom} %`}
        onChange={(e) => setZoom(Number(e.target.value))}
      />
      <span className="lib-zoom-value" aria-hidden="true">
        {zoom} %
      </span>
    </label>
  );
}

function LibraryToolbar() {
  const { t } = useTranslation();
  const openKey = useShortcut("editor.open");
  const pane = useLibrary((s) => s.shape.pane);
  const details = useLibrary((s) => s.shape.details === true);
  const setShape = useLibrary((s) => s.setShape);
  const selected = useLibrary((s) => s.selection.length);
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
      <button
        type="button"
        className="btn tb-btn"
        aria-pressed={pane}
        title={t("ui.library.filter.toggle")}
        onClick={() => setShape({ pane: !pane })}
      >
        <span className="tb-label">{t("ui.library.filter.title")}</span>
      </button>
      <SearchField />
      <SortControls />
      <ZoomControl />
      <CreateMenu />
      <button
        type="button"
        className="btn tb-btn"
        disabled={selected === 0}
        title={t("ui.library.tags.button")}
        onClick={() => editTags(useLibrary.getState().selection)}
      >
        <span className="tb-label">{t("ui.library.tags.button")}</span>
      </button>
      <button
        type="button"
        className="btn tb-btn"
        aria-pressed={details}
        title={t("ui.library.details.toggle")}
        onClick={() => setShape({ details: !details })}
      >
        <span className="tb-label">{t("ui.library.details.title")}</span>
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
      <Menu
        label={<Icon name="more" />}
        ariaLabel={t("ui.library.more")}
        title={t("ui.library.more")}
        triggerClass="btn icon"
        chevron={false}
        align="end"
        items={[
          // LIB-024.
          {
            id: "import_files",
            label: t("ui.library.import.files"),
            onSelect: () => void importFiles(),
          },
          {
            id: "import_folder",
            label: t("ui.library.import.folder"),
            onSelect: () => void importFolder(),
          },
          // LIB-031.
          {
            id: "merge",
            label: t("ui.library.merge.menu"),
            onSelect: () => void askMerge(),
          },
          {
            id: "rebuild",
            label: t("ui.library.rebuild"),
            separator: true,
            onSelect: askRebuild,
          },
          { id: "check", label: t("ui.library.check"), onSelect: () => void checkLibrary() },
          // LIB-033.
          {
            id: "export_all",
            label: t("ui.library.export.menu"),
            separator: true,
            onSelect: askExportLibrary,
          },
        ]}
      />
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

const NO_HIDDEN: string[] = [];

function Item({
  item,
  list,
  selected,
  focusable,
  onPick,
  onOpen,
  onMenu,
  onPress,
}: {
  item: LibraryItem;
  list: boolean;
  selected: boolean;
  focusable: boolean;
  onPick: (item: LibraryItem, e: MouseEvent) => void;
  onOpen: (item: LibraryItem) => void;
  onMenu: (item: LibraryItem, e: MouseEvent) => void;
  onPress: (item: LibraryItem, e: PointerEvent) => void;
}) {
  const { t, i18n } = useTranslation();
  const hidden = useLibrary((s) => s.shape.hidden) ?? NO_HIDDEN;
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
      className={`${list ? "lib-row" : "lib-item"}${selected ? " selected" : ""}${
        item.favorite ? " favorite" : ""
      }`}
      // Name, type, date and the app it came from (APP-044).
      aria-label={t(item.app ? "ui.library.item_label_app" : "ui.library.item_label", {
        name: item.name,
        type: typeName(item, t),
        date,
        app: item.app,
      })}
      // Name, type, size, date, app and file size (LIB-012).
      title={[
        item.name,
        typeName(item, t),
        size,
        date,
        item.app ?? "",
        fileSize(item.size, i18n.language),
      ]
        .filter(Boolean)
        .join("\n")}
      onClick={(e) => onPick(item, e)}
      onDoubleClick={() => onOpen(item)}
      onContextMenu={(e) => onMenu(item, e)}
      onPointerDown={(e) => onPress(item, e)}
    >
      {list ? (
        <>
          <span className="lc lc-name">
            <Star item={item} />
            {item.name}
            {item.cloud && <CloudBadge />}
            {item.conflict && <ConflictBadge />}
          </span>
          {!hidden.includes("captured") && <span className="lc">{date}</span>}
          {!hidden.includes("modified") && (
            <span className="lc">{shortDate(item.modified, i18n.language)}</span>
          )}
          {!hidden.includes("size") && (
            <span className="lc num">{fileSize(item.size, i18n.language)}</span>
          )}
          {!hidden.includes("type") && <span className="lc">{typeName(item, t)}</span>}
          {!hidden.includes("dimensions") && <span className="lc num">{size}</span>}
          {!hidden.includes("app") && <span className="lc">{item.app ?? ""}</span>}
          {!hidden.includes("site") && <span className="lc">{siteOf(item.url)}</span>}
          {!hidden.includes("tags") && <span className="lc">{item.tags.join(", ")}</span>}
        </>
      ) : (
        <>
          <div className="lib-thumb">
            <img src={libraryThumb(item)} alt="" loading="lazy" draggable={false} />
            {item.format !== "xshot" && (
              <span className="lib-badge" aria-hidden="true">
                {item.format === "jpeg" ? "JPG" : item.format.toUpperCase()}
              </span>
            )}
            <Star item={item} />
            {(item.cloud || item.conflict) && (
              <span className="lib-flags">
                {item.cloud && <CloudBadge />}
                {item.conflict && <ConflictBadge />}
              </span>
            )}
          </div>
          <div className="lib-name">{item.name}</div>
          <div className="lib-date">{date}</div>
        </>
      )}
    </div>
  );
}

/** Only in the cloud (LIB-029): opening it downloads it. */
function CloudBadge() {
  const { t } = useTranslation();
  return (
    <span className="lib-cloud" title={t("ui.library.cloud_help")}>
      {t("ui.library.cloud")}
    </span>
  );
}

/** A copy a sync service made (LIB-029): Resolve in its menu. */
function ConflictBadge() {
  const { t } = useTranslation();
  return (
    <span className="lib-cloud lib-conflict" title={t("ui.library.conflict_help")}>
      {t("ui.library.conflict")}
    </span>
  );
}

/** The favorite star (LIB-011): shown on hover, fixed when it is one. */
function Star({ item }: { item: LibraryItem }) {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className={`lib-star${item.favorite ? " on" : ""}`}
      tabIndex={-1}
      aria-pressed={item.favorite}
      aria-label={t(item.favorite ? "ui.library.unfavorite" : "ui.library.favorite")}
      title={t(item.favorite ? "ui.library.unfavorite" : "ui.library.favorite")}
      onClick={(e) => {
        e.stopPropagation();
        void toggleFavorite([item.id]);
      }}
      onDoubleClick={(e) => e.stopPropagation()}
      onPointerDown={(e) => e.stopPropagation()}
    >
      <Icon name="favorites" />
    </button>
  );
}

function shortDate(iso: string, lang: string): string {
  return new Date(iso).toLocaleString(lang, { dateStyle: "medium", timeStyle: "short" });
}

function siteOf(url: string | null): string {
  if (!url) return "";
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return "";
  }
}

/** The list's header (LIB-004): a click on a sortable column sorts, another reverses;
 * the edge of a column drags its width; the right-click menu shows and hides columns. */
function ListHeader() {
  const { t } = useTranslation();
  const shape = useLibrary((s) => s.shape);
  const setShape = useLibrary((s) => s.setShape);
  const columns = shownColumns(shape.hidden);
  const menu = (e: MouseEvent) => {
    e.preventDefault();
    const hidden = shape.hidden ?? [];
    useContextMenu.setState({
      open: true,
      ...menuPoint(e),
      label: t("ui.library.columns"),
      items: COLUMNS.filter((c) => c.key !== "name").map((c) => ({
        id: c.key,
        label: t(`ui.library.column.${c.key}`),
        checked: !hidden.includes(c.key),
        onSelect: () =>
          setShape({
            hidden: hidden.includes(c.key) ? hidden.filter((k) => k !== c.key) : [...hidden, c.key],
          }),
      })),
    });
  };
  const resize = (key: string, e: PointerEvent<HTMLSpanElement>) => {
    e.preventDefault();
    e.stopPropagation();
    const cell = e.currentTarget.parentElement;
    if (!cell) return;
    const start = { x: e.clientX, width: cell.getBoundingClientRect().width };
    const list = cell.closest<HTMLElement>(".library-content");
    let width = start.width;
    const move = (m: globalThis.PointerEvent) => {
      width = Math.max(MIN_WIDTH, start.width + m.clientX - start.x);
      // Live while dragging; kept when let go.
      list?.style.setProperty(
        "--lib-cols",
        template(columns, { ...(useLibrary.getState().shape.widths ?? {}), [key]: width }),
      );
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      // Letting go over the header is not a click that sorts.
      const eat = (c: globalThis.MouseEvent) => {
        c.stopPropagation();
        c.preventDefault();
      };
      window.addEventListener("click", eat, { capture: true, once: true });
      setTimeout(() => window.removeEventListener("click", eat, true), 300);
      setShape({ widths: { ...(useLibrary.getState().shape.widths ?? {}), [key]: width } });
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };
  const edge = (key: string) => (
    <span
      className="lc-resize"
      role="separator"
      aria-orientation="vertical"
      aria-label={t("ui.library.column_resize", { column: t(`ui.library.column.${key}`) })}
      onPointerDown={(e) => resize(key, e)}
      onDoubleClick={() =>
        setShape({
          widths: Object.fromEntries(Object.entries(shape.widths ?? {}).filter(([k]) => k !== key)),
        })
      }
    />
  );
  return (
    <div className="lib-head" role="presentation" onContextMenu={menu}>
      {columns.map((c) => {
        const sorted = c.sort === shape.sort;
        return c.sort ? (
          <button
            key={c.key}
            type="button"
            className={`lc lc-head${sorted ? " sorted" : ""}`}
            aria-sort={sorted ? (shape.order === "asc" ? "ascending" : "descending") : "none"}
            onClick={() =>
              setShape(
                sorted
                  ? { order: shape.order === "asc" ? "desc" : "asc" }
                  : { sort: c.sort as SortKey },
              )
            }
          >
            {t(`ui.library.column.${c.key}`)}
            {sorted && (
              <span className={`lib-dir ${shape.order}`} aria-hidden="true">
                <Icon name="up" />
              </span>
            )}
            {edge(c.key)}
          </button>
        ) : (
          <span key={c.key} className="lc lc-head">
            {t(`ui.library.column.${c.key}`)}
            {edge(c.key)}
          </span>
        );
      })}
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
  const list = useLibrary((s) => s.shape.layout === "list");
  const selection = useLibrary((s) => s.selection);
  const focus = useLibrary((s) => s.focus);
  const select = useLibrary((s) => s.select);
  const grid = useRef<HTMLDivElement>(null);
  const [band, setBand] = useState<Band | null>(null);
  /** The focus moves to the item only after a key, not after a click or a reload. */
  const keyed = useRef(false);
  const order = items.map((i) => i.id);
  const chosen = new Set(selection);
  // Only the rows in view are drawn (LIB-025): the scroll position, the size of the
  // area, and the height of a row as drawn.
  const [scroll, setScroll] = useState(0);
  const [box, setBox] = useState({ width: 800, height: 600, gap: 16 });
  const [tileHeight, setTileHeight] = useState(220);
  const zoom = useLibrary((s) => zoomOf(s.shape.zoom));
  // Items spring in when the view opens, not each time they scroll into view.
  const [fresh, setFresh] = useState(true);
  useEffect(() => {
    const timer = setTimeout(() => setFresh(false), 900);
    return () => clearTimeout(timer);
  }, []);
  useLayoutEffect(() => {
    const el = grid.current;
    if (!el) return;
    const measure = () => {
      const style = getComputedStyle(el);
      const pad = parseFloat(style.paddingLeft) + parseFloat(style.paddingRight);
      const inner = el.querySelector<HTMLElement>(".lib-window");
      const gap = inner ? parseFloat(getComputedStyle(inner).rowGap) || 0 : 16;
      setBox({ width: el.clientWidth - pad, height: el.clientHeight, gap });
      const first = el.querySelector<HTMLElement>("[data-item]");
      if (first && !list) setTileHeight(first.offsetHeight);
    };
    measure();
    const seen = new ResizeObserver(measure);
    seen.observe(el);
    return () => seen.disconnect();
  }, [list, items.length, zoom]);
  const tile = Math.round((THUMB * zoom) / 100) + TILE_PAD;
  const columns = list ? 1 : columnsFor(box.width, tile, box.gap);
  const rowHeight = list ? ROW : tileHeight + box.gap;
  const span = visibleSpan(items.length, columns, rowHeight, scroll, box.height);

  // Show in library (LIB-014): bring the item into view and give it the focus.
  const reveal = useLibrary((s) => s.reveal);
  useLayoutEffect(() => {
    if (!reveal) return;
    keyed.current = true;
    useLibrary.setState({ reveal: null });
  }, [reveal]);

  useLayoutEffect(() => {
    if (!keyed.current || !focus) return;
    const el = grid.current?.querySelector<HTMLElement>(`[data-item="${focus}"]`);
    const index = order.indexOf(focus);
    const g = grid.current;
    // Not drawn yet: scroll to its row; this runs again once it is.
    if (g && index >= 0) {
      const to = scrollToRow(Math.floor(index / columns), rowHeight, g.scrollTop, g.clientHeight);
      if (to !== null) {
        g.scrollTop = to;
        if (!el) return;
      }
    }
    keyed.current = false;
    el?.focus({ preventScroll: true });
  }, [focus, span.start, columns, rowHeight, order]);

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
    const rows = Math.max(1, Math.floor((grid.current?.clientHeight ?? 0) / rowHeight));
    return { columns, page: rows * columns };
  };

  /** A zoom `by` steps away (LIB-003), keeping the selected item in view. */
  const zoomBy = (by: number) => {
    const next = zoomOf(useLibrary.getState().shape.zoom) + by * ZOOM_STEP;
    const { focus: at, selection: picked } = useLibrary.getState();
    const keep = at ?? picked[0] ?? null;
    if (keep) {
      keyed.current = true;
      if (!at) select(picked, keep, useLibrary.getState().anchor);
    }
    setZoom(next);
  };

  const onKey = (e: KeyboardEvent) => {
    const primary = IS_MAC ? e.metaKey : e.ctrlKey;
    const { selection: current, anchor, focus: at } = useLibrary.getState();
    // LIB-003: Ctrl + / - (Cmd on macOS) change the thumbnail size.
    if (primary && !list && (e.key === "+" || e.key === "=" || e.key === "-")) {
      e.preventDefault();
      zoomBy(e.key === "-" ? -1 : 1);
      return;
    }
    if (primary && e.code === "KeyA") {
      e.preventDefault();
      select(order, at, at);
      return;
    }
    if ((!IS_MAC && e.key === "Enter" && e.altKey) || (IS_MAC && e.metaKey && e.code === "KeyI")) {
      // LIB-012: Alt+Enter (Cmd+I) opens Details.
      e.preventDefault();
      useLibrary.getState().setShape({ details: true });
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      void openItems(selectedItems());
      return;
    }
    if (e.key === " " && !primary && !e.altKey && !e.shiftKey) {
      // LIB-013: the quick view.
      e.preventDefault();
      openLibraryQuickView();
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
    const target = e.target as HTMLElement;
    const empty =
      target === e.currentTarget ||
      target.classList.contains("lib-sizer") ||
      target.classList.contains("lib-window");
    if (e.button !== 0 || !empty) return;
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
      className={`${list ? "library-list" : "library-grid"}${fresh ? " fresh" : ""}${
        !list && zoom < 80 ? " compact" : ""
      }`}
      role="listbox"
      aria-multiselectable="true"
      aria-label={t("ui.library.grid")}
      onKeyDown={onKey}
      onPointerDown={down}
      onPointerMove={moveBand}
      onPointerUp={upBand}
      onPointerCancel={upBand}
      onScroll={(e) => setScroll(e.currentTarget.scrollTop)}
      // LIB-003: Ctrl + wheel (Cmd on macOS) changes the thumbnail size.
      onWheel={(e) => {
        if (list || !(IS_MAC ? e.metaKey : e.ctrlKey)) return;
        e.preventDefault();
        zoomBy(e.deltaY < 0 ? 1 : -1);
      }}
    >
      <div
        className="lib-sizer"
        style={{ height: Math.max(0, span.rows * rowHeight - (list ? 0 : box.gap)) }}
      >
        <div
          className={`lib-window${list ? " list" : ""}`}
          style={{
            transform: `translateY(${span.firstRow * rowHeight}px)`,
            ...(list ? {} : { gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` }),
          }}
        >
          {items.slice(span.start, span.end).map((item) => (
            <Item
              key={item.id}
              item={item}
              list={list}
              selected={chosen.has(item.id)}
              focusable={item.id === firstFocusable}
              onPick={pick}
              onOpen={(i) => void openItems([i])}
              onMenu={menu}
              onPress={press}
            />
          ))}
        </div>
      </div>
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

/** The active filters as chips with ✕, and Clear filters (LIB-005); the number of
 * results, announced (LIB-007). */
function Chips() {
  const { t, i18n } = useTranslation();
  const filters = useLibrary((s) => s.filters);
  const query = useLibrary((s) => s.query);
  const count = useLibrary((s) => s.count);
  const setFilters = useLibrary((s) => s.setFilters);
  const setQuery = useLibrary((s) => s.setQuery);
  const active = chips(filters);
  if (active.length === 0 && !query) return null;
  const label = (section: string, value: string) => {
    if (section === "favorites") return t("ui.library.filter.favorites");
    if (section === "types") return t(`ui.library.filter.type_${value}`, value);
    if (section === "dates")
      return /^\d/.test(value) ? dateLabel(value, i18n.language) : t(`ui.library.filter.${value}`);
    if (section === "apps") return value || t("ui.library.filter.unknown_app");
    if (section === "sites") return value || t("ui.library.filter.no_site");
    return value;
  };
  return (
    <div className="lib-chips">
      <span className="lib-results" role="status" aria-live="polite">
        {t("ui.library.results", { count })}
      </span>
      {active.map((c) => (
        <span key={`${c.section}:${c.value}`} className="lib-chip">
          {label(c.section, c.value)}
          <button
            type="button"
            className="tag-x"
            aria-label={t("ui.library.filter.remove", { name: label(c.section, c.value) })}
            onClick={() =>
              setFilters(
                c.section === "favorites"
                  ? { ...filters, favorites: false }
                  : toggle(filters, c.section, c.value),
              )
            }
          >
            ✕
          </button>
        </span>
      ))}
      {(hasFilters(filters) || query) && (
        <button
          type="button"
          className="btn small"
          onClick={() => {
            setQuery("");
            setFilters(NO_FILTERS);
          }}
        >
          {t("ui.library.filter.clear")}
        </button>
      )}
    </div>
  );
}

/** "The filters were cleared to show the item" (LIB-014). */
function RevealNotice() {
  const { t } = useTranslation();
  const shown = useLibrary((s) => s.revealNotice);
  useEffect(() => {
    if (!shown) return;
    const timer = setTimeout(() => useLibrary.setState({ revealNotice: false }), 4000);
    return () => clearTimeout(timer);
  }, [shown]);
  if (!shown) return null;
  return (
    <div className="editor-toast" role="status">
      {t("ui.library.revealed_filters")}
    </div>
  );
}

function NoResults() {
  const { t } = useTranslation();
  const setFilters = useLibrary((s) => s.setFilters);
  const setQuery = useLibrary((s) => s.setQuery);
  return (
    <div className="editor-empty">
      <div className="empty-card">
        <h1>{t("ui.library.no_results")}</h1>
        <div className="row">
          <button
            type="button"
            className="btn primary"
            onClick={() => {
              setQuery("");
              setFilters(NO_FILTERS);
            }}
          >
            {t("ui.library.filter.clear")}
          </button>
        </div>
      </div>
    </div>
  );
}

/** The Library's status bar (LIB-026): how many items there are, shown and selected
 * (with their size), where the library is, and the tasks under way with Cancel. */
function LibraryStatus() {
  const { t, i18n } = useTranslation();
  const total = useLibrary((s) => s.total);
  const shown = useLibrary((s) => s.count);
  const items = useLibrary((s) => s.items);
  const selection = useLibrary((s) => s.selection);
  const root = useLibrary((s) => s.root);
  const place = useLibrary((s) => s.place);
  const service = useLibrary((s) => s.service);
  const tasks = useLibraryTasks((s) => s.tasks);
  const said = useLibraryTasks((s) => s.said);
  const [expanded, setExpanded] = useState(statusExpanded);
  // Exports report their progress (LIB-018, LIB-033).
  useEffect(() => {
    const off = listen<{ kind: string; payload: Record<string, unknown> }>("xs://editor", (e) => {
      const p = e.payload.payload;
      if (e.payload.kind !== "progress") return;
      // A merge (LIB-031) reports the same way, with its own Cancel.
      const merge = p.task === "library_merge";
      const id = merge
        ? "merge"
        : p.task === "library_export"
          ? "export"
          : p.task === "batch"
            ? "batch"
            : null;
      if (!id) return;
      if (p.done === null)
        taskDone(id, t(merge ? "ui.library.status.merge_done" : "ui.library.status.export_done"));
      else
        taskProgress({
          id,
          label: merge ? "ui.library.status.merging" : "ui.library.status.exporting",
          done: Number(p.index ?? 0),
          total: Number(p.total ?? 0),
          cancel: merge ? cancelMerge : () => void run("editor.cancel").catch(() => undefined),
        });
    });
    return () => void off.then((f) => f());
  }, [t]);
  const size = totalSize(items, selection);
  const where =
    place === "synced"
      ? service
        ? t("ui.library.status.synced_with", { service: t(`ui.library.service.${service}`) })
        : t("ui.library.synced")
      : t(`ui.library.status.place_${place}`);
  const toggle = () => {
    keepStatusExpanded(!expanded);
    setExpanded(!expanded);
  };
  return (
    <footer className={`status-bar library-status${expanded ? " expanded" : ""}`}>
      <div className="lib-status-row">
        <span className="status-item">{t("ui.library.count", { count: total })}</span>
        {shown !== total && (
          <span className="status-item">{t("ui.library.status.shown", { count: shown })}</span>
        )}
        {selection.length > 0 && (
          <span className="status-item">
            {t("ui.library.selected", { count: selection.length })} ·{" "}
            {fileSize(size, i18n.language)}
          </span>
        )}
        {tasks.length > 0 && !expanded && (
          <span className="status-item lib-task-short">{t(tasks[0]?.label ?? "")}</span>
        )}
        <span className="grow" />
        <span
          className={`lib-place lib-place-${place}`}
          title={
            place === "synced"
              ? t("ui.library.synced_help", { path: root })
              : place === "unavailable"
                ? t("ui.library.status.unavailable_help", { path: root })
                : root
          }
        >
          {where}
        </span>
        {/* LIB-030: out of reach, it still answers from this computer's index. */}
        {place === "unavailable" && expanded && (
          <span className="status-item muted">{t("ui.library.status.unavailable_note")}</span>
        )}
        {expanded && (
          <span className="lib-root muted" title={root}>
            {root}
          </span>
        )}
        <button
          type="button"
          className="btn icon small lib-status-toggle"
          aria-expanded={expanded}
          aria-label={t(expanded ? "ui.library.status.collapse" : "ui.library.status.expand")}
          title={t(expanded ? "ui.library.status.collapse" : "ui.library.status.expand")}
          onClick={toggle}
        >
          <span className={`lf-caret${expanded ? "" : " open"}`} aria-hidden="true">
            ▸
          </span>
        </button>
      </div>
      {expanded &&
        tasks.map((task) => (
          <div key={task.id} className="lib-status-row lib-task">
            <span>
              {task.total
                ? t("ui.library.status.progress", {
                    what: t(task.label),
                    done: task.done ?? 0,
                    total: task.total,
                  })
                : t(task.label)}
            </span>
            <progress
              max={Math.max(1, task.total ?? 1)}
              {...(task.total ? { value: task.done ?? 0 } : {})}
              aria-label={t(task.label)}
            />
            {task.cancel && (
              <button type="button" className="btn small" onClick={task.cancel}>
                {t("ui.editor.cancel")}
              </button>
            )}
          </div>
        ))}
      <div className="sr-only" role="status" aria-live="polite">
        {said}
      </div>
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
  const empty = useLibrary((s) => s.total === 0);
  const none = useLibrary((s) => s.items.length === 0);
  const pane = useLibrary((s) => s.shape.pane);
  const details = useLibrary((s) => s.shape.details === true);
  const list = useLibrary((s) => s.shape.layout === "list");
  const hidden = useLibrary((s) => s.shape.hidden) ?? NO_HIDDEN;
  const widths = useLibrary((s) => s.shape.widths);

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
          <>
            {pane && <FilterPane />}
            <div
              className="library-content"
              style={{ "--lib-cols": template(shownColumns(hidden), widths) } as CSSProperties}
            >
              <Chips />
              {list && !none && <ListHeader />}
              {none ? <NoResults /> : <Grid />}
            </div>
            {details && <DetailsPane />}
          </>
        )}
      </div>
      <LibraryStatus />
      <RevealNotice />
      <MaintenanceDialogs />
      <MergeDialogs />
      <LibraryExportDialog />
      <ImportDialog />
    </>
  );
}

// The right-click menu of recent tray items (TRY-002, TRY-004, TRY-008 to TRY-010,
// TRY-012, TRY-016): one item, or the selection when the item is in it.
import type { MenuItem } from "../components/Menu";
import {
  askDelete,
  askRename,
  copyPath,
  docRef,
  locate,
  removeFromTray,
  showInFolder,
} from "../library/actions";
import { exportItems } from "../library/exportState";
import { showQuickView } from "../library/quickViewState";
import { openPicker } from "./templateUi";
import {
  closeAll,
  closeDocument,
  closeDocuments,
  closeOthers,
  closeUnpinned,
  togglePin,
} from "./actions";
import { shortcutOf } from "./menus";
import { docUrl, xsdocUrl, type DocSummary } from "./state";
import {
  copyAllOf,
  detailsOfDoc,
  duplicateDocs,
  favoriteDocs,
  revealDoc,
  tagDocs,
} from "./trayActions";

const IS_MAC = navigator.userAgent.includes("Mac");

/** The quick view of the tray from `doc` (LIB-013). */
function quickView(doc: DocSummary, items: DocSummary[]): void {
  showQuickView({
    entries: items.map((d) => ({
      id: d.documentId,
      name: d.name,
      src: xsdocUrl(`${d.documentId}/final?v=${d.version}`),
      thumb: docUrl(d.documentId, "thumb", d.version),
      width: d.width,
      height: d.height,
      date: null,
    })),
    at: items.findIndex((d) => d.documentId === doc.documentId),
    open: () => undefined,
    done: () => undefined,
  });
}

/** The menu of a tray item; `items` is the tray, `selected` its selection. */
export function trayMenu(
  doc: DocSummary,
  items: DocSummary[],
  t: (key: string) => string,
  selected: string[] = [],
): MenuItem[] {
  if (doc.missing) {
    return [
      { id: "locate", label: t("menu.locate"), onSelect: () => void locate(doc) },
      { id: "remove", label: t("menu.remove_from_tray"), onSelect: () => removeFromTray(doc) },
    ];
  }
  const count = items.length;
  const several = selected.length > 1 && selected.includes(doc.documentId);
  const ids = several ? selected : [doc.documentId];
  const docs = items.filter((d) => ids.includes(d.documentId) && !d.missing && d.path);
  const files = docs.map(docRef);
  const allPinned = docs.every((d) => d.pinned);
  // TRY-004, TRY-010, TRY-016: what applies to one item and to several.
  const pin: MenuItem = {
    id: "pin",
    label: t(allPinned ? "menu.unpin_item" : "menu.pin_item"),
    shortcut: shortcutOf("editor.pin_item"),
    onSelect: () => void togglePin(ids),
  };
  const common: MenuItem[] = [
    {
      id: "duplicate",
      label: t("menu.duplicate_item"),
      shortcut: shortcutOf("editor.duplicate_item"),
      disabled: docs.length === 0,
      onSelect: () => void duplicateDocs(docs.map((d) => d.documentId)),
    },
    {
      id: "tags",
      label: t("menu.tags_item"),
      separator: true,
      disabled: docs.length === 0,
      onSelect: () => void tagDocs(ids),
    },
    {
      id: "favorite",
      label: t("menu.favorite_item"),
      disabled: docs.length === 0,
      onSelect: () => void favoriteDocs(ids),
    },
    {
      id: "export",
      label: t("menu.export_items"),
      disabled: docs.length === 0,
      onSelect: () => void exportItems(docs.map((d) => ({ name: d.name, path: d.path ?? "" }))),
    },
  ];
  if (several) {
    return [
      pin,
      ...common,
      // EDT-095: the selected items, in the tray's order, into a template.
      {
        id: "combine",
        label: `${t("menu.create")}: ${t("menu.combine_template")}`,
        shortcut: shortcutOf("editor.new_from_template"),
        separator: true,
        onSelect: () => openPicker(docs.map((d) => ({ documentId: d.documentId }))),
      },
      {
        id: "close",
        label: t("menu.close"),
        separator: true,
        onSelect: () => void closeDocuments(ids),
      },
      {
        id: "delete",
        label: t("menu.delete_item"),
        shortcut: shortcutOf("editor.delete_item"),
        disabled: files.length === 0,
        onSelect: () => askDelete(files),
      },
    ];
  }
  const file = docRef(doc);
  const noFile = !doc.path;
  return [
    {
      id: "quick_view",
      label: t("menu.quick_view"),
      shortcut: t("ui.library.quick.key"),
      onSelect: () => quickView(doc, items),
    },
    {
      id: "details",
      label: t("menu.details_tray"),
      disabled: noFile,
      onSelect: () => void detailsOfDoc(doc.documentId),
    },
    {
      id: "reveal",
      label: t("menu.show_in_library"),
      disabled: noFile,
      onSelect: () => void revealDoc(doc.documentId),
    },
    { ...pin, separator: true },
    ...common,
    {
      id: "copy_all",
      label: t("menu.copy_all"),
      onSelect: () => void copyAllOf(doc.documentId),
    },
    {
      id: "close",
      label: t("menu.close"),
      shortcut: shortcutOf("editor.close"),
      separator: true,
      onSelect: () => void closeDocument(doc.documentId),
    },
    {
      id: "close_others",
      label: t("menu.close_others"),
      shortcut: shortcutOf("editor.close_others"),
      disabled: count < 2,
      onSelect: () => void closeOthers(doc.documentId),
    },
    {
      id: "close_unpinned",
      label: t("menu.close_unpinned"),
      disabled: !items.some((d) => !d.pinned),
      onSelect: () => void closeUnpinned(),
    },
    {
      id: "close_all",
      label: t("menu.close_all"),
      shortcut: shortcutOf("editor.close_all"),
      onSelect: () => void closeAll(),
    },
    {
      id: "rename",
      label: t("menu.rename_item"),
      shortcut: shortcutOf("editor.rename_item"),
      disabled: noFile,
      separator: true,
      onSelect: () => askRename(file),
    },
    {
      id: "show",
      label: t(IS_MAC ? "menu.show_in_finder" : "menu.show_in_folder"),
      disabled: noFile,
      onSelect: () => showInFolder(file),
    },
    {
      id: "copy_path",
      label: t("menu.copy_path"),
      disabled: noFile,
      onSelect: () => copyPath(file),
    },
    {
      id: "delete",
      label: t("menu.delete_item"),
      shortcut: shortcutOf("editor.delete_item"),
      disabled: noFile,
      separator: true,
      onSelect: () => askDelete([file]),
    },
  ];
}

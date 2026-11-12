// The right-click menu of the Library view's selection (LIB-002).
import type { MenuItem } from "../components/Menu";
import { shortcutOf } from "../editor/menus";
import { askDelete, askRename, copyPath, itemRef, openItems, showInFolder } from "./actions";
import type { LibraryItem } from "./state";
import { openPicker } from "../editor/templateUi";
import { editTags, toggleFavorite } from "./marks";
import { askResolve } from "./maintenanceState";
import { openLibraryQuickView } from "./quickViewState";
import { exportItems } from "./exportState";
import { duplicateItems } from "../editor/trayActions";
import { useLibrary } from "./state";

const IS_MAC = navigator.userAgent.includes("Mac");

/** The menu of the selection (LIB-002), in the spec's order; entries of later builds
 * (quick view, share, tags, favorites, duplicate, create, export, print, details) join
 * it when they exist. */
export function itemMenu(items: LibraryItem[], t: (key: string) => string): MenuItem[] {
  const one = items.length === 1 ? items[0] : undefined;
  return [
    { id: "open", label: t("menu.open_item"), onSelect: () => void openItems(items) },
    // LIB-013.
    {
      id: "quick_view",
      label: t("menu.quick_view"),
      shortcut: t("ui.library.quick.key"),
      onSelect: openLibraryQuickView,
    },
    // EDT-095: the selected captures, in order, into a template's zones.
    {
      id: "combine",
      label: t("menu.combine_template"),
      shortcut: shortcutOf("editor.new_from_template"),
      onSelect: () => openPicker(items.map((i) => i.path)),
    },
    // A sync conflict (LIB-029).
    ...(one?.conflict
      ? [
          {
            id: "resolve",
            label: t("menu.resolve_conflict"),
            separator: true,
            onSelect: () => askResolve({ id: one.id, name: one.name }),
          },
        ]
      : []),
    // LIB-009, LIB-011.
    {
      id: "tags",
      label: t("menu.tags_item"),
      separator: true,
      onSelect: () => editTags(items.map((i) => i.id)),
    },
    {
      id: "favorite",
      label: t(items.every((i) => i.favorite) ? "menu.unfavorite_item" : "menu.favorite_item"),
      onSelect: () => void toggleFavorite(items.map((i) => i.id)),
    },
    {
      id: "rename",
      label: t("menu.rename_item"),
      shortcut: shortcutOf("editor.rename_item"),
      disabled: !one,
      separator: true,
      onSelect: () => one && askRename(itemRef(one)),
    },
    {
      id: "show",
      label: t(IS_MAC ? "menu.show_in_finder" : "menu.show_in_folder"),
      disabled: !one,
      onSelect: () => one && showInFolder(itemRef(one)),
    },
    {
      id: "copy_path",
      label: t("menu.copy_path"),
      disabled: !one,
      onSelect: () => one && copyPath(itemRef(one)),
    },
    // TRY-010.
    {
      id: "duplicate",
      label: t("menu.duplicate_item"),
      shortcut: shortcutOf("editor.duplicate_item"),
      onSelect: () => void duplicateItems(items.map((i) => i.id)),
    },
    // LIB-018: one item with Save as, several with the batch export.
    {
      id: "export",
      label: t("menu.export_items"),
      separator: true,
      onSelect: () => void exportItems(items),
    },
    // LIB-012: the Details pane, which follows the selection.
    {
      id: "details",
      label: t("menu.details_item"),
      shortcut: IS_MAC ? "⌘I" : "Alt+Enter",
      onSelect: () => useLibrary.getState().setShape({ details: true }),
    },
    {
      id: "delete",
      label: t("menu.delete_item"),
      shortcut: shortcutOf("editor.delete_item"),
      separator: true,
      onSelect: () => askDelete(items.map(itemRef)),
    },
  ];
}

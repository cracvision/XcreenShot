// The right-click menu of the Library view's selection (LIB-002).
import type { MenuItem } from "../components/Menu";
import { shortcutOf } from "../editor/menus";
import { askDelete, askRename, copyPath, itemRef, openItems, showInFolder } from "./actions";
import type { LibraryItem } from "./state";

const IS_MAC = navigator.userAgent.includes("Mac");

/** The menu of the selection (LIB-002), in the spec's order; entries of later builds
 * (quick view, share, tags, favorites, duplicate, create, export, print, details) join
 * it when they exist. */
export function itemMenu(items: LibraryItem[], t: (key: string) => string): MenuItem[] {
  const one = items.length === 1 ? items[0] : undefined;
  return [
    { id: "open", label: t("menu.open_item"), onSelect: () => void openItems(items) },
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
    {
      id: "delete",
      label: t("menu.delete_item"),
      shortcut: shortcutOf("editor.delete_item"),
      separator: true,
      onSelect: () => askDelete(items.map(itemRef)),
    },
  ];
}

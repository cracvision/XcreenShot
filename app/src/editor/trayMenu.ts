// The right-click menu of a recent tray item (TRY-002, TRY-008, TRY-009).
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
import { closeAll, closeDocument, closeOthers, closeUnpinned, togglePin } from "./actions";
import { shortcutOf } from "./menus";
import type { DocSummary } from "./state";

const IS_MAC = navigator.userAgent.includes("Mac");

/** The menu of a tray item (TRY-002, TRY-004, TRY-008, TRY-009); `items` is the tray. */
export function trayMenu(
  doc: DocSummary,
  items: DocSummary[],
  t: (key: string) => string,
): MenuItem[] {
  const count = items.length;
  if (doc.missing) {
    return [
      { id: "locate", label: t("menu.locate"), onSelect: () => void locate(doc) },
      { id: "remove", label: t("menu.remove_from_tray"), onSelect: () => removeFromTray(doc) },
    ];
  }
  const file = docRef(doc);
  const noFile = !doc.path;
  return [
    // TRY-004.
    {
      id: "pin",
      label: t(doc.pinned ? "menu.unpin_item" : "menu.pin_item"),
      shortcut: shortcutOf("editor.pin_item"),
      onSelect: () => void togglePin([doc.documentId]),
    },
    {
      id: "close",
      label: t("menu.close"),
      separator: true,
      shortcut: shortcutOf("editor.close"),
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

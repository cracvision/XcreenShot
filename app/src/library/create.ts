// Create from the Library (LIB-016): new things made from the selection, in the order
// it is shown; the originals do not change.
import type { MenuItem } from "../components/Menu";
import { shortcutOf } from "../editor/menus";
import { printFiles } from "../editor/printState";
import { openPicker } from "../editor/templateUi";
import type { LibraryItem } from "./state";

/** The entries of Create: a new image from a template now; narrated slides and
 * combining videos arrive with the video features (M4). */
export function createItems(items: LibraryItem[], t: (key: string) => string): MenuItem[] {
  const images = items.filter((i) => i.kind === "image");
  const later = t("ui.library.create.later");
  return [
    {
      id: "from_template",
      label: t("ui.library.create.from_template"),
      shortcut: shortcutOf("editor.new_from_template"),
      disabled: images.length === 0,
      ...(images.length === 0 ? { title: t("ui.library.create.need_images") } : {}),
      onSelect: () => openPicker(images.map((i) => i.path)),
    },
    {
      id: "slides",
      label: t("ui.library.create.slides"),
      disabled: true,
      title: later,
      onSelect: () => undefined,
    },
    {
      id: "videos",
      label: t("ui.library.create.videos"),
      disabled: true,
      title: later,
      onSelect: () => undefined,
    },
  ];
}

/** Print… of the selection (LIB-017): images only, one per page. */
export function printItems(items: LibraryItem[]): void {
  printFiles(items.filter((i) => i.kind === "image").map((i) => i.path));
}

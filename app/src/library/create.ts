// Create from the Library (LIB-016): new things made from the selection, in the order
// it is shown; the originals do not change.
import type { MenuItem } from "../components/Menu";
import { shortcutOf } from "../editor/menus";
import { printFiles } from "../editor/printState";
import { fail } from "../editor/actions";
import { startSlides } from "../editor/slidesState";
import { openPicker } from "../editor/templateUi";
import { useVideo } from "../editor/videoState";
import { run } from "../lib/commands";
import { useMainView, type LibraryItem } from "./state";

/** Narrated slides of library images (VID-071): opened, then made slides in order. */
async function slidesOf(images: LibraryItem[]): Promise<void> {
  const ids: string[] = [];
  for (const i of images) {
    const doc = await run<{ documentId: string }>("document.open", { path: i.path, show: false });
    ids.push(doc.documentId);
  }
  useMainView.getState().setView("editor");
  await startSlides(ids);
}

/** The entries of Create: a new image from a template, narrated slides (VID-071) and
 * combined videos (VID-067). */
export function createItems(items: LibraryItem[], t: (key: string) => string): MenuItem[] {
  const images = items.filter((i) => i.kind === "image");
  const videos = items.filter((i) => i.kind === "video");
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
      disabled: images.length === 0,
      ...(images.length === 0 ? { title: t("ui.library.create.need_images") } : {}),
      onSelect: () => void slidesOf(images).catch((e: unknown) => fail(e)),
    },
    {
      id: "videos",
      label: t("ui.library.create.videos"),
      disabled: videos.length === 0,
      ...(videos.length === 0 ? { title: t("ui.library.create.need_videos") } : {}),
      onSelect: () => {
        useMainView.getState().setView("editor");
        useVideo.setState({
          combineDialog: true,
          combineWith: videos.map((v) => ({ path: v.path, name: v.name })),
        });
      },
    },
  ];
}

/** Print… of the selection (LIB-017): images only, one per page. */
export function printItems(items: LibraryItem[]): void {
  printFiles(items.filter((i) => i.kind === "image").map((i) => i.path));
}

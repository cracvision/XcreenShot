// The editor's menus seen from the window (EDT-002): the state of the menu bar
// entries, its orders (`editor.action`), and the right-click menus of the canvas and
// the toolbar. Entries show their current shortcut (EDT-116).
import { listen } from "@tauri-apps/api/event";
import { Menu } from "@tauri-apps/api/menu";
import i18next from "i18next";
import { useEffect } from "react";
import { display } from "../lib/accelerator";
import { run } from "../lib/commands";
import { useApp, useSetting } from "../store";
import { performAction, toggleToolbarLabels } from "./actions";
import { useEditor, usePanel } from "./state";

/** "Name<Tab>Shortcut": Windows aligns the shortcut to the right. */
function label(key: string, slot?: string): string {
  const text = i18next.t(key);
  const { shortcuts, layout } = useApp.getState();
  const row = slot ? shortcuts?.rows.find((r) => r.command === slot) : undefined;
  if (!row?.shortcut) return text;
  return `${text}\t${display(row.shortcut, shortcuts?.platform ?? "windows", layout)}`;
}

function entry(action: string, enabled = true) {
  return {
    id: `editor.${action}`,
    text: label(`menu.${action}`, `editor.${action}`),
    enabled,
    action: () => void performAction(action),
  };
}

const SEPARATOR = { item: "Separator" } as const;

/** Right click on the canvas: what applies to the object under the pointer, or to the
 * image (EDT-002). Grouping, order and flattening arrive with them (M2). */
export async function canvasMenu(onObject: boolean): Promise<void> {
  const items = onObject
    ? [entry("cut"), entry("copy"), SEPARATOR, entry("delete")]
    : [entry("paste"), entry("paste_in_place"), SEPARATOR, entry("copy_all")];
  const menu = await Menu.new({ items });
  await menu.popup();
}

/** Right click on the toolbar: Icons only (EDT-003). */
export async function toolbarMenu(): Promise<void> {
  const labels = useApp.getState().settings.find((s) => s.key === "editor.toolbarLabels");
  const menu = await Menu.new({
    items: [
      {
        id: "editor.icons_only",
        text: i18next.t("menu.icons_only"),
        checked: labels?.value === false,
        action: toggleToolbarLabels,
      },
    ],
  });
  await menu.popup();
}

/** Tells the menu bar which entries apply now, and runs what it orders. */
export function useMenuBar(): void {
  const doc = useEditor((s) => s.doc);
  const selected = useEditor((s) => s.selection.length > 0);
  const anyOpen = useEditor((s) => s.documents.length > 0);
  const panel = usePanel((s) => s.visible);
  const labels = useSetting("editor.toolbarLabels", true);
  const hasDoc = doc !== null;
  const canUndo = doc?.canUndo ?? false;
  const canRedo = doc?.canRedo ?? false;

  useEffect(() => {
    const onDoc = [
      "save",
      "save_as",
      "export",
      "close",
      "copy_all",
      "zoom_in",
      "zoom_out",
      "zoom_actual",
      "zoom_fit",
      "zoom_original",
    ];
    const enabled: Record<string, boolean> = {
      "editor.close_all": anyOpen,
      "editor.undo": canUndo,
      "editor.redo": canRedo,
    };
    for (const a of onDoc) enabled[`editor.${a}`] = hasDoc;
    for (const a of ["cut", "copy", "delete", "deselect"]) enabled[`editor.${a}`] = selected;
    const checked = { "editor.side_panel": panel, "editor.toolbar_labels": labels };
    run("editor.menu_state", { enabled, checked }).catch(() => undefined);
  }, [hasDoc, canUndo, canRedo, selected, anyOpen, panel, labels]);

  useEffect(() => {
    const off = listen<{ kind: string; payload: { action?: string } }>("xs://editor", (e) => {
      if (e.payload.kind === "action" && e.payload.payload.action)
        performAction(e.payload.payload.action);
    });
    return () => void off.then((f) => f());
  }, []);
}

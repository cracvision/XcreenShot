// The editor's menus seen from the window (EDT-002): the state of the menu bar
// entries, its orders (`editor.action`), and the right-click menus of the canvas and
// the toolbar, drawn by the app so they animate like every other menu (APP-012).
// Entries show their current shortcut (EDT-116).
import { listen } from "@tauri-apps/api/event";
import i18next from "i18next";
import type { MouseEvent } from "react";
import { useEffect } from "react";
import { create } from "zustand";
import type { MenuItem } from "../components/Menu";
import { display } from "../lib/accelerator";
import { run } from "../lib/commands";
import { useApp, useSetting } from "../store";
import { performAction, toggleToolbarLabels } from "./actions";
import { useEditor, usePanel } from "./state";

interface Point {
  x: number;
  y: number;
}

interface ContextMenuState extends Point {
  open: boolean;
  label: string;
  items: MenuItem[];
}

/** The right-click menu open in the editor, if any. */
export const useContextMenu = create<ContextMenuState>(() => ({
  open: false,
  x: 0,
  y: 0,
  label: "",
  items: [],
}));

export function closeContextMenu(): void {
  useContextMenu.setState({ open: false });
}

/** The current shortcut of an editor row, written for this platform. */
export function shortcutOf(slot: string): string | undefined {
  const { shortcuts, layout } = useApp.getState();
  const row = shortcuts?.rows.find((r) => r.command === slot);
  return row?.shortcut
    ? display(row.shortcut, shortcuts?.platform ?? "windows", layout)
    : undefined;
}

function entry(action: string, separator = false): MenuItem {
  return {
    id: `editor.${action}`,
    label: i18next.t(`menu.${action}`),
    shortcut: shortcutOf(`editor.${action}`),
    separator,
    onSelect: () => void performAction(action),
  };
}

/** Where to open a menu: at the pointer, or by the element for the menu key. */
export function menuPoint(e: MouseEvent): Point {
  if (e.clientX === 0 && e.clientY === 0) {
    const r = (e.target as HTMLElement).getBoundingClientRect();
    return { x: r.left + 12, y: r.top + 12 };
  }
  return { x: e.clientX, y: e.clientY };
}

/** Right click on the canvas: what applies to the object under the pointer, or to the
 * image (EDT-002). Grouping, order and flattening arrive with them (M2). */
export function canvasMenu(onObject: boolean, at: Point): void {
  const items = onObject
    ? [entry("cut"), entry("copy"), entry("delete", true)]
    : [entry("paste"), entry("paste_in_place"), entry("copy_all", true)];
  useContextMenu.setState({ open: true, ...at, label: i18next.t("ui.editor.canvas_menu"), items });
}

/** Right click on the toolbar: Icons only (EDT-003). */
export function toolbarMenu(at: Point): void {
  const labels = useApp.getState().settings.find((s) => s.key === "editor.toolbarLabels");
  useContextMenu.setState({
    open: true,
    ...at,
    label: i18next.t("ui.editor.toolbar"),
    items: [
      {
        id: "editor.icons_only",
        label: i18next.t("menu.icons_only"),
        checked: labels?.value === false,
        onSelect: toggleToolbarLabels,
      },
    ],
  });
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
      "ai_ask",
      "ai_name",
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

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
import { performAction, setLocked, toggleToolbarLabels } from "./actions";
import { useEditor, usePanel } from "./state";
import { useLibrary, useMainView, useTray } from "../library/state";
import { neighbour } from "./tray";

interface Point {
  x: number;
  y: number;
}

/** A menu as the shell describes it (`xs_editor_menus`, EDT-002). */
export type MenuNode =
  | {
      kind: "command";
      id: string;
      label: string;
      command: string;
      params: Record<string, unknown>;
      checked: string | null;
    }
  | { kind: "submenu"; label: string; items: MenuNode[] }
  | { kind: "placeholder"; label: string }
  | { kind: "separator" };

/** A label's access key: "&File" is "File" with F, "Ay&uda" is "Ayuda" with u. */
export function mnemonic(label: string): { text: string; key: string | null; at: number } {
  const at = label.search(/&[^&]/);
  const text = label.replace(/&(.)/g, "$1");
  if (at < 0) return { text, key: null, at: -1 };
  return { text, key: label.charAt(at + 1).toLowerCase(), at };
}

/** The entries of a menu, with their state, shortcut and command. */
export function menuItems(
  nodes: MenuNode[],
  t: (key: string) => string,
  state: { enabled: Record<string, boolean>; checked: Record<string, boolean> },
): MenuItem[] {
  const out: MenuItem[] = [];
  let separator = false;
  for (const node of nodes) {
    if (node.kind === "separator") {
      separator = true;
      continue;
    }
    if (node.kind === "submenu") {
      out.push({
        id: node.label,
        label: mnemonic(t(node.label)).text,
        separator,
        submenu: menuItems(node.items, t, state),
        onSelect: () => undefined,
      });
    } else if (node.kind === "placeholder") {
      out.push({
        id: node.label,
        label: mnemonic(t(node.label)).text,
        separator,
        disabled: true,
        onSelect: () => undefined,
      });
    } else {
      out.push({
        id: node.id,
        label: mnemonic(t(node.label)).text,
        shortcut: shortcutOf(node.id) ?? shortcutOf(node.command),
        disabled: state.enabled[node.id] === false,
        checked: node.checked ? (state.checked[node.checked] ?? false) : undefined,
        separator,
        onSelect: () => void run(node.command, node.params).catch(() => undefined),
      });
    }
    separator = false;
  }
  return out;
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

/** Which menu bar entries apply now and which are ticked, by entry id (EDT-002). */
export const useMenuState = create<{
  enabled: Record<string, boolean>;
  checked: Record<string, boolean>;
}>(() => ({ enabled: {}, checked: {} }));

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
    ? [
        entry("cut"),
        entry("copy"),
        entry("duplicate"),
        entry("delete", true),
        entry("group", true),
        entry("ungroup"),
        entry("bring_front", true),
        entry("bring_forward"),
        entry("send_backward"),
        entry("send_back"),
      ]
    : [entry("paste"), entry("paste_in_place"), entry("copy_all", true)];
  useContextMenu.setState({ open: true, ...at, label: i18next.t("ui.editor.canvas_menu"), items });
}

/** Right click on a locked object: Unlock (EDT-085). */
export function lockedMenu(id: string, at: Point): void {
  useContextMenu.setState({
    open: true,
    ...at,
    label: i18next.t("ui.editor.canvas_menu"),
    items: [
      {
        id: "editor.unlock",
        label: i18next.t("menu.unlock"),
        onSelect: () => void setLocked([id], false),
      },
    ],
  });
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
  const inLibrary = useMainView((s) => s.view === "library");
  const selected = useEditor((s) => s.selection.length > 0) && !inLibrary;
  const selectedCount = useEditor((s) => s.selection.length);
  const lockedCount = useEditor((s) => s.doc?.objects.filter((o) => o.locked).length ?? 0);
  const groupedSelected = useEditor(
    (s) =>
      s.doc?.objects.some((o) => s.selection.includes(o.id) && (o.groups ?? []).length > 0) ??
      false,
  );
  const anyOpen = useEditor((s) => s.documents.length > 0);
  const panel = usePanel((s) => s.visible);
  const labels = useSetting("editor.toolbarLabels", true);
  const tray = useTray((s) => s.visible);
  const inTray = useEditor((s) => s.documents.filter((d) => d.shown).length);
  const hasPrev = useEditor((s) => neighbour(s.documents, s.active, -1) !== null);
  const hasNext = useEditor((s) => neighbour(s.documents, s.active, 1) !== null);
  // The file of the document shown, while it is where it was (TRY-002).
  const hasFile = useEditor((s) => {
    const d = s.documents.find((x) => x.documentId === s.active);
    return Boolean(d?.path) && !d?.missing;
  });
  const picked = useLibrary((s) => s.selection.length);
  // In the Library view the document entries do not apply (LIB-001).
  const hasDoc = doc !== null && !inLibrary;
  const objectCount = inLibrary ? 0 : (doc?.objects.length ?? 0);
  const canUndo = (doc?.canUndo ?? false) && !inLibrary;
  const canRedo = (doc?.canRedo ?? false) && !inLibrary;

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
      "editor.close_others": !inLibrary && inTray > 1,
      "editor.prev_item": hasDoc && hasPrev,
      "editor.next_item": hasDoc && hasNext,
      "editor.rename_item": inLibrary ? picked === 1 : hasDoc && hasFile,
      "editor.show_in_folder": inLibrary ? picked > 0 : hasDoc && hasFile,
      "editor.delete_item": inLibrary ? picked > 0 : hasDoc && hasFile,
      "editor.paste": !inLibrary,
      "editor.paste_in_place": !inLibrary,
    };
    for (const a of onDoc) enabled[`editor.${a}`] = hasDoc;
    for (const a of [
      "cut",
      "copy",
      "delete",
      "deselect",
      "duplicate",
      "bring_front",
      "bring_forward",
      "send_backward",
      "send_back",
    ])
      enabled[`editor.${a}`] = selected;
    for (const a of [
      "align_left",
      "align_hcenter",
      "align_right",
      "align_top",
      "align_vcenter",
      "align_bottom",
    ])
      enabled[`editor.${a}`] = selected;
    for (const a of ["distribute_h", "distribute_v"]) enabled[`editor.${a}`] = selectedCount >= 3;
    enabled["editor.lock"] = selected;
    enabled["editor.group"] = selectedCount >= 2;
    enabled["editor.ungroup"] = groupedSelected;
    enabled["editor.unlock_all"] = lockedCount > 0;
    enabled["editor.select_all"] = objectCount > 0;
    const checked = {
      "editor.side_panel": panel,
      "editor.toolbar_labels": labels,
      "editor.tray": tray,
    };
    // The drawn menu bar (Windows) reads it here; the system one (macOS) from the shell.
    useMenuState.setState({ enabled, checked });
    run("editor.menu_state", { enabled, checked }).catch(() => undefined);
  }, [
    hasDoc,
    objectCount,
    selectedCount,
    lockedCount,
    groupedSelected,
    canUndo,
    canRedo,
    selected,
    anyOpen,
    panel,
    labels,
    tray,
    inLibrary,
    inTray,
    hasPrev,
    hasNext,
    hasFile,
    picked,
  ]);

  useEffect(() => {
    const off = listen<{ kind: string; payload: { action?: string } }>("xs://editor", (e) => {
      if (e.payload.kind === "action" && e.payload.payload.action)
        performAction(e.payload.payload.action);
    });
    return () => void off.then((f) => f());
  }, []);
}

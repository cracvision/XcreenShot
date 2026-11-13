// The editor's menus seen from the window (EDT-002): the state of the menu bar
// entries, its orders (`editor.action`), and the right-click menus of the canvas and
// the toolbar, drawn by the app so they animate like every other menu (APP-012).
// Entries show their current shortcut (EDT-116).
import { hasSelectAll, idsOfKind } from "./selectKind";
import { listen } from "@tauri-apps/api/event";
import i18next from "i18next";
import type { MouseEvent } from "react";
import { useEffect } from "react";
import { create } from "zustand";
import type { MenuItem } from "../components/Menu";
import { display } from "../lib/accelerator";
import { run } from "../lib/commands";
import { useApp, useSetting } from "../store";
import {
  performAction,
  setLocked,
  stepSequence,
  toggleToolbarLabels,
  unflattenLayer,
} from "./actions";
import { useEditor, usePanel, type FlatLayer, type StepObject, type ZoneObject } from "./state";
import { useLibrary, useMainView, useTray } from "../library/state";
import { neighbour } from "./tray";
import { TYPES } from "./effects";
import { useClipboard, type ClipboardInfo } from "./imageMenu";
import { useFullscreen } from "./fullscreen";
import { captureIntoZone, replaceZoneImage, templateCommand } from "./templateActions";
import { useZones } from "./templateUi";
import { useVideo } from "./videoState";

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
 * image (EDT-002), and Unflatten on a flattened place (EDT-091). */
/** Right-click inside a pixel selection (EDT-044). */
export function selectionMenu(at: Point, onImage: boolean): void {
  const items: MenuItem[] = [
    entry("cut"),
    entry("copy"),
    entry("delete"),
    entry("invert_selection", true),
    ...(onImage ? [entry("float_selection"), entry("crop_selection")] : []),
    entry("deselect", true),
  ];
  useContextMenu.setState({ open: true, ...at, label: i18next.t("ui.editor.canvas_menu"), items });
}

export function canvasMenu(
  onObject: boolean,
  at: Point,
  layer?: FlatLayer,
  step?: StepObject,
  image?: string,
  kind?: string,
): void {
  const items: MenuItem[] = onObject
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
        entry("flatten", true),
      ]
    : [entry("paste"), entry("paste_in_place"), entry("copy_all", true)];
  if (onObject && kind && hasSelectAll(kind)) {
    items.push({
      id: "editor.select_kind",
      label: i18next.t("ui.editor.select_kind.menu"),
      separator: true,
      onSelect: () =>
        useEditor.getState().setSelection(idsOfKind(useEditor.getState().doc?.objects ?? [], kind)),
    });
  }
  if (step) {
    // Edit value and Renumber (EDT-035).
    items.splice(
      0,
      0,
      {
        id: "editor.step_edit",
        label: i18next.t("menu.step_edit"),
        onSelect: () => useEditor.setState({ stepEdit: { id: step.id, value: step.value } }),
      },
      {
        id: "editor.step_renumber",
        label: i18next.t("menu.step_renumber"),
        onSelect: () => void stepSequence("renumber", step.style.sequence),
      },
    );
    const first = items[2];
    if (first) items[2] = { ...first, separator: true };
  }
  if (image) {
    // A floating image becomes a template's zone (EDT-101).
    items.push({
      id: "editor.convert_zone",
      label: i18next.t("ui.editor.template.convert_zone"),
      separator: true,
      onSelect: () => void templateCommand("convert_zone", { objectId: image }),
    });
  }
  if (!onObject && layer) {
    items.push({
      id: "editor.unflatten",
      label: i18next.t("menu.unflatten"),
      separator: true,
      disabled: Boolean(layer.permanent),
      ...(layer.permanent ? { title: i18next.t(`ui.editor.unflatten_${layer.permanent}`) } : {}),
      onSelect: () => void unflattenLayer(layer.id),
    });
  }
  useContextMenu.setState({ open: true, ...at, label: i18next.t("ui.editor.canvas_menu"), items });
}

/** Right click on a template's zone (EDT-098): capture, replace, paste, framing, remove
 * the image or the caption. */
export function zoneMenu(zone: ZoneObject, at: Point): void {
  const tr = (k: string) => i18next.t(`ui.editor.template.${k}`);
  const filled = Boolean(zone.image);
  const z = zone.id;
  const frame = (params: Record<string, unknown>) =>
    void templateCommand("frame_zone", { zone: z, ...params });
  const items: MenuItem[] = [
    { id: "zone.capture", label: tr("capture"), onSelect: () => void captureIntoZone(z) },
    {
      id: "zone.replace",
      label: tr(filled ? "replace" : "choose"),
      onSelect: () => void replaceZoneImage(z),
    },
    {
      id: "zone.paste",
      label: i18next.t("menu.paste"),
      shortcut: shortcutOf("editor.paste"),
      onSelect: () => void templateCommand("fill_zone", { zone: z, clipboard: true }),
    },
    {
      id: "zone.frame",
      label: tr("frame"),
      disabled: !filled,
      separator: true,
      onSelect: () => useZones.setState({ framing: z, active: z }),
    },
    {
      id: "zone.zoom_in",
      label: tr("zoom_in"),
      disabled: !filled,
      onSelect: () => frame({ zoomBy: 1 }),
    },
    {
      id: "zone.zoom_out",
      label: tr("zoom_out"),
      disabled: !filled,
      onSelect: () => frame({ zoomBy: -1 }),
    },
    {
      id: "zone.reset",
      label: tr("reset_frame"),
      disabled: !filled,
      onSelect: () => frame({ reset: true }),
    },
    {
      id: "zone.clear",
      label: tr("remove_image"),
      disabled: !filled,
      separator: true,
      onSelect: () => void templateCommand("clear_zone", { zone: z }),
    },
    {
      id: "zone.caption",
      label: tr("remove_caption"),
      disabled: !zone.slot,
      onSelect: () => void templateCommand("remove_caption", { zone: z }),
    },
  ];
  useContextMenu.setState({ open: true, ...at, label: tr("zone_menu"), items });
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

/** Entries of the Image menu that apply to any document shown. */
/** The Video menu (VID-056 to VID-064). */
const VIDEO_ACTIONS = [
  "video_play",
  "video_prev_frame",
  "video_next_frame",
  "video_start",
  "video_end",
  "video_go_to",
  "video_cut",
  "video_save_selection",
  "video_save_frame",
  "video_gif",
  "video_extract_text",
  "video_silent_copy",
  "video_info",
];

const IMAGE_ACTIONS = [
  "resize_image",
  "canvas_size",
  "canvas_color",
  "rotate_right",
  "rotate_left",
  "rotate_180",
  "rotate_custom",
  "flip_horizontal",
  "flip_vertical",
  "corners",
  "resolution",
  "revert",
  "crop_to_canvas",
  "fit_canvas",
  "trim",
  ...TYPES.map((t) => `effect_${t.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`)}`),
];

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
  const snap = useSetting("editor.snap", true);
  const grid = useSetting("editor.pixelGrid", true);
  const fullscreen = useFullscreen((s) => s.on);
  const clip = useClipboard((s) => s.info);
  const hasClip = Boolean(clip && (clip.image || clip.files > 0 || clip.text));
  const tray = useTray((s) => s.visible);
  const inTray = useEditor((s) => s.documents.filter((d) => d.shown).length);
  const hasPrev = useEditor((s) => neighbour(s.documents, s.active, -1, s.tray) !== null);
  const hasNext = useEditor((s) => neighbour(s.documents, s.active, 1, s.tray) !== null);
  // The file of the document shown, while it is where it was (TRY-002).
  const hasFile = useEditor((s) => {
    const d = s.documents.find((x) => x.documentId === s.active);
    return Boolean(d?.path) && !d?.missing;
  });
  const picked = useLibrary((s) => s.selection.length);
  // In the Library view the document entries do not apply (LIB-001).
  const hasDoc = doc !== null && !inLibrary;
  const libraryDoc = doc?.kind === "library";
  const objectCount = inLibrary ? 0 : (doc?.objects.length ?? 0);
  const canUnflatten = !inLibrary && (doc?.flattened ?? []).some((l) => !l.permanent);
  // A pixel selection (EDT-044): on the image, or on a floating image.
  const pixelSel = inLibrary ? null : (doc?.pixelSelection ?? null);
  const hasPixels = pixelSel !== null;
  const pixelsOnImage = hasPixels && !pixelSel?.target;
  const canUndo = (doc?.canUndo ?? false) && !inLibrary;
  // A video shown (ADR 0008): its menu on, the image's off.
  const isVideo = !!doc?.video && !inLibrary;
  const videoSound = doc?.video?.sound ?? false;
  const isGif = isVideo && doc?.video?.codec === "GIF";
  const videoCuttable = useVideo((s) => {
    const length = doc?.video?.length ?? 0;
    return s.end - s.start < length - 1e-6;
  });
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
      "editor.pin_item": !inLibrary && hasDoc,
      "editor.shortcut_sheet": true,
      "editor.move_computer": true,
      "editor.duplicate_item": inLibrary ? picked > 0 : hasDoc && hasFile,
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
    enabled["editor.equal_width"] = selectedCount >= 2;
    enabled["editor.equal_height"] = selectedCount >= 2;
    enabled["editor.ungroup"] = groupedSelected;
    enabled["editor.unlock_all"] = lockedCount > 0;
    enabled["editor.select_all"] = objectCount > 0;
    enabled["editor.flatten"] = selected;
    enabled["editor.flatten_all"] = objectCount > 0;
    enabled["editor.unflatten_all"] = canUnflatten;
    for (const a of ["cut", "copy", "delete", "deselect"])
      enabled[`editor.${a}`] = (enabled[`editor.${a}`] ?? false) || hasPixels;
    // The Image menu (EDT-069 to EDT-077) and Image > Effects (EDT-053); not for a
    // video (VID-065).
    for (const a of IMAGE_ACTIONS) enabled[`editor.${a}`] = hasDoc && !isVideo;
    enabled["editor.snap"] = !inLibrary;
    enabled["editor.new_image"] = true;
    enabled["editor.batch_export"] = true;
    enabled["editor.customize_toolbar"] = !inLibrary;
    enabled["editor.pixel_grid"] = hasDoc;
    enabled["editor.fullscreen"] = true;
    enabled["editor.print"] = inLibrary ? picked > 0 : hasDoc;
    // Templates (EDT-095, EDT-101).
    enabled["editor.new_from_template"] = true;
    enabled["editor.apply_template"] = hasDoc && !inLibrary;
    enabled["editor.save_as_template"] = hasDoc && !inLibrary;
    enabled["editor.import_template"] = true;
    enabled["editor.show_in_library"] = hasDoc && libraryDoc;
    enabled["editor.new_from_clipboard"] = hasClip;
    enabled["editor.flip_objects_horizontal"] = selected;
    enabled["editor.flip_objects_vertical"] = selected;
    enabled["editor.invert_selection"] = hasPixels;
    enabled["editor.float_selection"] = pixelsOnImage;
    enabled["editor.crop_selection"] = pixelsOnImage;
    // The Video menu (VID-056 to VID-064): with a video shown.
    for (const a of VIDEO_ACTIONS) enabled[`editor.${a}`] = hasDoc && isVideo;
    enabled["editor.video_cut"] = hasDoc && isVideo && videoCuttable;
    enabled["editor.video_silent_copy"] = hasDoc && isVideo && videoSound;
    // A GIF plays and shows its information; it is not edited (VID-068).
    if (isGif) for (const a of VIDEO_ACTIONS) enabled[`editor.${a}`] = a === "video_info";
    // Image things a video does not have (VID-065).
    if (isVideo)
      for (const a of [
        "print",
        "apply_template",
        "save_as_template",
        "pixel_grid",
        "ai_ask",
        "ai_name",
      ])
        enabled[`editor.${a}`] = false;
    const checked = {
      "editor.side_panel": panel,
      "editor.toolbar_labels": labels,
      "editor.snap": snap,
      "editor.pixel_grid": grid,
      "editor.fullscreen": fullscreen,
      "editor.tray": tray,
    };
    // The drawn menu bar (Windows) reads it here; the system one (macOS) from the shell.
    useMenuState.setState({ enabled, checked });
    run("editor.menu_state", { enabled, checked }).catch(() => undefined);
  }, [
    hasDoc,
    isVideo,
    isGif,
    videoSound,
    videoCuttable,
    libraryDoc,
    objectCount,
    selectedCount,
    lockedCount,
    groupedSelected,
    canUnflatten,
    canUndo,
    canRedo,
    selected,
    anyOpen,
    panel,
    labels,
    snap,
    grid,
    fullscreen,
    hasClip,
    tray,
    inLibrary,
    inTray,
    hasPrev,
    hasNext,
    hasFile,
    picked,
    hasPixels,
    pixelsOnImage,
  ]);

  // What the clipboard has, when the window gets the focus (EDT-104).
  useEffect(() => {
    const read = () =>
      void run<ClipboardInfo>("editor.clipboard")
        .then((info) => useClipboard.setState({ info }))
        .catch(() => undefined);
    // Not at startup: only once the window is in use.
    if (document.hasFocus()) read();
    window.addEventListener("focus", read);
    return () => window.removeEventListener("focus", read);
  }, []);

  useEffect(() => {
    const off = listen<{ kind: string; payload: { action?: string } }>("xs://editor", (e) => {
      if (e.payload.kind === "action" && e.payload.payload.action)
        performAction(e.payload.payload.action);
    });
    return () => void off.then((f) => f());
  }, []);
}

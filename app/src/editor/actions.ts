// What the editor's buttons, keys and dialogs do: each one runs a command (APP-027).
// The only work done here is asking the system for a file name and showing the
// answer of the command.
import { openShortcutSheet } from "./sheetData";
import { duplicateDocs, duplicateItems, trayTargets } from "./trayActions";
import { open, save } from "@tauri-apps/plugin-dialog";
import i18next from "i18next";
import { run } from "../lib/commands";
import type { Rect } from "./boxes";
import type { Point } from "./geometry";
import {
  usePanel,
  useEditor,
  type Pending,
  type RecoveryItem,
  type TextObject,
  type Tool,
  type CalloutObject,
} from "./state";
import { setting, useApp } from "../store";
import { openAi } from "./aiState";
import {
  askDelete,
  askRename,
  docRef,
  itemRef,
  offerUndo,
  openItems,
  showInFolder,
  type FileRef,
} from "../library/actions";
import { selectedItems, showView, useMainView, useTray } from "../library/state";
import { focusedTrayItem, neighbour, trayOrder } from "./tray";
import { extentPoints } from "./objects";
import { alignMoves, distributeMoves, frameOf, type AlignTo, type Extent } from "./align";
import { textBox } from "./callouts";
import { selectionAction, selectPixels } from "./pixelSelection";
import { importTheme } from "./themesUi";
import { openImageDialog } from "./imageMenu";
import { turnOnEffect } from "./effects";
import { useExportDialog } from "./exportOptions";
import { useBatchDialog } from "./batchExport";
import { printFiles, usePrintDialog } from "./printState";
import { useToolbar } from "./toolbarLayout";
import { toggleFullscreen } from "./fullscreen";
import { applyTemplate, askSaveTemplate, importTemplate, pickTemplate } from "./templateActions";
import { useZones } from "./templateUi";

const t = (key: string, options: Record<string, unknown> = {}) => i18next.t(key, options);
const LAST_FORMAT = "xs.editor.lastFormat";

export function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

function lastFormat(): "xshot" | "png" | "jpg" {
  try {
    const v = localStorage.getItem(LAST_FORMAT);
    if (v === "png" || v === "jpg" || v === "xshot") return v;
  } catch {
    // Storage unavailable: use the default.
  }
  return "png";
}

function rememberFormat(path: string): void {
  const ext = path.split(".").pop()?.toLowerCase();
  const format = ext === "jpeg" ? "jpg" : ext;
  try {
    if (format === "png" || format === "jpg" || format === "xshot")
      localStorage.setItem(LAST_FORMAT, format);
  } catch {
    // Not remembered; nothing else changes.
  }
}

/** Save as and Export offer the last used format first ("Remember the last", EDT-107). */
function filters(withXshot: boolean) {
  const all = [
    { key: "png", name: "PNG", extensions: ["png"] },
    { key: "jpg", name: "JPG", extensions: ["jpg", "jpeg"] },
    ...(withXshot
      ? [{ key: "xshot", name: t("ui.editor.format_xshot"), extensions: ["xshot"] }]
      : []),
  ];
  const first = lastFormat();
  return [...all.filter((f) => f.key === first), ...all.filter((f) => f.key !== first)].map(
    ({ name, extensions }) => ({ name, extensions }),
  );
}

export async function openFiles(paths?: string[]): Promise<void> {
  try {
    const chosen =
      paths ??
      (await open({
        multiple: true,
        filters: [
          {
            name: t("ui.editor.open_filter"),
            extensions: ["xshot", "png", "jpg", "jpeg", "xtemplate"],
          },
        ],
      }));
    if (!chosen) return;
    const all = Array.isArray(chosen) ? chosen : [chosen];
    // A theme or template file is imported (EDT-018, EDT-101); the rest are opened.
    for (const path of all.filter((p) => /\.xtheme$/i.test(p))) await importTheme(path);
    for (const path of all.filter((p) => /\.xtemplate$/i.test(p))) await importTemplate(path);
    const docs = all.filter((p) => !/\.(xtheme|xtemplate)$/i.test(p));
    if (docs.length === 0) return;
    // All are opened; the last one is shown (EDT-105), in the Editor view.
    for (const path of docs) {
      await run("document.open", { path, show: true });
    }
    showView("editor");
  } catch (e) {
    fail(e);
  }
}

export async function saveActive(confirmFlatten = false, dontAskAgain = false): Promise<void> {
  const id = useEditor.getState().active;
  if (!id) return;
  try {
    const r = await run<{ status: string }>("document.save", {
      documentId: id,
      confirmFlatten,
      dontAskAgain,
    });
    if (r.status === "needs_decision") useEditor.setState({ flatten: id });
  } catch (e) {
    fail(e);
  }
}

export async function saveAs(): Promise<void> {
  const { active, doc } = useEditor.getState();
  if (!active || !doc) return;
  try {
    const path = await save({ defaultPath: doc.name, filters: filters(true) });
    if (!path) return;
    await run("document.save", { documentId: active, path });
    rememberFormat(path);
  } catch (e) {
    fail(e);
  }
}

/** Export, and the File destination of Share (SHR-018): a copy in PNG or JPG. */
/** File > Export (EDT-110 to EDT-112): the Export dialog chooses the format and its
 * options, then asks where. */
export function exportImage(): Promise<void> {
  const { active, doc } = useEditor.getState();
  if (active && doc) useExportDialog.setState({ ids: [active] });
  return Promise.resolve();
}

export async function copyAll(): Promise<void> {
  const id = useEditor.getState().active;
  if (!id) return;
  try {
    await run("document.copy_all", { documentId: id });
    useEditor.getState().showToast(t("ui.editor.copied"));
  } catch (e) {
    fail(e);
  }
}

async function closeWith(ids: string[] | "all", save: string[], discard: string[]): Promise<void> {
  const before = useEditor.getState();
  const targets = ids === "all" ? before.documents.map((d) => d.documentId) : ids;
  // Files to reopen with Undo when several close at once (TRY-008), oldest first.
  const reopen = before.documents
    .filter((d) => targets.includes(d.documentId) && d.shown && d.path && !d.missing)
    .filter((d) => !discard.includes(d.documentId) || !d.dirty)
    .map((d) => d.path as string);
  const shown = before.documents.find((d) => d.documentId === before.active)?.path ?? null;
  const params =
    ids === "all"
      ? { all: true, save, discard }
      : ids.length === 1
        ? { documentId: ids[0], save, discard }
        : { documentIds: ids, save, discard };
  const r = await run<{ status: string; pending: Pending[] }>("document.close", params);
  if (r.status === "needs_decision") {
    useEditor.setState({ decision: { kind: "close", pending: r.pending, targets: ids } });
    return;
  }
  if (reopen.length > 1) {
    offerUndo(t("ui.files.closed", { count: reopen.length }), async () => {
      try {
        for (const path of reopen) await run("document.open", { path, show: true });
        if (shown && reopen.includes(shown))
          await run("document.open", { path: shown, show: true });
      } catch (e) {
        fail(e);
      }
    });
  }
}

/** Close all (EDT-108, TRY-008): one question for every file from elsewhere with
 * changes; Undo reopens them. */
export async function closeAll(): Promise<void> {
  try {
    await closeWith("all", [], []);
  } catch (e) {
    fail(e);
  }
}

/** Pins the items, or unpins them when all are pinned already (TRY-004). */
export async function togglePin(ids: string[]): Promise<void> {
  const { documents } = useEditor.getState();
  const all =
    ids.length > 0 && ids.every((id) => documents.find((d) => d.documentId === id)?.pinned);
  try {
    await run(all ? "tray.unpin" : "tray.pin", { documentIds: ids });
  } catch (e) {
    fail(e);
  }
}

/** Closes these tray items (TRY-012), asking about unsaved changes as Close does. */
export async function closeDocuments(ids: string[]): Promise<void> {
  if (ids.length === 0) return;
  try {
    await closeWith(ids, [], []);
  } catch (e) {
    fail(e);
  }
}

/** Close unpinned (TRY-008): every tray item that is not pinned. */
export async function closeUnpinned(): Promise<void> {
  const { documents, tray } = useEditor.getState();
  const free = trayOrder(documents, tray)
    .filter((d) => !d.pinned)
    .map((d) => d.documentId);
  if (free.length === 0) return;
  try {
    await closeWith(free, [], []);
  } catch (e) {
    fail(e);
  }
}

/** Close others (TRY-008): every tray item but `keep`. */
export async function closeOthers(keep: string): Promise<void> {
  const { documents, tray } = useEditor.getState();
  const others = trayOrder(documents, tray)
    .map((d) => d.documentId)
    .filter((id) => id !== keep);
  if (others.length === 0) return;
  try {
    await closeWith(others, [], []);
  } catch (e) {
    fail(e);
  }
}

export async function closeDocument(id?: string): Promise<void> {
  const target = id ?? useEditor.getState().active;
  if (!target) return;
  try {
    await closeWith([target], [], []);
  } catch (e) {
    fail(e);
  }
}

/** Answer to "Save changes?" before closing or quitting (EDT-108, APP-004). */
export async function decide(choice: "save" | "discard" | "cancel"): Promise<void> {
  const decision = useEditor.getState().decision;
  useEditor.setState({ decision: null });
  if (!decision || choice === "cancel") return;
  const ids = decision.pending.map((p) => p.documentId);
  const save = choice === "save" ? ids : [];
  const discard = choice === "discard" ? ids : [];
  try {
    if (decision.kind === "close") {
      await closeWith(decision.targets, save, discard);
    } else {
      await run("document.close", { all: true, save, discard });
      await run("app.quit", { force: true });
    }
  } catch (e) {
    fail(e);
  }
}

export function undo(): void {
  const id = useEditor.getState().active;
  if (id) run("document.undo", { documentId: id }).catch(fail);
}

export function redo(): void {
  const id = useEditor.getState().active;
  if (id) run("document.redo", { documentId: id }).catch(fail);
}

export function selectTool(tool: Tool): void {
  const { tool: before, doc } = useEditor.getState();
  // Leaving Simplify locks its shapes when the user asked for it (EDT-041).
  if (
    before === "simplify" &&
    tool !== "simplify" &&
    doc &&
    setting("editor.simplifyLockOnExit", false)
  ) {
    const ids = doc.objects.filter((o) => o.type === "simplify" && !o.locked).map((o) => o.id);
    if (ids.length > 0) void setLocked(ids, true);
  }
  if (before !== tool) run("editor.select_tool", { tool }).catch(fail);
  // Selecting pixels works on what is under the pointer, not on selected objects.
  if (tool === "select" || tool === "wand") useEditor.getState().setSelection([]);
  // The stamps are chosen in the Styles tab (EDT-031).
  if (tool === "stamp") usePanel.getState().set({ visible: true, tab: "styles" });
}

/** Loads the installed fonts the first time the font list is needed (EDT-026). */
export async function loadFonts(): Promise<void> {
  if (useEditor.getState().fonts.length > 0) return;
  try {
    const r = await run<{ fonts: string[] }>("editor.fonts");
    useEditor.setState({ fonts: r.fonts });
  } catch (e) {
    fail(e);
  }
}

/** The last duplicate: its copies, where they were made and the step used, so
 *  duplicating the copy again repeats how it was moved (EDT-088). */
let lastDuplicate: {
  copies: string[];
  at: [number, number] | null;
  step: [number, number];
} | null = null;

/** The top-left corner of some objects, to compare where copies went. */
function cornerOf(ids: string[]): [number, number] | null {
  const doc = useEditor.getState().doc;
  const pts = (doc?.objects ?? []).filter((o) => ids.includes(o.id)).flatMap(extentPoints);
  if (pts.length === 0) return null;
  return [Math.min(...pts.map((p) => p[0])), Math.min(...pts.map((p) => p[1]))];
}

/** Edit > Duplicate (EDT-088): copies 10 px right and down; duplicating the copy again
 *  repeats where the last copy went. */
export async function duplicateSelection(offset?: [number, number]): Promise<string[]> {
  const { active, selection, setSelection } = useEditor.getState();
  if (!active || selection.length === 0) return [];
  let step = offset ?? ([10, 10] as [number, number]);
  const last = lastDuplicate;
  if (!offset && last && last.copies.join() === selection.join()) {
    // The copy moved since it was made: the next one moves the same; else the same step.
    const now = cornerOf(last.copies);
    const moved =
      now && last.at ? ([now[0] - last.at[0], now[1] - last.at[1]] as [number, number]) : null;
    step = moved && (moved[0] !== 0 || moved[1] !== 0) ? moved : last.step;
  }
  try {
    const r = await run<{ ids: string[] }>("document.duplicate", {
      documentId: active,
      objectIds: selection,
      offset: step,
    });
    setSelection(r.ids);
    // Where the copies are once the document says so (EDT-088).
    const at = cornerOf(selection);
    lastDuplicate = {
      copies: r.ids,
      at: at ? [at[0] + step[0], at[1] + step[1]] : null,
      step,
    };
    return r.ids;
  } catch (e) {
    fail(e);
    return [];
  }
}

/** Align or distribute the selection (EDT-083): against the selection's own box with
 *  two or more, against the canvas with one. One undo step. */
export function arrangeSelection(action: string): void {
  const { active, doc, selection } = useEditor.getState();
  if (!active || !doc) return;
  const items: Extent[] = doc.objects
    .filter((o) => selection.includes(o.id))
    .map((o) => {
      const pts = extentPoints(o);
      return {
        id: o.id,
        x0: Math.min(...pts.map((p) => p[0])),
        y0: Math.min(...pts.map((p) => p[1])),
        x1: Math.max(...pts.map((p) => p[0])),
        y1: Math.max(...pts.map((p) => p[1])),
      };
    })
    .filter((i) => Number.isFinite(i.x0));
  if (items.length === 0) return;
  const canvas: Extent = { id: "", x0: 0, y0: 0, x1: doc.canvas.width, y1: doc.canvas.height };
  const moves = action.startsWith("distribute")
    ? distributeMoves(items, action === "distribute_h" ? "h" : "v")
    : alignMoves(
        items,
        action.slice("align_".length) as AlignTo,
        items.length > 1 ? frameOf(items) : canvas,
      );
  if (Object.keys(moves).length === 0) return;
  run("document.update_object", {
    documentId: active,
    objectIds: Object.keys(moves),
    moves,
  }).catch(fail);
}

/** Restart sequence or Renumber the steps of a sequence (EDT-035). */
export async function stepSequence(
  action: "restart" | "renumber",
  sequence?: string,
): Promise<void> {
  const { active } = useEditor.getState();
  if (!active) return;
  await run("document.steps", {
    documentId: active,
    action,
    ...(sequence ? { sequence } : {}),
  }).catch(fail);
}

/** Gives a step the value typed in Edit value (EDT-035). */
export async function setStepValue(id: string, value: string): Promise<void> {
  const { active } = useEditor.getState();
  const v = value.trim().slice(0, 6);
  if (!active || !v) return;
  await run("document.update_object", {
    documentId: active,
    objectId: id,
    set: { value: v },
  }).catch(fail);
}

/** Flattens the selection, or every object (EDT-091). */
export async function flattenNow(all: boolean): Promise<void> {
  const { active, selection } = useEditor.getState();
  if (!active) return;
  await run(
    "document.flatten",
    all ? { documentId: active, all } : { documentId: active, objectIds: selection },
  )
    .then(() => useEditor.getState().setSelection([]))
    .catch(fail);
}

/** Makes a flattened layer (or, with `null`, every one that can) objects again, and
 * selects them (EDT-091). */
export async function unflattenLayer(layer: string | null): Promise<void> {
  const { active } = useEditor.getState();
  if (!active) return;
  await run<{ ids: string[] }>(
    "document.unflatten",
    layer ? { documentId: active, layerId: layer } : { documentId: active, all: true },
  )
    .then((r) => useEditor.getState().setSelection(r.ids))
    .catch(fail);
}

/** Locks or unlocks objects, in one undo step (EDT-085). */
export async function setLocked(ids: string[], locked: boolean): Promise<void> {
  const { active } = useEditor.getState();
  if (!active || ids.length === 0) return;
  await run("document.update_object", {
    documentId: active,
    objectIds: ids,
    set: { locked },
  }).catch(fail);
}

export function deleteSelection(): void {
  const { active, selection } = useEditor.getState();
  if (!active || selection.length === 0) return;
  run("document.delete_object", { documentId: active, objectIds: selection }).catch(fail);
}

export function showDocument(id: string): void {
  run("editor.show", { documentId: id }).catch(fail);
}

export function openCapturePanel(): void {
  run("capture.panel", { show: true }).catch(fail);
}

/** Help > Recover work (APP-047): what a session that ended unexpectedly left. */
export async function showRecovery(): Promise<void> {
  try {
    const { items } = await run<{ items: RecoveryItem[] }>("editor.recovery_list");
    if (items.length === 0) useEditor.getState().showToast(t("ui.editor.nothing_to_recover"));
    useEditor.setState({ recovery: items, recoveryShown: true });
  } catch (e) {
    fail(e);
  }
}

/** Icons only, or icons and names, in the toolbar (EDT-003, EDT-118). */
export function toggleToolbarLabels(): void {
  const current = useApp.getState().settings.find((s) => s.key === "editor.toolbarLabels");
  run("settings.set", { key: "editor.toolbarLabels", value: current?.value === false }).catch(fail);
}

/** The tray item an action is about: the one with the focus in the tray, else the
 * document shown (TRY-008, TRY-009). */
function trayTarget(): FileRef | null {
  const { documents, active } = useEditor.getState();
  const id = focusedTrayItem() ?? active;
  const doc = documents.find((d) => d.documentId === id);
  return doc ? docRef(doc) : null;
}

/** The files an action is about: the Library view's selection, or the tray item. */
function fileTargets(): FileRef[] {
  if (useMainView.getState().view === "library") return selectedItems().map(itemRef);
  const target = trayTarget();
  return target ? [target] : [];
}

/** What applies in the Library view too; the rest is about the document shown. */
const IN_LIBRARY = new Set([
  "open",
  "close_all",
  "view_editor",
  "view_library",
  "tray",
  "rename_item",
  "duplicate_item",
  "shortcut_sheet",
  "print",
  "show_in_folder",
  "delete_item",
  "open_item",
  "recover",
  "toolbar_labels",
  "new_image",
  "new_from_clipboard",
  "batch_export",
  "fullscreen",
]);

/** What an editor menu entry or shortcut does (`editor.action`, EDT-002, EDT-116).
 * False when it does not apply now, so the key goes on to the page. */
export function performAction(action: string): boolean {
  const { selection, setSelection, doc, documents, active } = useEditor.getState();
  const selected = selection.length > 0;
  // A pixel selection takes Cut, Copy, Delete and Deselect (EDT-044).
  const pixels = doc?.pixelSelection ?? null;
  if (pixels && useMainView.getState().view !== "library") {
    switch (action) {
      case "cut":
      case "copy":
      case "delete":
        void selectionAction(action);
        return true;
      case "deselect":
        void selectPixels({ shape: "none" });
        setSelection([]);
        return true;
    }
  }
  if (useMainView.getState().view === "library" && !IN_LIBRARY.has(action)) return false;
  switch (action) {
    case "view_editor":
      showView("editor");
      return true;
    case "view_library":
      showView("library");
      return true;
    case "tray":
      useTray.getState().toggle();
      return true;
    case "prev_item":
    case "next_item": {
      if (!doc) return false;
      const id = neighbour(
        documents,
        active,
        action === "next_item" ? 1 : -1,
        useEditor.getState().tray,
      );
      if (id) showDocument(id);
      return true;
    }
    case "shortcut_sheet":
      // APP-023.
      openShortcutSheet();
      return true;
    case "pin_item": {
      // TRY-004: the tray item with the focus (and the selection it is in), or the
      // one shown.
      const ids = trayTargets().filter((id) => documents.some((d) => d.documentId === id));
      if (ids.length === 0) return false;
      void togglePin(ids);
      return true;
    }
    case "duplicate_item": {
      // TRY-010: the Library's selection, or the tray's items.
      if (useMainView.getState().view === "library") {
        const ids = selectedItems().map((i) => i.id);
        if (ids.length === 0) return false;
        void duplicateItems(ids);
        return true;
      }
      const ids = trayTargets().filter((id) =>
        documents.some((d) => d.documentId === id && d.path && !d.missing),
      );
      if (ids.length === 0) return false;
      void duplicateDocs(ids);
      return true;
    }
    case "close_others": {
      const target = trayTarget();
      if (!target?.documentId) return false;
      void closeOthers(target.documentId);
      return true;
    }
    case "open_item": {
      if (useMainView.getState().view !== "library") return false;
      void openItems(selectedItems());
      return true;
    }
    case "rename_item": {
      const [file, ...more] = fileTargets();
      if (!file?.path || more.length > 0) return false;
      askRename(file);
      return true;
    }
    case "show_in_folder": {
      const file = fileTargets()[0];
      if (!file?.path) return false;
      showInFolder(file);
      return true;
    }
    case "delete_item": {
      const files = fileTargets().filter((f) => f.path);
      if (files.length === 0) return false;
      askDelete(files);
      return true;
    }
    case "open":
      void openFiles();
      return true;
    case "save":
      if (!doc) return false;
      void saveActive();
      return true;
    case "save_as":
      if (!doc) return false;
      void saveAs();
      return true;
    case "export":
      if (!doc) return false;
      void exportImage();
      return true;
    case "close":
      if (!doc) return false;
      void closeDocument();
      return true;
    case "close_all":
      void closeAll();
      return true;
    case "undo":
      undo();
      return true;
    case "redo":
      redo();
      return true;
    case "cut":
      if (!selected) return false;
      void cutObjects();
      return true;
    case "copy":
      if (!selected) return false;
      void copyObjects();
      return true;
    case "paste":
    case "paste_in_place": {
      // A template's zone clicked last takes the clipboard's image (EDT-098).
      const zone = useZones.getState().active;
      if (action === "paste" && doc && zone && doc.objects.some((o) => o.id === zone)) {
        run("document.template", {
          documentId: doc.documentId,
          action: "fill_zone",
          zone,
          clipboard: true,
        }).catch(() => void pasteObjects(false));
        return true;
      }
      void pasteObjects(action === "paste_in_place");
      return true;
    }
    case "copy_all":
      if (!doc) return false;
      void copyAll();
      return true;
    case "delete":
      if (!selected) return false;
      deleteSelection();
      return true;
    case "duplicate":
      if (!selected) return false;
      void duplicateSelection();
      return true;
    case "bring_front":
    case "bring_forward":
    case "send_backward":
    case "send_back":
      if (!selected || !doc) return false;
      run("document.arrange", {
        documentId: doc.documentId,
        objectIds: selection,
        to: {
          bring_front: "front",
          bring_forward: "forward",
          send_backward: "backward",
          send_back: "back",
        }[action],
      }).catch(fail);
      return true;
    case "align_left":
    case "align_hcenter":
    case "align_right":
    case "align_top":
    case "align_vcenter":
    case "align_bottom":
    case "distribute_h":
    case "distribute_v":
      if (!selected) return false;
      arrangeSelection(action);
      return true;
    case "import_theme":
      void importTheme();
      return true;
    case "show_in_library":
      // LIB-014: only documents of the library.
      if (!doc?.path || doc.kind !== "library") return false;
      run("library.reveal", { path: doc.path }).catch(fail);
      return true;
    case "import_template":
      void importTemplate();
      return true;
    case "new_from_template":
      // EDT-095: with images selected in the Library view, Combine in template.
      pickTemplate(
        useMainView.getState().view === "library" ? selectedItems().map((i) => i.path) : [],
      );
      return true;
    case "apply_template":
      if (!doc) return false;
      applyTemplate();
      return true;
    case "save_as_template":
      if (!doc) return false;
      askSaveTemplate();
      return true;
    case "invert_selection":
      if (!pixels) return false;
      void selectPixels({ shape: "invert" });
      return true;
    case "float_selection":
      // The pixels become a floating image, ready to move (EDT-044).
      if (!pixels || pixels.target) return false;
      void selectionAction("float").then((r) => {
        if (!r?.ids?.length) return;
        selectTool("move");
        setSelection(r.ids);
      });
      return true;
    case "crop_selection":
      if (!pixels || pixels.target) return false;
      void selectionAction("crop");
      return true;
    case "select_all":
      // With Selection or the Magic wand, every pixel (EDT-043).
      if (doc && (useEditor.getState().tool === "select" || useEditor.getState().tool === "wand")) {
        void selectPixels({ shape: "all" });
        return true;
      }
      // Every object but the locked ones (EDT-079, EDT-085).
      if (!doc || doc.objects.length === 0) return false;
      setSelection(doc.objects.filter((o) => !o.locked).map((o) => o.id));
      return true;
    case "flatten":
    case "flatten_all": {
      // Merge into the image (EDT-091); a blur cannot come back, so it asks first.
      const all = action === "flatten_all";
      const picked = doc?.objects.filter((o) => all || selection.includes(o.id)) ?? [];
      if (!doc || picked.length === 0) return false;
      if (picked.some((o) => o.type === "blur")) useEditor.setState({ flattenAsk: { all } });
      else void flattenNow(all);
      return true;
    }
    case "unflatten_all":
      if (!doc?.flattened?.some((l) => !l.permanent)) return false;
      void unflattenLayer(null);
      return true;
    case "equal_width":
    case "equal_height":
      // The same width or height as the largest (EDT-083, EDT-080).
      if (!doc || selection.length < 2) return false;
      run("document.transform", {
        documentId: doc.documentId,
        objectIds: selection,
        equal: action === "equal_width" ? "width" : "height",
      }).catch(fail);
      return true;
    case "group":
      // Two or more objects become one group (EDT-082).
      if (!doc || selection.length < 2) return false;
      run("document.group", { documentId: doc.documentId, objectIds: selection }).catch(fail);
      return true;
    case "ungroup": {
      const grouped =
        doc?.objects.filter((o) => selection.includes(o.id) && (o.groups ?? []).length > 0) ?? [];
      if (!doc || grouped.length === 0) return false;
      run("document.group", {
        documentId: doc.documentId,
        objectIds: grouped.map((o) => o.id),
        ungroup: true,
      }).catch(fail);
      return true;
    }
    case "lock": {
      // Locks the selection, or unlocks it when it is all locked already (EDT-085).
      if (!selected || !doc) return false;
      const all = doc.objects.filter((o) => selection.includes(o.id)).every((o) => o.locked);
      void setLocked(selection, !all);
      if (!all) setSelection([]);
      return true;
    }
    case "unlock_all": {
      const locked = doc?.objects.filter((o) => o.locked).map((o) => o.id) ?? [];
      if (locked.length === 0) return false;
      void setLocked(locked, false);
      return true;
    }
    case "deselect":
      if (!selected) return false;
      setSelection([]);
      return true;
    case "side_panel":
      usePanel.getState().set({ visible: !usePanel.getState().visible });
      return true;
    case "toolbar_labels":
      toggleToolbarLabels();
      return true;
    case "customize_toolbar":
      // View > Toolbar > Customize (EDT-004).
      useToolbar.setState({ customizing: true });
      return true;
    case "pixel_grid": {
      // View > Pixel grid (EDT-008).
      const current = useApp.getState().settings.find((s) => s.key === "editor.pixelGrid");
      run("settings.set", { key: "editor.pixelGrid", value: current?.value === false }).catch(fail);
      return true;
    }
    case "fullscreen":
      // View > Full screen (EDT-009).
      void toggleFullscreen();
      return true;
    case "snap": {
      // Image > Snapping (EDT-084).
      const current = useApp.getState().settings.find((s) => s.key === "editor.snap");
      run("settings.set", { key: "editor.snap", value: current?.value === false }).catch(fail);
      return true;
    }
    case "recover":
      void showRecovery();
      return true;
    case "ai_ask":
      if (!doc) return false;
      void openAi("ask");
      return true;
    case "ai_name":
      if (!doc) return false;
      void openAi("name");
      return true;
    // The Image menu (EDT-069 to EDT-077) and Edit > Revert to original (EDT-076).
    case "resize_image":
      if (!doc) return false;
      openImageDialog("resize");
      return true;
    case "rotate_custom":
      if (!doc) return false;
      openImageDialog("rotate");
      return true;
    case "new_image":
      // File > New image (EDT-103), also from the Library view.
      openImageDialog("new_image");
      return true;
    case "print":
      // File > Print (EDT-114); in the Library, its selection (LIB-017).
      if (useMainView.getState().view === "library") {
        const paths = selectedItems().map((i) => i.path);
        if (paths.length === 0) return false;
        printFiles(paths);
        return true;
      }
      if (!doc) return false;
      usePrintDialog.setState({ open: true, paths: undefined });
      return true;
    case "batch_export": {
      // EDT-113: the selection of the Library view, or the document shown.
      const inLibrary = useMainView.getState().view === "library";
      const files = inLibrary ? selectedItems().map((i) => i.path) : doc?.path ? [doc.path] : [];
      useBatchDialog.setState({ files });
      return true;
    }
    case "new_from_clipboard":
      // EDT-104: the document opens in the editor.
      run("document.new_from_clipboard")
        .then(() => showView("editor"))
        .catch(fail);
      return true;
    case "canvas_size":
    case "canvas_color":
    case "corners":
    case "resolution":
    case "revert":
    case "trim":
      if (!doc) return false;
      openImageDialog(action);
      return true;
    case "crop_to_canvas":
    case "fit_canvas":
      // EDT-051: what cannot be done, or is left sticking out, is said.
      if (!doc) return false;
      run<{ changed?: boolean; message?: string; warnings?: string[] }>(`document.${action}`, {
        documentId: doc.documentId,
      })
        .then((r) => {
          const say = r.warnings?.[0] ?? (r.changed === false ? r.message : undefined);
          if (say) useEditor.getState().showToast(say);
        })
        .catch(fail);
      return true;
    case "rotate_right":
    case "rotate_left":
    case "rotate_180": {
      if (!doc) return false;
      const degrees = action === "rotate_right" ? 90 : action === "rotate_left" ? -90 : 180;
      run("document.rotate", { documentId: doc.documentId, degrees }).catch(fail);
      return true;
    }
    case "flip_horizontal":
    case "flip_vertical":
      if (!doc) return false;
      run("document.flip", {
        documentId: doc.documentId,
        direction: action === "flip_horizontal" ? "horizontal" : "vertical",
      }).catch(fail);
      return true;
    case "flip_objects_horizontal":
    case "flip_objects_vertical":
      // Object > Flip: only the selected objects (EDT-073).
      if (!doc || !selected) return false;
      run("document.flip", {
        documentId: doc.documentId,
        direction: action === "flip_objects_horizontal" ? "horizontal" : "vertical",
        objectIds: selection,
      }).catch(fail);
      return true;
    default:
      // Image > Effects (EDT-053): `effect_page_curl` turns on pageCurl.
      if (action.startsWith("effect_"))
        return turnOnEffect(
          action.slice(7).replace(/_([a-z])/g, (_, c: string) => c.toUpperCase()),
        );
      return false;
  }
}

export async function recover(ids: string[], restore: boolean): Promise<void> {
  try {
    await run(restore ? "editor.recovery_restore" : "editor.recovery_discard", { ids });
  } catch (e) {
    fail(e);
  }
  useEditor.setState((s) => ({ recovery: s.recovery.filter((r) => !ids.includes(r.id)) }));
}

/** Starts typing a new text box at `at`, with a fixed `width` or growing (EDT-026). */
export function startNewText(at: Point, width: number | null): void {
  const { styles } = useEditor.getState();
  if (!styles) return;
  useEditor.setState({
    selection: [],
    textEdit: { id: null, at, width, text: "", rotation: 0, style: styles.text },
  });
}

/** Types into an existing text object (double click, Enter). */
export function startTextEdit(o: TextObject): void {
  useEditor.setState({
    selection: [o.id],
    textEdit: {
      id: o.id,
      at: o.at,
      width: o.width,
      text: o.text,
      rotation: o.rotation,
      style: o.style,
    },
  });
}

/** Types into a callout's body (EDT-028): double click, or right after drawing it. */
export function startCalloutEdit(o: CalloutObject): void {
  const b = textBox(o);
  useEditor.setState({
    selection: [o.id],
    textEdit: {
      id: o.id,
      at: [b.x, b.y],
      width: b.w,
      text: o.text,
      rotation: o.rotation,
      style: {
        font: o.style.font,
        size: o.style.size,
        bold: o.style.bold,
        italic: o.style.italic,
        color: o.style.textColor,
        align: o.style.align,
        opacity: 100,
      },
      // The field covers the body with its color, or a gradient's first (EDT-020).
      callout: {
        fill:
          typeof o.style.fill === "object" && o.style.fill
            ? (o.style.fill.stops[0]?.color ?? null)
            : o.style.fill,
      },
    },
  });
}

/** Ends typing: a new box becomes a text object; an empty box is removed (EDT-026). */
export async function finishTextEdit(text: string): Promise<void> {
  const { textEdit: edit, active } = useEditor.getState();
  useEditor.setState({ textEdit: null });
  if (!edit || !active) return;
  try {
    // A callout keeps its body even without text (EDT-028).
    if (edit.callout && edit.id) {
      if (text !== edit.text)
        await run("document.update_object", {
          documentId: active,
          objectId: edit.id,
          set: { text },
        });
      return;
    }
    if (edit.id === null) {
      if (!text.trim()) return;
      const r = await run<{ ids: string[] }>("document.annotate", {
        documentId: active,
        objects: [{ type: "text", at: edit.at, width: edit.width, text }],
      });
      if (r.ids[0] && setting("editor.autoSelectAfterDraw", true))
        useEditor.setState({ selection: [r.ids[0]] });
    } else if (!text.trim()) {
      await run("document.delete_object", { documentId: active, objectId: edit.id });
    } else if (text !== edit.text) {
      await run("document.update_object", { documentId: active, objectId: edit.id, set: { text } });
    }
  } catch (e) {
    fail(e);
  }
}

/** Applies the crop being adjusted (EDT-049). */
export async function applyCrop(): Promise<void> {
  const { active, cropRect, doc } = useEditor.getState();
  if (!active || !cropRect || !doc) return;
  const rect: Rect = {
    x: Math.round(cropRect.x),
    y: Math.round(cropRect.y),
    w: Math.round(cropRect.w),
    h: Math.round(cropRect.h),
  };
  // Nothing to crop: no undo step.
  if (rect.x === 0 && rect.y === 0 && rect.w === doc.canvas.width && rect.h === doc.canvas.height)
    return;
  try {
    const r = await run<{ width: number; height: number }>("document.crop", {
      documentId: active,
      rect,
    });
    // The view keeps the image in place when the new document arrives (state.ts).
    useEditor.setState({ cropRect: { x: 0, y: 0, w: r.width, h: r.height } });
  } catch (e) {
    fail(e);
  }
}

/** Puts the crop box back on the whole canvas (Esc, EDT-049). */
export function resetCrop(): void {
  const { doc, tool } = useEditor.getState();
  useEditor.setState({
    cropRect:
      tool === "crop" && doc ? { x: 0, y: 0, w: doc.canvas.width, h: doc.canvas.height } : null,
  });
}

/** Copies the selected objects (EDT-089): editable here, a PNG for other apps. */
export async function copyObjects(cut = false): Promise<void> {
  const { active, selection } = useEditor.getState();
  if (!active || selection.length === 0) return;
  try {
    await run("document.copy_objects", { documentId: active, objectIds: selection, cut });
  } catch (e) {
    fail(e);
  }
}

export async function cutObjects(): Promise<void> {
  await copyObjects(true);
}

/** Pastes the clipboard (EDT-089); `inPlace` keeps the copied objects' place. */
export async function pasteObjects(inPlace: boolean): Promise<void> {
  const { active, views } = useEditor.getState();
  const view = active ? views[active] : undefined;
  try {
    const r = await run<{ ids: string[] }>("document.paste", {
      ...(active ? { documentId: active } : {}),
      inPlace,
      ...(view ? { center: [view.cx, view.cy] } : {}),
    });
    if (r.ids.length > 0) useEditor.setState({ selection: r.ids });
  } catch (e) {
    fail(e);
  }
}

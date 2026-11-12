// What the editor's buttons, keys and dialogs do: each one runs a command (APP-027).
// The only work done here is asking the system for a file name and showing the
// answer of the command.
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

const t = (key: string, options: Record<string, unknown> = {}) => i18next.t(key, options);
const LAST_FORMAT = "xs.editor.lastFormat";

function fail(e: unknown): void {
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
          { name: t("ui.editor.open_filter"), extensions: ["xshot", "png", "jpg", "jpeg"] },
        ],
      }));
    if (!chosen) return;
    // All are opened; the last one is shown (EDT-105), in the Editor view.
    for (const path of Array.isArray(chosen) ? chosen : [chosen]) {
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
export async function exportImage(): Promise<void> {
  const { active, doc } = useEditor.getState();
  if (!active || !doc) return;
  try {
    const path = await save({ defaultPath: doc.name, filters: filters(false) });
    if (!path) return;
    const format = /\.jpe?g$/i.test(path) ? "jpg" : "png";
    // The system dialog already asked before replacing a file.
    await run("document.export", { documentId: active, format, path, onConflict: "replace" });
    rememberFormat(path);
    useEditor.getState().showToast(t("ui.editor.exported"));
  } catch (e) {
    fail(e);
  }
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

/** Close others (TRY-008): every tray item but `keep`. */
export async function closeOthers(keep: string): Promise<void> {
  const others = trayOrder(useEditor.getState().documents)
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
  if (useEditor.getState().tool !== tool) run("editor.select_tool", { tool }).catch(fail);
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
  "show_in_folder",
  "delete_item",
  "open_item",
  "recover",
  "toolbar_labels",
]);

/** What an editor menu entry or shortcut does (`editor.action`, EDT-002, EDT-116).
 * False when it does not apply now, so the key goes on to the page. */
export function performAction(action: string): boolean {
  const { selection, setSelection, doc, documents, active } = useEditor.getState();
  const selected = selection.length > 0;
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
      const id = neighbour(documents, active, action === "next_item" ? 1 : -1);
      if (id) showDocument(id);
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
    case "paste_in_place":
      void pasteObjects(action === "paste_in_place");
      return true;
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
    case "select_all":
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
    default:
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
      callout: { fill: o.style.fill },
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

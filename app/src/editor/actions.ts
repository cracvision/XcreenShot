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
} from "./state";
import { setting, useApp } from "../store";

const t = (key: string) => i18next.t(key);
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
    // All are opened; the last one is shown (EDT-105).
    for (const path of Array.isArray(chosen) ? chosen : [chosen]) {
      await run("document.open", { path, show: true });
    }
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
  const params =
    ids === "all" ? { all: true, save, discard } : { documentId: ids[0], save, discard };
  const r = await run<{ status: string; pending: Pending[] }>("document.close", params);
  if (r.status === "needs_decision") {
    useEditor.setState({ decision: { kind: "close", pending: r.pending } });
  }
}

/** Close all (EDT-108): one question for every file from elsewhere with changes. */
export async function closeAll(): Promise<void> {
  try {
    await closeWith("all", [], []);
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
      await closeWith(ids.length === 1 ? ids : "all", save, discard);
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

/** What an editor menu entry or shortcut does (`editor.action`, EDT-002, EDT-116).
 * False when it does not apply now, so the key goes on to the page. */
export function performAction(action: string): boolean {
  const { selection, setSelection, doc } = useEditor.getState();
  const selected = selection.length > 0;
  switch (action) {
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

/** Ends typing: a new box becomes a text object; an empty box is removed (EDT-026). */
export async function finishTextEdit(text: string): Promise<void> {
  const { textEdit: edit, active } = useEditor.getState();
  useEditor.setState({ textEdit: null });
  if (!edit || !active) return;
  try {
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

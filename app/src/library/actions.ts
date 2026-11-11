// What the Library view and the recent tray do with files (LIB-002, LIB-015, TRY-002,
// TRY-006, TRY-009): open, rename, show in the folder, copy the path, find a moved
// file, and delete with Undo. Each one runs a command (APP-027).
import { open } from "@tauri-apps/plugin-dialog";
import i18next from "i18next";
import { create } from "zustand";
import { isCommandError, run } from "../lib/commands";
import { setting } from "../store";
import { useEditor, type DocSummary } from "../editor/state";
import { showView, useLibrary, type LibraryItem } from "./state";

const t = (key: string, options: Record<string, unknown> = {}) => i18next.t(key, options);

/** A file the actions work on: a library item, an open document, or both. */
export interface FileRef {
  name: string;
  path: string | null;
  /** Its library item, if it is in the library. */
  itemId?: string;
  /** Its open document, if it is open in the editor. */
  documentId?: string;
}

export interface DeleteRequest {
  files: FileRef[];
  /** How many have changes that are not in their file yet. */
  unsaved: number;
  /** No trash where they are: deleting is for good (LIB-015). */
  forever: boolean;
}

/** A notice with Undo, for 10 s (LIB-015, TRY-008). */
export interface UndoNotice {
  text: string;
  seq: number;
  undo: () => Promise<void>;
}

export const useFiles = create<{
  deleting: DeleteRequest | null;
  renaming: FileRef | null;
  notice: UndoNotice | null;
}>(() => ({ deleting: null, renaming: null, notice: null }));

const UNDO_MS = 10_000;
let noticeSeq = 0;
let noticeTimer: ReturnType<typeof setTimeout> | undefined;

export function offerUndo(text: string, undo: () => Promise<void>): void {
  clearTimeout(noticeTimer);
  const seq = ++noticeSeq;
  useFiles.setState({ notice: { text, seq, undo } });
  noticeTimer = setTimeout(() => {
    if (useFiles.getState().notice?.seq === seq) useFiles.setState({ notice: null });
  }, UNDO_MS);
}

export function dismissNotice(): void {
  clearTimeout(noticeTimer);
  useFiles.setState({ notice: null });
}

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

function toast(text: string): void {
  useEditor.getState().showToast(text);
}

/** Same file, whatever the slashes and, on Windows, the case. */
export function samePath(a: string | null, b: string | null): boolean {
  if (!a || !b) return false;
  const norm = (p: string) => {
    const s = p.replace(/\//g, "\\");
    return navigator.userAgent.includes("Windows") ? s.toLowerCase() : s;
  };
  return norm(a) === norm(b);
}

/** The open document of a file, if any. */
function openDoc(path: string | null): DocSummary | undefined {
  return useEditor.getState().documents.find((d) => samePath(d.path, path));
}

export function itemRef(item: LibraryItem): FileRef {
  const doc = openDoc(item.path);
  return {
    name: item.name,
    path: item.path,
    itemId: item.id,
    ...(doc ? { documentId: doc.documentId } : {}),
  };
}

export function docRef(doc: DocSummary): FileRef {
  const item = useLibrary.getState().items.find((i) => samePath(i.path, doc.path));
  return {
    name: doc.name,
    path: doc.path,
    documentId: doc.documentId,
    ...(item ? { itemId: item.id } : {}),
  };
}

/** Opens items in the editor and shows it (LIB-001): already open ones are only
 * selected (TRY-002). The last one is shown. */
export async function openItems(items: LibraryItem[]): Promise<void> {
  try {
    for (const item of items) await run("document.open", { path: item.path, show: true });
    if (items.length > 0) showView("editor");
  } catch (e) {
    fail(e);
  }
}

/** Delete (LIB-015, TRY-009): asks first unless the preference says not to. */
export function askDelete(files: FileRef[]): void {
  if (files.length === 0) return;
  const { documents } = useEditor.getState();
  const unsaved = files.filter(
    (f) => f.documentId && documents.find((d) => d.documentId === f.documentId)?.dirty,
  ).length;
  if (setting("library.confirmDelete", true)) {
    useFiles.setState({ deleting: { files, unsaved, forever: false } });
  } else {
    void deleteFiles(files, false);
  }
}

/** Runs `library.delete`; without a trash, asks again for a delete for good. */
export async function deleteFiles(
  files: FileRef[],
  forever: boolean,
  dontAskAgain = false,
): Promise<void> {
  useFiles.setState({ deleting: null });
  try {
    if (dontAskAgain) await run("settings.set", { key: "library.confirmDelete", value: false });
    const ids = files.flatMap((f) => (f.itemId ? [f.itemId] : []));
    const paths = files.flatMap((f) => (!f.itemId && f.path ? [f.path] : []));
    const r = await run<{ deleted: number; undo: string | null }>("library.delete", {
      ...(ids.length > 0 ? { ids } : {}),
      ...(paths.length > 0 ? { paths } : {}),
      permanent: forever,
    });
    const name = files[0]?.name ?? "";
    if (r.undo) {
      const undo = r.undo;
      offerUndo(t("ui.files.deleted", { count: r.deleted, name }), async () => {
        dismissNotice();
        try {
          await run("library.restore", { undo });
          toast(t("ui.files.restored"));
        } catch (e) {
          fail(e);
        }
      });
    } else {
      toast(t("ui.files.deleted_forever", { count: r.deleted, name }));
    }
  } catch (e) {
    if (isCommandError(e) && e.details?.reason === "no_trash" && !forever) {
      // No trash there: the delete would be for good, so it always asks (LIB-015).
      const { documents } = useEditor.getState();
      const unsaved = files.filter(
        (f) => f.documentId && documents.find((d) => d.documentId === f.documentId)?.dirty,
      ).length;
      useFiles.setState({ deleting: { files, unsaved, forever: true } });
      return;
    }
    fail(e);
  }
}

export function askRename(file: FileRef): void {
  if (file.path) useFiles.setState({ renaming: file });
}

/** Renames a file in its folder (TRY-006): an error message for the dialog, or null. */
export async function renameFile(file: FileRef, name: string): Promise<string | null> {
  try {
    const r = file.itemId
      ? await run<{ name: string }>("library.rename", { id: file.itemId, name })
      : await run<{ name: string }>("document.rename", { documentId: file.documentId, name });
    useFiles.setState({ renaming: null });
    toast(t("ui.files.renamed", { name: r.name }));
    return null;
  } catch (e) {
    return isCommandError(e) ? e.message : String(e);
  }
}

/** Show in Explorer / Finder (EDT-109). */
export function showInFolder(file: FileRef): void {
  if (!file.path) return;
  run("library.show_in_folder", { path: file.path }).catch(fail);
}

export function copyPath(file: FileRef): void {
  if (!file.path) return;
  navigator.clipboard
    .writeText(file.path)
    .then(() => toast(t("ui.files.path_copied")))
    .catch(fail);
}

/** Find a file moved outside XcreenShot (TRY-002): a file of the same kind. */
export async function locate(doc: DocSummary): Promise<void> {
  try {
    const ext = doc.format === "jpg" ? ["jpg", "jpeg"] : [doc.format];
    const path = await open({
      multiple: false,
      filters: [{ name: t("ui.files.locate_filter"), extensions: ext }],
    });
    if (!path || Array.isArray(path)) return;
    await run("document.locate", { documentId: doc.documentId, path });
  } catch (e) {
    fail(e);
  }
}

/** Remove from the tray (TRY-002): the file, wherever it is, stays. Changes of a file
 * that is gone cannot be saved back, so they are dropped. */
export function removeFromTray(doc: DocSummary): void {
  run("document.close", { documentIds: [doc.documentId], discard: [doc.documentId] }).catch(fail);
}

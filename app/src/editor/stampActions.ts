// Own stamps, packs and emoji (EDT-032 to EDT-034): commands of the core with the
// editor's notices and errors, and the state of their dialogs.
import i18next from "i18next";
import { open, save } from "@tauri-apps/plugin-dialog";
import { create } from "zustand";
import { run } from "../lib/commands";
import { useEditor } from "./state";

export const PICTURE_FILES = ["png", "jpg", "jpeg", "gif", "webp", "svg"];
export const PACK_FILE = "xstamps";

export interface StampCategory {
  id: string;
  name: string;
  own?: boolean;
  count?: number;
  pack?: { name: string; version: string; author: string; license: string } | null;
}

export interface StampEntry {
  id: string;
  category: string;
  name: string;
  keywords?: string;
  own?: boolean;
}

export interface StampsFound {
  categories: StampCategory[];
  stamps: StampEntry[];
}

/** A pack already here with the same name (EDT-034): update or keep both. */
export interface PackConflict {
  path: string;
  name: string;
  version: string;
  installed: string;
  canUpdate: boolean;
}

interface StampsUi {
  /** Organize stamps is open, on this own category. */
  organize: string | null;
  /** File > Save as stamp is open. */
  saveAs: boolean;
  conflict: PackConflict | null;
  /** Bumped when the own stamps change, so lists load again. */
  version: number;
}

export const useStampsUi = create<StampsUi>(() => ({
  organize: null,
  saveAs: false,
  conflict: null,
  version: 0,
}));

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

export function stampsChanged(): void {
  useStampsUi.setState((s) => ({ version: s.version + 1 }));
}

export function organizeStamps(category = "own:mine"): void {
  useStampsUi.setState({ organize: category });
}

/** File > Save as stamp (EDT-032). */
export function askSaveAsStamp(): void {
  if (useEditor.getState().active) useStampsUi.setState({ saveAs: true });
}

export async function saveAsStamp(
  name: string,
  keywords: string,
  category: string,
): Promise<boolean> {
  try {
    const r = await run<{ name: string }>("document.save_as_stamp", {
      documentId: useEditor.getState().active,
      name,
      keywords,
      category,
    });
    useEditor.getState().showToast(i18next.t("ui.editor.stamps.saved", { name: r.name }));
    stampsChanged();
    return true;
  } catch (e) {
    fail(e);
    return false;
  }
}

/** Add stamps… (EDT-032): picture files to an own category; asks for them if none. */
export async function addStampFiles(paths?: string[], category = "own:mine"): Promise<void> {
  try {
    const chosen =
      paths ??
      ((await open({
        multiple: true,
        filters: [{ name: i18next.t("ui.editor.stamps.pictures"), extensions: PICTURE_FILES }],
      })) as string[] | null);
    if (!chosen || chosen.length === 0) return;
    const r = await run<{ added: unknown[]; failed: { file: string; reason: string }[] }>(
      "stamps.add",
      { files: chosen, category },
    );
    const failed = r.failed[0];
    if (failed)
      useEditor
        .getState()
        .showToast(
          i18next.t("ui.editor.stamps.add_failed", { name: failed.file, reason: failed.reason }),
        );
    else if (r.added.length > 0)
      useEditor
        .getState()
        .showToast(i18next.t("ui.editor.stamps.added", { count: r.added.length }));
    stampsChanged();
  } catch (e) {
    fail(e);
  }
}

/** File > Import > Stamps, a .xstamps opened or dropped (EDT-034). */
export async function importStampPack(path?: string, existing?: "update" | "both"): Promise<void> {
  try {
    const chosen =
      path ??
      ((await open({
        filters: [{ name: i18next.t("ui.editor.stamps.pack_file"), extensions: [PACK_FILE] }],
      })) as string | null);
    if (!chosen) return;
    const r = await run<{
      status: string;
      count?: number;
      category?: string;
      name?: string;
      version?: string;
      installed?: string;
      canUpdate?: boolean;
    }>("stamps.import", { path: chosen, ...(existing ? { existing } : {}) });
    if (r.status === "exists") {
      useStampsUi.setState({
        conflict: {
          path: chosen,
          name: r.name ?? "",
          version: r.version ?? "",
          installed: r.installed ?? "",
          canUpdate: !!r.canUpdate,
        },
      });
      return;
    }
    useEditor.getState().showToast(
      i18next.t(r.status === "updated" ? "ui.editor.stamps.updated" : "ui.editor.stamps.imported", {
        count: r.count ?? 0,
      }),
    );
    stampsChanged();
  } catch (e) {
    fail(e);
  }
}

/** Export category… (EDT-034). */
export async function exportStampCategory(
  category: string,
  name: string,
  author: string,
  license: string,
): Promise<void> {
  try {
    const path = await save({
      defaultPath: `${name}.${PACK_FILE}`,
      filters: [{ name: i18next.t("ui.editor.stamps.pack_file"), extensions: [PACK_FILE] }],
    });
    if (!path) return;
    const r = await run<{ count: number }>("stamps.export", {
      category,
      path,
      name,
      author,
      license,
    });
    useEditor.getState().showToast(i18next.t("ui.editor.stamps.exported", { count: r.count }));
  } catch (e) {
    fail(e);
  }
}

/** Files dropped on the stamp collection become own stamps (EDT-032): whether they were. */
export function dropOnStamps(paths: string[], position: { x: number; y: number }): boolean {
  const k = window.devicePixelRatio || 1;
  const at = document.elementFromPoint(position.x / k, position.y / k);
  if (!at?.closest(".stamp-browser")) return false;
  const pictures = paths.filter((p) =>
    PICTURE_FILES.some((e) => p.toLowerCase().endsWith(`.${e}`)),
  );
  if (pictures.length === 0) return false;
  void addStampFiles(pictures);
  return true;
}

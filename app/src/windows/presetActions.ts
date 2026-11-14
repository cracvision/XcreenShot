// What the panel's Presets section does (CAP-058 to CAP-064): the commands, and which
// dialog is open. The section and the panel's More menu share it.
import { open, save } from "@tauri-apps/plugin-dialog";
import i18next from "i18next";
import { create } from "zustand";
import { isCommandError, run } from "../lib/commands";
import type { Preset } from "./presetsState";

export interface ImportRow {
  name: string;
  kind: string;
  exists: boolean;
  shortcut: string | null;
  shortcutTaken: boolean;
  problem: string | null;
}

type Dialog =
  | { kind: "edit"; preset: Preset }
  | { kind: "new"; presetKind: "universal" | "image" }
  | { kind: "shortcut"; preset: Preset }
  | { kind: "delete"; presets: Preset[] }
  | { kind: "update"; preset: Preset }
  | { kind: "import"; paths: string[]; rows: ImportRow[] }
  | { kind: "builtins"; modified: string[] };

interface PresetUi {
  presets: Preset[];
  dialog: Dialog | null;
  /** The row being renamed in place. */
  renaming: string | null;
  /** Undo of the last delete, for 10 s (CAP-060). */
  undo: { token: string; text: string } | null;
  message: { text: string; error: boolean } | null;
}

export const usePresetUi = create<PresetUi>(() => ({
  presets: [],
  dialog: null,
  renaming: null,
  undo: null,
  message: null,
}));

const t = (key: string, o?: Record<string, unknown>) => i18next.t(key, o ?? {});

function fail(e: unknown): void {
  usePresetUi.setState({
    message: { text: isCommandError(e) ? e.message : String(e), error: true },
  });
}

function say(text: string): void {
  usePresetUi.setState({ message: { text, error: false } });
}

export async function loadPresets(): Promise<void> {
  try {
    const r = await run<{ presets: Preset[] }>("preset.list");
    usePresetUi.setState({ presets: r.presets });
  } catch (e) {
    fail(e);
  }
}

export async function runPreset(p: Preset): Promise<void> {
  usePresetUi.setState({ message: null });
  try {
    await run("preset.run", { id: p.id });
  } catch (e) {
    if (isCommandError(e) && e.code === "cancelled") return;
    fail(e);
  }
}

/** Creates a preset and leaves its name editable in the row (CAP-059). */
export async function createPreset(params: Record<string, unknown>): Promise<Preset | null> {
  try {
    const p = await run<Preset>("preset.create", params);
    await loadPresets();
    usePresetUi.setState({ renaming: p.id, dialog: null });
    return p;
  } catch (e) {
    fail(e);
    return null;
  }
}

export async function renamePreset(p: Preset, to: string): Promise<void> {
  usePresetUi.setState({ renaming: null });
  if (!to.trim() || to.trim() === p.name) return;
  try {
    await run("preset.rename", { id: p.id, to });
    await loadPresets();
  } catch (e) {
    fail(e);
  }
}

export async function savePreset(
  p: Preset,
  name: string,
  settings: Record<string, unknown>,
): Promise<boolean> {
  try {
    await run("preset.update", { id: p.id, settings });
    if (name.trim() && name.trim() !== p.name) await run("preset.rename", { id: p.id, to: name });
    usePresetUi.setState({ dialog: null });
    await loadPresets();
    return true;
  } catch (e) {
    fail(e);
    return false;
  }
}

export async function duplicatePreset(p: Preset): Promise<void> {
  try {
    await run("preset.duplicate", { id: p.id });
    await loadPresets();
  } catch (e) {
    fail(e);
  }
}

export async function updateFromCurrent(p: Preset): Promise<void> {
  usePresetUi.setState({ dialog: null });
  try {
    await run("preset.update", { id: p.id, fromCurrent: true });
    await loadPresets();
    say(t("ui.panel.presets.updated", { name: p.name }));
  } catch (e) {
    fail(e);
  }
}

let undoTimer: ReturnType<typeof setTimeout> | undefined;

export async function deletePresets(list: Preset[], dontAskAgain = false): Promise<void> {
  usePresetUi.setState({ dialog: null });
  try {
    if (dontAskAgain)
      await run("settings.set", { key: "capture.confirmPresetDelete", value: false });
    const r = await run<{ deleted: number; undo: string }>("preset.delete", {
      ids: list.map((p) => p.id),
    });
    await loadPresets();
    const text = t("ui.panel.presets.deleted", { count: r.deleted, name: list[0]?.name ?? "" });
    usePresetUi.setState({ undo: { token: r.undo, text } });
    clearTimeout(undoTimer);
    undoTimer = setTimeout(() => usePresetUi.setState({ undo: null }), 10_000);
  } catch (e) {
    fail(e);
  }
}

export async function undoDelete(): Promise<void> {
  const undo = usePresetUi.getState().undo;
  if (!undo) return;
  usePresetUi.setState({ undo: null });
  clearTimeout(undoTimer);
  try {
    await run("preset.restore", { undo: undo.token });
    await loadPresets();
  } catch (e) {
    fail(e);
  }
}

export async function movePreset(p: Preset, index: number): Promise<void> {
  try {
    await run("preset.move", { id: p.id, index });
    await loadPresets();
  } catch (e) {
    fail(e);
  }
}

const FILTERS = () => [{ name: t("ui.panel.presets.file_type"), extensions: ["xpreset"] }];

export async function exportPresets(list: Preset[] | null): Promise<void> {
  const one = list?.length === 1 ? list[0] : null;
  const path = await save({
    defaultPath: `${one ? one.name : t("ui.panel.presets.file_name")}.xpreset`,
    filters: FILTERS(),
  });
  if (!path) return;
  try {
    const r = await run<{ count: number }>("preset.export", {
      out: path,
      ...(list ? { ids: list.map((p) => p.id) } : {}),
    });
    say(t("ui.panel.presets.exported", { count: r.count }));
  } catch (e) {
    fail(e);
  }
}

/** Import presets…: the files' presets first, to choose (CAP-063). */
export async function askImport(chosen?: string[]): Promise<void> {
  const picked = chosen ?? (await open({ multiple: true, filters: FILTERS() })) ?? ([] as string[]);
  const paths = Array.isArray(picked) ? picked : [picked];
  if (paths.length === 0) return;
  try {
    const r = await run<{ presets: ImportRow[] }>("preset.import", { paths, dryRun: true });
    usePresetUi.setState({ dialog: { kind: "import", paths, rows: r.presets } });
  } catch (e) {
    fail(e);
  }
}

export async function importPresets(
  paths: string[],
  only: string[],
  onConflict: "keepBoth" | "replace" | "skip",
): Promise<void> {
  usePresetUi.setState({ dialog: null });
  try {
    const r = await run<{ imported: number; withoutShortcut: string[] }>("preset.import", {
      paths,
      only,
      onConflict,
    });
    await loadPresets();
    const parts = [t("ui.panel.presets.imported", { count: r.imported })];
    if (r.withoutShortcut.length > 0)
      parts.push(t("ui.panel.presets.without_shortcut", { names: r.withoutShortcut.join(", ") }));
    say(parts.join(" "));
  } catch (e) {
    fail(e);
  }
}

/** Restore built-in presets: what was deleted comes back; changed ones are asked. */
export async function restoreBuiltins(reset = false): Promise<void> {
  usePresetUi.setState({ dialog: null });
  try {
    const r = await run<{ restored: string[]; modified: string[] }>("preset.restore_builtins", {
      reset,
    });
    await loadPresets();
    if (!reset && r.modified.length > 0) {
      usePresetUi.setState({ dialog: { kind: "builtins", modified: r.modified } });
      return;
    }
    say(
      r.restored.length > 0 || reset
        ? t("ui.panel.presets.builtins_restored")
        : t("ui.panel.presets.builtins_there"),
    );
  } catch (e) {
    fail(e);
  }
}

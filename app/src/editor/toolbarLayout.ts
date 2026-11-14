// The editor toolbar's items (EDT-003, EDT-004): what is on it, in order, kept by the
// core (`editor.toolbar`); what is not is in More. Pure helpers for the customizer.
import { create } from "zustand";
import { run } from "../lib/commands";
import type { Tool } from "./state";

export interface ToolbarData {
  items: string[];
  default: string[];
  available: { tools: Tool[]; actions: string[]; effects: string[] };
}

export const useToolbar = create<{
  data: ToolbarData | null;
  customizing: boolean;
  /** A tool chosen in More: at the end of the bar until another one (EDT-003). */
  temp: Tool | null;
}>(() => ({ data: null, customizing: false, temp: null }));

export async function loadToolbar(): Promise<void> {
  try {
    useToolbar.setState({ data: await run<ToolbarData>("editor.toolbar") });
  } catch {
    // The bar keeps what it had.
  }
}

/** Keeps a new order; the bar changes at once. */
export async function saveToolbar(items: string[]): Promise<void> {
  const data = useToolbar.getState().data;
  if (data) useToolbar.setState({ data: { ...data, items } });
  try {
    useToolbar.setState({
      data: await run<ToolbarData>("editor.toolbar", { action: "set", items }),
    });
  } catch {
    await loadToolbar();
  }
}

export async function resetToolbar(): Promise<void> {
  try {
    useToolbar.setState({ data: await run<ToolbarData>("editor.toolbar", { action: "reset" }) });
  } catch {
    // Nothing changed.
  }
}

/** The tools on the bar, in order: Ctrl+1…9 (EDT-115). */
export function barTools(items: string[]): Tool[] {
  return items.filter((i) => i.startsWith("tool:")).map((i) => i.slice(5) as Tool);
}

/** More's groups (EDT-003). */
export const TOOL_GROUPS: { key: "annotation" | "pixels" | "image"; tools: Tool[] }[] = [
  {
    key: "annotation",
    tools: [
      "favorites",
      "move",
      "arrow",
      "line",
      "text",
      "callout",
      "shape",
      "stamp",
      "step",
      "highlight",
      "pen",
      "magnify",
      "spotlight",
      "blur",
      "simplify",
    ],
  },
  { key: "pixels", tools: ["select", "wand", "fill", "eraser", "cutout"] },
  { key: "image", tools: ["crop", "measure"] },
];

/** `items` with the one at `from` moved to `to` (its index after the move). */
export function moveItem(items: string[], from: number, to: number): string[] {
  const next = [...items];
  const [item] = next.splice(from, 1);
  if (item === undefined) return items;
  next.splice(Math.max(0, Math.min(next.length, to)), 0, item);
  return next;
}

/** `item` put at `at`; a tool or command already on the bar moves there. */
export function insertItem(items: string[], item: string, at: number): string[] {
  const repeatable = item === "sep" || item === "space";
  const from = repeatable ? -1 : items.indexOf(item);
  if (from >= 0) return moveItem(items, from, from < at ? at - 1 : at);
  const next = [...items];
  next.splice(Math.max(0, Math.min(next.length, at)), 0, item);
  return next;
}

export function removeAt(items: string[], index: number): string[] {
  return items.filter((_, i) => i !== index);
}

/** The icon of a command on the bar. */
export function actionIcon(action: string): string {
  if (action.startsWith("effect_")) return "effects";
  return (
    {
      save: "save",
      save_as: "save_as",
      export: "export",
      new_from_clipboard: "paste_new",
      copy_all: "copy",
      revert: "revert",
      trim: "trim",
      undo: "undo",
      redo: "redo",
      print: "print",
    }[action] ?? "more"
  );
}

/** The name of a command or effect on the bar. */
export function actionName(t: (k: string) => string, action: string): string {
  if (action.startsWith("effect_")) {
    const type = action.slice(7).replace(/_([a-z])/g, (_, c: string) => c.toUpperCase());
    return t(`ui.editor.fx.${type}.name`);
  }
  return t(`ui.editor.toolbar_item.${action}`);
}

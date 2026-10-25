// The editor's tools (EDT-003, EDT-115).
import type { Tool } from "./state";

/** One key per tool, the same in English and Spanish (EDT-115). */
export const TOOL_KEYS: Record<Tool, string> = {
  move: "V",
  arrow: "A",
  text: "T",
  shape: "U",
  blur: "B",
  crop: "R",
};

/** The tool a key activates, by physical key. */
export const TOOL_BY_CODE: Record<string, Tool> = Object.fromEntries(
  Object.entries(TOOL_KEYS).map(([tool, key]) => [`Key${key}`, tool as Tool]),
);

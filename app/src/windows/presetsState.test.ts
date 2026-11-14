import { describe, expect, it } from "vitest";
import { dropIndex, filterPresets, keyTarget, type Preset } from "./presetsState";

const preset = (name: string, summary = ""): Preset => ({
  id: name,
  name,
  kind: "image",
  settings: {},
  summary,
  builtin: null,
  shortcut: null,
  shortcutDisplay: null,
  slot: `preset.${name}`,
  running: false,
  warnings: [],
});

describe("presets list (CAP-058, CAP-061)", () => {
  it("filters by name or summary without case or accents", () => {
    const list = [preset("Región al portapapeles", "Región · Sin editor"), preset("Ventana 3 s")];
    expect(filterPresets(list, "region").map((p) => p.name)).toEqual(["Región al portapapeles"]);
    expect(filterPresets(list, "SIN EDITOR")).toHaveLength(1);
    expect(filterPresets(list, " ")).toHaveLength(2);
  });

  it("moves with the arrows, Home and End", () => {
    expect(keyTarget(0, 3, "ArrowDown")).toBe(1);
    expect(keyTarget(2, 3, "ArrowDown")).toBe(2);
    expect(keyTarget(0, 3, "ArrowUp")).toBe(0);
    expect(keyTarget(1, 3, "End")).toBe(2);
    expect(keyTarget(2, 3, "Home")).toBe(0);
    expect(keyTarget(0, 3, "a")).toBeNull();
    expect(keyTarget(0, 0, "ArrowDown")).toBeNull();
  });

  it("drops where the line shows", () => {
    // The third row dropped before the first becomes the first.
    expect(dropIndex(2, 0, false)).toBe(0);
    // The first dropped after the third becomes the last.
    expect(dropIndex(0, 2, true)).toBe(2);
    // Dropped on itself, it stays.
    expect(dropIndex(1, 1, false)).toBe(1);
    expect(dropIndex(1, 1, true)).toBe(1);
  });
});

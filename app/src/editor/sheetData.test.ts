import { describe, expect, it } from "vitest";
import { CONTEXTUAL, matches } from "./sheetData";

describe("shortcut sheet search (APP-023)", () => {
  it("finds an action by its name or its keys, without accents", () => {
    const arrow = { title: "Herramienta Flecha", keys: "A" };
    expect(matches(arrow, "flecha")).toBe(true);
    expect(matches({ title: "Línea", keys: "L" }, "linea")).toBe(true);
    expect(matches({ title: "Guardar", keys: "Ctrl+S" }, "ctrl+s")).toBe(true);
    expect(matches(arrow, "texto")).toBe(false);
    expect(matches(arrow, "  ")).toBe(true);
  });
});

describe("the shortcut sheet's keys of a moment (VID-076)", () => {
  it("lists the player's and live draw's keys", () => {
    const video = CONTEXTUAL.find((g) => g.id === "video");
    const labels = video?.rows.map((r) => r.label) ?? [];
    expect(labels).toContain("ui.shortcuts.k_play");
    expect(labels).toContain("ui.shortcuts.k_frame");
    expect(labels).toContain("ui.shortcuts.k_draw_tools");
    expect(video?.rows.find((r) => r.label === "ui.shortcuts.k_draw_tools")?.keys).toContain("O");
  });
});

import { describe, expect, it } from "vitest";
import { matches } from "./sheetData";

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

import i18next from "i18next";
import { beforeAll, describe as group, expect, it } from "vitest";
import "../i18n";
import { describe } from "./announce";
import type { DocObject } from "./state";

const text = {
  id: "t",
  type: "text",
  at: [320, 140],
  width: null,
  text: "Paso 1",
  rotation: 0,
  style: {
    font: "Segoe UI",
    size: 14,
    bold: false,
    italic: false,
    color: "#F2545B",
    align: "left",
    opacity: 100,
  },
  layout: { width: 200, height: 80 },
} as DocObject;

group("screen reader announcements (EDT-013)", () => {
  beforeAll(async () => {
    await i18next.changeLanguage("es");
  });

  it("say the type, the text, the position and the size", () => {
    expect(describe(text)).toBe("Texto: Paso 1, en 320, 140, 200 por 80 píxeles");
  });

  it("name other objects by their tool", () => {
    const arrow = { id: "a", type: "arrow", from: [10, 20], to: [110, 20] } as DocObject;
    expect(describe(arrow)).toMatch(/^Flecha, en 10, /);
  });
});

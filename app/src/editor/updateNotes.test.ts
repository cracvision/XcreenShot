import { describe, expect, it } from "vitest";
import { notesFor } from "./updateNotes";

describe("release notes (APP-036)", () => {
  it("in the app's language, English without a translation", () => {
    const both = JSON.stringify({ en: "New stamps", es: "Sellos nuevos" });
    expect(notesFor(both, "es")).toBe("Sellos nuevos");
    expect(notesFor(both, "es-MX")).toBe("Sellos nuevos");
    expect(notesFor(JSON.stringify({ en: "Only English" }), "es")).toBe("Only English");
    expect(notesFor("Plain notes", "es")).toBe("Plain notes");
    expect(notesFor(null, "en")).toBe("");
  });
});

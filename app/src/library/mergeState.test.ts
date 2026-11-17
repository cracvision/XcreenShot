import i18next from "i18next";
import { describe, expect, it, vi } from "vitest";
import "../i18n";
import { baseName, mergeSummary, type MergeReport } from "./mergeState";

vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

const report = (r: Partial<MergeReport>): MergeReport => ({
  added: 0,
  duplicates: 0,
  enriched: 0,
  renamed: [],
  failed: [],
  cancelled: false,
  ...r,
});

describe("merge report (LIB-031)", () => {
  it("says what was added, what was already there and what changed name", async () => {
    await i18next.changeLanguage("en");
    const t = i18next.t.bind(i18next);
    const lines = mergeSummary(
      report({
        added: 280,
        duplicates: 20,
        enriched: 1,
        renamed: [{ from: "2026-10/a.png", path: "C:/lib/2026-10/a (2).png" }],
      }),
      t,
    );
    expect(lines).toEqual([
      "280 items added to your library.",
      "20 were already in your library and were not copied.",
      "1 of yours gained tags or the favorite.",
      "1 got another name because its name was taken:",
    ]);
  });

  it("a cancelled merge says nothing changed, in Spanish too", async () => {
    await i18next.changeLanguage("es");
    const t = i18next.t.bind(i18next);
    expect(mergeSummary(report({ cancelled: true, added: 3 }), t)).toEqual([
      "No cambió nada: lo que se había copiado se quitó.",
    ]);
    expect(mergeSummary(report({ added: 1 }), t)).toEqual([
      "Se añadió 1 elemento a tu biblioteca.",
    ]);
    await i18next.changeLanguage("en");
  });

  it("names come from the end of a path", () => {
    expect(baseName("C:\\lib\\2026-10\\a (2).png")).toBe("a (2).png");
    expect(baseName("2026-10/a.png")).toBe("a.png");
  });
});

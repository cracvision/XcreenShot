import { describe, expect, it } from "vitest";
import { nextLanguages } from "./languageList";

describe("recognition languages (SMT-004)", () => {
  it("Automatic alone, or up to 3 languages", () => {
    expect(nextLanguages(["auto"], "es", true)).toEqual(["es"]);
    expect(nextLanguages(["es"], "pt", true)).toEqual(["es", "pt"]);
    expect(nextLanguages(["es", "pt", "en"], "de", true)).toEqual(["pt", "en", "de"]);
    expect(nextLanguages(["es"], "es", false)).toEqual(["auto"]);
    expect(nextLanguages(["es", "pt"], "auto", true)).toEqual(["auto"]);
  });
});

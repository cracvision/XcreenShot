import { afterEach, describe, expect, it } from "vitest";
import { applyAccent, applyGlass, glassLevel } from "./theme";

afterEach(() => {
  delete document.documentElement.dataset.accent;
  document.documentElement.style.removeProperty("--glass");
});

describe("the glass intensity (APP-012, Cristal)", () => {
  it("goes from solid at 0 to full at 100", () => {
    expect(glassLevel(0)).toBe(0);
    expect(glassLevel(60)).toBe(0.6);
    expect(glassLevel(100)).toBe(1);
  });

  it("keeps out-of-range or missing values sensible", () => {
    expect(glassLevel(250)).toBe(1);
    expect(glassLevel(-5)).toBe(0);
    expect(glassLevel(undefined)).toBe(0.6);
  });

  it("is applied as a CSS variable", () => {
    applyGlass(25);
    expect(document.documentElement.style.getPropertyValue("--glass")).toBe("0.25");
  });
});

describe("the accent color (APP-012, Cristal)", () => {
  it("marks the root for the accents other than blue", () => {
    applyAccent("teal");
    expect(document.documentElement.dataset.accent).toBe("teal");
    applyAccent("blue");
    expect(document.documentElement.dataset.accent).toBeUndefined();
    applyAccent("purple");
    expect(document.documentElement.dataset.accent).toBeUndefined();
  });
});

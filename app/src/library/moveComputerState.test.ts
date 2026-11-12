import i18next from "i18next";
import { describe, expect, it } from "vitest";
import "../i18n";
import { fileName, libraryStep } from "./moveComputerState";

describe("move to another computer (APP-055, LIB-032)", () => {
  it("a local library is copied by hand; a synced one arrives by its service", async () => {
    await i18next.changeLanguage("es");
    const t = i18next.t.bind(i18next);
    const local = libraryStep({ place: "local", service: null }, t);
    expect(local.synced).toBe(false);
    expect(local.text).toContain("Copia esta carpeta");
    const synced = libraryStep({ place: "synced", service: "onedrive" }, t);
    expect(synced.synced).toBe(true);
    expect(synced.text).toContain("OneDrive");
    const unknown = libraryStep({ place: "synced", service: null }, t);
    expect(unknown.text).toContain("carpeta sincronizada");
    await i18next.changeLanguage("en");
  });

  it("names the saved file", () => {
    expect(fileName("D:\\copias\\XcreenShot.xsettings")).toBe("XcreenShot.xsettings");
  });
});

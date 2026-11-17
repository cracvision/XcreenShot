import { describe, expect, it } from "vitest";
import { insideFolder, splitTarget } from "./exportState";

describe("export from the library (LIB-018, LIB-033)", () => {
  it("reads the folder, name and format of a Save as path", () => {
    expect(splitTarget(String.raw`C:\Users\Ana\Desktop\Informe.jpeg`)).toEqual({
      folder: String.raw`C:\Users\Ana\Desktop`,
      stem: "Informe",
      format: "jpg",
    });
    expect(splitTarget("/Users/ana/Informe v2.tif").format).toBe("tiff");
    expect(splitTarget("/Users/ana/Informe v2")).toEqual({
      folder: "/Users/ana",
      stem: "Informe v2",
      format: "png",
    });
    expect(splitTarget("/Users/ana/Notas.txt")).toEqual({
      folder: "/Users/ana",
      stem: "Notas.txt",
      format: "png",
    });
  });

  it("knows a folder inside the library", () => {
    expect(insideFolder("C:/Lib", "C:/Lib")).toBe(true);
    expect(insideFolder("C:/Lib", "C:/Lib/2026-11/a.xshot")).toBe(true);
    expect(insideFolder("C:/Lib", "C:/Library")).toBe(false);
    expect(insideFolder("", "C:/Lib")).toBe(false);
  });
});

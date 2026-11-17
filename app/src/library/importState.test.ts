import { describe, expect, it } from "vitest";
import { looksLikeFolder } from "./importState";

describe("import to the library (LIB-024)", () => {
  it("tells folders from images by their name", () => {
    expect(looksLikeFolder("D:/Proyectos/Proyecto X")).toBe(true);
    expect(looksLikeFolder("D:/Proyectos/v1.2")).toBe(true);
    expect(looksLikeFolder("D:/Fotos/IMG_001.JPG")).toBe(false);
    expect(looksLikeFolder("D:/Capturas/Informe.xshot")).toBe(false);
  });
});

import { beforeEach, describe, expect, it } from "vitest";
import {
  DEFAULT_OPTIONS,
  fileSize,
  lastExportFormat,
  rememberExport,
  rememberedOptions,
} from "./exportOptions";

describe("export options (EDT-112)", () => {
  beforeEach(() => localStorage.clear());

  it("remembers the options of each format and the last format", () => {
    expect(lastExportFormat()).toBe("png");
    rememberExport("tiff", { ...DEFAULT_OPTIONS, tiffCompression: "zip" });
    expect(rememberedOptions("tiff").tiffCompression).toBe("zip");
    expect(rememberedOptions("png").tiffCompression).toBe("lzw");
    expect(lastExportFormat()).toBe("tiff");
  });

  it("writes sizes for people", () => {
    expect(fileSize(500)).toBe("500 B");
    expect(fileSize(2048)).toBe("2 KB");
    expect(fileSize(3 * 1024 * 1024 + 200 * 1024, "es")).toBe("3,2 MB");
  });
});

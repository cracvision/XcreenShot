import { describe, expect, it } from "vitest";
import { fileSize, when } from "./format";

describe("library formats", () => {
  it("writes sizes up to gigabytes", () => {
    expect(fileSize(512, "en")).toBe("512 B");
    expect(fileSize(2048, "en")).toBe("2 KB");
    expect(fileSize(3.25 * 1024 * 1024, "es")).toBe("3,3 MB");
    expect(fileSize(5 * 1024 ** 3, "en")).toBe("5 GB");
  });

  it("shows a dash for a missing or broken date", () => {
    expect(when(null, "en")).toBe("—");
    expect(when("not a date", "en")).toBe("—");
    expect(when("2026-11-12T10:00:00Z", "en")).not.toBe("—");
  });
});

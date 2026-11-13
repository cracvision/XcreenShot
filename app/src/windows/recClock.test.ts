import { describe, expect, it } from "vitest";
import { clock } from "./recClock";

describe("clock", () => {
  it("shows hours, minutes and seconds, never empty", () => {
    expect(clock(0)).toBe("00:00:00");
    expect(clock(61.9)).toBe("00:01:01");
    expect(clock(3 * 3600 + 7)).toBe("03:00:07");
    expect(clock(-5)).toBe("00:00:00");
  });
});

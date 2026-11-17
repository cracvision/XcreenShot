import { describe, expect, it } from "vitest";
import { clock, timeLeft } from "./recClock";

describe("clock", () => {
  it("shows hours, minutes and seconds, never empty", () => {
    expect(clock(0)).toBe("00:00:00");
    expect(clock(61.9)).toBe("00:01:01");
    expect(clock(3 * 3600 + 7)).toBe("03:00:07");
    expect(clock(-5)).toBe("00:00:00");
  });
});

describe("the timer (VID-015)", () => {
  it("says what is left of the time recorded, none without a timer", () => {
    expect(timeLeft(30, 12.4)).toBeCloseTo(17.6);
    expect(timeLeft(30, 31)).toBe(0);
    expect(timeLeft(null, 10)).toBeNull();
    expect(timeLeft(undefined, 10)).toBeNull();
  });
});

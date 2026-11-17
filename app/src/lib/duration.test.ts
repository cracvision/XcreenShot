import { describe, expect, it } from "vitest";
import { joinSeconds, shortDuration, splitSeconds, timerProblem, TIMER_MAX } from "./duration";

describe("timer length (VID-015)", () => {
  it("splits and joins hours, minutes and seconds", () => {
    expect(splitSeconds(3725)).toEqual({ h: 1, m: 2, s: 5 });
    expect(joinSeconds(1, 2, 5)).toBe(3725);
    expect(joinSeconds(0, Number.NaN, 30)).toBe(30);
    expect(joinSeconds(0, 90, 0)).toBe(5400);
  });

  it("says a length briefly", () => {
    expect([30, 60, 300, 3600, 5400, 3725].map(shortDuration)).toEqual([
      "30 s",
      "1 min",
      "5 min",
      "1 h",
      "1 h 30 min",
      "1:02:05",
    ]);
  });

  it("goes from 5 s to 12 h, and more than what is recorded", () => {
    expect(timerProblem(4)).toBe("range");
    expect(timerProblem(5)).toBeNull();
    expect(timerProblem(TIMER_MAX)).toBeNull();
    expect(timerProblem(TIMER_MAX + 1)).toBe("range");
    expect(timerProblem(60, 75)).toBe("short");
    expect(timerProblem(60, 60)).toBe("short");
    expect(timerProblem(90, 75)).toBeNull();
  });
});

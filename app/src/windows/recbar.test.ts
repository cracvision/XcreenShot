import { describe, expect, it } from "vitest";
import { longHint } from "./recClock";

describe("the recording bar (VID-023)", () => {
  it("suggests another clip once, for half a minute, after an hour", () => {
    expect(longHint(3599, true)).toBe(false);
    expect(longHint(3600, true)).toBe(true);
    expect(longHint(3629, true)).toBe(true);
    expect(longHint(3631, true)).toBe(false);
    expect(longHint(3610, false)).toBe(false);
  });
});

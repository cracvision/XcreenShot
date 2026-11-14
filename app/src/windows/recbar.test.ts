import { describe, expect, it } from "vitest";
import { longHint } from "./recClock";
import { meterFill, soundWarning } from "./recSound";

describe("the recording bar (VID-023)", () => {
  it("suggests another clip once, for half a minute, after an hour", () => {
    expect(longHint(3599, true)).toBe(false);
    expect(longHint(3600, true)).toBe(true);
    expect(longHint(3629, true)).toBe(true);
    expect(longHint(3631, true)).toBe(false);
    expect(longHint(3610, false)).toBe(false);
  });
});

describe("the sound of the recording bar (VID-019, VID-039, VID-043)", () => {
  const mic = {
    on: true,
    level: 0.2,
    clip: false,
    quietSeconds: 0,
    muted: false,
    device: "USB",
    wireless: false,
    missing: false,
    none: false,
  };
  const at = (patch: Partial<typeof mic>, status: "ready" | "recording" = "recording") => ({
    status,
    microphone: { ...mic, ...patch },
  });

  it("warns when the microphone is muted or nothing comes from it for 10 s", () => {
    expect(soundWarning(at({}), true)).toBeNull();
    expect(soundWarning(at({ muted: true }), true)).toBe("mic_muted");
    expect(soundWarning(at({ quietSeconds: 9.9 }), true)).toBeNull();
    expect(soundWarning(at({ quietSeconds: 10 }), true)).toBe("mic_quiet");
    // The preference off: no warning of those.
    expect(soundWarning(at({ muted: true, quietSeconds: 30 }), false)).toBeNull();
    // The microphone off: nothing to warn about.
    expect(soundWarning(at({ on: false, muted: true }), true)).toBeNull();
  });

  it("says when the chosen microphone is not there, before recording", () => {
    expect(soundWarning(at({ missing: true }, "ready"), true)).toBe("mic_missing");
    expect(soundWarning(at({ missing: true }), true)).toBeNull();
    expect(soundWarning(at({ none: true }), false)).toBe("mic_none");
  });

  it("draws the meter in decibels", () => {
    expect(meterFill(0)).toBe(0);
    expect(meterFill(1)).toBe(1);
    expect(meterFill(0.001)).toBeCloseTo(0);
    expect(meterFill(0.1)).toBeCloseTo(2 / 3);
  });
});

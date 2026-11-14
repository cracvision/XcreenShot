import { describe, expect, it } from "vitest";
import { longHint } from "./recClock";
import { barWarning, cameraWarning, meterFill, soundWarning, warningActions } from "./recSound";

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
    blocked: false,
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

  it("says first when the system's privacy blocks the microphone (VID-077)", () => {
    expect(soundWarning(at({ blocked: true, quietSeconds: 30 }, "ready"), false)).toBe(
      "mic_blocked",
    );
    expect(soundWarning(at({ blocked: true, on: false }), true)).toBeNull();
  });

  it("draws the meter in decibels", () => {
    expect(meterFill(0)).toBe(0);
    expect(meterFill(1)).toBe(1);
    expect(meterFill(0.001)).toBeCloseTo(0);
    expect(meterFill(0.1)).toBeCloseTo(2 / 3);
  });
});

describe("the camera's warnings (VID-038, VID-043)", () => {
  const cam = {
    on: true,
    enabled: true,
    available: true,
    open: true,
    device: "USB camera",
    missing: false,
    trouble: null,
    placement: "pip",
  } as const;
  const at = (patch: object, status: "ready" | "recording" = "ready") => ({
    status,
    camera: { ...cam, ...patch },
  });

  it("says why the camera gives no picture, with a way out", () => {
    expect(cameraWarning(at({}))).toBeNull();
    expect(cameraWarning(at({ trouble: "in_use" }))).toBe("camera_in_use");
    expect(warningActions("camera_in_use").map((a) => a.label)).toEqual([
      "retry",
      "without_camera",
    ]);
    expect(warningActions("camera_blocked")[0]?.params).toEqual({ permission: "camera" });
    // The camera off, or none at all: nothing to say.
    expect(cameraWarning(at({ on: false, trouble: "black" }))).toBeNull();
    expect(cameraWarning(at({ available: false, trouble: "gone" }))).toBeNull();
    // A missing camera is said before recording.
    expect(cameraWarning(at({ missing: true }))).toBe("camera_missing");
    expect(cameraWarning(at({ missing: true }, "recording"))).toBeNull();
  });

  it("puts the camera's warning before the microphone's", () => {
    const both = {
      ...at({ trouble: "black" }),
      microphone: {
        on: true,
        level: 0,
        clip: false,
        quietSeconds: 30,
        muted: true,
        device: null,
        wireless: false,
        missing: false,
        none: false,
      },
    };
    expect(barWarning(both, true)).toBe("camera_black");
  });
});

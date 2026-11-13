import { describe, expect, it } from "vitest";
import {
  clampSelection,
  formatTime,
  length,
  parseTime,
  skipCut,
  snap,
  toEdited,
  toSource,
  wholeSelected,
  type Span,
} from "./videoTimeline";

// The same cases as `xs_video::timeline` (ADR 0008).
describe("video timeline", () => {
  const cut: Span[] = [
    [0, 10],
    [20, 60],
  ];

  it("joins both sides of a cut (VID-059)", () => {
    expect(length(cut)).toBe(50);
    expect(toSource(cut, 9.5)).toBe(9.5);
    expect(toSource(cut, 10.5)).toBe(20.5);
    expect(toEdited(cut, 25)).toBe(15);
    expect(toEdited(cut, 15)).toBe(10);
  });

  it("jumps over cuts while playing", () => {
    expect(skipCut(cut, 5)).toBe(5);
    expect(skipCut(cut, 12)).toBe(20);
    expect(skipCut(cut, 60)).toBeNull();
  });

  it("writes and reads times (VID-056)", () => {
    expect(formatTime(75.45)).toBe("01:15.4");
    expect(formatTime(3725)).toBe("1:02:05.0");
    expect(formatTime(5, true)).toBe("0:00:05.0");
    expect(parseTime("01:15")).toBe(75);
    expect(parseTime("75")).toBe(75);
    expect(parseTime("1:02:03")).toBe(3723);
    expect(parseTime("1:15,5")).toBe(75.5);
    expect(parseTime("abc")).toBeNull();
    expect(parseTime("1:2:3:4")).toBeNull();
  });

  it("snaps a handle to the playhead within 8 px (VID-058)", () => {
    // 10 px per second: 0.5 s is 5 px.
    expect(snap(10.5, 10, 10)).toBe(10);
    expect(snap(11, 10, 10)).toBe(11);
  });

  it("keeps a selection of at least one frame inside the video", () => {
    expect(clampSelection(5, 5, 60, 1 / 30)).toEqual({ start: 5, end: 5 + 1 / 30 });
    expect(clampSelection(20, 10, 60, 0.1)).toEqual({ start: 10, end: 20 });
    expect(clampSelection(-3, 99, 60, 0.1)).toEqual({ start: 0, end: 60 });
    expect(wholeSelected(0, 60, 60)).toBe(true);
    expect(wholeSelected(0, 59, 60)).toBe(false);
  });
});

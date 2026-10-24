import { describe, expect, it } from "vitest";
import { display, fromEvent, releasedCombination } from "./accelerator";

const key = (
  code: string,
  mods: Partial<Record<"ctrl" | "alt" | "shift" | "meta", boolean>> = {},
) => ({
  code,
  ctrlKey: !!mods.ctrl,
  altKey: !!mods.alt,
  shiftKey: !!mods.shift,
  metaKey: !!mods.meta,
});

describe("fromEvent", () => {
  it("builds the canonical form in a fixed modifier order", () => {
    expect(fromEvent(key("KeyK", { shift: true, ctrl: true, alt: true }))).toBe(
      "Ctrl+Alt+Shift+KeyK",
    );
    expect(fromEvent(key("Digit5", { meta: true, shift: true }))).toBe("Shift+Super+Digit5");
  });

  it("waits while only modifiers are held", () => {
    expect(fromEvent(key("ShiftLeft", { shift: true }))).toBeNull();
  });
});

describe("releasedCombination", () => {
  it("keeps what was pressed", () => {
    expect(releasedCombination("Ctrl+KeyK", key("KeyK", { ctrl: true }))).toBe("Ctrl+KeyK");
  });

  it("takes PrtScn on release, where Windows reports it", () => {
    expect(releasedCombination(null, key("PrintScreen", { ctrl: true, shift: true }))).toBe(
      "Ctrl+Shift+PrintScreen",
    );
    expect(releasedCombination(null, key("PrintScreen"))).toBe("PrintScreen");
  });

  it("ignores releasing only modifiers", () => {
    expect(releasedCombination(null, key("ShiftLeft", { ctrl: true }))).toBeNull();
  });
});

describe("display", () => {
  it("uses platform conventions", () => {
    expect(display("Ctrl+Alt+Shift+KeyK", "windows")).toBe("Ctrl+Alt+Shift+K");
    expect(display("Ctrl+Alt+Shift+KeyK", "macos")).toBe("⌃⌥⇧K");
    expect(display("Ctrl+Shift+PrintScreen", "windows")).toBe("Ctrl+Shift+PrtScn");
  });

  it("shows the character of the active layout for the same physical key", () => {
    const spanish = new Map([["Semicolon", "ñ"]]);
    expect(display("Ctrl+Alt+Semicolon", "windows", spanish)).toBe("Ctrl+Alt+Ñ");
    expect(display("Ctrl+Alt+Semicolon", "windows")).toBe("Ctrl+Alt+;");
  });
});

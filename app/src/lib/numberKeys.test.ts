import type { KeyboardEvent } from "react";
import { describe, expect, it, vi } from "vitest";
import { tenStep } from "./numberKeys";

const press = (key: string, shiftKey: boolean, value: string, min = "", max = "") => {
  const input = Object.assign(document.createElement("input"), { type: "number", value, min, max });
  const e = { key, shiftKey, currentTarget: input, preventDefault: vi.fn() };
  return e as unknown as KeyboardEvent<HTMLInputElement>;
};

describe("number fields (APP-043)", () => {
  it("Shift+arrows step by 10 within min and max", () => {
    const apply = vi.fn();
    tenStep(press("ArrowUp", true, "10"), apply);
    expect(apply).toHaveBeenLastCalledWith(20);
    tenStep(press("ArrowDown", true, "5", "1", "60"), apply);
    expect(apply).toHaveBeenLastCalledWith(1);
    tenStep(press("ArrowUp", true, "55", "1", "60"), apply);
    expect(apply).toHaveBeenLastCalledWith(60);
  });

  it("leaves the plain arrows to the field", () => {
    const apply = vi.fn();
    tenStep(press("ArrowUp", false, "10"), apply);
    expect(apply).not.toHaveBeenCalled();
  });
});

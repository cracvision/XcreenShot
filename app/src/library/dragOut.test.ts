import { invoke } from "@tauri-apps/api/core";
import type { PointerEvent as ReactPointerEvent } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import "../i18n";
import { dismissNotice, useFiles } from "./actions";
import { draggingOut, pastThreshold, watchDrag } from "./dragOut";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("../lib/commands", () => ({ run: vi.fn(), isCommandError: () => false }));

const calls = vi.mocked(invoke);

/** The parts of a React pointer event the gesture reads. */
const press = (x: number, y: number) =>
  ({ button: 0, clientX: x, clientY: y, pointerId: 1 }) as unknown as ReactPointerEvent;

const pointer = (type: string, x: number, y: number) => {
  const e = new MouseEvent(type, { clientX: x, clientY: y, bubbles: true });
  Object.defineProperty(e, "pointerId", { value: 1 });
  window.dispatchEvent(e);
};

const settle = () => new Promise((r) => setTimeout(r, 0));

beforeEach(() => {
  calls.mockReset();
  dismissNotice();
});

afterEach(async () => {
  // Let the drag finish before the next test.
  await new Promise((r) => setTimeout(r, 550));
});

describe("drag out (TRY-015)", () => {
  it("starts only past a few pixels", () => {
    expect(pastThreshold(4, 0)).toBe(false);
    expect(pastThreshold(0, -4)).toBe(false);
    expect(pastThreshold(3, 3)).toBe(true);
  });

  it("prepares the files, then hands them to the system", async () => {
    calls.mockImplementation((cmd) =>
      Promise.resolve(
        cmd === "xs_drag_prepare" ? { files: ["C:\\a.png"], image: null } : "dropped",
      ),
    );
    const target = vi.fn(() => ({ docs: ["d1"], paths: [] }));
    watchDrag(press(10, 10), target);
    pointer("pointermove", 12, 11);
    expect(target).not.toHaveBeenCalled();
    pointer("pointermove", 20, 10);
    expect(target).toHaveBeenCalledTimes(1);
    expect(draggingOut()).toBe(true);
    await settle();
    await settle();
    expect(calls).toHaveBeenNthCalledWith(1, "xs_drag_prepare", { docs: ["d1"], paths: [] });
    expect(calls).toHaveBeenNthCalledWith(2, "xs_drag_start", {
      files: ["C:\\a.png"],
      image: null,
    });
  });

  it("a press that ends where it began is a click, not a drag", () => {
    const target = vi.fn(() => ({ docs: ["d1"], paths: [] }));
    watchDrag(press(10, 10), target);
    pointer("pointerup", 10, 10);
    pointer("pointermove", 40, 40);
    expect(target).not.toHaveBeenCalled();
    expect(calls).not.toHaveBeenCalled();
  });

  it("letting go before the files are ready offers Copy all instead", async () => {
    let ready: (v: unknown) => void = () => undefined;
    calls.mockImplementation(() => new Promise((r) => (ready = r)));
    watchDrag(press(0, 0), () => ({ docs: [], paths: ["C:\\lib\\a.xshot"] }));
    pointer("pointermove", 30, 0);
    pointer("pointerup", 30, 0);
    ready({ files: ["C:\\tmp\\a.png"], image: null });
    await settle();
    await settle();
    expect(calls).toHaveBeenCalledTimes(1);
    const notice = useFiles.getState().notice;
    expect(notice?.text).toMatch(/not ready/);
    expect(notice?.action?.label).toBe("Copy all");
  });
});

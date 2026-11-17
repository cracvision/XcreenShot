import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  closeQuickView,
  fitted,
  guideEntries,
  previewSide,
  showQuickView,
  stepQuickView,
  useQuickView,
  type QuickEntry,
} from "./quickViewState";

const entry = (id: string): QuickEntry => ({
  id,
  name: id,
  src: `big/${id}`,
  thumb: `small/${id}`,
  width: 100,
  height: 50,
  date: null,
});

describe("quick view (LIB-013)", () => {
  beforeEach(() => useQuickView.setState({ view: null }));

  it("never shows an image larger than its pixels, and fits a big one", () => {
    expect(fitted(400, 300, { width: 1000, height: 800 }, 1)).toEqual({ width: 400, height: 300 });
    expect(fitted(400, 300, { width: 1000, height: 800 }, 2)).toEqual({ width: 200, height: 150 });
    expect(fitted(4000, 1000, { width: 1000, height: 800 }, 1)).toEqual({
      width: 1000,
      height: 250,
    });
    expect(fitted(null, 300, { width: 1000, height: 800 }, 1)).toBeNull();
  });

  it("asks for previews in steps of 512 pixels", () => {
    expect(previewSide(1000, 700) % 512).toBe(0);
    expect(previewSide(1000, 700)).toBeGreaterThanOrEqual(1000);
    expect(previewSide(20000, 100)).toBe(8192);
  });

  it("walks the entries, stops at the ends and hands back the last one seen", () => {
    const done = vi.fn();
    const open = vi.fn();
    showQuickView({ entries: ["a", "b", "c"].map(entry), at: 1, open, done });
    stepQuickView(1);
    stepQuickView(1);
    expect(useQuickView.getState().view?.at).toBe(2);
    stepQuickView(-1);
    closeQuickView();
    expect(done).toHaveBeenCalledWith("b");
    expect(open).not.toHaveBeenCalled();
    expect(useQuickView.getState().view).toBeNull();

    showQuickView({ entries: ["a"].map(entry), at: 5, open, done });
    expect(useQuickView.getState().view?.at).toBe(0);
    closeQuickView(true);
    expect(open).toHaveBeenCalledWith("a");
  });
});

describe("a guide in the quick view (LIB-019)", () => {
  it("walks through its steps, one picture each", () => {
    const entries = guideEntries({
      id: "g1",
      name: "Pedido",
      path: "C:/lib/pedido.xshot",
      kind: "guide",
      format: "xshot",
      size: 1,
      modified: "2026-11-14T09:00:00Z",
      captured: "2026-11-14T09:00:00Z",
      width: 900,
      height: 3000,
      app: null,
      url: null,
      favorite: false,
      tags: [],
      steps: 3,
    });
    expect(entries.map((e) => e.id)).toEqual(["g1#1", "g1#2", "g1#3"]);
    expect(entries[1]?.src).toContain("library/g1/step/2");
  });
});

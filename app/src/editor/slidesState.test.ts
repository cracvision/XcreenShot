import { describe, expect, it } from "vitest";
import { place, slideKey } from "./slidesState";

describe("narrated slides (VID-072, VID-074)", () => {
  it("places a slide whole and centred, as the video does", () => {
    expect(place(1920, 1080, 800, 600, false)).toEqual({ x: 560, y: 240, w: 800, h: 600 });
    const big = place(1280, 720, 2560, 1600, false);
    expect([big.w, big.h, big.x, big.y]).toEqual([1152, 720, 64, 0]);
    expect(place(1920, 1080, 800, 600, true).h).toBe(1080);
    const near = place(1920, 1080, 800, 600, false, 2, [0.25, 0.25]);
    expect([near.x + 0.25 * near.w, near.y + 0.25 * near.h]).toEqual([960, 540]);
  });

  it("goes through the slides with the keys of a presentation remote", () => {
    const k = (key: string, ctrlKey = false) => slideKey({ key, ctrlKey, metaKey: false });
    expect(k("ArrowRight")).toBe("next");
    expect(k("PageDown")).toBe("next");
    expect(k(" ")).toBe("next");
    expect(k("PageUp")).toBe("prev");
    expect(k("]", true)).toBe("next");
    expect(k("[", true)).toBe("prev");
    expect(k("Home")).toBe("first");
    expect(k("End")).toBe("last");
    expect(k("0")).toBe("reset");
    expect(k("ArrowRight", true)).toBeNull();
    expect(k("a")).toBeNull();
  });
});

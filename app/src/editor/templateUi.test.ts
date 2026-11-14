import { describe, expect, it } from "vitest";
import type { ZoneObject } from "./state";
import {
  bands,
  dividerAt,
  fits,
  grouped,
  placement,
  snapOffset,
  thumbCount,
  zoneAt,
  type PickerTemplate,
  type Template,
} from "./templateUi";

const zone = (over: Partial<ZoneObject> = {}): ZoneObject => ({
  id: "o1",
  type: "zone",
  rect: { x: 100, y: 50, w: 400, h: 200 },
  image: "ins-a.png",
  fit: "fit",
  zoom: 1,
  offset: [0, 0],
  ...over,
});

const template = (over: Partial<Template["settings"]> = {}): Template => ({
  id: "steps-column",
  width: 1200,
  settings: {
    scale: "fit",
    title: true,
    captions: true,
    footer: false,
    author: "system",
    authorText: "",
    font: null,
    textColor: null,
    background: "#FFFFFF",
    numbering: true,
    numberColor: null,
    numberShape: "circle",
    reversed: false,
    theme: null,
    ...over,
  },
  sections: [
    { id: "s1", kind: "title", height: 110 },
    { id: "s2", kind: "step", height: 720 },
    { id: "s3", kind: "step", height: 720 },
    { id: "s4", kind: "footer", height: 70 },
  ],
});

const picker = (id: string, max: number, family = "steps"): PickerTemplate => ({
  id,
  name: id,
  family,
  min: 1,
  max,
  own: false,
});

describe("templates in the editor (EDT-095 to EDT-099)", () => {
  it("lays sections out without the ones turned off", () => {
    expect(bands(template()).map((b) => [b.section.id, b.top])).toEqual([
      ["s1", 0],
      ["s2", 110],
      ["s3", 830],
    ]);
    expect(bands(template({ title: false, footer: true })).map((b) => b.top)).toEqual([
      0, 720, 1440,
    ]);
  });

  it("finds the divider under the pointer, not after the last section", () => {
    const t = template();
    expect(dividerAt(t, 112, 4)?.id).toBe("s1");
    expect(dividerAt(t, 830, 4)?.id).toBe("s2");
    expect(dividerAt(t, 1550, 4)).toBeNull();
  });

  it("places pictures fitted or filling, zoomed and moved", () => {
    // 200×200 in 400×200: fitted at 1, centered.
    expect(placement(zone(), { w: 200, h: 200 })).toEqual({ x: 200, y: 50, k: 1 });
    // Filling: scale 2, centered vertically (crops).
    expect(placement(zone({ fit: "fill" }), { w: 200, h: 200 })).toEqual({ x: 100, y: -50, k: 2 });
    const z = placement(zone({ zoom: 1.2, offset: [10, -5] }), { w: 200, h: 200 });
    expect(z.k).toBeCloseTo(1.2);
    expect(z.x).toBeCloseTo(100 + (400 - 240) / 2 + 10);
  });

  it("snaps framing to the center and the edges", () => {
    const z = zone({ fit: "fill" });
    // Picture 400 px tall at scale 2 in a 200 px zone: edges at ±100.
    expect(snapOffset(z, { w: 200, h: 200 }, [3, 97], 6)).toEqual([0, 100]);
    expect(snapOffset(z, { w: 200, h: 200 }, [30, 40], 6)).toEqual([30, 40]);
  });

  it("finds the zone under a point", () => {
    const objects = [zone(), zone({ id: "o2", rect: { x: 0, y: 0, w: 50, h: 50 } })];
    expect(zoneAt(objects, [120, 60])?.id).toBe("o1");
    expect(zoneAt(objects, [10, 10])?.id).toBe("o2");
    expect(zoneAt(objects, [700, 700])).toBeNull();
  });

  it("filters the picker for N images and by name", () => {
    const list = [
      picker("steps-column", 20),
      picker("compare-side", 2, "compare"),
      picker("collage-grid", 9, "collage"),
      { ...picker("own:Plan", 3, "weird"), own: true },
    ];
    // Six images hide the two-zone templates.
    const six = grouped(list, "", 6, true).flatMap((g) => g.items.map((t) => t.id));
    expect(six).toEqual(["steps-column", "collage-grid"]);
    expect(fits(list[1] as PickerTemplate, 0)).toBe(true);
    // An unknown family goes to My templates; the search ignores accents and case.
    const all = grouped(list, "", 6, false);
    expect(all.at(-1)).toMatchObject({ family: "mine" });
    expect(grouped(list, "COMPÁRE", 0, false)[0]?.items[0]?.id).toBe("compare-side");
    expect(thumbCount(list[1] as PickerTemplate, 6)).toBe(2);
    expect(thumbCount(list[0] as PickerTemplate, 0)).toBe(3);
  });
});

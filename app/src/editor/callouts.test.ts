// The same case as `text_box_matches_the_editor` in crates/xs-image/src/callouts.rs.
import { describe, expect, it } from "vitest";
import { textBox } from "./callouts";
import type { CalloutObject } from "./state";

describe("callout text box (EDT-028)", () => {
  it("leaves the padding and the outline around the text", () => {
    const c = {
      id: "o1",
      type: "callout",
      rect: { x: 0, y: 0, w: 200, h: 60 },
      rotation: 0,
      text: "",
      tails: [],
      style: { shape: "roundedRectangle", padding: 10, outlineWidth: 3 },
    } as unknown as CalloutObject;
    expect(textBox(c)).toEqual({ x: 13, y: 13, w: 174, h: 34 });
    const cloud = { ...c, style: { ...c.style, shape: "cloud" } } as CalloutObject;
    expect(textBox(cloud)).toEqual({ x: 37, y: 20.2, w: 126, h: 19.6 });
  });
});

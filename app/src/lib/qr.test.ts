import { describe, expect, it } from "vitest";
import { codesIn, splitUrl, type QrCode } from "./qr";

describe("QR codes on screen (SMT-038)", () => {
  const code = (x: number, y: number): QrCode => ({
    text: "t",
    rect: [x, y, 100, 100],
    kind: "text",
  });

  it("keeps the codes whose middle is in the selection", () => {
    const codes = [code(0, 0), code(500, 500), code(-2000, 300)];
    expect(codesIn(codes, { x: 0, y: 0, width: 400, height: 400 })).toEqual([codes[0]]);
    // Half a code is enough when its middle is in.
    expect(codesIn(codes, { x: 540, y: 540, width: 200, height: 200 })).toEqual([codes[1]]);
    // Another monitor, left of the main one.
    expect(codesIn(codes, { x: -2100, y: 0, width: 400, height: 800 })).toEqual([codes[2]]);
    expect(codesIn(codes, { x: 200, y: 200, width: 100, height: 100 })).toEqual([]);
  });

  it("splits the domain out of an address", () => {
    expect(splitUrl("https://example.com/a?b=1")).toEqual(["https://", "example.com", "/a?b=1"]);
  });
});

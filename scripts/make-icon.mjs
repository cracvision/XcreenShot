// Draws XcreenShot's icon, the logo simplified (Carlos, 2026-11-05): the X in the logo's
// sweep (cyan, blue, violet, magenta) inside the blue-to-violet selection frame, and
// writes:
//   app/src-tauri/icons/app-icon.png   1024 px on a navy tile, input for `tauri icon`
//   app/src-tauri/icons/tray.png       64 px glyph for the tray / menu bar icon
// Usage: node scripts/make-icon.mjs
import { writeFileSync, mkdirSync } from "node:fs";
import { deflateSync } from "node:zlib";
import { fileURLToPath } from "node:url";

const crcTable = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
const crc32 = (buf) => {
  let c = 0xffffffff;
  for (const b of buf) c = crcTable[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
};
function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const td = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(td));
  return Buffer.concat([len, td, crc]);
}
function png(size, rgba) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // RGBA
  const raw = Buffer.alloc((size * 4 + 1) * size);
  for (let y = 0; y < size; y++) {
    raw[y * (size * 4 + 1)] = 0;
    rgba.copy(raw, y * (size * 4 + 1) + 1, y * size * 4, (y + 1) * size * 4);
  }
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

const hex = (h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16));
/** Color at `t` (0..1) along evenly spaced stops. */
function gradient(stops, t) {
  const s = stops.map(hex);
  const x = Math.max(0, Math.min(1, t)) * (s.length - 1);
  const i = Math.min(Math.floor(x), s.length - 2);
  const f = x - i;
  return s[i].map((c, k) => c + (s[i + 1][k] - c) * f);
}

// Geometry in unit coordinates (0..1).
const segDist = (px, py, [ax, ay, bx, by]) => {
  const dx = bx - ax;
  const dy = by - ay;
  const t = Math.max(
    0,
    Math.min(1, ((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)),
  );
  return Math.hypot(px - (ax + t * dx), py - (ay + t * dy));
};
function brackets(a, arm) {
  const b = 1 - a;
  return [
    [a, a, a + arm, a],
    [a, a, a, a + arm],
    [b, a, b - arm, a],
    [b, a, b, a + arm],
    [a, b, a + arm, b],
    [a, b, a, b - arm],
    [b, b, b - arm, b],
    [b, b, b, b - arm],
  ];
}
function roundRect(px, py, cx, cy, half, r) {
  const qx = Math.max(Math.abs(px - cx) - (half - r), 0);
  const qy = Math.max(Math.abs(py - cy) - (half - r), 0);
  return Math.hypot(qx, qy) <= r;
}

const FRAME = ["#0c9afc", "#2f6bff", "#7f19fc"];
const SWEEP = ["#14b3fc", "#0064fb", "#6b1cf2", "#fa29ae"];

/**
 * Paints each sample in order: tile, frame, corner handles, X. Without `tile` the
 * background stays transparent (tray glyph).
 */
function render(size, o) {
  const out = Buffer.alloc(size * size * 4);
  const frame = brackets(o.inset, o.arm);
  const corners = [
    [o.inset, o.inset],
    [1 - o.inset, o.inset],
    [o.inset, 1 - o.inset],
    [1 - o.inset, 1 - o.inset],
  ];
  const x0 = 0.5 - o.x;
  const x1 = 0.5 + o.x;
  const strokes = [
    [x0, x0, x1, x1],
    [x1, x0, x0, x1],
  ];
  const S = 4;
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      let r = 0;
      let g = 0;
      let b = 0;
      let a = 0;
      for (let sy = 0; sy < S; sy++) {
        for (let sx = 0; sx < S; sx++) {
          const u = (x + (sx + 0.5) / S) / size;
          const v = (y + (sy + 0.5) / S) / size;
          let c = null;
          if (o.tile && roundRect(u, v, 0.5, 0.5, 0.5 - o.margin, 0.2)) {
            // Navy of the wordmark, a little violet at the top, with a soft gloss.
            c = gradient(
              ["#2a2470", "#14123a"],
              (v - o.margin) / (1 - 2 * o.margin),
            );
            const gloss = Math.max(0, 0.5 - v) * 0.16;
            c = c.map((k) => k + (255 - k) * gloss);
          }
          if (frame.some((s) => segDist(u, v, s) <= o.stroke / 2))
            c = gradient(FRAME, (u + v) / 2);
          if (o.handle) {
            for (const [cx, cy] of corners) {
              if (roundRect(u, v, cx, cy, o.handle, o.handle * 0.3)) {
                const inner = roundRect(
                  u,
                  v,
                  cx,
                  cy,
                  o.handle * 0.55,
                  o.handle * 0.15,
                );
                c = inner ? [255, 255, 255] : gradient(FRAME, (cx + cy) / 2);
              }
            }
          }
          if (strokes.some((s) => segDist(u, v, s) <= o.xStroke / 2)) {
            c = gradient(SWEEP, (u + v - 2 * x0) / (2 * (x1 - x0)));
          }
          if (c) {
            r += c[0];
            g += c[1];
            b += c[2];
            a += 1;
          }
        }
      }
      const i = (y * size + x) * 4;
      out[i] = a ? Math.round(r / a) : 0;
      out[i + 1] = a ? Math.round(g / a) : 0;
      out[i + 2] = a ? Math.round(b / a) : 0;
      out[i + 3] = Math.round((a / (S * S)) * 255);
    }
  }
  return out;
}

const dir = fileURLToPath(new URL("../app/src-tauri/icons/", import.meta.url));
mkdirSync(dir, { recursive: true });
writeFileSync(
  dir + "app-icon.png",
  png(
    1024,
    render(1024, {
      tile: true,
      margin: 0.06,
      inset: 0.22,
      arm: 0.17,
      stroke: 0.058,
      handle: 0.05,
      x: 0.15,
      xStroke: 0.115,
    }),
  ),
);
// The tray keeps the old proportions: when paused, the shell clears the X (the middle
// 60 %) and draws a slash there.
writeFileSync(
  dir + "tray.png",
  png(
    64,
    render(64, {
      tile: false,
      inset: 0.1,
      arm: 0.24,
      stroke: 0.11,
      handle: 0,
      x: 0.18,
      xStroke: 0.17,
    }),
  ),
);
console.log("icons written to", dir);

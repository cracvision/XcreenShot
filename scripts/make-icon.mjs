// Draws XcreenShot's own icon (viewfinder brackets around an X) and writes:
//   app/src-tauri/icons/app-icon.png   1024 px, input for `tauri icon`
//   app/src-tauri/icons/tray.png       64 px glyph for the tray / menu bar icon
// Usage: node scripts/make-icon.mjs
import { writeFileSync, mkdirSync } from "node:fs";
import { deflateSync } from "node:zlib";
import { fileURLToPath } from "node:url";

const ACCENT = [14, 140, 127]; // #0E8C7F
const WHITE = [255, 255, 255];

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

// Geometry in unit coordinates (0..1).
const segDist = (px, py, [ax, ay, bx, by]) => {
  const dx = bx - ax;
  const dy = by - ay;
  const t = Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)));
  return Math.hypot(px - (ax + t * dx), py - (ay + t * dy));
};
function glyphSegments(inset, arm) {
  const a = inset;
  const b = 1 - inset;
  const segs = [
    [a, a, a + arm, a], [a, a, a, a + arm],
    [b, a, b - arm, a], [b, a, b, a + arm],
    [a, b, a + arm, b], [a, b, a, b - arm],
    [b, b, b - arm, b], [b, b, b, b - arm],
  ];
  const c = 0.5;
  const x = 0.13;
  segs.push([c - x, c - x, c + x, c + x], [c + x, c - x, c - x, c + x]);
  return segs;
}
function insideRoundRect(px, py, r) {
  const qx = Math.max(Math.abs(px - 0.5) - (0.5 - r), 0);
  const qy = Math.max(Math.abs(py - 0.5) - (0.5 - r), 0);
  return Math.hypot(qx, qy) <= r;
}

function render(size, { background, glyphColor, stroke, inset, arm, margin }) {
  const out = Buffer.alloc(size * size * 4);
  const segs = glyphSegments(inset, arm);
  const S = 4;
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      let bg = 0;
      let fg = 0;
      for (let sy = 0; sy < S; sy++) {
        for (let sx = 0; sx < S; sx++) {
          const u = (x + (sx + 0.5) / S) / size;
          const v = (y + (sy + 0.5) / S) / size;
          const mu = (u - margin) / (1 - 2 * margin);
          const mv = (v - margin) / (1 - 2 * margin);
          if (background && mu >= 0 && mu <= 1 && mv >= 0 && mv <= 1 && insideRoundRect(mu, mv, 0.22)) bg++;
          if (segs.some((s) => segDist(u, v, s) <= stroke / 2)) fg++;
        }
      }
      const n = S * S;
      const i = (y * size + x) * 4;
      const fa = fg / n;
      const ba = background ? bg / n : 0;
      const a = fa + ba * (1 - fa);
      const mix = (k) =>
        a === 0 ? 0 : Math.round((glyphColor[k] * fa + (background ?? [0, 0, 0])[k] * ba * (1 - fa)) / a);
      out[i] = mix(0);
      out[i + 1] = mix(1);
      out[i + 2] = mix(2);
      out[i + 3] = Math.round(a * 255);
    }
  }
  return out;
}

const dir = fileURLToPath(new URL("../app/src-tauri/icons/", import.meta.url));
mkdirSync(dir, { recursive: true });
writeFileSync(
  dir + "app-icon.png",
  png(1024, render(1024, { background: ACCENT, glyphColor: WHITE, stroke: 0.06, inset: 0.27, arm: 0.15, margin: 0.06 })),
);
writeFileSync(
  dir + "tray.png",
  png(64, render(64, { background: null, glyphColor: ACCENT, stroke: 0.11, inset: 0.1, arm: 0.24, margin: 0 })),
);
console.log("icons written to", dir);

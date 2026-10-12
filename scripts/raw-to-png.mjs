// Converts `<name>-<W>x<H>.rgba` dumps into PNGs scaled ×4 on a dark background,
// for reviewing tray icon states. Usage: node scripts/raw-to-png.mjs <dir>
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { deflateSync } from "node:zlib";
import { join } from "node:path";

const crcTable = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
const crc32 = (b) => {
  let c = 0xffffffff;
  for (const x of b) c = crcTable[(c ^ x) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
};
const chunk = (t, d) => {
  const l = Buffer.alloc(4);
  l.writeUInt32BE(d.length);
  const td = Buffer.concat([Buffer.from(t), d]);
  const c = Buffer.alloc(4);
  c.writeUInt32BE(crc32(td));
  return Buffer.concat([l, td, c]);
};

const dir = process.argv[2];
for (const f of readdirSync(dir).filter((f) => f.endsWith(".rgba"))) {
  const [w, h] = f
    .match(/(\d+)x(\d+)\.rgba$/)
    .slice(1)
    .map(Number);
  const src = readFileSync(join(dir, f));
  const S = 4;
  const W = w * S;
  const H = h * S;
  const raw = Buffer.alloc((W * 3 + 1) * H);
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      const i = (Math.floor(y / S) * w + Math.floor(x / S)) * 4;
      const a = src[i + 3] / 255;
      const o = y * (W * 3 + 1) + 1 + x * 3;
      for (let k = 0; k < 3; k++)
        raw[o + k] = Math.round(src[i + k] * a + 32 * (1 - a));
    }
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(W, 0);
  ihdr.writeUInt32BE(H, 4);
  ihdr[8] = 8;
  ihdr[9] = 2;
  const png = Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(raw)),
    chunk("IEND", Buffer.alloc(0)),
  ]);
  writeFileSync(join(dir, f.replace(".rgba", ".png")), png);
}

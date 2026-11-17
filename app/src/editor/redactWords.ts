// The words read on the canvas, for redacting by hand (SMT-014, SMT-015): which words a
// drag or a Shift+click covers, one rectangle per line, and the text under a redaction.
import type { WordLine } from "./redactState";

export type Rect = [number, number, number, number];

/** A word: its line and its place in the line. */
export interface At {
  line: number;
  word: number;
}

export function union(rects: Rect[]): Rect {
  const x0 = Math.min(...rects.map((r) => r[0]));
  const y0 = Math.min(...rects.map((r) => r[1]));
  const x1 = Math.max(...rects.map((r) => r[0] + r[2]));
  const y1 = Math.max(...rects.map((r) => r[1] + r[3]));
  return [x0, y0, x1 - x0, y1 - y0];
}

/** The centre of `r` is inside one of `boxes`. */
export function covered(r: Rect, boxes: Rect[]): boolean {
  const cx = r[0] + r[2] / 2;
  const cy = r[1] + r[3] / 2;
  return boxes.some((b) => cx >= b[0] && cx <= b[0] + b[2] && cy >= b[1] && cy <= b[1] + b[3]);
}

function ordered(a: At, b: At): [At, At] {
  return a.line < b.line || (a.line === b.line && a.word <= b.word) ? [a, b] : [b, a];
}

/** `at` is between `a` and `b` (either order), in reading order. */
export function between(at: At, a: At, b: At): boolean {
  const [from, to] = ordered(a, b);
  const after = at.line > from.line || (at.line === from.line && at.word >= from.word);
  const before = at.line < to.line || (at.line === to.line && at.word <= to.word);
  return after && before;
}

/** The words from `a` to `b` in reading order, one rectangle per line. */
export function spanRects(lines: WordLine[], a: At, b: At): Rect[] {
  const [from, to] = ordered(a, b);
  const out: Rect[] = [];
  for (let l = from.line; l <= to.line; l++) {
    const words = lines[l]?.words ?? [];
    const first = l === from.line ? from.word : 0;
    const last = l === to.line ? to.word : words.length - 1;
    const picked = words.slice(first, last + 1).map((w) => w.rect);
    if (picked.length > 0) out.push(union(picked));
  }
  return out;
}

/** The whole line of a word. */
export function lineRect(lines: WordLine[], line: number): Rect | null {
  const words = lines[line]?.words ?? [];
  return words.length > 0 ? union(words.map((w) => w.rect)) : null;
}

/** The words under a redaction, in reading order. */
export function textUnder(lines: WordLine[], r: Rect): string {
  return lines
    .flatMap((l) => l.words)
    .filter((w) => covered(w.rect, [r]))
    .map((w) => w.text)
    .join(" ");
}

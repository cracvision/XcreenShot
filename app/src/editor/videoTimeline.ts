// A video's timeline in the editor (VID-056 to VID-059, ADR 0008): the copy of
// `xs_video::timeline` that the player needs every frame. The core sends the parts of
// the file that are kept after the cuts; times on the timeline are of the cut video.

/** A stretch of the file, in seconds: [start, end]. */
export type Span = [number, number];

const EPS = 1e-6;

/** How long the video is once cut. */
export function length(kept: Span[]): number {
  return kept.reduce((sum, [a, b]) => sum + Math.max(0, b - a), 0);
}

/** A time of the cut video, in the file. */
export function toSource(kept: Span[], t: number): number {
  let left = Math.max(0, t);
  for (const [a, b] of kept) {
    if (left <= b - a + EPS) return Math.min(a + left, b);
    left -= b - a;
  }
  return kept.length > 0 ? kept[kept.length - 1][1] : 0;
}

/** A time of the file, in the cut video; a time that was cut goes to the cut. */
export function toEdited(kept: Span[], t: number): number {
  let before = 0;
  for (const [a, b] of kept) {
    if (t < a) return before;
    if (t <= b) return before + (t - a);
    before += b - a;
  }
  return before;
}

/** While playing, a time of the file inside a cut jumps to the next kept part; past
 * the end, `null`. */
export function skipCut(kept: Span[], t: number): number | null {
  for (const [a, b] of kept) {
    if (t < a - EPS) return a;
    if (t < b - EPS) return t;
  }
  return null;
}

/** `m:ss.f`, or `h:mm:ss.f` from an hour (VID-056). */
export function formatTime(seconds: number, long = false): string {
  const tenths = Math.max(0, Math.floor(seconds * 10 + EPS));
  const f = tenths % 10;
  const s = Math.floor(tenths / 10) % 60;
  const m = Math.floor(tenths / 600) % 60;
  const h = Math.floor(tenths / 36000);
  const ss = String(s).padStart(2, "0");
  const mm = String(m).padStart(2, "0");
  return long || h > 0 ? `${h}:${mm}:${ss}.${f}` : `${mm}:${ss}.${f}`;
}

/** Go to… (VID-056): "75", "1:15", "01:15.5" or "1:02:03" as seconds; `null` if not a
 * time. */
export function parseTime(text: string): number | null {
  const parts = text.trim().replace(",", ".").split(":");
  if (parts.length === 0 || parts.length > 3 || parts.some((p) => !/^\d+(\.\d+)?$/.test(p)))
    return null;
  return parts.reduce((total, p) => total * 60 + Number(p), 0);
}

/** A selection handle near the playhead sticks to it (VID-058): within `px` pixels. */
export function snap(time: number, playhead: number, pxPerSecond: number, px = 8): number {
  return Math.abs(time - playhead) * pxPerSecond <= px ? playhead : time;
}

/** The selection kept inside the video and at least one frame long (VID-058). */
export function clampSelection(
  start: number,
  end: number,
  total: number,
  frame: number,
): { start: number; end: number } {
  let s = Math.min(Math.max(0, start), total);
  let e = Math.min(Math.max(0, end), total);
  if (e < s) [s, e] = [e, s];
  if (e - s < frame) {
    if (s + frame <= total) e = s + frame;
    else s = Math.max(0, e - frame);
  }
  return { start: s, end: e };
}

/** The whole video selected: Cut is off then (VID-059). */
export function wholeSelected(start: number, end: number, total: number): boolean {
  return start <= EPS && end >= total - EPS;
}

// The part of the canvas the floating bars leave visible (Cristal, EDT-001), and the thin
// scroll bars drawn over it.

/** How much of the canvas area each floating bar covers, in CSS px. */
export interface Insets {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

/** The gap between a floating bar and the edge of the workspace. */
export const EDGE = 12;

/** The insets from the bars' measured sizes: each covers its size plus its gap to the edge;
 * a bar that is not there covers nothing. */
export function insetsFrom(top: number, side: number, bottom: number): Insets {
  return {
    top: top > 0 ? top + EDGE : 0,
    right: side > 0 ? side + EDGE : 0,
    bottom: bottom > 0 ? bottom + EDGE : 0,
    left: 0,
  };
}

/** A scroll bar's thumb: where it starts and how long it is along a track of `track` px,
 * for `content` px of content seen through `visible` px scrolled by `scroll`. Null when
 * everything fits. */
export function thumbOf(
  content: number,
  visible: number,
  scroll: number,
  track: number,
): { start: number; length: number } | null {
  if (content <= visible + 0.5 || track <= 0) return null;
  const length = Math.max(Math.min(32, track), (visible / content) * track);
  const range = content - visible;
  const start = (Math.min(Math.max(scroll, 0), range) / range) * (track - length);
  return { start, length };
}

/** How far to scroll for a thumb dragged `delta` px along its track. */
export function scrollForDrag(
  content: number,
  visible: number,
  track: number,
  thumbLength: number,
  delta: number,
): number {
  const free = track - thumbLength;
  return free > 0 ? (delta * (content - visible)) / free : 0;
}

// How compact a bar must be to fit its width (EDT-001, LIB-002): level 0 shows
// everything; each level up hides or folds one more thing. The bar never lets its items
// run over each other: it climbs while its content is wider than it is, and comes down
// when the level below is known (or tried) to fit.
import { useLayoutEffect, useRef, useState, type RefObject } from "react";

/** The next level, from the current one, how wide its content is, how wide the bar is,
 * and the widths already measured at other levels. */
export function nextLevel(
  level: number,
  needed: number,
  width: number,
  max: number,
  known: Map<number, number>,
): number {
  if (needed > width + 0.5) return Math.min(max, level + 1);
  if (level === 0) return 0;
  const below = known.get(level - 1);
  // Not measured yet: try it (if it does not fit, the bar comes back up and remembers).
  if (below === undefined || below <= width + 0.5) return level - 1;
  return level;
}

/** How wide the bar's content is, without the room its flexible gaps take. */
export function contentWidth(bar: HTMLElement): number {
  let slack = 0;
  for (const el of bar.querySelectorAll<HTMLElement>(".tb-spacer, .tb-space")) {
    const min = parseFloat(getComputedStyle(el).minWidth) || 0;
    slack += Math.max(0, el.offsetWidth - min);
  }
  return bar.scrollWidth - slack;
}

/** The level `ref`'s bar needs now. `key` names what is on it: when it changes, the
 * widths measured before no longer count. */
export function useFit(ref: RefObject<HTMLElement>, max: number, key: string): number {
  const [level, setLevel] = useState(0);
  const [tick, setTick] = useState(0);
  const known = useRef(new Map<number, number>());
  const lastKey = useRef(key);

  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const observer = new ResizeObserver(() => setTick((t) => t + 1));
    observer.observe(el);
    return () => observer.disconnect();
  }, [ref]);

  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (lastKey.current !== key) {
      lastKey.current = key;
      known.current.clear();
    }
    const needed = contentWidth(el);
    known.current.set(level, needed);
    const next = nextLevel(level, needed, el.clientWidth, max, known.current);
    if (next !== level) setLevel(next);
  }, [ref, level, max, key, tick]);

  return Math.min(level, max);
}

/** The `fit-N` classes of a level: each hides one more kind of label. */
export function fitClasses(level: number): string {
  return [1, 2, 3]
    .filter((n) => level >= n)
    .map((n) => ` fit-${n}`)
    .join("");
}

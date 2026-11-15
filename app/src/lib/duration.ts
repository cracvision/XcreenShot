// A recording timer's length (VID-015): hours, minutes and seconds, from 5 s to 12 h.

/** The timer's range, in seconds (VID-015). */
export const TIMER_MIN = 5;
export const TIMER_MAX = 12 * 3600;

/** Hours, minutes and seconds of a number of seconds. */
export function splitSeconds(total: number): { h: number; m: number; s: number } {
  const t = Math.max(0, Math.round(total));
  return { h: Math.floor(t / 3600), m: Math.floor((t % 3600) / 60), s: t % 60 };
}

/** Seconds of hours, minutes and seconds; what is not a number counts as 0. */
export function joinSeconds(h: number, m: number, s: number): number {
  const n = (v: number) => (Number.isFinite(v) ? Math.max(0, Math.floor(v)) : 0);
  return n(h) * 3600 + n(m) * 60 + n(s);
}

/** A short length: 30 s, 5 min, 1 h, 1 h 30 min, or 1:02:05. */
export function shortDuration(seconds: number): string {
  const { h, m, s } = splitSeconds(seconds);
  if (s && (h || m)) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  return (
    [h ? `${h} h` : "", m ? `${m} min` : "", s ? `${s} s` : ""].filter(Boolean).join(" ") || "0 s"
  );
}

/** Why a timer of `seconds` cannot be set: outside 5 s to 12 h, or not more than what
 * is `recorded` already. */
export function timerProblem(seconds: number, recorded = 0): "range" | "short" | null {
  if (seconds < TIMER_MIN || seconds > TIMER_MAX) return "range";
  if (recorded > 0 && seconds <= recorded) return "short";
  return null;
}

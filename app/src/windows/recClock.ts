/** `hh:mm:ss` of a number of seconds (the recording bar's time, VID-011). */
export function clock(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  const two = (n: number) => String(n).padStart(2, "0");
  return `${two(Math.floor(s / 3600))}:${two(Math.floor((s % 3600) / 60))}:${two(s % 60)}`;
}

/** An hour recorded: a suggestion, shown once for half a minute (VID-023). */
export function longHint(elapsed: number, recording: boolean): boolean {
  return recording && elapsed >= 3600 && elapsed < 3630;
}

/** The timer's quick choices (VID-015), in seconds. */
export const TIMER_CHOICES = [30, 60, 300, 900, 1800, 3600];

/** What is left before the timer stops the recording, or none without a timer. */
export function timeLeft(limit: number | null | undefined, elapsed: number): number | null {
  return limit ? Math.max(0, limit - elapsed) : null;
}

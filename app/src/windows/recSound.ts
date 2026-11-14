// The sound of the recording bar (VID-011, VID-019, VID-039, VID-040, VID-043): its
// meters, asked for a few times a second, and the warnings they lead to.
import { useEffect, useState } from "react";
import { run } from "../lib/commands";

export interface MicLevel {
  on: boolean;
  /** The loudest sample lately, 0 to 1 (after the automatic level). */
  level: number;
  clip: boolean;
  /** How long nothing has come from it: a muted headset (VID-019). */
  quietSeconds: number;
  /** Muted in the system, or its volume at zero (VID-019). */
  muted: boolean;
  device: string | null;
  wireless: boolean;
  /** The chosen one is not plugged in: the default is used (VID-043). */
  missing: boolean;
  /** No input at all. */
  none: boolean;
}

export interface SoundLevels {
  status: "ready" | "recording" | "paused" | "idle";
  microphone?: MicLevel;
  systemAudio?: { on: boolean; level: number; clip: boolean };
  /** The video has a sound track: the sounds can be switched while it records. */
  track?: boolean;
}

/** The meters while `active`, about eight times a second. */
export function useLevels(active: boolean): SoundLevels | null {
  const [levels, setLevels] = useState<SoundLevels | null>(null);
  useEffect(() => {
    if (!active) return;
    let alive = true;
    const ask = () =>
      void run<SoundLevels>("record.level")
        .then((l) => {
          if (alive) setLevels(l);
        })
        .catch(() => undefined);
    ask();
    const timer = window.setInterval(ask, 120);
    return () => {
      alive = false;
      window.clearInterval(timer);
    };
  }, [active]);
  return levels;
}

/** How full a meter is (0 to 1) for a level: in decibels, from -60 dB to the top, as
 * the ear hears it. */
export function meterFill(level: number): number {
  if (level <= 0) return 0;
  return Math.min(1, Math.max(0, (20 * Math.log10(level) + 60) / 60));
}

export type SoundWarning = "mic_none" | "mic_muted" | "mic_missing" | "mic_quiet";

/** The warning to show now, if any. Muted and silent follow the preference "Warn when
 * the microphone is muted" (VID-019); a missing device is said in Ready (VID-043). */
export function soundWarning(l: SoundLevels | null, alerts: boolean): SoundWarning | null {
  const m = l?.microphone;
  if (!l || !m?.on) return null;
  if (m.none) return "mic_none";
  if (alerts && m.muted) return "mic_muted";
  if (m.missing && l.status === "ready") return "mic_missing";
  if (alerts && m.quietSeconds >= 10) return "mic_quiet";
  return null;
}

/** A warning closed stays away for 2 minutes (VID-019). */
export const DISMISS_MS = 120_000;

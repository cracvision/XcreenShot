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
  /** The system's privacy settings do not let XcreenShot use it (VID-077). */
  blocked?: boolean;
  /** Someone talks while it is off (VID-019). */
  talking?: boolean;
}

/** The camera now (VID-031, VID-038). */
export interface CameraState {
  /** Switched on and allowed in videos. */
  on: boolean;
  /** Use the camera in videos at all: off, its switch goes away. */
  enabled: boolean;
  /** Some camera is plugged in. */
  available: boolean;
  open: boolean;
  device: string | null;
  /** The chosen one is not plugged in: another is used (VID-043). */
  missing: boolean;
  /** Why it gives no picture. */
  trouble: "blocked" | "in_use" | "gone" | "black" | "failed" | null;
  placement: string;
}

export interface SoundLevels {
  status: "ready" | "recording" | "paused" | "idle";
  camera?: CameraState;
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

export type SoundWarning =
  "mic_blocked" | "mic_none" | "mic_muted" | "mic_missing" | "mic_quiet" | "mic_talking";

/** The warning to show now, if any. Blocked by the system's privacy comes first
 * (VID-077); muted and silent follow the preference "Warn when the microphone is
 * muted" (VID-019); a missing device is said in Ready (VID-043). */
export function soundWarning(l: SoundLevels | null, alerts: boolean): SoundWarning | null {
  const m = l?.microphone;
  // Off, and someone talks: it may have been forgotten (VID-019).
  if (l && m && !m.on) return alerts && m.talking ? "mic_talking" : null;
  if (!l || !m?.on) return null;
  if (m.blocked) return "mic_blocked";
  if (m.none) return "mic_none";
  if (alerts && m.muted) return "mic_muted";
  if (m.missing && l.status === "ready") return "mic_missing";
  if (alerts && m.quietSeconds >= 10) return "mic_quiet";
  return null;
}

export type CameraWarning =
  | "camera_blocked"
  | "camera_in_use"
  | "camera_gone"
  | "camera_black"
  | "camera_failed"
  | "camera_missing";

export type BarWarning = SoundWarning | CameraWarning;

/** The camera's warning now, if any (VID-038, VID-043). */
export function cameraWarning(l: SoundLevels | null): CameraWarning | null {
  const c = l?.camera;
  if (!l || !c?.on || !c.enabled || !c.available) return null;
  if (c.trouble) return `camera_${c.trouble}`;
  if (c.missing && l.status === "ready") return "camera_missing";
  return null;
}

/** The warning under the bar: the camera's first, then the sound's. */
export function barWarning(l: SoundLevels | null, alerts: boolean): BarWarning | null {
  return cameraWarning(l) ?? soundWarning(l, alerts);
}

/** A way out of a warning: a button that runs a command. */
export interface StripAction {
  label: string;
  command: string;
  params?: Record<string, unknown>;
}

const RETRY: StripAction = { label: "retry", command: "record.camera", params: { retry: true } };
const WITHOUT_CAMERA: StripAction = {
  label: "without_camera",
  command: "record.camera",
  params: { on: false },
};

/** The buttons of each warning (VID-019, VID-038, VID-077). */
export function warningActions(w: BarWarning): StripAction[] {
  switch (w) {
    case "mic_blocked":
    case "camera_blocked":
      return [
        {
          label: "fix_permissions",
          command: "app.open_permission_settings",
          params: { permission: w === "mic_blocked" ? "microphone" : "camera" },
        },
      ];
    case "camera_in_use":
    case "camera_gone":
    case "camera_black":
    case "camera_failed":
      return [RETRY, WITHOUT_CAMERA];
    case "mic_missing":
    case "camera_missing":
      return [];
    case "mic_talking":
      return [{ label: "mic_on", command: "record.sound", params: { microphone: true } }];
    default:
      return [{ label: "open_sound_settings", command: "record.sound_settings" }];
  }
}

/** A warning closed stays away for 2 minutes (VID-019). */
export const DISMISS_MS = 120_000;

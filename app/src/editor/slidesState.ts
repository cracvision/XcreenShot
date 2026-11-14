// Narrated slides in the editor (VID-071 to VID-074): the mode's state as the core
// sends it, the recording's state, and the slide's place in the video (the same rule
// as the engine's, crates/xs-video/src/slides.rs).
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { run } from "../lib/commands";

export interface SlideInfo {
  documentId: string;
  name: string;
  width: number;
  height: number;
  zoom: number;
  center: [number, number];
}

export interface SlidesMode {
  active: boolean;
  index: number;
  count: number;
  size: [number, number];
  background: string;
  enlarge: boolean;
  slides: SlideInfo[];
}

export interface RecState {
  status: "idle" | "countdown" | "recording" | "paused" | "finalizing";
  elapsed: number;
}

interface Store {
  mode: SlidesMode | null;
  rec: RecState;
  /** When the recording's state arrived, to move the time on between updates. */
  since: number;
}

export const useSlides = create<Store>(() => ({
  mode: null,
  rec: { status: "idle", elapsed: 0 },
  since: 0,
}));

const take = (m: Partial<SlidesMode> & { active: boolean }) =>
  useSlides.setState({ mode: m.active ? (m as SlidesMode) : null });

/** Follows the core: the mode's changes, and the recording while the mode is open. */
export async function followSlides(): Promise<void> {
  await listen<{ kind: string; payload: SlidesMode }>("xs://editor", (e) => {
    if (e.payload.kind === "slides") take(e.payload.payload);
  });
  await listen<RecState>("xs://slides-record", (e) =>
    useSlides.setState({ rec: e.payload, since: Date.now() }),
  );
  void run<SlidesMode>("slides.state")
    .then(take)
    .catch(() => undefined);
}

/** Asks the core for the recording's state now (it also comes as events). */
export function pollRecording(): void {
  void run<RecState>("record.status")
    .then((s) =>
      useSlides.setState({
        rec: {
          status: s.status === ("ready" as string) ? "idle" : s.status,
          elapsed: s.elapsed ?? 0,
        },
        since: Date.now(),
      }),
    )
    .catch(() => undefined);
}

/** Opens the mode with these images (default: the one in the editor). */
export function startSlides(documentIds: string[] = []): Promise<unknown> {
  return run("slides.start", documentIds.length > 0 ? { documentIds } : {});
}

export interface Box {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** The image's place in a `w` × `h` video: whole and centred, at its size if it fits
 * (or enlarged), smaller if not; zoomed with `center` in the middle (VID-074). */
export function place(
  w: number,
  h: number,
  iw: number,
  ih: number,
  enlarge: boolean,
  zoom = 1,
  center: [number, number] = [0.5, 0.5],
): Box {
  const fit = Math.min(w / iw, h / ih);
  const base = fit < 1 || enlarge ? fit : 1;
  const s = base * Math.min(8, Math.max(1, zoom));
  const dw = iw * s;
  const dh = ih * s;
  return { x: w / 2 - center[0] * dw, y: h / 2 - center[1] * dh, w: dw, h: dh };
}

/** The video sizes offered (VID-074). */
export const SLIDE_SIZES: [number, number][] = [
  [1920, 1080],
  [1280, 720],
  [2560, 1440],
  [3840, 2160],
  [1080, 1080],
  [1080, 1920],
];

/** The slide keys (VID-072): where each one goes, or null. */
export function slideKey(e: {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
}): "next" | "prev" | "first" | "last" | "reset" | null {
  const mod = e.ctrlKey || e.metaKey;
  switch (e.key) {
    case "ArrowRight":
    case "ArrowDown":
    case "PageDown":
    case " ":
      return mod ? null : "next";
    case "ArrowLeft":
    case "ArrowUp":
    case "PageUp":
      return mod ? null : "prev";
    case "]":
      return mod ? "next" : null;
    case "[":
      return mod ? "prev" : null;
    case "Home":
      return "first";
    case "End":
      return "last";
    case "0":
      return mod ? null : "reset";
    default:
      return null;
  }
}

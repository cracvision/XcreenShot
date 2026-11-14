// Narrated slides in the editor (VID-071 to VID-074): the mode's state as the core
// sends it, the recording's state, and the slide's place in the video (the same rule
// as the engine's, crates/xs-video/src/slides.rs).
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { run } from "../lib/commands";

export interface SlideInfo {
  documentId: string;
  /** Changes when its picture does (an annotation, VID-073). */
  version: number;
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
  /** Annotations added while narrating, to keep or remove once the mode closes
   * (VID-073). */
  pendingAnnotations: number;
  rec: RecState;
  /** When the recording's state arrived, to move the time on between updates. */
  since: number;
}

export const useSlides = create<Store>(() => ({
  mode: null,
  pendingAnnotations: 0,
  rec: { status: "idle", elapsed: 0 },
  since: 0,
}));

const take = (m: Partial<SlidesMode> & { active: boolean; annotations?: number }) =>
  useSlides.setState(
    m.active ? { mode: m as SlidesMode } : { mode: null, pendingAnnotations: m.annotations ?? 0 },
  );

/** The annotations added while narrating: kept, or removed from all the images. */
export function settleAnnotations(keep: boolean): void {
  useSlides.setState({ pendingAnnotations: 0 });
  void run("slides.annotations", { keep }).catch(() => undefined);
}

/** The annotation tools of the slides (VID-073). */
export type SlideTool = "arrow" | "shape" | "highlight" | "text" | "step";
export const SLIDE_TOOLS: SlideTool[] = ["arrow", "shape", "highlight", "text", "step"];

/** The object a gesture makes, in the image's pixels: a drag from `a` to `b`, or a
 * click at `a` (text and steps). Null when it is too small to be meant. */
export function slideObject(
  tool: SlideTool,
  a: [number, number],
  b: [number, number],
  text = "",
): Record<string, unknown> | null {
  const rect = {
    x: Math.min(a[0], b[0]),
    y: Math.min(a[1], b[1]),
    w: Math.abs(b[0] - a[0]),
    h: Math.abs(b[1] - a[1]),
  };
  switch (tool) {
    case "arrow":
      return Math.hypot(b[0] - a[0], b[1] - a[1]) < 4 ? null : { type: "arrow", from: a, to: b };
    case "shape":
    case "highlight":
      return rect.w < 4 || rect.h < 4 ? null : { type: tool, rect };
    case "step":
      return { type: "step", at: a };
    case "text":
      return text.trim() ? { type: "text", at: a, text: text.trim() } : null;
  }
}

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

// The video player's state (VID-056 to VID-058, ADR 0008): the <video> element, where
// the playhead is, the selection on the timeline and the playback settings. Times are
// of the video as it is now (after its cuts); the element plays the file and skips the
// parts cut out. Playback settings never change the file.
import { listen } from "@tauri-apps/api/event";
import i18next from "i18next";
import { create } from "zustand";
import { isCommandError, run } from "../lib/commands";
import { useEditor, usePanel, type DocDetails } from "./state";
import {
  clampSelection,
  frameStep,
  skipCut,
  toEdited,
  toSource,
  wholeSelected,
} from "./videoTimeline";
import { actionOff } from "../features";

/** Speeds offered (VID-057). */
export const SPEEDS = [0.5, 0.75, 1, 1.25, 1.5, 2];

interface VideoState {
  documentId: string | null;
  element: HTMLVideoElement | null;
  /** Playhead, in seconds of the cut video. */
  time: number;
  playing: boolean;
  /** The selection on the timeline (VID-058); the whole video by default. */
  start: number;
  end: number;
  muted: boolean;
  volume: number;
  speed: number;
  /** A long job (saving, a part, a silent copy): 0..1, or null. */
  progress: number | null;
  busy: string | null;
  /** A selection handle being dragged shows its frame; the playhead stays (VID-058). */
  previewing: boolean;
  /** Go to… is open: the time can be typed (VID-056). */
  goTo: boolean;
  /** The Create GIF dialog is open (VID-068). */
  gifDialog: boolean;
  /** The Combine videos window is open (VID-067). */
  combineDialog: boolean;
}

export const useVideo = create<VideoState>(() => ({
  documentId: null,
  element: null,
  time: 0,
  playing: false,
  start: 0,
  end: 0,
  muted: false,
  volume: 1,
  speed: 1,
  progress: null,
  busy: null,
  previewing: false,
  goTo: false,
  gifDialog: false,
  combineDialog: false,
}));

function video(): DocDetails["video"] {
  return useEditor.getState().doc?.video ?? null;
}

/** The player's own keys (VID-076): with a video shown, the editor leaves them to it. */
const PLAYER_KEYS = new Set([
  " ",
  "k",
  "ArrowLeft",
  "ArrowRight",
  "Home",
  "End",
  "i",
  "I",
  "o",
  "O",
  "Escape",
  "Delete",
  "Backspace",
]);

export function playerKey(e: KeyboardEvent): boolean {
  return (
    !e.ctrlKey &&
    !e.metaKey &&
    !e.altKey &&
    PLAYER_KEYS.has(e.key) &&
    !!useEditor.getState().doc?.video
  );
}

/** One frame, in seconds. */
export function frameLength(): number {
  return 1 / Math.max(1, video()?.fps ?? 30);
}

/** A new video or new cuts: the playhead stays where it was, the selection is all. */
export function attach(doc: DocDetails, element: HTMLVideoElement | null): void {
  const v = doc.video;
  if (!v) return;
  const same = useVideo.getState().documentId === doc.documentId;
  const time = same ? Math.min(useVideo.getState().time, v.length) : 0;
  useVideo.setState({ documentId: doc.documentId, element, start: 0, end: v.length, time });
  if (element) {
    element.muted = useVideo.getState().muted;
    element.volume = useVideo.getState().volume;
    element.playbackRate = useVideo.getState().speed;
    element.preservesPitch = true;
    // The same moment of the video as it is now (a cut moves the file's times).
    if (same) element.currentTime = toSource(v.kept, time);
  }
}

/** Shows the frame at `time` without moving the playhead (a handle being dragged). */
export function preview(time: number | null): void {
  const { element } = useVideo.getState();
  const v = video();
  if (!element || !v) return;
  if (time === null) {
    useVideo.setState({ previewing: false });
    element.currentTime = toSource(v.kept, useVideo.getState().time);
    return;
  }
  useVideo.setState({ previewing: true });
  element.currentTime = toSource(v.kept, Math.min(Math.max(0, time), v.length));
}

/** The element reports where it is: inside a cut it jumps on; past the end it stops. */
export function tick(): void {
  const { element, previewing } = useVideo.getState();
  const v = video();
  if (!element || !v || previewing) return;
  const at = skipCut(v.kept, element.currentTime);
  if (at === null) {
    element.pause();
    useVideo.setState({ time: v.length, playing: false });
    return;
  }
  if (at !== element.currentTime) element.currentTime = at;
  useVideo.setState({ time: toEdited(v.kept, at), playing: !element.paused });
}

/** Moves the playhead (VID-056); the picture follows at once. */
export function seek(time: number): void {
  const { element } = useVideo.getState();
  const v = video();
  if (!v) return;
  const t = Math.min(Math.max(0, time), v.length);
  useVideo.setState({ time: t });
  if (element) element.currentTime = toSource(v.kept, t);
}

export function playPause(): void {
  const { element, time } = useVideo.getState();
  const v = video();
  if (!element || !v) return;
  if (element.paused) {
    // At the end, playing starts again from the beginning.
    if (time >= v.length - frameLength() / 2) seek(0);
    void element.play();
    // The computer does not sleep while a video plays (VID-021): WebView2 asks for it.
  } else {
    element.pause();
  }
  useVideo.setState({ playing: !element.paused });
}

/** One frame back or forward (VID-056), paused. */
export function stepFrame(direction: -1 | 1): void {
  const { element, time } = useVideo.getState();
  element?.pause();
  useVideo.setState({ playing: false });
  seek(frameStep(time, direction, frameLength()));
}

/** The selection, kept inside the video and at least one frame long (VID-058). */
export function select(start: number, end: number): void {
  const v = video();
  if (!v) return;
  useVideo.setState(clampSelection(start, end, v.length, frameLength()));
}

export function selectAll(): boolean {
  const v = video();
  const { start, end } = useVideo.getState();
  if (!v || wholeSelected(start, end, v.length)) return false;
  useVideo.setState({ start: 0, end: v.length });
  return true;
}

export function canCut(): boolean {
  const v = video();
  const { start, end } = useVideo.getState();
  return !!v && !wholeSelected(start, end, v.length);
}

function failed(e: unknown): void {
  useEditor.getState().showError(e);
}

/** Runs a long video command with its progress shown. */
async function job<T>(
  task: string,
  command: string,
  params: Record<string, unknown>,
): Promise<T | null> {
  const doc = useEditor.getState().doc;
  if (!doc) return null;
  useVideo.getState().element?.pause();
  useVideo.setState({ busy: task, progress: 0, playing: false });
  try {
    return await run<T>(command, { documentId: doc.documentId, ...params });
  } catch (e) {
    // Cancelled on purpose: said, not an error (VID-068).
    if (isCommandError(e) && e.code === "cancelled") useEditor.getState().showToast(e.message);
    else failed(e);
    return null;
  } finally {
    useVideo.setState({ busy: null, progress: null });
  }
}

/** Cut selection (VID-059): one undo step; the playhead goes to the cut. */
export async function cutSelection(): Promise<void> {
  const doc = useEditor.getState().doc;
  const { start, end } = useVideo.getState();
  if (!doc || !canCut()) return;
  try {
    await run("video.cut", { documentId: doc.documentId, start, end });
    // The playhead goes where the cut is.
    seek(start);
  } catch (e) {
    failed(e);
  }
}

/** Save selection as video (VID-060). */
export async function saveSelection(): Promise<void> {
  const { start, end } = useVideo.getState();
  const r = await job<{ path: string }>("selection", "video.save_selection", { start, end });
  if (r) useEditor.getState().showToast(i18next.t("ui.video.saved_selection"));
}

/** Save frame (VID-061): pauses first, then the image opens in the editor. */
export async function saveFrame(): Promise<void> {
  useVideo.getState().element?.pause();
  useVideo.setState({ playing: false });
  const doc = useEditor.getState().doc;
  if (!doc) return;
  try {
    await run("video.save_frame", { documentId: doc.documentId, time: useVideo.getState().time });
  } catch (e) {
    failed(e);
  }
}

/** Extract text from the frame (VID-062): pauses first. */
export async function extractText(): Promise<void> {
  useVideo.getState().element?.pause();
  useVideo.setState({ playing: false });
  const doc = useEditor.getState().doc;
  if (!doc) return;
  try {
    await run("video.extract_text", {
      documentId: doc.documentId,
      time: useVideo.getState().time,
      show: true,
    });
  } catch (e) {
    failed(e);
  }
}

/** Create GIF (VID-068): in the background; the GIF opens in the editor when done. */
export async function createGif(params: Record<string, unknown>): Promise<void> {
  await job("gif", "video.gif", params);
}

/** Cancel the GIF being made (VID-068). */
export function cancelGif(): void {
  const doc = useEditor.getState().doc;
  if (doc) void run("video.cancel", { documentId: doc.documentId }).catch(() => undefined);
}

/** Silent copy (VID-063). */
export async function silentCopy(): Promise<void> {
  const r = await job<{ path: string }>("silent", "video.silent_copy", {});
  if (r) useEditor.getState().showToast(i18next.t("ui.video.saved_silent"));
}

/** Playback settings (VID-056, VID-057): they only change how it plays, never the file. */
export function setPlayback(change: { muted?: boolean; volume?: number; speed?: number }): void {
  useVideo.setState(change);
  const { element, muted, volume, speed } = useVideo.getState();
  if (!element) return;
  element.muted = muted;
  element.volume = volume;
  element.playbackRate = speed;
}

/** The Video menu (VID-056 to VID-064): with a video shown. */
export function videoAction(action: string): boolean {
  // Combine videos opens also without a video shown (VID-067).
  if (action === "video_combine") {
    useVideo.getState().element?.pause();
    useVideo.setState({ combineDialog: true, playing: false });
    return true;
  }
  const v = video();
  if (!v) return false;
  switch (action) {
    case "video_play":
      playPause();
      break;
    case "video_prev_frame":
      stepFrame(-1);
      break;
    case "video_next_frame":
      stepFrame(1);
      break;
    case "video_start":
      seek(0);
      break;
    case "video_end":
      seek(v.length);
      break;
    case "video_go_to":
      useVideo.getState().element?.pause();
      useVideo.setState({ goTo: true, playing: false });
      break;
    case "video_cut":
      void cutSelection();
      break;
    case "video_save_selection":
      void saveSelection();
      break;
    case "video_save_frame":
      void saveFrame();
      break;
    case "video_extract_text":
      if (actionOff(action)) return false;
      void extractText();
      break;
    case "video_silent_copy":
      if (!v.sound) return false;
      void silentCopy();
      break;
    case "video_gif":
      if (v.codec === "GIF") return false;
      useVideo.getState().element?.pause();
      useVideo.setState({ gifDialog: true, playing: false });
      break;
    case "video_info":
      // The side panel shows it (VID-064).
      usePanel.getState().set({ visible: true });
      break;
    default:
      return false;
  }
  return true;
}

/** Saving a video with cuts encodes it again: its progress shows on the bar. */
export function watchProgress(): () => void {
  const off = listen<{
    kind: string;
    payload: { documentId?: string; task?: string; progress?: number };
  }>("xs://editor", (e) => {
    if (e.payload.kind !== "video_progress") return;
    const { documentId, task, progress } = e.payload.payload;
    if (documentId !== useVideo.getState().documentId) return;
    useVideo.setState({
      progress: progress ?? null,
      busy: progress !== undefined && progress < 1 ? (task ?? "save") : null,
    });
  });
  return () => void off.then((f) => f());
}

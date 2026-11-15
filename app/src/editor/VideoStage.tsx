// A video open in the editor (VID-056, VID-057, ADR 0008): the picture fills the part of
// the workspace the floating bars leave, fitted. The system's player (WebView2) plays the
// file from the `xsdoc` protocol; the playback bar (VideoBar) and the keys drive it.
// It opens paused on its first frame; a click on the picture plays or pauses.
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import type { Insets } from "./insets";
import { requestZoom, takeZoomRequest, useEditor, xsdocUrl, type DocDetails } from "./state";
import { shownZoom, useVideoZoom, zoomAfter } from "./videoZoom";
import {
  attach,
  canCut,
  cutSelection,
  playPause,
  seek,
  select,
  selectAll,
  stepFrame,
  tick,
  useVideo,
  watchProgress,
} from "./videoState";

/** The keys of the player (VID-076): only while a video is shown and no field has the
 * focus. They go before the editor's own keys. */
function useVideoKeys() {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      if (target?.closest("input, textarea, select, [contenteditable='true'], [role='dialog']"))
        return;
      const { time, start, end } = useVideo.getState();
      const mod = e.ctrlKey || e.metaKey;
      // Ctrl/Cmd+Enter: to the start; with Shift, to the end (VID-076).
      if (mod && !e.altKey && e.key === "Enter") {
        seek(e.shiftKey ? Number.MAX_SAFE_INTEGER : 0);
        e.preventDefault();
        e.stopImmediatePropagation();
        return;
      }
      if (mod || e.altKey) return;
      let handled = true;
      switch (e.key) {
        case " ":
        case "k":
          playPause();
          break;
        case "ArrowLeft":
          if (e.shiftKey) seek(Math.max(0, time - 1));
          else stepFrame(-1);
          break;
        case "ArrowRight":
          if (e.shiftKey) seek(time + 1);
          else stepFrame(1);
          break;
        case "Home":
          seek(0);
          break;
        case "End":
          seek(Number.MAX_SAFE_INTEGER);
          break;
        case "i":
        case "I":
          select(time, Math.max(end, time));
          break;
        case "o":
        case "O":
          select(Math.min(start, time), time);
          break;
        case "Escape":
          // With all already selected, Esc is the editor's again (VID-058).
          handled = selectAll();
          break;
        case "Delete":
        case "Backspace":
          if (canCut()) void cutSelection();
          break;
        default:
          handled = false;
      }
      if (handled) {
        e.preventDefault();
        e.stopImmediatePropagation();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, []);
}

export function VideoStage({ doc, insets }: { doc: DocDetails; insets: Insets }) {
  const { t } = useTranslation();
  const ref = useRef<HTMLVideoElement | null>(null);
  const pictureRef = useRef<HTMLElement | null>(null);
  useVideoKeys();
  useEffect(watchProgress, []);

  // The editor's zoom (VID-056): fitted, or a percent of the video's pixels.
  const zoomed = useVideoZoom((s) => (s.documentId === doc.documentId ? s.zoom : null));
  const request = useEditor((s) => s.zoomRequest);
  useEffect(() => {
    if (useVideoZoom.getState().documentId !== doc.documentId)
      useVideoZoom.setState({ documentId: doc.documentId, zoom: null });
  }, [doc.documentId]);
  useEffect(() => {
    if (!request) return;
    const r = takeZoomRequest(doc.documentId);
    if (!r) return;
    useVideoZoom.setState((z) => ({ zoom: zoomAfter(r.mode, r.value, z.shown) }));
  }, [request, doc.documentId]);
  // What is shown, fitted or not, for the status bar.
  useEffect(() => {
    const el = pictureRef.current;
    const width = doc.video?.width ?? 0;
    if (!el || !width) return;
    const say = () =>
      useVideoZoom.setState({ shown: shownZoom(el.clientWidth, width, window.devicePixelRatio) });
    say();
    const watch = new ResizeObserver(say);
    watch.observe(el);
    return () => watch.disconnect();
  }, [doc.documentId, doc.video?.width]);

  // New video, new cuts or a saved file: the player takes it and stays where it was.
  useEffect(() => {
    attach(doc, ref.current);
  }, [doc]);

  // While playing, the playhead follows each frame shown.
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    let frame = 0;
    const loop = () => {
      tick();
      frame = el.requestVideoFrameCallback(loop);
    };
    frame = el.requestVideoFrameCallback(loop);
    // While it plays, the screen stays on (VID-021).
    let lock: WakeLockSentinel | null = null;
    const release = () => {
      void lock?.release().catch(() => undefined);
      lock = null;
    };
    const onPlay = () => {
      useVideo.setState({ playing: true });
      void navigator.wakeLock
        ?.request("screen")
        .then((l) => {
          lock = l;
        })
        .catch(() => undefined);
    };
    const onPause = () => {
      tick();
      useVideo.setState({ playing: false });
      release();
    };
    el.addEventListener("play", onPlay);
    el.addEventListener("pause", onPause);
    el.addEventListener("seeked", tick);
    return () => {
      el.cancelVideoFrameCallback(frame);
      el.removeEventListener("play", onPlay);
      el.removeEventListener("pause", onPause);
      el.removeEventListener("seeked", tick);
      el.pause();
      release();
    };
  }, [doc.documentId]);

  const v = doc.video;
  // Zoomed, the picture takes its size and the stage scrolls.
  const size =
    zoomed != null && v
      ? {
          width: (v.width * zoomed) / 100 / window.devicePixelRatio,
          maxWidth: "none",
          maxHeight: "none",
        }
      : {};
  return (
    <div className="canvas-area video-area">
      <div
        className={`video-stage${zoomed != null ? " zoomed" : ""}`}
        style={{
          top: insets.top,
          right: insets.right,
          bottom: insets.bottom,
          left: insets.left,
        }}
        onWheel={(e) => {
          // Ctrl+wheel zooms, as on the canvas (EDT-007).
          if (!e.ctrlKey) return;
          e.preventDefault();
          requestZoom(e.deltaY < 0 ? "in" : "out");
        }}
      >
        {v?.codec === "GIF" ? (
          <img
            ref={(el) => {
              pictureRef.current = el;
            }}
            className="video-picture"
            src={xsdocUrl(`${doc.documentId}/video?v=${doc.version}`)}
            alt={t("ui.video.picture", { name: doc.name })}
            style={{ aspectRatio: `${v.width} / ${v.height}`, ...size }}
            draggable={false}
          />
        ) : (
          <video
            ref={(el) => {
              ref.current = el;
              pictureRef.current = el;
            }}
            className="video-picture"
            // The version changes when the file does (saved with its cuts).
            src={xsdocUrl(`${doc.documentId}/video?v=${doc.version}`)}
            preload="auto"
            playsInline
            aria-label={t("ui.video.picture", { name: doc.name })}
            style={{ aspectRatio: v ? `${v.width} / ${v.height}` : undefined, ...size }}
            onClick={playPause}
            onError={() => useVideo.setState({ playing: false })}
          />
        )}
      </div>
    </div>
  );
}

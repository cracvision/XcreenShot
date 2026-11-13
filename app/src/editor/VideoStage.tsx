// A video open in the editor (VID-056, VID-057, ADR 0008): the picture fills the part of
// the workspace the floating bars leave, fitted. The system's player (WebView2) plays the
// file from the `xsdoc` protocol; the playback bar (VideoBar) and the keys drive it.
// It opens paused on its first frame; a click on the picture plays or pauses.
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import type { Insets } from "./insets";
import { xsdocUrl, type DocDetails } from "./state";
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
      if (e.ctrlKey || e.metaKey || e.altKey) return;
      const { time, start, end } = useVideo.getState();
      let handled = true;
      switch (e.key) {
        case " ":
        case "k":
          playPause();
          break;
        case "ArrowLeft":
          stepFrame(-1);
          break;
        case "ArrowRight":
          stepFrame(1);
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
  const ref = useRef<HTMLVideoElement>(null);
  useVideoKeys();
  useEffect(watchProgress, []);

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
    const onPlay = () => useVideo.setState({ playing: true });
    const onPause = () => {
      tick();
      useVideo.setState({ playing: false });
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
    };
  }, [doc.documentId]);

  const v = doc.video;
  return (
    <div className="canvas-area video-area">
      <div
        className="video-stage"
        style={{
          top: insets.top,
          right: insets.right,
          bottom: insets.bottom,
          left: insets.left,
        }}
      >
        {v?.codec === "GIF" ? (
          <img
            className="video-picture"
            src={xsdocUrl(`${doc.documentId}/video?v=${doc.version}`)}
            alt={t("ui.video.picture", { name: doc.name })}
            style={{ aspectRatio: `${v.width} / ${v.height}` }}
            draggable={false}
          />
        ) : (
          <video
            ref={ref}
            className="video-picture"
            // The version changes when the file does (saved with its cuts).
            src={xsdocUrl(`${doc.documentId}/video?v=${doc.version}`)}
            preload="auto"
            playsInline
            aria-label={t("ui.video.picture", { name: doc.name })}
            style={{ aspectRatio: v ? `${v.width} / ${v.height}` : undefined }}
            onClick={playPause}
            onError={() => useVideo.setState({ playing: false })}
          />
        )}
      </div>
    </div>
  );
}

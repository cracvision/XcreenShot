// The playback bar of a video open in the editor (VID-056 to VID-061, ADR 0008): a glass
// bar over the status bar. Its timeline shows the video as it is now (after its cuts),
// with the playhead, the marks where it was cut and the selection between its start and
// end handles; below, the controls. Playback settings never change the file.
import { useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import { useTranslation } from "react-i18next";
import { Menu } from "../components/Menu";
import { Icon } from "./icons";
import { useEditor } from "./state";
import {
  cancelGif,
  canCut,
  cutSelection,
  extractText,
  frameLength,
  playPause,
  preview,
  saveFrame,
  saveSelection,
  seek,
  select,
  setPlayback,
  silentCopy,
  SPEEDS,
  stepFrame,
  useVideo,
  videoAction,
} from "./videoState";
import { formatTime, parseTime, snap } from "./videoTimeline";

type Drag = "playhead" | "start" | "end";

/** Holding a frame button keeps stepping (VID-056). */
function useRepeat(action: () => void) {
  const timer = useRef<number | undefined>(undefined);
  const stop = () => {
    window.clearTimeout(timer.current);
    window.clearInterval(timer.current);
    timer.current = undefined;
  };
  return {
    onPointerDown: (e: ReactPointerEvent) => {
      if (e.button !== 0) return;
      action();
      stop();
      timer.current = window.setTimeout(() => {
        timer.current = window.setInterval(action, 1000 / 15);
      }, 400);
    },
    onPointerUp: stop,
    onPointerLeave: stop,
    onPointerCancel: stop,
    // The keyboard (Enter, Space) still works once per press.
    onClick: (e: React.MouseEvent) => {
      if (e.detail === 0) action();
    },
  };
}

function Timeline() {
  const { t } = useTranslation();
  const v = useEditor((s) => s.doc?.video);
  const time = useVideo((s) => s.time);
  const start = useVideo((s) => s.start);
  const end = useVideo((s) => s.end);
  const track = useRef<HTMLDivElement>(null);
  const [drag, setDrag] = useState<{ what: Drag; at: number } | null>(null);
  if (!v) return null;
  const total = Math.max(v.length, 1e-6);
  const pct = (s: number) => `${(Math.min(Math.max(s, 0), total) / total) * 100}%`;

  const timeAt = (clientX: number) => {
    const r = track.current?.getBoundingClientRect();
    if (!r || r.width <= 0) return 0;
    return (Math.min(Math.max(clientX - r.left, 0), r.width) / r.width) * total;
  };
  const pxPerSecond = () => (track.current?.getBoundingClientRect().width ?? 1) / total;

  const move = (what: Drag, clientX: number) => {
    const at = timeAt(clientX);
    if (what === "playhead") {
      seek(at);
      setDrag({ what, at });
      return;
    }
    // A handle sticks to the playhead within 8 px and shows its own frame (VID-058).
    const s = snap(at, useVideo.getState().time, pxPerSecond());
    const { start: a, end: b } = useVideo.getState();
    if (what === "start") select(Math.min(s, b - frameLength()), b);
    else select(a, Math.max(s, a + frameLength()));
    preview(s);
    setDrag({ what, at: s });
  };

  // One set of pointer handlers on the track: a handle says which it is.
  const onPointerDown = (e: ReactPointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    e.preventDefault();
    const what = ((e.target as HTMLElement).dataset.drag as Drag | undefined) ?? "playhead";
    e.currentTarget.setPointerCapture(e.pointerId);
    useVideo.getState().element?.pause();
    move(what, e.clientX);
  };
  const onPointerMove = (e: ReactPointerEvent) => {
    if (drag) move(drag.what, e.clientX);
  };
  const onPointerUp = () => {
    if (drag && drag.what !== "playhead") preview(null);
    setDrag(null);
  };

  // Keyboard: arrows move one frame, Shift+arrows one second (APP-041).
  const keys = (what: Drag) => (e: React.KeyboardEvent) => {
    const step = e.shiftKey ? 1 : frameLength();
    const delta = e.key === "ArrowLeft" ? -step : e.key === "ArrowRight" ? step : 0;
    if (!delta) return;
    e.preventDefault();
    e.stopPropagation();
    const s = useVideo.getState();
    if (what === "playhead") seek(s.time + delta);
    else if (what === "start") select(Math.min(s.start + delta, s.end - frameLength()), s.end);
    else select(s.start, Math.max(s.end + delta, s.start + frameLength()));
  };

  const label = drag && drag.what !== "playhead";
  return (
    <div className="vb-timeline">
      <div
        className="vb-track"
        ref={track}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
      >
        <div className="vb-rail" />
        <div className="vb-range" style={{ left: pct(start), width: pct(end - start) }} />
        {v.marks.map((m) => (
          <span key={m} className="vb-mark" style={{ left: pct(m) }} aria-hidden="true" />
        ))}
        <div
          className="vb-playhead"
          style={{ left: pct(time) }}
          role="slider"
          tabIndex={0}
          aria-label={t("ui.video.playhead")}
          aria-valuemin={0}
          aria-valuemax={total}
          aria-valuenow={time}
          aria-valuetext={formatTime(time)}
          onKeyDown={keys("playhead")}
        />
        <div
          className="vb-handle start"
          style={{ left: pct(start) }}
          role="slider"
          tabIndex={0}
          aria-label={t("ui.video.start_handle")}
          aria-valuemin={0}
          aria-valuemax={total}
          aria-valuenow={start}
          aria-valuetext={formatTime(start)}
          data-drag="start"
          onKeyDown={keys("start")}
        />
        <div
          className="vb-handle end"
          style={{ left: pct(end) }}
          role="slider"
          tabIndex={0}
          aria-label={t("ui.video.end_handle")}
          aria-valuemin={0}
          aria-valuemax={total}
          aria-valuenow={end}
          aria-valuetext={formatTime(end)}
          data-drag="end"
          onKeyDown={keys("end")}
        />
        {label && (
          <span className="vb-label" style={{ left: pct(drag.at) }} role="status">
            {formatTime(drag.at)} · {t("ui.video.seconds", { value: (end - start).toFixed(1) })}
          </span>
        )}
      </div>
    </div>
  );
}

/** Elapsed and total time; a click turns it into Go to… (VID-056). */
/** Go to… (VID-056): a time typed as 01:15, 75 or 1:02:03. */
function GoTo({ initial }: { initial: string }) {
  const { t } = useTranslation();
  const [text, setText] = useState(initial);
  const close = () => useVideo.setState({ goTo: false });
  return (
    <input
      className="vb-goto"
      autoFocus
      value={text}
      aria-label={t("ui.video.go_to")}
      placeholder="01:15"
      onFocus={(e) => e.target.select()}
      onChange={(e) => setText(e.target.value)}
      onBlur={close}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          const at = parseTime(text);
          if (at !== null) seek(at);
          close();
        } else if (e.key === "Escape") close();
      }}
    />
  );
}

/** Elapsed and total time; a click (or Video > Go to…) types a time (VID-056). */
function Clock() {
  const { t } = useTranslation();
  const v = useEditor((s) => s.doc?.video);
  const time = useVideo((s) => s.time);
  const editing = useVideo((s) => s.goTo);
  if (!v) return null;
  const long = v.length >= 3600;
  if (editing) return <GoTo initial={formatTime(time, long)} />;
  return (
    <button
      type="button"
      className="vb-clock"
      title={t("ui.video.go_to")}
      onClick={() => useVideo.setState({ goTo: true })}
    >
      {formatTime(time, long)} <span className="muted">/ {formatTime(v.length, long)}</span>
    </button>
  );
}

export function VideoBar() {
  const { t } = useTranslation();
  const v = useEditor((s) => s.doc?.video);
  const playing = useVideo((s) => s.playing);
  const start = useVideo((s) => s.start);
  const end = useVideo((s) => s.end);
  const muted = useVideo((s) => s.muted);
  const volume = useVideo((s) => s.volume);
  const speed = useVideo((s) => s.speed);
  const busy = useVideo((s) => s.busy);
  const progress = useVideo((s) => s.progress);
  const back = useRepeat(() => stepFrame(-1));
  const next = useRepeat(() => stepFrame(1));
  // A GIF plays by itself, in a loop: no controls (VID-068).
  if (!v || v.codec === "GIF") return null;
  const cuttable = canCut();
  return (
    <div className="video-bar" role="group" aria-label={t("ui.video.bar")}>
      <Timeline />
      <div className="vb-controls">
        <button
          type="button"
          className="btn icon-btn"
          title={t("ui.video.prev_frame")}
          aria-label={t("ui.video.prev_frame")}
          {...back}
        >
          <Icon name="frame_back" />
        </button>
        <button
          type="button"
          className="btn primary icon-btn vb-play"
          title={playing ? t("ui.video.pause") : t("ui.video.play")}
          aria-label={playing ? t("ui.video.pause") : t("ui.video.play")}
          onClick={playPause}
        >
          <Icon name={playing ? "pause" : "play"} />
        </button>
        <button
          type="button"
          className="btn icon-btn"
          title={t("ui.video.next_frame")}
          aria-label={t("ui.video.next_frame")}
          {...next}
        >
          <Icon name="frame_next" />
        </button>
        <Clock />
        <span className="vb-selection muted">
          {t("ui.video.selection", {
            from: formatTime(start),
            to: formatTime(end),
            seconds: (end - start).toFixed(1),
          })}
        </span>
        <span className="vb-fill" />
        {busy !== null && (
          <span className="vb-progress" role="status">
            {t(`ui.video.busy_${busy}`, { percent: Math.round((progress ?? 0) * 100) })}
            <progress max={1} value={progress ?? 0} />
            {busy === "gif" && (
              <button type="button" className="btn small" onClick={cancelGif}>
                {t("ui.editor.cancel")}
              </button>
            )}
          </span>
        )}
        <button
          type="button"
          className="btn"
          disabled={!cuttable || busy !== null}
          title={cuttable ? t("ui.video.cut_hint") : t("ui.video.cut_all")}
          onClick={() => void cutSelection()}
        >
          <Icon name="scissors" />
          <span className="tb-label">{t("ui.video.cut")}</span>
        </button>
        <button
          type="button"
          className="btn"
          disabled={busy !== null}
          title={t("ui.video.save_frame")}
          onClick={() => void saveFrame()}
        >
          <Icon name="frame" />
          <span className="tb-label">{t("ui.video.save_frame")}</span>
        </button>
        <button
          type="button"
          className="btn"
          disabled={busy !== null}
          title={t("ui.video.gif.title")}
          onClick={() => videoAction("video_gif")}
        >
          <span className="tb-label">{t("ui.video.gif.button")}</span>
        </button>
        <Menu
          label={<Icon name="more" />}
          ariaLabel={t("ui.video.more")}
          title={t("ui.video.more")}
          triggerClass="btn icon-btn"
          chevron={false}
          side="top"
          align="end"
          disabled={busy !== null}
          items={[
            {
              id: "save_selection",
              label: t("ui.video.save_selection"),
              onSelect: () => void saveSelection(),
            },
            {
              id: "extract_text",
              label: t("ui.video.extract_text"),
              onSelect: () => void extractText(),
            },
            {
              id: "silent_copy",
              label: t("ui.video.silent_copy"),
              disabled: !v.sound,
              ...(v.sound ? {} : { title: t("errors.video_no_sound") }),
              onSelect: () => void silentCopy(),
            },
          ]}
        />
        <select
          className="vb-speed"
          value={speed}
          aria-label={t("ui.video.speed")}
          title={t("ui.video.speed")}
          onChange={(e) => setPlayback({ speed: Number(e.target.value) })}
        >
          {SPEEDS.map((s) => (
            <option key={s} value={s}>
              {`${s.toLocaleString()}×`}
            </option>
          ))}
        </select>
        <button
          type="button"
          className="btn icon-btn"
          aria-pressed={muted}
          disabled={!v.sound}
          title={muted ? t("ui.video.unmute") : t("ui.video.mute")}
          aria-label={muted ? t("ui.video.unmute") : t("ui.video.mute")}
          onClick={() => setPlayback({ muted: !muted })}
        >
          <Icon name={muted || !v.sound ? "mute" : "volume"} />
        </button>
        <input
          type="range"
          className="vb-volume"
          min={0}
          max={1}
          step={0.05}
          value={muted ? 0 : volume}
          disabled={!v.sound}
          aria-label={t("ui.video.volume")}
          onChange={(e) => setPlayback({ volume: Number(e.target.value), muted: false })}
        />
      </div>
    </div>
  );
}

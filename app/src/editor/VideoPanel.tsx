// The side panel of a video (VID-064, VID-065, ADR 0008): what the video holds and what
// can be done with it. Image tools and effects do not apply to videos: the panel says
// what to do instead.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { Icon } from "./icons";
import type { DocDetails } from "./state";
import { extractText, saveFrame, saveSelection, silentCopy, useVideo } from "./videoState";
import { formatTime } from "./videoTimeline";

interface FullInfo {
  bytes?: number;
  created?: string;
  frames?: number;
  fpsAverage?: number;
  fpsMax?: number;
  bitrate?: number;
  audio?: { channels: number; rate: number }[];
}

function megabytes(bytes: number): string {
  return `${(bytes / 1_048_576).toLocaleString(undefined, { maximumFractionDigits: 1 })} MB`;
}

export function VideoPanel({ doc }: { doc: DocDetails }) {
  const { t } = useTranslation();
  const busy = useVideo((s) => s.busy);
  // The rest (VID-064) reads the whole file: it comes a moment later, for this file.
  const key = `${doc.path ?? ""}#${doc.version}`;
  const [loaded, setLoaded] = useState<{ key: string; info: FullInfo } | null>(null);
  const full = loaded?.key === key ? loaded.info : null;
  const v = doc.video;
  useEffect(() => {
    if (!doc.path) return;
    let live = true;
    void run<FullInfo>("video.info", { path: doc.path })
      .then((info) => live && setLoaded({ key, info }))
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [doc.path, key]);

  if (!v) return null;
  const rows: [string, string][] = [
    [t("ui.video.info.size"), `${v.width} × ${v.height}`],
    [t("ui.video.info.duration"), formatTime(v.length)],
    [t("ui.video.info.fps"), `${v.fps}`],
    [t("ui.video.info.codec"), v.codec],
    [t("ui.video.info.sound"), v.sound ? t("ui.video.info.yes") : t("ui.video.info.no")],
  ];
  if (v.length < v.duration - 0.05)
    rows.splice(2, 0, [t("ui.video.info.file_duration"), formatTime(v.duration)]);
  if (full) {
    if (full.fpsAverage !== undefined)
      rows.push([
        t("ui.video.info.fps_measured"),
        t("ui.video.info.fps_values", { average: full.fpsAverage, max: full.fpsMax ?? 0 }),
      ]);
    if (full.bitrate)
      rows.push([t("ui.video.info.bitrate"), `${Math.round(full.bitrate / 1000)} kbps`]);
    for (const [i, a] of (full.audio ?? []).entries())
      rows.push([
        t("ui.video.info.track", { n: i + 1 }),
        t("ui.video.info.track_value", { channels: a.channels, rate: a.rate }),
      ]);
    if (full.bytes) rows.push([t("ui.video.info.file_size"), megabytes(full.bytes)]);
    if (full.created)
      rows.push([t("ui.video.info.created"), new Date(full.created).toLocaleString()]);
  }
  if (doc.path) rows.push([t("ui.video.info.location"), doc.path]);

  const copy = () =>
    void navigator.clipboard.writeText(rows.map(([k, val]) => `${k}: ${val}`).join("\n"));

  return (
    <div className="video-panel">
      <dl className="video-info">
        {rows.map(([k, val]) => (
          <div key={k}>
            <dt>{k}</dt>
            <dd>{val}</dd>
          </div>
        ))}
      </dl>
      <button type="button" className="btn small" onClick={copy}>
        {t("ui.video.info.copy")}
      </button>
      <h3 className="side-subtitle">{t("ui.video.actions")}</h3>
      <div className="video-actions">
        <button
          type="button"
          className="btn"
          disabled={busy !== null}
          onClick={() => void saveFrame()}
        >
          <Icon name="frame" />
          {t("ui.video.save_frame")}
        </button>
        <button
          type="button"
          className="btn"
          disabled={busy !== null}
          onClick={() => void saveSelection()}
        >
          {t("ui.video.save_selection")}
        </button>
        <button
          type="button"
          className="btn"
          disabled={busy !== null}
          onClick={() => void extractText()}
        >
          {t("ui.video.extract_text")}
        </button>
        <button
          type="button"
          className="btn"
          disabled={busy !== null || !v.sound}
          title={v.sound ? undefined : t("errors.video_no_sound")}
          onClick={() => void silentCopy()}
        >
          {t("ui.video.silent_copy")}
        </button>
      </div>
      <p className="muted video-note">{t("errors.video_no_image_tools")}</p>
    </div>
  );
}

// The Ready summary (VID-006): beside the recording bar while Ready, what is about to be
// recorded — the video's size, frames and codec, the sound, the camera, the pointer's
// effects, the timer, where it opens and how much fits on the disk — with Record,
// Capture image and Cancel, and "Don't show again". It is part of the bar's window, so
// it never shows in the video.
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { useSetting } from "../store";
import { clock } from "./recClock";

/** Its height in CSS px, for the bar's window to make room. */
export const SUMMARY_H = 214;

export interface SummaryState {
  width?: number;
  height?: number;
  areaWidth?: number;
  areaHeight?: number;
  reduced?: boolean;
  fps?: number;
  codec?: string;
  minutesFree?: number | null;
  open?: "editor" | "none";
  limit?: number | null;
  cursor?: boolean;
  systemAudio?: boolean;
}

export function ReadySummary(props: {
  state: SummaryState;
  onRecord: () => void;
  onImage: () => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  const s = props.state;
  const mic = useSetting("video.microphone", true);
  const camera = useSetting("video.camera", false);
  const highlight = useSetting("video.highlight", false);
  const clicks = useSetting("video.clicks", false);
  const keys = useSetting("video.keys", false);
  const liveDraw = useSetting("video.liveDraw", true);
  const combine = useSetting("video.combineAudioTracks", true);
  const yes = (on: boolean | undefined) => t(on ? "ui.recbar.ready_on" : "ui.recbar.ready_off");
  const size =
    s.width && s.height
      ? s.reduced && s.areaWidth && s.areaHeight
        ? t("ui.recbar.ready_size_reduced", {
            width: s.width,
            height: s.height,
            areaWidth: s.areaWidth,
            areaHeight: s.areaHeight,
          })
        : t("ui.recbar.size", { width: s.width, height: s.height })
      : "—";
  const effects = [
    s.cursor ? t("ui.recbar.cursor") : null,
    s.cursor && highlight ? t("ui.recbar.ready_highlight") : null,
    s.cursor && clicks ? t("ui.recbar.clicks") : null,
    keys ? t("ui.recbar.ready_keys") : null,
    liveDraw ? t("ui.recbar.ready_draw") : null,
  ].filter((e): e is string => e !== null);
  const minutes = s.minutesFree;
  const disk =
    minutes == null
      ? "—"
      : minutes >= 120
        ? t("ui.recbar.ready_disk_hours", { hours: Math.floor(minutes / 60) })
        : t("ui.recbar.ready_disk_minutes", { minutes });
  const rows: [string, string][] = [
    [t("ui.recbar.ready_video"), `${size} · ${s.fps ?? 0} fps · ${s.codec ?? "H.264"}`],
    [
      t("ui.recbar.ready_sound"),
      t("ui.recbar.ready_sound_value", { mic: yes(mic), system: yes(s.systemAudio) }) +
        (mic && s.systemAudio && !combine ? ` · ${t("ui.recbar.tracks_apart")}` : ""),
    ],
    [t("ui.recbar.ready_camera"), yes(camera)],
    [t("ui.recbar.ready_effects"), effects.length ? effects.join(", ") : t("ui.recbar.ready_none")],
    [t("ui.recbar.timer"), s.limit ? clock(s.limit) : t("ui.recbar.timer_none")],
    [
      t("ui.recbar.ready_open"),
      t(s.open === "none" ? "ui.recbar.ready_open_library" : "ui.recbar.ready_open_editor"),
    ],
    [t("ui.recbar.ready_disk"), disk],
  ];
  const low = minutes != null && minutes < 10;
  return (
    <section className="recbar-summary" aria-label={t("ui.recbar.ready_title")}>
      <dl>
        {rows.map(([k, v]) => (
          <div key={k} className={k === t("ui.recbar.ready_disk") && low ? "low" : undefined}>
            <dt>{k}</dt>
            <dd>{v}</dd>
          </div>
        ))}
      </dl>
      <div className="recbar-summary-foot">
        <label className="panel-check">
          <input
            type="checkbox"
            onChange={(e) =>
              void run("settings.set", {
                key: "video.readySummary",
                value: !e.target.checked,
              }).catch(() => undefined)
            }
          />
          {t("ui.recbar.ready_hide")}
        </label>
        <span className="grow" />
        <button type="button" className="btn small" onClick={props.onCancel}>
          {t("ui.recbar.cancel")} (Esc)
        </button>
        <button type="button" className="btn small" onClick={props.onImage}>
          {t("ui.recbar.capture_image")} (I)
        </button>
        <button type="button" className="btn small primary" onClick={props.onRecord}>
          {t("ui.recbar.record")} (Enter)
        </button>
      </div>
    </section>
  );
}

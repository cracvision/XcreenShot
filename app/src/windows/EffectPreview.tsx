// A preview of the pointer's effects (VID-045, VID-046): the highlight around the
// pointer and a click's wave, with the colours, size, opacity and shape chosen now.
import { useTranslation } from "react-i18next";
import { useApp, settingValue } from "../store";

const CLICK_SIZES: Record<string, number> = { small: 40, medium: 64, large: 96 };

/** The preview's pixels are these many of the video's: it fits a 120 px highlight. */
const K = 0.55;

export function EffectPreview({ highlight, clicks }: { highlight: boolean; clicks: boolean }) {
  const { t } = useTranslation();
  const settings = useApp((s) => s.settings);
  const value = (key: string) => settingValue(settings, key);
  const str = (key: string, fallback: string) => {
    const v = value(key);
    return typeof v === "string" ? v : fallback;
  };
  const num = (key: string, fallback: number) => {
    const v = value(key);
    return typeof v === "number" ? v : fallback;
  };
  const hColor = str("video.highlightColor", "#FFD000");
  const hSize = num("video.highlightSize", 48) * K;
  const hOpacity = num("video.highlightOpacity", 40) / 100;
  const ring = str("video.highlightShape", "circle") === "ring";
  const cColor = str("video.clickColor", "#2F6FDE");
  const cSize = (CLICK_SIZES[str("video.clickSize", "medium")] ?? 64) * K;
  const parts = [
    highlight ? t("ui.panel.video_highlight") : null,
    clicks ? t("ui.panel.video_clicks") : null,
  ].filter(Boolean);
  // The pointer: on the highlight, or where the click is when there is none.
  const [px, py] = highlight ? [52, 44] : [118, 44];
  return (
    <svg
      className="effect-preview"
      viewBox="0 0 168 88"
      width={168}
      height={88}
      role="img"
      aria-label={t("ui.panel.video_effects_preview", { effects: parts.join(", ") })}
    >
      <rect x={0.5} y={0.5} width={167} height={87} rx={8} className="effect-preview-bg" />
      {highlight &&
        (ring ? (
          <circle
            cx={52}
            cy={44}
            r={hSize / 2 - 1}
            fill="none"
            stroke={hColor}
            strokeOpacity={hOpacity}
            strokeWidth={4 * K}
          />
        ) : (
          <circle cx={52} cy={44} r={hSize / 2} fill={hColor} fillOpacity={hOpacity} />
        ))}
      {clicks && (
        <>
          <circle cx={118} cy={44} r={cSize / 2} fill={cColor} fillOpacity={0.18} />
          <circle
            cx={118}
            cy={44}
            r={cSize / 2}
            fill="none"
            stroke={cColor}
            strokeOpacity={0.85}
            strokeWidth={2}
          />
        </>
      )}
      <path
        d="M0 0 L0 17 L4.5 13 L7.5 20 L10 19 L7 12 L12.5 12 Z"
        transform={`translate(${px} ${py})`}
        className="effect-preview-pointer"
        strokeWidth={1}
        strokeLinejoin="round"
      />
    </svg>
  );
}

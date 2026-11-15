// Hours, minutes and seconds in three small fields: the timer's own length (VID-015).
import { useTranslation } from "react-i18next";
import { joinSeconds, splitSeconds } from "../lib/duration";

interface Props {
  /** Seconds. */
  value: number;
  onChange: (seconds: number) => void;
  /** What the three fields are, for screen readers. */
  label: string;
  autoFocus?: boolean;
  /** Enter in a field. */
  onEnter?: () => void;
}

export function DurationField({ value, onChange, label, autoFocus, onEnter }: Props) {
  const { t } = useTranslation();
  const parts = splitSeconds(value);
  const part = (key: "h" | "m" | "s", max: number, first: boolean) => (
    <label className="duration-part">
      <input
        type="number"
        min={0}
        max={max}
        step={1}
        inputMode="numeric"
        value={parts[key]}
        aria-label={t(`ui.duration.${key}_long`)}
        autoFocus={autoFocus && first}
        onFocus={(e) => e.currentTarget.select()}
        onKeyDown={(e) => {
          if (e.key === "Enter" && onEnter) {
            e.preventDefault();
            onEnter();
          }
        }}
        onChange={(e) => {
          const n = Math.min(max, Math.max(0, Math.floor(Number(e.target.value) || 0)));
          const next = { ...parts, [key]: n };
          onChange(joinSeconds(next.h, next.m, next.s));
        }}
      />
      <span aria-hidden="true">{t(`ui.duration.${key}`)}</span>
    </label>
  );
  return (
    <span className="duration" role="group" aria-label={label}>
      {part("h", 12, true)}
      {part("m", 59, false)}
      {part("s", 59, false)}
    </span>
  );
}

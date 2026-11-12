// Fields of the side panel: a number with its slider (EDT-014: decimals, Shift+arrows
// for ten steps) and a list of choices. Shared by Properties and Effects.
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useDraft } from "./useDraft";

export function NumberField(props: {
  label: string;
  /** `undefined` when the selected objects differ (Mixed, EDT-014). */
  value: number | undefined;
  min: number;
  max: number;
  step: number;
  unit: string;
  onCommit: (v: number) => void;
}) {
  const { label, min, max, step, unit, onCommit } = props;
  const { t } = useTranslation();
  const mixed = props.value === undefined;
  const value = props.value ?? min;
  const [draft, setDraft] = useDraft(mixed ? "" : String(value));
  const [sliding, setSliding] = useState<number | null>(null);
  const id = `f-${label}`;
  const commit = (v: number) => {
    const clamped = Math.min(max, Math.max(min, v));
    if (Number.isFinite(clamped) && (mixed || clamped !== value)) onCommit(clamped);
  };
  const typed = () => {
    const v = Number(draft.replace(",", "."));
    if (draft.trim() !== "" && Number.isFinite(v)) commit(v);
    setDraft(null);
  };
  return (
    <div className="prop-field">
      <label htmlFor={id}>{label}</label>
      <div className="prop-number">
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={sliding ?? value}
          aria-label={label}
          onChange={(e) => setSliding(Number(e.target.value))}
          onKeyDown={(e) => {
            // Shift + arrows: ten steps at once (EDT-013); alone, the slider's own step.
            const dir = { ArrowUp: 1, ArrowRight: 1, ArrowDown: -1, ArrowLeft: -1 }[e.key];
            if (!e.shiftKey || dir === undefined) return;
            e.preventDefault();
            commit((sliding ?? value) + dir * step * 10);
            setSliding(null);
          }}
          onPointerUp={() => {
            if (sliding !== null) commit(sliding);
            setSliding(null);
          }}
          onKeyUp={() => {
            if (sliding !== null) commit(sliding);
            setSliding(null);
          }}
        />
        <input
          id={id}
          type="text"
          inputMode="decimal"
          value={sliding !== null ? String(sliding) : draft}
          placeholder={mixed ? t("ui.editor.mixed") : ""}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={typed}
          onKeyDown={(e) => {
            if (e.key === "Enter") typed();
            if (e.key === "ArrowUp" || e.key === "ArrowDown") {
              e.preventDefault();
              const d = (e.key === "ArrowUp" ? 1 : -1) * (e.shiftKey ? 10 : 1);
              commit(value + d);
            }
          }}
        />
        <span className="muted">{unit}</span>
      </div>
    </div>
  );
}

export function SelectField<T extends string>(props: {
  label: string;
  value: T | undefined;
  options: { value: T; label: string; title?: string }[];
  onCommit: (v: T) => void;
}) {
  const { t } = useTranslation();
  const id = `sel-${props.label}`;
  return (
    <div className="prop-field">
      <label htmlFor={id}>{props.label}</label>
      <select
        id={id}
        value={props.value ?? ""}
        onChange={(e) => props.onCommit(e.target.value as T)}
      >
        {props.value === undefined && <option value="">{t("ui.editor.mixed")}</option>}
        {props.options.map((o) => (
          <option key={o.value} value={o.value} title={o.title}>
            {o.label}
          </option>
        ))}
      </select>
    </div>
  );
}

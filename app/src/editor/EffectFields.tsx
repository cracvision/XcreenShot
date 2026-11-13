// The options of one effect (EDT-053 to EDT-068): its numbers, choices, colors, checks
// and filters, as the Effects tab and the capture panel (CAP-051) show them.
import { useTranslation } from "react-i18next";
import { ColorPicker } from "./ColorPicker";
import { NumberField, SelectField } from "./Fields";
import { change, FIELDS, read, type Effect, type Field } from "./effectOptions";

/** The fields of `effect`; each change is the top-level change to apply. `capture`:
 * among the effects on capture, with the options that only exist there (CAP-051). */
export function EffectFields({
  effect,
  onChange,
  capture = false,
}: {
  effect: Effect;
  onChange: (changes: Record<string, unknown>) => void;
  capture?: boolean;
}) {
  const { t } = useTranslation();
  const fieldNode = (f: Field) => {
    const e = effect;
    const type = effect.type;
    const v = read(e, f.k);
    const set = (value: unknown) => onChange(change(e, f.k, value));
    const lbl = t(`ui.editor.fx.${type}.${f.k.replaceAll(".", "_")}`);
    switch (f.kind) {
      case "number":
        return (
          <NumberField
            key={f.k}
            label={lbl}
            value={typeof v === "number" ? v : Number(v ?? f.min)}
            min={f.min}
            max={f.max}
            step={f.step ?? 1}
            unit={f.unit ?? ""}
            onCommit={set}
          />
        );
      case "select":
        return (
          <SelectField
            key={f.k}
            label={lbl}
            value={String(v ?? f.options[0])}
            options={f.options.map((o) => ({ value: o, label: t(`ui.editor.fx.opt.${o}`) }))}
            onCommit={(o) => set(type === "rotate" ? Number(o) : o)}
          />
        );
      case "color":
        return (
          <ColorPicker
            key={f.k}
            label={lbl}
            value={(v as string | null | undefined) ?? (f.transparent ? null : "#000000")}
            allowTransparent={f.transparent ?? false}
            onCommit={set}
          />
        );
      case "check":
        return (
          <label key={f.k} className="panel-check">
            <input type="checkbox" checked={v === true} onChange={(ev) => set(ev.target.checked)} />
            {lbl}
          </label>
        );
      case "text":
        return (
          <label key={f.k} className="prop-field">
            <span>{lbl}</span>
            <input
              type="text"
              defaultValue={String(v ?? "")}
              onBlur={(ev) => ev.target.value !== String(v ?? "") && set(ev.target.value)}
              onKeyDown={(ev) => ev.key === "Enter" && (ev.target as HTMLInputElement).blur()}
            />
          </label>
        );
      case "filter": {
        const on =
          v === true ||
          (typeof v === "object" && v !== null && (v as { on?: boolean }).on === true);
        const value =
          typeof v === "object" && v !== null ? (v as { value?: number }).value : undefined;
        return (
          <div key={f.k} className="fx-filter">
            <label className="panel-check">
              <input
                type="checkbox"
                checked={on}
                onChange={(ev) => set({ on: ev.target.checked, value: value ?? f.max ?? 0 })}
              />
              {lbl}
            </label>
            {on && f.min !== undefined && f.max !== undefined && (
              <NumberField
                label={`${lbl} · ${t("ui.editor.fx.value")}`}
                value={value ?? f.max}
                min={f.min}
                max={f.max}
                step={1}
                unit={f.unit ?? ""}
                onCommit={(n) => set({ on: true, value: n })}
              />
            )}
          </div>
        );
      }
    }
  };

  const shown = (FIELDS[effect.type] ?? []).filter(
    (f) => capture || !(f.kind === "check" && f.capture),
  );
  return <>{shown.map((f) => fieldNode(f))}</>;
}

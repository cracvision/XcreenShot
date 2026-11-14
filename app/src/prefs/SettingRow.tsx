import { useEffect, useRef, useState, type CSSProperties } from "react";
import { useTranslation } from "react-i18next";
import { Icon } from "../components/Icon";
import { ContextMenu } from "../components/Menu";
import { MicDeviceSelect } from "../components/MicDeviceSelect";
import { isCommandError, run, type SettingEntry } from "../lib/commands";
import { applyGlass } from "../theme";

interface Props {
  setting: SettingEntry;
  highlighted: boolean;
}

/** One preference: label, description and a control chosen from its schema (APP-006). */
export function SettingRow({ setting, highlighted }: Props) {
  const { t } = useTranslation();
  const [error, setError] = useState<string | null>(null);
  const [menu, setMenu] = useState({ open: false, x: 0, y: 0 });
  const id = `setting-${setting.key}`;
  const optionsBase = `settings.${setting.key.replaceAll(".", "_")}.options`;

  const change = async (value: unknown) => {
    setError(null);
    try {
      await run("settings.set", { key: setting.key, value });
    } catch (e) {
      setError(isCommandError(e) ? e.message : String(e));
    }
  };

  let control;
  if (setting.key === "video.microphoneDevice" || setting.key === "video.cameraDevice") {
    // A device chosen from those plugged in, not its id typed (VID-031, VID-039).
    control = (
      <MicDeviceSelect
        id={id}
        kind={setting.key === "video.cameraDevice" ? "camera" : "microphone"}
        value={String(setting.value ?? "")}
        disabled={setting.locked}
        describedBy={`${id}-desc`}
        onChange={(v) => void change(v)}
      />
    );
  } else if (setting.key === "general.glassIntensity") {
    control = <GlassSlider id={id} setting={setting} change={change} />;
  } else if (setting.key === "general.accent" && setting.schema.enum) {
    control = (
      <AccentPicker
        labelledBy={`${id}-label`}
        describedBy={`${id}-desc`}
        setting={setting}
        names={(v) => t(`${optionsBase}.${v}`)}
        change={change}
      />
    );
  } else if (setting.schema.format === "color") {
    // A colour as #RRGGBB, with the system's colour picker.
    control = (
      <input
        id={id}
        type="color"
        value={String(setting.value ?? "#000000").toLowerCase()}
        disabled={setting.locked}
        aria-describedby={`${id}-desc`}
        onChange={(e) => {
          const v = e.target.value.toUpperCase();
          if (v !== setting.value) void change(v);
        }}
      />
    );
  } else if (setting.schema.type === "boolean") {
    control = (
      <input
        id={id}
        type="checkbox"
        role="switch"
        className="switch"
        checked={setting.value === true}
        disabled={setting.locked}
        aria-describedby={`${id}-desc`}
        onChange={(e) => void change(e.target.checked)}
      />
    );
  } else if (setting.schema.type === "integer") {
    // A whole number, saved when the field is left (APP-006).
    control = (
      <input
        id={id}
        type="number"
        key={String(setting.value)}
        defaultValue={Number(setting.value)}
        min={setting.schema.minimum}
        max={setting.schema.maximum}
        disabled={setting.locked}
        aria-describedby={`${id}-desc`}
        onBlur={(e) => {
          const v = Math.round(Number(e.target.value));
          if (Number.isFinite(v) && v !== setting.value) void change(v);
        }}
      />
    );
  } else if (
    !setting.schema.enum &&
    (setting.schema.type === "string" ||
      (Array.isArray(setting.schema.type) && setting.schema.type.includes("string")))
  ) {
    // Free text; empty is null when allowed.
    const nullable = Array.isArray(setting.schema.type) && setting.schema.type.includes("null");
    control = (
      <input
        id={id}
        type="text"
        key={String(setting.value)}
        defaultValue={setting.value === null ? "" : String(setting.value ?? "")}
        disabled={setting.locked}
        aria-describedby={`${id}-desc`}
        onBlur={(e) => {
          const text = e.target.value.trim();
          const v = text === "" && nullable ? null : text;
          if (v !== setting.value) void change(v);
        }}
      />
    );
  } else if (setting.schema.enum) {
    control = (
      <select
        id={id}
        value={String(setting.value)}
        disabled={setting.locked}
        aria-describedby={`${id}-desc`}
        onChange={(e) => void change(e.target.value)}
      >
        {setting.schema.enum.map((v) => (
          <option key={v} value={v}>
            {t(`${optionsBase}.${v}`)}
          </option>
        ))}
      </select>
    );
  }

  return (
    <div
      className={`setting${highlighted ? " highlight" : ""}`}
      data-key={setting.key}
      onContextMenu={(e) => {
        e.preventDefault();
        setMenu({ open: true, x: e.clientX, y: e.clientY });
      }}
    >
      <label className="label" htmlFor={id} id={`${id}-label`}>
        {setting.label}
      </label>
      <div className="control">{control}</div>
      <div className="description" id={`${id}-desc`}>
        {setting.description}
        {setting.locked && (
          <div className="locked">
            <Icon name="lock" />
            {t("ui.prefs.locked")}
          </div>
        )}
        {error && (
          <div className="error-text" role="alert">
            <Icon name="alert" />
            {error}
          </div>
        )}
      </div>
      <ContextMenu
        open={menu.open}
        x={menu.x}
        y={menu.y}
        label={setting.label}
        onClose={() => setMenu((m) => ({ ...m, open: false }))}
        items={[
          {
            id: "copy-key",
            label: t("ui.prefs.copy_key"),
            trailing: <code>{setting.key}</code>,
            onSelect: () => void navigator.clipboard.writeText(setting.key),
          },
        ]}
      />
    </div>
  );
}

/** The glass intensity (APP-012, Cristal): previews while it moves, saved a moment after. */
function GlassSlider({
  id,
  setting,
  change,
}: {
  id: string;
  setting: SettingEntry;
  change: (value: unknown) => Promise<void>;
}) {
  // While it moves, the slider shows its own value; once saved, the setting's.
  const [moving, setMoving] = useState<number | null>(null);
  const value = moving ?? Number(setting.value);
  const timer = useRef<ReturnType<typeof setTimeout>>();
  useEffect(() => () => clearTimeout(timer.current), []);
  return (
    <div className="range-row">
      <input
        id={id}
        type="range"
        className="range"
        min={0}
        max={100}
        step={5}
        value={value}
        style={{ "--fill": `${value}%` } as CSSProperties}
        disabled={setting.locked}
        aria-describedby={`${id}-desc`}
        aria-valuetext={`${value} %`}
        onChange={(e) => {
          const next = Number(e.target.value);
          setMoving(next);
          applyGlass(next);
          clearTimeout(timer.current);
          timer.current = setTimeout(() => void change(next).then(() => setMoving(null)), 300);
        }}
      />
      <output htmlFor={id}>{value} %</output>
    </div>
  );
}

/** The accent color (APP-012, Cristal): one swatch per choice, as radio buttons. */
function AccentPicker({
  labelledBy,
  describedBy,
  setting,
  names,
  change,
}: {
  labelledBy: string;
  describedBy: string;
  setting: SettingEntry;
  names: (value: string) => string;
  change: (value: unknown) => Promise<void>;
}) {
  const options = setting.schema.enum ?? [];
  const current = String(setting.value);
  const group = useRef<HTMLDivElement>(null);
  const pick = (i: number) => {
    const v = options[(i + options.length) % options.length];
    if (v === undefined) return;
    void change(v);
    group.current?.querySelectorAll<HTMLButtonElement>("button")[options.indexOf(v)]?.focus();
  };
  return (
    <div
      ref={group}
      className="accent-picker"
      role="radiogroup"
      aria-labelledby={labelledBy}
      aria-describedby={describedBy}
      onKeyDown={(e) => {
        const i = options.indexOf(current);
        if (e.key === "ArrowRight" || e.key === "ArrowDown") {
          e.preventDefault();
          pick(i + 1);
        } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
          e.preventDefault();
          pick(i - 1);
        }
      }}
    >
      {options.map((v) => (
        <button
          key={v}
          type="button"
          role="radio"
          className={`accent-swatch ${v}`}
          aria-checked={v === current}
          aria-label={names(v)}
          title={names(v)}
          tabIndex={v === current ? 0 : -1}
          disabled={setting.locked}
          onClick={() => void change(v)}
        />
      ))}
    </div>
  );
}

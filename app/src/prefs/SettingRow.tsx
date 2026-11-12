import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Icon } from "../components/Icon";
import { ContextMenu } from "../components/Menu";
import { isCommandError, run, type SettingEntry } from "../lib/commands";

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
  if (setting.schema.type === "boolean") {
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
      <label className="label" htmlFor={id}>
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

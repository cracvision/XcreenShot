import { useState } from "react";
import { useTranslation } from "react-i18next";
import { isCommandError, run, type SettingEntry } from "../lib/commands";

interface Props {
  setting: SettingEntry;
  highlighted: boolean;
}

/** One preference: label, description and a control chosen from its schema (APP-006). */
export function SettingRow({ setting, highlighted }: Props) {
  const { t } = useTranslation();
  const [error, setError] = useState<string | null>(null);
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
  const id = `setting-${setting.key}`;
  const optionsBase = `settings.${setting.key.replace(".", "_")}.options`;

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

  const copyKey = async () => {
    setMenu(null);
    await navigator.clipboard.writeText(setting.key);
  };

  return (
    <div
      className={`setting${highlighted ? " highlight" : ""}`}
      data-key={setting.key}
      onContextMenu={(e) => {
        e.preventDefault();
        setMenu({ x: e.clientX, y: e.clientY });
      }}
    >
      <label className="label" htmlFor={id}>
        {setting.label}
      </label>
      <div className="control">{control}</div>
      <div className="description" id={`${id}-desc`}>
        {setting.description}
        {setting.locked && <div className="locked">🔒 {t("ui.prefs.locked")}</div>}
        {error && (
          <div className="error-text" role="alert">
            {error}
          </div>
        )}
      </div>
      {menu && (
        <div
          className="context-menu"
          style={{ left: menu.x, top: menu.y }}
          onMouseLeave={() => setMenu(null)}
        >
          <button autoFocus onClick={() => void copyKey()} onBlur={() => setMenu(null)}>
            {t("ui.prefs.copy_key")} <code>{setting.key}</code>
          </button>
        </div>
      )}
    </div>
  );
}

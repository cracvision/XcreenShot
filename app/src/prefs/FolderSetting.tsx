// A setting that is a folder, chosen with the system's dialog or none (the receive
// folder, CAP-043).
import { open } from "@tauri-apps/plugin-dialog";
import { useTranslation } from "react-i18next";
import { run, type SettingEntry } from "../lib/commands";

export function FolderSetting({
  setting,
  highlighted,
}: {
  setting: SettingEntry;
  highlighted: boolean;
}) {
  const { t } = useTranslation();
  const value = typeof setting.value === "string" ? setting.value : "";
  const set = (v: string) => void run("settings.set", { key: setting.key, value: v });
  const choose = async () => {
    const picked = await open({ directory: true, multiple: false });
    if (typeof picked === "string") set(picked);
  };
  return (
    <div className={`setting${highlighted ? " highlight" : ""}`} data-key={setting.key}>
      <span className="label">{setting.label}</span>
      <div className="control row">
        <button
          type="button"
          className="btn"
          disabled={setting.locked}
          onClick={() => void choose()}
        >
          {t("ui.prefs.folder_choose")}
        </button>
        {value && (
          <button type="button" className="btn" disabled={setting.locked} onClick={() => set("")}>
            {t("ui.prefs.folder_clear")}
          </button>
        )}
      </div>
      <div className="description">
        {setting.description}
        <div>
          <code className="path">{value || t("ui.prefs.folder_none")}</code>
        </div>
      </div>
    </div>
  );
}

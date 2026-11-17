// Preferences > Advanced, the parts that are not settings (APP-009): where the app keeps
// its data, and the terminal command. And Preferences > Experimental features
// (APP-034), empty until a feature ships as a trial.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";

export function AdvancedExtras() {
  const { t } = useTranslation();
  const [data, setData] = useState<{ data: string; config: string } | null>(null);
  const [cli, setCli] = useState<string | null | undefined>(undefined);
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    run<{ data: string; config: string }>("app.open_data_folder", { open: false })
      .then(setData)
      .catch(() => undefined);
    run<{ command: string }>("app.mcp_config")
      .then((r) => setCli(r.command))
      .catch(() => setCli(null));
  }, []);
  return (
    <>
      {data && (
        <div className="setting">
          <span className="label">{t("ui.prefs.data_folder")}</span>
          <div className="control">
            <button type="button" className="btn" onClick={() => void run("app.open_data_folder")}>
              {t("ui.prefs.open")}
            </button>
          </div>
          <div className="description">
            <code className="path">{data.data}</code>
            <div>{t("ui.prefs.data_folder_help", { config: data.config })}</div>
          </div>
        </div>
      )}
      {cli !== undefined && (
        <div className="setting">
          <span className="label">{t("ui.prefs.cli_title")}</span>
          <div className="control">
            {cli && (
              <button
                type="button"
                className="btn"
                onClick={() => void navigator.clipboard.writeText(cli).then(() => setCopied(true))}
              >
                {t("ui.prefs.copy_path")}
              </button>
            )}
          </div>
          <div className="description" role="status">
            {cli ? <code className="path">{cli}</code> : t("ui.prefs.cli_missing")}
            <div>{copied ? t("ui.prefs.path_copied") : t("ui.prefs.cli_help")}</div>
          </div>
        </div>
      )}
    </>
  );
}

/** Features on trial (APP-034): the notice, then their switches (or that there are none). */
export function ExperimentalSection({ empty }: { empty: boolean }) {
  const { t } = useTranslation();
  return (
    <>
      <p className="section-intro">{t("ui.prefs.experimental_intro")}</p>
      {empty && <p className="empty">{t("ui.prefs.experimental_none")}</p>}
    </>
  );
}

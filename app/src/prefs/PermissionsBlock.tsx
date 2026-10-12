import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { useApp } from "../store";

/** Preferences > General > Permissions and "Fix permissions" (APP-024). */
export function PermissionsBlock() {
  const { t } = useTranslation();
  const permissions = useApp((s) => s.permissions);

  if (permissions.length === 0) {
    return <p className="muted">{t("ui.permissions.none_needed")}</p>;
  }
  return (
    <div>
      <p className="muted" style={{ marginTop: 0 }}>
        {t("ui.permissions.help")}
      </p>
      {permissions.map((p) => (
        <div className="permission" key={p.permission}>
          <div>
            <strong>{t(`ui.permissions.name.${p.permission}`)}</strong>
            {" · "}
            <span>{t(`ui.permissions.state.${p.state}`)}</span>
          </div>
          <div className="row">
            {p.state !== "granted" && p.can_request && (
              <button
                className="btn small"
                onClick={() => void run("app.request_permission", { permission: p.permission })}
              >
                {t("ui.permissions.request")}
              </button>
            )}
            {p.state !== "granted" && (
              <button
                className="btn small"
                onClick={() =>
                  void run("app.open_permission_settings", { permission: p.permission })
                }
              >
                {t("ui.permissions.open_settings")}
              </button>
            )}
          </div>
          <div className="muted" style={{ fontSize: 13 }}>
            {t(`ui.permissions.purpose.${p.permission}`)}
          </div>
        </div>
      ))}
    </div>
  );
}

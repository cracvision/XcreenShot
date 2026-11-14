import { useTranslation } from "react-i18next";
import { run, type PermissionStatus } from "../lib/commands";
import { useApp } from "../store";

const IS_MAC = navigator.userAgent.includes("Mac");

const BADGE: Record<PermissionStatus["state"], string> = {
  granted: "ok",
  denied: "err",
  not_determined: "warn",
};

/** Preferences > General > Permissions and "Fix permissions" (APP-024). */
export function PermissionsBlock() {
  const { t } = useTranslation();
  const permissions = useApp((s) => s.permissions);

  if (permissions.length === 0) {
    return <p className="muted">{t("ui.permissions.none_needed")}</p>;
  }
  return (
    <div>
      {/* The help is about the macOS privacy list (APP-024). */}
      {IS_MAC && (
        <p className="muted" style={{ marginTop: 0 }}>
          {t("ui.permissions.help")}
        </p>
      )}
      {permissions.map((p) => (
        <div className="permission" key={p.permission}>
          <div className="head">
            <strong>{t(`ui.permissions.name.${p.permission}`)}</strong>
            <span className={`badge ${BADGE[p.state]}`}>
              {t(`ui.permissions.state.${p.state}`)}
            </span>
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
          <div className="purpose">{t(`ui.permissions.purpose.${p.permission}`)}</div>
        </div>
      ))}
    </div>
  );
}

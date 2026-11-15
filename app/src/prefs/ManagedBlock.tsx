// Advanced > Managed configuration (APP-035): its state and, with View, where the
// file is, its date, what it fixes or starts and what was ignored.
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useManaged } from "../lib/managed";

export function ManagedBlock() {
  const { t, i18n } = useTranslation();
  const m = useManaged();
  const [open, setOpen] = useState(false);
  if (!m) return null;
  const date = m.modified
    ? new Date(m.modified).toLocaleString(i18n.language, {
        dateStyle: "medium",
        timeStyle: "short",
      })
    : "";
  const list = (o: Record<string, unknown>) =>
    Object.entries(o).map(([k, v]) => (
      <li key={k}>
        <code>{k}</code> = <code>{JSON.stringify(v)}</code>
      </li>
    ));
  return (
    <section className="managed-block" aria-label={t("ui.managed.title")}>
      <h2>{t("ui.managed.title")}</h2>
      <p className="muted">{t(`ui.managed.status_${m.status}`)}</p>
      {m.status !== "inactive" && (
        <button
          type="button"
          className="btn small"
          aria-expanded={open}
          onClick={() => setOpen((o) => !o)}
        >
          {t("ui.managed.view")}
        </button>
      )}
      {open && (
        <div className="managed-details">
          <p>
            {t("ui.managed.file", { path: m.path })}
            {date && ` · ${date}`}
          </p>
          {Object.keys(m.locked).length > 0 && (
            <>
              <h3>{t("ui.managed.locked")}</h3>
              <ul>{list(m.locked)}</ul>
            </>
          )}
          {Object.keys(m.defaults).length > 0 && (
            <>
              <h3>{t("ui.managed.defaults")}</h3>
              <ul>{list(m.defaults)}</ul>
            </>
          )}
          {Object.keys(m.policy).length > 0 && (
            <>
              <h3>{t("ui.managed.policy")}</h3>
              <ul>{list(m.policy)}</ul>
            </>
          )}
          {m.errors.length > 0 && (
            <>
              <h3>{t("ui.managed.errors")}</h3>
              <ul className="managed-errors">
                {m.errors.map((e) => (
                  <li key={e}>{e}</li>
                ))}
              </ul>
            </>
          )}
          <p className="muted">{t("ui.managed.restart")}</p>
        </div>
      )}
    </section>
  );
}

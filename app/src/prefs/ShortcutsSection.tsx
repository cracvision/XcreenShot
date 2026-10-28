import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { display } from "../lib/accelerator";
import { isCommandError, run, type ShortcutRow } from "../lib/commands";
import { useApp } from "../store";
import { SettingRow } from "./SettingRow";
import { ShortcutRecorder } from "./ShortcutRecorder";

type Filter = "all" | "global" | "app" | "editor" | "conflict" | "custom";

interface Reassign {
  row: ShortcutRow;
  shortcut: string;
  otherTitle: string;
}

interface ProfilePreview {
  profile: string;
  changes: { command: string; from: string | null; to: string | null }[];
  conflicts: { slot: string; with: string; shortcut: string }[];
}

/** Preferences > Shortcuts (APP-018 to APP-021). */
export function ShortcutsSection({ highlightKey }: { highlightKey: string | null }) {
  const { t } = useTranslation();
  const { shortcuts, layout, settings } = useApp();
  const [filter, setFilter] = useState<Filter>("all");
  const [query, setQuery] = useState("");
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [reassign, setReassign] = useState<Reassign | null>(null);
  const [profilePreview, setProfilePreview] = useState<ProfilePreview | null>(null);

  const platform = shortcuts?.platform ?? "windows";
  const rowKey = (r: ShortcutRow) => `${r.command}@${r.scope}`;
  const titleOf = useCallback(
    (command: string) => shortcuts?.rows.find((r) => r.command === command)?.title ?? command,
    [shortcuts],
  );

  const assign = useCallback(
    async (row: ShortcutRow, shortcut: string | null, reassignOther = false) => {
      setErrors((e) => ({ ...e, [rowKey(row)]: "" }));
      try {
        await run("shortcuts.set", {
          command: row.command,
          scope: row.scope,
          shortcut,
          reassign: reassignOther,
        });
      } catch (e) {
        if (isCommandError(e) && e.details?.["conflict"] === "duplicate" && shortcut) {
          setReassign({ row, shortcut, otherTitle: titleOf(String(e.details["command"])) });
        } else {
          setErrors((x) => ({ ...x, [rowKey(row)]: isCommandError(e) ? e.message : String(e) }));
        }
      }
    },
    [titleOf],
  );

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase();
    return (shortcuts?.rows ?? []).filter((r) => {
      if (filter === "global" && r.scope !== "global") return false;
      if (filter === "app" && r.scope !== "app") return false;
      if (filter === "editor" && r.scope !== "editor") return false;
      if (filter === "conflict" && !r.conflict) return false;
      if (filter === "custom" && !r.custom) return false;
      if (!q) return true;
      const shown = r.shortcut ? display(r.shortcut, platform, layout).toLowerCase() : "";
      return r.title.toLowerCase().includes(q) || r.command.includes(q) || shown.includes(q);
    });
  }, [shortcuts, filter, query, platform, layout]);

  const changeProfile = async (profile: string) => {
    const preview = await run<Omit<ProfilePreview, "profile">>("shortcuts.set_profile", {
      profile,
      dryRun: true,
    });
    setProfilePreview({ profile, ...preview });
  };

  const applyProfile = async (resetCustom: boolean) => {
    if (!profilePreview) return;
    await run("shortcuts.set_profile", { profile: profilePreview.profile, resetCustom });
    setProfilePreview(null);
  };

  if (!shortcuts) return null;
  const takeOver = settings.find((s) => s.key === "shortcuts.takeOver");

  return (
    <div>
      <div className="setting">
        <label className="label" htmlFor="shortcut-profile">
          {t("settings.shortcuts_profile.label")}
        </label>
        <div className="control">
          <select
            id="shortcut-profile"
            value={shortcuts.profile}
            onChange={(e) => void changeProfile(e.target.value)}
          >
            <option value="coexistence">
              {t("settings.shortcuts_profile.options.coexistence")}
            </option>
            <option value="compatibility">
              {t("settings.shortcuts_profile.options.compatibility")}
            </option>
          </select>
        </div>
        <div className="description">{t(`ui.shortcuts.profile_help_${shortcuts.profile}`)}</div>
      </div>
      {takeOver && <SettingRow setting={takeOver} highlighted={highlightKey === takeOver.key} />}

      <div className="toolbar">
        <input
          type="search"
          placeholder={t("ui.shortcuts.search")}
          aria-label={t("ui.shortcuts.search")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <label>
          {t("ui.shortcuts.filter")}{" "}
          <select value={filter} onChange={(e) => setFilter(e.target.value as Filter)}>
            {(["all", "global", "app", "editor", "conflict", "custom"] as const).map((f) => (
              <option key={f} value={f}>
                {t(`ui.shortcuts.filter_${f}`)}
              </option>
            ))}
          </select>
        </label>
      </div>

      <table className="shortcuts">
        <thead>
          <tr>
            <th scope="col">{t("ui.shortcuts.col_command")}</th>
            <th scope="col">{t("ui.shortcuts.col_shortcut")}</th>
            <th scope="col">{t("ui.shortcuts.col_status")}</th>
            <th scope="col" />
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={rowKey(r)}>
              <td>
                <div>{r.title}</div>
                <div className="muted" style={{ fontSize: 12 }}>
                  {t(`ui.shortcuts.scope_${r.scope}`)}
                </div>
              </td>
              <td>
                <ShortcutRecorder
                  value={r.shortcut}
                  platform={platform}
                  layout={layout}
                  conflict={r.conflict}
                  label={r.title}
                  onCommit={(s) => void assign(r, s)}
                />
                {errors[rowKey(r)] && (
                  <div className="error-text" role="alert">
                    {errors[rowKey(r)]}
                  </div>
                )}
              </td>
              <td>
                <span className={r.conflict ? "status-conflict" : "muted"}>{r.statusText}</span>
                {r.conflict && (
                  <div className="row" style={{ marginTop: 4 }}>
                    <button className="btn small" onClick={() => void run("shortcuts.retry")}>
                      {t("ui.shortcuts.retry")}
                    </button>
                    {r.status.kind === "system_assigned" && r.status.settings_uri && (
                      <button
                        className="btn small"
                        onClick={() => void run("shortcuts.open_system_settings")}
                      >
                        {t("ui.shortcuts.open_system_settings")}
                      </button>
                    )}
                  </div>
                )}
              </td>
              <td>
                <div className="cell-actions">
                  <button
                    className="btn icon"
                    title={t("ui.shortcuts.restore")}
                    aria-label={`${t("ui.shortcuts.restore")}: ${r.title}`}
                    disabled={!r.custom}
                    onClick={() =>
                      void run("shortcuts.reset", { command: r.command, scope: r.scope })
                    }
                  >
                    ↺
                  </button>
                  <button
                    className="btn icon"
                    title={t("ui.shortcuts.remove")}
                    aria-label={`${t("ui.shortcuts.remove")}: ${r.title}`}
                    disabled={!r.shortcut}
                    onClick={() => void assign(r, null)}
                  >
                    ✕
                  </button>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="muted" style={{ fontSize: 13 }}>
        {t("ui.shortcuts.contextual_note")}
      </p>

      {reassign && (
        <Dialog
          title={t("ui.shortcuts.reassign_title")}
          onCancel={() => setReassign(null)}
          actions={
            <>
              <button className="btn" onClick={() => setReassign(null)}>
                {t("ui.shortcuts.cancel")}
              </button>
              <button
                className="btn primary"
                onClick={() => {
                  const r = reassign;
                  setReassign(null);
                  void assign(r.row, r.shortcut, true);
                }}
              >
                {t("ui.shortcuts.reassign")}
              </button>
            </>
          }
        >
          <p style={{ margin: 0 }}>
            {t("ui.shortcuts.reassign_body", {
              shortcut: display(reassign.shortcut, platform, layout),
              command: reassign.otherTitle,
            })}
          </p>
        </Dialog>
      )}

      {profilePreview && (
        <Dialog
          title={t("ui.shortcuts.profile_title")}
          onCancel={() => setProfilePreview(null)}
          actions={
            <>
              <button className="btn" onClick={() => setProfilePreview(null)}>
                {t("ui.shortcuts.cancel")}
              </button>
              <button className="btn" onClick={() => void applyProfile(true)}>
                {t("ui.shortcuts.profile_apply_reset")}
              </button>
              <button className="btn primary" onClick={() => void applyProfile(false)}>
                {t("ui.shortcuts.profile_apply")}
              </button>
            </>
          }
        >
          {profilePreview.changes.length === 0 ? (
            <p style={{ margin: 0 }}>{t("ui.shortcuts.profile_no_changes")}</p>
          ) : (
            <>
              <p style={{ margin: 0 }}>{t("ui.shortcuts.profile_changes")}</p>
              <ul>
                {profilePreview.changes.map((c) => (
                  <li key={c.command + c.from}>
                    {titleOf(c.command)}: <kbd>{c.from ?? t("ui.shortcuts.none")}</kbd> →{" "}
                    <kbd>{c.to ?? t("ui.shortcuts.none")}</kbd>
                  </li>
                ))}
              </ul>
            </>
          )}
          {profilePreview.conflicts.length > 0 && (
            <>
              <p style={{ margin: 0 }}>{t("ui.shortcuts.profile_conflicts")}</p>
              <ul>
                {profilePreview.conflicts.map((c) => (
                  <li key={c.slot + c.with}>
                    <kbd>{c.shortcut}</kbd>
                  </li>
                ))}
              </ul>
            </>
          )}
        </Dialog>
      )}
    </div>
  );
}

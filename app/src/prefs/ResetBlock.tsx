// Restore defaults (APP-010): "Reset this section" at the end of each section, "Reset
// all settings…" in Advanced, and "Show warnings and hints again" in General. Each runs
// a command (settings.reset, app.warnings); captures and the library are never touched.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { isCommandError, run } from "../lib/commands";
import { useApp } from "../store";

function Confirm({
  title,
  body,
  action,
  onDone,
}: {
  title: string;
  body: string;
  action: string;
  onDone: (confirmed: boolean) => void;
}) {
  const { t } = useTranslation();
  return (
    <Dialog
      title={title}
      onCancel={() => onDone(false)}
      actions={
        <>
          <button type="button" className="btn" onClick={() => onDone(false)}>
            {t("ui.common.cancel")}
          </button>
          <button type="button" className="btn primary" onClick={() => onDone(true)}>
            {action}
          </button>
        </>
      }
    >
      <p>{body}</p>
    </Dialog>
  );
}

/** One button that resets a section, or everything, after asking. */
function ResetButton({ section }: { section: string | null }) {
  const { t } = useTranslation();
  const [asking, setAsking] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const all = section === null;
  const name = all ? "" : t(`ui.prefs.section.${section}`);
  const done = async (confirmed: boolean) => {
    setAsking(false);
    if (!confirmed) return;
    try {
      const r = await run<{ reset: string[] }>("settings.reset", section ? { section } : {});
      await useApp.getState().refreshSettings();
      setMessage(t("ui.prefs.reset_done", { count: r.reset.length }));
    } catch (e) {
      setMessage(isCommandError(e) ? e.message : String(e));
    }
  };
  return (
    <div className="reset-block">
      <button type="button" className="btn" onClick={() => setAsking(true)}>
        {all ? t("ui.prefs.reset_all") : t("ui.prefs.reset_section")}
      </button>
      {message && (
        <span className="muted" role="status">
          {message}
        </span>
      )}
      {asking && (
        <Confirm
          title={all ? t("ui.prefs.reset_all_title") : t("ui.prefs.reset_section_title", { name })}
          body={all ? t("ui.prefs.reset_all_body") : t("ui.prefs.reset_section_body", { name })}
          action={t("ui.prefs.reset")}
          onDone={(c) => void done(c)}
        />
      )}
    </div>
  );
}

/** At the end of a section (APP-010). */
export function ResetSection({ section }: { section: string }) {
  return <ResetButton section={section} />;
}

/** In Advanced: every setting, the shortcut profile and every shortcut (APP-010). */
export function ResetAll() {
  return <ResetButton section={null} />;
}

/** In General: the warnings answered with "Don't ask again" and the hints seen. */
export function ShowWarningsAgain() {
  const { t } = useTranslation();
  const [hidden, setHidden] = useState<number | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  useEffect(() => {
    run<{ hidden: number }>("app.warnings")
      .then((r) => setHidden(r.hidden))
      .catch(() => setHidden(0));
  }, []);
  const restore = async () => {
    const r = await run<{ restored: number }>("app.warnings", { reset: true });
    await useApp.getState().refreshSettings();
    setHidden(0);
    setMessage(t("ui.prefs.warnings_restored", { count: r.restored }));
  };
  return (
    <div className="setting">
      <span className="label">{t("ui.prefs.warnings_title")}</span>
      <div className="control">
        <button type="button" className="btn" disabled={!hidden} onClick={() => void restore()}>
          {t("ui.prefs.warnings_again")}
        </button>
      </div>
      <div className="description" role="status">
        {message ??
          (hidden ? t("ui.prefs.warnings_hidden", { count: hidden }) : t("ui.prefs.warnings_none"))}
      </div>
    </div>
  );
}

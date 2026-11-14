// Help > Move XcreenShot to another computer… (APP-055, LIB-032): one dialog with the
// two things to take (the library folder and an .xsettings file) and what to do on the
// new computer.
import { save } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { isCommandError, run } from "../lib/commands";
import { fileName, libraryStep, useMoveComputer } from "./moveComputerState";

interface Where {
  root: string;
  place: string;
  service: string | null;
}

export function MoveComputer() {
  const { t } = useTranslation();
  const open = useMoveComputer((s) => s.open);
  const [where, setWhere] = useState<Where | null>(null);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    if (!open) return;
    run<Where>("library.list", { limit: 1 })
      .then(setWhere)
      .catch(() => undefined);
  }, [open]);

  if (!open) return null;
  const close = () => {
    setStatus("");
    setError("");
    useMoveComputer.setState({ open: false });
  };
  const exportSettings = async () => {
    setError("");
    const path = await save({
      defaultPath: "XcreenShot.xsettings",
      filters: [{ name: t("ui.prefs.transfer.file_type"), extensions: ["xsettings"] }],
    });
    if (!path) return;
    try {
      await run("settings.export", { out: path });
      setStatus(t("ui.move_computer.exported", { name: fileName(path) }));
    } catch (e) {
      setError(isCommandError(e) ? e.message : String(e));
    }
  };
  const step = where ? libraryStep(where, t) : null;
  return (
    <Dialog
      title={t("ui.move_computer.title")}
      onCancel={close}
      actions={
        <button type="button" className="btn primary" data-autofocus onClick={close}>
          {t("ui.editor.close")}
        </button>
      }
    >
      <div className="move-computer">
        <p>{t("ui.move_computer.intro")}</p>
        <section aria-labelledby="mc-library">
          <h3 id="mc-library">{t("ui.move_computer.library_title")}</h3>
          {step && <p>{step.text}</p>}
          {where && (
            <div className="mc-row">
              <code className="path" title={where.root}>
                {where.root}
              </code>
              <button
                type="button"
                className="btn"
                onClick={() => void run("library.show_in_folder").catch(() => undefined)}
              >
                {t("ui.move_computer.show")}
              </button>
            </div>
          )}
        </section>
        <section aria-labelledby="mc-settings">
          <h3 id="mc-settings">{t("ui.move_computer.settings_title")}</h3>
          <p className="muted">{t("ui.move_computer.settings_help")}</p>
          <div className="mc-row">
            <button type="button" className="btn" onClick={() => void exportSettings()}>
              {t("ui.move_computer.export")}
            </button>
            <span role="status">
              {error ? <span className="warning-text">{error}</span> : status}
            </span>
          </div>
        </section>
        <section aria-labelledby="mc-new">
          <h3 id="mc-new">{t("ui.move_computer.new_title")}</h3>
          <p>{t("ui.move_computer.new_steps")}</p>
          <p className="muted">{t("ui.move_computer.formats")}</p>
        </section>
      </div>
    </Dialog>
  );
}

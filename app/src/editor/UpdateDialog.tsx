// A new version (APP-036): the current and the new one, its notes in the app's language
// (English when there is no translation), and Install and restart, Install when
// closing, Not now. From Help > Check for updates, also "you have the latest".
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import { useEditor } from "./state";
import { notesFor } from "./updateNotes";

interface Found {
  available: boolean;
  current: string;
  version: string | null;
  notes: string | null;
}

export function UpdateDialog() {
  const { t, i18n } = useTranslation();
  const [found, setFound] = useState<Found | null>(null);
  useEffect(() => {
    const off = listen<{ kind: string; payload: Found }>("xs://editor", (e) => {
      if (e.payload.kind === "update") setFound(e.payload.payload);
    });
    return () => void off.then((f) => f());
  }, []);
  if (!found) return null;
  const close = () => setFound(null);
  const install = (when: "now" | "quit") =>
    void run("app.install_update", { when })
      .then(() => {
        if (when === "quit") useEditor.getState().showToast(t("ui.update.at_quit"));
        close();
      })
      .catch((e: unknown) => {
        close();
        useEditor.getState().showError(e);
      });
  if (!found.available)
    return (
      <Dialog
        title={t("ui.update.title")}
        onCancel={close}
        actions={
          <button type="button" className="btn primary" onClick={close}>
            {t("ui.update.close")}
          </button>
        }
      >
        <p>{t("ui.update.latest", { current: found.current })}</p>
      </Dialog>
    );
  const notes = notesFor(found.notes, i18n.language);
  return (
    <Dialog
      title={t("ui.update.title")}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.update.later")}
          </button>
          <button type="button" className="btn" onClick={() => install("quit")}>
            {t("ui.update.install_quit")}
          </button>
          <button type="button" className="btn primary" onClick={() => install("now")}>
            {t("ui.update.install_now")}
          </button>
        </>
      }
    >
      <p>{t("ui.update.available", { version: found.version, current: found.current })}</p>
      {notes && (
        <>
          <h3>{t("ui.update.notes")}</h3>
          <div className="update-notes">{notes}</div>
        </>
      )}
    </Dialog>
  );
}

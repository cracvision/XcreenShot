// "Waiting to send to: File" over the canvas, with Send (Ctrl+Enter) and Don't send
// (CAP-052): a capture opened in the editor goes to its destinations only when the
// user says so, as it is then.
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { useEditor } from "./state";

export function PendingShare() {
  const { t } = useTranslation();
  const active = useEditor((s) => s.active);
  const destinations = useEditor((s) => (s.active ? s.pendingShare[s.active] : undefined));
  const answer = (send: boolean) => {
    if (active)
      void run("capture.send_pending", { documentId: active, send }).catch(() => undefined);
  };

  useEffect(() => {
    if (!destinations || !active) return;
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const typing =
        target &&
        (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable);
      if (!typing && e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        void run("capture.send_pending", { documentId: active, send: true }).catch(() => undefined);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [destinations, active]);

  if (!destinations || !active) return null;
  return (
    <div
      className="pending-share"
      role="region"
      aria-label={t("ui.editor.pending_share", { destinations: destinations.join(", ") })}
    >
      <span>{t("ui.editor.pending_share", { destinations: destinations.join(", ") })}</span>
      <button
        type="button"
        className="btn primary small"
        title="Ctrl+Enter"
        onClick={() => answer(true)}
      >
        {t("ui.editor.send")}
      </button>
      <button type="button" className="btn small" onClick={() => answer(false)}>
        {t("ui.editor.dont_send")}
      </button>
    </div>
  );
}

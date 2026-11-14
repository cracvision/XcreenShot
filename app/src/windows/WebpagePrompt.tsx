// The web address box (CAP-037): the address (filled from the clipboard when it holds
// one), Enter captures and Esc cancels. While the page loads it shows the address and
// Cancel; it closes when the capture is made.
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { isCommandError, run } from "../lib/commands";
import { useAppShortcuts } from "../lib/appShortcuts";

export function WebpagePrompt() {
  const { t } = useTranslation();
  useAppShortcuts();
  const [url, setUrl] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const close = useCallback(() => void getCurrentWebviewWindow().close(), []);

  useEffect(() => {
    void run<{ url: string }>("capture.webpage_suggest")
      .then((s) => setUrl(s.url))
      .catch(() => undefined);
  }, []);

  const start = async () => {
    if (!url.trim() || loading) return;
    setError(null);
    setLoading(true);
    try {
      await run("capture.webpage", { url: url.trim() });
      close();
    } catch (e) {
      setLoading(false);
      if (isCommandError(e) && e.code === "cancelled") return;
      setError(isCommandError(e) ? e.message : String(e));
    }
  };

  const cancel = () => {
    if (loading) void run("capture.webpage_cancel").catch(() => undefined);
    else close();
  };

  return (
    <form
      className="webpage-prompt"
      onSubmit={(e) => {
        e.preventDefault();
        void start();
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          cancel();
        }
      }}
    >
      <label className="panel-field">
        <span>{t("ui.webpage.url")}</span>
        <input
          type="url"
          autoFocus
          spellCheck={false}
          value={url}
          disabled={loading}
          placeholder="https://"
          onChange={(e) => setUrl(e.target.value)}
          onFocus={(e) => e.target.select()}
        />
      </label>
      {loading ? (
        <div className="pdf-progress" role="status">
          <progress />
          <span>{t("ui.webpage.loading", { url: url.trim() })}</span>
        </div>
      ) : (
        <p className="muted webpage-note">{error ?? t("ui.webpage.private")}</p>
      )}
      {error && loading === false && (
        <span className="sr-only" role="alert">
          {error}
        </span>
      )}
      <div className="pdf-actions">
        <button type="submit" className="btn primary" disabled={!url.trim() || loading}>
          {t("ui.webpage.capture")}
        </button>
        <button type="button" className="btn" onClick={cancel}>
          {t("ui.webpage.cancel")}
        </button>
      </div>
    </form>
  );
}

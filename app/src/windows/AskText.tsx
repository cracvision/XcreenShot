// The text of Capture details, asked when a capture ends (CAP-051): Enter adds it, Esc
// (or closing the box) leaves it empty; the capture goes on either way.
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";

export function AskText() {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const answered = useRef(false);

  const answer = (value: string) => {
    if (answered.current) return;
    answered.current = true;
    void run("capture.details_text", { text: value }).catch(() => undefined);
  };

  useEffect(() => {
    // Closing the box is no text.
    const off = getCurrentWebviewWindow().onCloseRequested(() => answer(""));
    return () => void off.then((f) => f());
  }, []);

  return (
    <form
      className="webpage-prompt"
      onSubmit={(e) => {
        e.preventDefault();
        answer(text.trim());
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          answer("");
        }
      }}
    >
      <label className="panel-field">
        <span>{t("ui.ask_text.label")}</span>
        <input
          type="text"
          autoFocus
          value={text}
          maxLength={500}
          onChange={(e) => setText(e.target.value)}
        />
      </label>
      <p className="muted webpage-note">{t("ui.ask_text.hint")}</p>
      <div className="pdf-actions">
        <button type="submit" className="btn primary">
          {t("ui.ask_text.add")}
        </button>
        <button type="button" className="btn" onClick={() => answer("")}>
          {t("ui.ask_text.skip")}
        </button>
      </div>
    </form>
  );
}

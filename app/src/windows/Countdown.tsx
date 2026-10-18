// Delay countdown (CAP-048). The window never takes focus: Cancel works with a click
// and Esc is a temporary global shortcut owned by the shell.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

interface Info {
  seconds: number;
  /** Epoch milliseconds when the capture happens. */
  endsAt: number;
}

export function Countdown() {
  const { t } = useTranslation();
  const [info, setInfo] = useState<Info | null>(null);
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    void invoke<Info | null>("xs_countdown_current").then((i) => i && setInfo(i));
    const un = listen<Info>("xs://countdown", (e) => {
      setNow(Date.now());
      setInfo(e.payload);
    });
    return () => void un.then((f) => f());
  }, []);

  useEffect(() => {
    if (!info) return;
    const timer = window.setInterval(() => setNow(Date.now()), 100);
    return () => window.clearInterval(timer);
  }, [info]);

  const left = info ? Math.max(0, Math.ceil((info.endsAt - now) / 1000)) : 0;

  return (
    <div className="countdown" role="timer" aria-live="off">
      <div className="countdown-number" aria-hidden="true">
        {left}
      </div>
      <div className="countdown-text">
        <div>{t("ui.countdown.label", { seconds: left })}</div>
        <div className="muted">{t("ui.countdown.hint")}</div>
      </div>
      <button type="button" className="btn" onClick={() => void invoke("xs_countdown_cancel")}>
        {t("ui.countdown.cancel")}
      </button>
    </div>
  );
}

// One-time hints (APP-014): short local tips, one at a time, each with "Got it" and
// "Don't show more hints". No videos, tours, surveys or news.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { settingValue, useApp } from "../store";

const IS_WINDOWS = navigator.userAgent.includes("Windows");

interface HintDef {
  id: string;
  /** Whether it applies now. */
  when: (state: { trayItems: number }) => boolean;
}

/** In the order they may appear, one at a time. */
const HINTS: HintDef[] = [
  // Windows hides new tray icons: where to make it always visible.
  { id: "tray_icon", when: () => IS_WINDOWS },
  { id: "tray_drag", when: ({ trayItems }) => trayItems > 0 },
];

/** The first hint not seen yet that applies, as a small bubble (APP-014). */
export function HintBubble({ trayItems }: { trayItems: number }) {
  const { t } = useTranslation();
  const [seen, setSeen] = useState<string[] | null>(null);
  const [enabled, setEnabled] = useState(false);
  useEffect(() => {
    run<{ enabled: boolean; seen: string[] }>("app.hints")
      .then((r) => {
        setEnabled(r.enabled);
        setSeen(r.seen);
      })
      .catch(() => undefined);
  }, []);
  const on = useApp((s) => settingValue(s.settings, "general.hints") !== false);
  if (!seen || !enabled || !on) return null;
  const hint = HINTS.find((h) => !seen.includes(h.id) && h.when({ trayItems }));
  if (!hint) return null;
  const gotIt = () => {
    setSeen([...seen, hint.id]);
    void run("app.hints", { seen: hint.id }).catch(() => undefined);
  };
  const noMore = () => {
    setEnabled(false);
    void run("app.hints", { seen: hint.id }).catch(() => undefined);
    void run("settings.set", { key: "general.hints", value: false }).catch(() => undefined);
  };
  return (
    <aside className="hint-bubble" role="note" aria-label={t("ui.hints.label")}>
      <p>{t(`ui.hints.${hint.id}`)}</p>
      <div className="row">
        <button type="button" className="btn small primary" onClick={gotIt}>
          {t("ui.hints.got_it")}
        </button>
        <button type="button" className="btn small" onClick={noMore}>
          {t("ui.hints.no_more")}
        </button>
      </div>
    </aside>
  );
}

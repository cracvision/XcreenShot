// Main window placeholder for M0: the editor and library arrive in M1. It proves the
// exit criterion: the same test command answers the UI, the global shortcut and the CLI.
import { useState } from "react";
import { useTranslation } from "react-i18next";
import logo from "../../src-tauri/icons/128x128.png";
import { display } from "../lib/accelerator";
import { useAppShortcuts } from "../lib/appShortcuts";
import { isCommandError, run } from "../lib/commands";
import { useApp } from "../store";

export function Home() {
  const { t } = useTranslation();
  useAppShortcuts();
  const { ping, shortcuts, activity, layout } = useApp();
  const [result, setResult] = useState<string | null>(null);

  const pingRow = shortcuts?.rows.find((r) => r.command === "app.ping" && r.scope === "global");
  const platform = shortcuts?.platform ?? ping?.platform ?? "windows";
  const pingShortcut = pingRow?.shortcut
    ? display(pingRow.shortcut, platform, layout)
    : t("ui.shortcuts.none");

  const runPing = async () => {
    try {
      setResult(JSON.stringify(await run("app.ping"), null, 2));
    } catch (e) {
      setResult(isCommandError(e) ? `${e.code}: ${e.message}` : String(e));
    }
  };

  return (
    <div className="home">
      <header>
        <img src={logo} alt="" />
        <div className="grow">
          <h1>{t("ui.home.title")}</h1>
          <div className="muted">{t("ui.home.subtitle")}</div>
        </div>
        <button className="btn" onClick={() => void run("settings.open")}>
          {t("ui.home.open_preferences")}
        </button>
      </header>

      <section className="card" aria-labelledby="test-title">
        <h2 id="test-title">{t("ui.home.test_title")}</h2>
        <p className="muted" style={{ margin: 0 }}>
          {t("ui.home.test_help")}
        </p>
        <div className="row">
          <button className="btn primary" onClick={() => void runPing()}>
            {t("ui.home.run_ping")}
          </button>
          <span>
            {t("ui.home.shortcut_hint", { shortcut: "" })}
            <button
              className="btn small"
              title={t("ui.home.change_shortcut")}
              onClick={() => void run("settings.open", { section: "shortcuts" })}
            >
              <kbd>{pingShortcut}</kbd>
            </button>
          </span>
        </div>
        <div className="row">
          <span className="muted">{t("ui.home.cli_hint")}</span>
          <code>xcreenshot app ping</code>
        </div>
        {result && (
          <div>
            <div className="muted">{t("ui.home.result")}</div>
            <pre>{result}</pre>
          </div>
        )}
      </section>

      <section className="card" aria-labelledby="activity-title">
        <h2 id="activity-title">{t("ui.home.activity")}</h2>
        <ul className="activity" aria-live="polite">
          {activity.length === 0 && <li className="muted">{t("ui.home.activity_empty")}</li>}
          {activity.map((a) => (
            <li key={a.seq}>
              <span className={`badge ${a.ok ? "ok" : "err"}`}>{a.ok ? "OK" : a.error?.code}</span>
              <code>{a.id}</code>
              <span className="muted">{t(`source.${a.source}`)}</span>
              <span className="muted">{new Date(Number(a.at)).toLocaleTimeString()}</span>
            </li>
          ))}
        </ul>
      </section>
      {ping && (
        <div className="muted" style={{ fontSize: 12 }}>
          XcreenShot {ping.version} · {ping.platform} · {ping.arch}
        </div>
      )}
    </div>
  );
}

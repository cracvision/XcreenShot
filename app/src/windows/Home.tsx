// Main window placeholder for M0: the editor and library arrive in M1. It proves the
// exit criterion: the same test command answers the UI, the global shortcut and the CLI.
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { LanguageToggle } from "../components/LanguageToggle";
import logo from "../../src-tauri/icons/128x128.png";
import { display } from "../lib/accelerator";
import { useAppShortcuts } from "../lib/appShortcuts";
import { isCommandError, run } from "../lib/commands";
import { useApp } from "../store";

export function Home() {
  const { t } = useTranslation();
  useAppShortcuts();
  const { ping, shortcuts, activity, layout, lastCapture } = useApp();
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
        <LanguageToggle />
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
            <kbd>{pingShortcut}</kbd>
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

      <section className="card" aria-labelledby="capture-title">
        <h2 id="capture-title">{t("ui.home.last_capture")}</h2>
        <p className="muted" style={{ margin: 0 }}>
          {t("ui.home.capture_help")}
        </p>
        <div className="row">
          <button
            className="btn primary"
            onClick={() => void run("capture.universal").catch(() => undefined)}
          >
            {t("ui.home.capture_universal")}
          </button>
          <button
            className="btn"
            onClick={() => void run("capture.fullscreen", { delay: 3 }).catch(() => undefined)}
          >
            {t("ui.home.capture_fullscreen")}
          </button>
          <code>xcreenshot capture fullscreen</code>
        </div>
        {lastCapture ? (
          <figure className="capture-preview">
            {lastCapture.thumbnail && (
              <img
                src={lastCapture.thumbnail}
                alt={`${t("ui.home.last_capture")} ${lastCapture.result.width}×${lastCapture.result.height}`}
              />
            )}
            <figcaption className="muted">
              {lastCapture.result.width}×{lastCapture.result.height} px
              {lastCapture.result.metadata?.app?.name
                ? ` · ${lastCapture.result.metadata.app.name}`
                : ""}
              <br />
              {t("ui.home.saved_in")} <code>{lastCapture.result.path}</code>
            </figcaption>
          </figure>
        ) : (
          <p className="muted" style={{ margin: 0 }}>
            {t("ui.home.no_capture")}
          </p>
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
          XcreenShot {ping.version} ({ping.build}) · {ping.platform} · {ping.arch}
        </div>
      )}
    </div>
  );
}

// The welcome of the first run (APP-005): language, shortcuts, startup and library,
// and a test capture. Every step can be skipped; whatever is skipped keeps its default,
// and everything can be changed later in Preferences. No account, no data, no survey.
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { LanguageToggle } from "../components/LanguageToggle";
import { run } from "../lib/commands";
import { useApp } from "../store";

const IS_MAC = navigator.userAgent.includes("Mac");

interface ShortcutRow {
  command: string;
  scope: string;
  title: string;
  display: string | null;
  status?: { state: string };
}

const MAIN = ["capture.capture", "capture.universal", "capture.repeat_last"];

/** Closes the welcome; with `then`, runs a command afterwards (the test capture). */
async function finish(then?: string): Promise<void> {
  if (then) {
    // The capture goes on in the app after this window is gone.
    void run(then).catch(() => undefined);
    await new Promise((r) => setTimeout(r, 50));
  } else {
    await run("app.show").catch(() => undefined);
  }
  await getCurrentWebviewWindow().close();
}

function Shortcuts() {
  const { t } = useTranslation();
  const [answer, setAnswer] = useState<"yes" | "no" | null>(null);
  const [compat, setCompat] = useState(false);
  const [rows, setRows] = useState<ShortcutRow[]>([]);
  const load = () =>
    run<{ rows: ShortcutRow[] }>("shortcuts.list")
      .then((r) => setRows(r.rows.filter((x) => x.scope === "global" && MAIN.includes(x.command))))
      .catch(() => undefined);
  useEffect(() => void load(), []);
  const applyCompat = async () => {
    await run("shortcuts.set_profile", { profile: "compatibility" });
    setCompat(true);
    await load();
  };
  return (
    <>
      <p>{t("ui.welcome.shortcuts_question")}</p>
      <div className="row" role="group" aria-label={t("ui.welcome.shortcuts_question")}>
        {(["yes", "no"] as const).map((a) => (
          <button
            key={a}
            type="button"
            className={`btn${answer === a ? " primary" : ""}`}
            aria-pressed={answer === a}
            onClick={() => setAnswer(a)}
          >
            {t(`ui.welcome.shortcuts_${a}`)}
          </button>
        ))}
      </div>
      {answer === "yes" && <p className="muted">{t("ui.welcome.shortcuts_yes_body")}</p>}
      {answer === "no" && (
        <>
          <p className="muted">{t("ui.welcome.shortcuts_no_body")}</p>
          {compat ? (
            <p role="status">{t("ui.welcome.shortcuts_compat_on")}</p>
          ) : (
            <button type="button" className="btn" onClick={() => void applyCompat()}>
              {t("ui.welcome.shortcuts_use_compat")}
            </button>
          )}
        </>
      )}
      <p className="welcome-sub">{t("ui.welcome.shortcuts_main")}</p>
      <ul className="welcome-keys">
        {rows.map((r) => (
          <li key={r.command}>
            <span>{r.title}</span>
            <kbd>{r.display ?? "—"}</kbd>
            {r.status && r.status.state !== "active" && r.display && (
              <span className="warning-text">{t("ui.welcome.shortcuts_failed")}</span>
            )}
          </li>
        ))}
      </ul>
    </>
  );
}

function Startup() {
  const { t } = useTranslation();
  const settings = useApp((s) => s.settings);
  const login = settings.find((s) => s.key === "general.launchAtLogin");
  const [root, setRoot] = useState<string | null>(null);
  useEffect(() => {
    run<{ root: string }>("library.list", { limit: 1 })
      .then((r) => setRoot(r.root))
      .catch(() => undefined);
  }, []);
  return (
    <>
      {login && (
        <label className="welcome-check">
          <input
            type="checkbox"
            role="switch"
            className="switch"
            checked={login.value === true}
            onChange={(e) =>
              void run("settings.set", {
                key: "general.launchAtLogin",
                value: e.target.checked,
              }).then(() => useApp.getState().refreshSettings())
            }
          />
          <span>
            {t("ui.welcome.startup_login")}
            <small>{t("ui.welcome.startup_login_help")}</small>
          </span>
        </label>
      )}
      {root && (
        <>
          <p className="welcome-sub">{t("ui.welcome.library_where")}</p>
          <code className="path">{root}</code>
          <p className="muted">{t("ui.welcome.library_change_later")}</p>
        </>
      )}
    </>
  );
}

function Done() {
  const { t } = useTranslation();
  const [keys, setKeys] = useState("");
  useEffect(() => {
    run<{ rows: ShortcutRow[] }>("shortcuts.list")
      .then((r) =>
        setKeys(
          r.rows.find((x) => x.command === "capture.capture" && x.scope === "global")?.display ??
            "",
        ),
      )
      .catch(() => undefined);
  }, []);
  return (
    <>
      <p>{t("ui.welcome.done_body", { keys: keys || "—" })}</p>
      <button type="button" className="btn" onClick={() => void finish("capture.capture")}>
        {t("ui.welcome.done_try")}
      </button>
    </>
  );
}

export function Welcome() {
  const { t } = useTranslation();
  // The permissions step is macOS only; there, macOS asks for Screen Recording.
  const steps = ["language", "shortcuts", ...(IS_MAC ? ["permissions"] : []), "startup", "done"];
  const [at, setAt] = useState(0);
  const step = steps[at];
  const last = at === steps.length - 1;
  return (
    <div className="welcome">
      <header>
        <h1>{t("ui.welcome.title")}</h1>
        <p className="muted">{t("ui.welcome.step", { n: at + 1, total: steps.length })}</p>
      </header>
      <section key={step} className="welcome-step section-enter" aria-live="polite">
        <h2>{t(`ui.welcome.${step}_title`)}</h2>
        {step === "language" && (
          <>
            <p>{t("ui.welcome.language_body")}</p>
            <LanguageToggle />
          </>
        )}
        {step === "shortcuts" && <Shortcuts />}
        {step === "permissions" && (
          <button
            type="button"
            className="btn"
            onClick={() => void run("app.request_permission", { permission: "screen_recording" })}
          >
            {t("ui.permissions.request")}
          </button>
        )}
        {step === "startup" && <Startup />}
        {step === "done" && <Done />}
      </section>
      <footer className="row">
        {!last && (
          <button type="button" className="btn" onClick={() => void finish()}>
            {t("ui.welcome.skip")}
          </button>
        )}
        <span className="spacer" />
        {at > 0 && (
          <button type="button" className="btn" onClick={() => setAt(at - 1)}>
            {t("ui.welcome.back")}
          </button>
        )}
        <button
          type="button"
          className="btn primary"
          onClick={() => (last ? void finish() : setAt(at + 1))}
        >
          {last ? t("ui.welcome.finish") : t("ui.welcome.next")}
        </button>
      </footer>
    </div>
  );
}

// The Help window (APP-015): local pages in the language of the interface, and About
// XcreenShot with the version, privacy, licenses and "Check communication".
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { LanguageToggle } from "../components/LanguageToggle";
import { useAppShortcuts } from "../lib/appShortcuts";
import { isCommandError, run } from "../lib/commands";
import licenses from "../help/licenses.json";
import { blocks, helpPage, pieces, TOPICS, type Topic } from "../help/pages";
import { arrowFocus } from "../components/arrowFocus";

function initialTopic(): Topic {
  const hash = window.location.hash.slice(1);
  return hash === "about" || (TOPICS as readonly string[]).includes(hash)
    ? (hash as Topic)
    : "start";
}

function Line({ text }: { text: string }) {
  return (
    <>
      {pieces(text).map((p, i) =>
        p.kind === "bold" ? (
          <strong key={i}>{p.text}</strong>
        ) : p.kind === "keys" ? (
          <kbd key={i}>{p.text}</kbd>
        ) : (
          <span key={i}>{p.text}</span>
        ),
      )}
    </>
  );
}

function Page({ topic }: { topic: (typeof TOPICS)[number] }) {
  const { i18n } = useTranslation();
  const page = helpPage(i18n.language, topic);
  return (
    <article className="help-page">
      <h1>{page.title}</h1>
      {blocks(page.body).map((b, i) =>
        b.kind === "h2" ? (
          <h2 key={i}>{b.text}</h2>
        ) : b.kind === "p" ? (
          <p key={i}>
            <Line text={b.text} />
          </p>
        ) : (
          <ul key={i}>
            {b.items.map((item, j) => (
              <li key={j}>
                <Line text={item} />
              </li>
            ))}
          </ul>
        ),
      )}
    </article>
  );
}

interface AboutInfo {
  version: string;
  build: string;
  date: string;
  platform: string;
  arch: string;
  channel: "development" | "stable";
  dataDir: string;
  configDir: string;
  managed: string;
}

const LICENSED = [...licenses.rust, ...licenses.js];

function About() {
  const { t } = useTranslation();
  const [info, setInfo] = useState<AboutInfo | null>(null);
  const [check, setCheck] = useState<string | null>(null);
  useEffect(() => {
    run<AboutInfo>("app.about")
      .then(setInfo)
      .catch(() => undefined);
  }, []);
  const ping = async () => {
    setCheck(t("ui.help.checking"));
    const started = performance.now();
    try {
      const r = await run<{ version: string }>("app.ping");
      const ms = Math.round(performance.now() - started);
      setCheck(t("ui.help.check_ok", { ms, version: r.version }));
    } catch (e) {
      setCheck(t("ui.help.check_failed", { reason: isCommandError(e) ? e.message : String(e) }));
    }
  };
  const byLicense = new Map<string, number>();
  for (const c of LICENSED) byLicense.set(c.license, (byLicense.get(c.license) ?? 0) + 1);
  return (
    <article className="help-page">
      <h1>{t("ui.help.about")}</h1>
      {info && (
        <dl className="about-facts">
          <dt>{t("ui.help.version")}</dt>
          <dd>{info.version}</dd>
          <dt>{t("ui.help.build")}</dt>
          <dd>
            {info.build}
            {info.date && ` · ${info.date}`}
          </dd>
          <dt>{t("ui.help.arch")}</dt>
          <dd>
            {info.platform === "macos" ? "macOS" : "Windows"} · {info.arch}
          </dd>
          <dt>{t("ui.help.channel")}</dt>
          <dd>{t(`ui.help.channel_${info.channel}`)}</dd>
          <dt>{t("ui.help.data")}</dt>
          <dd>
            <code className="path">{info.dataDir}</code>
          </dd>
          <dt>{t("ui.help.settings")}</dt>
          <dd>
            <code className="path">{info.configDir}</code>
          </dd>
          <dt>{t("ui.help.managed")}</dt>
          <dd>{t("ui.help.managed_inactive")}</dd>
        </dl>
      )}
      <div className="row">
        <button type="button" className="btn" onClick={() => void ping()}>
          {t("ui.help.check")}
        </button>
        <button
          type="button"
          className="btn"
          onClick={() => void run("app.diagnostics", { copy: true }).catch(() => undefined)}
        >
          {t("ui.help.copy_diagnostics")}
        </button>
      </div>
      {check && (
        <p className="muted" role="status">
          {check}
        </p>
      )}
      <h2>{t("ui.help.privacy_title")}</h2>
      <p>{t("ui.help.privacy")}</p>
      <div className="row">
        <button
          type="button"
          className="btn"
          onClick={() => void run("app.feedback").catch(() => undefined)}
        >
          {t("ui.help.feedback")}
        </button>
      </div>
      <p className="muted">{t("ui.help.feedback_help")}</p>
      <h2>{t("ui.help.licenses_title")}</h2>
      <p>{t("ui.help.licenses_help", { count: LICENSED.length })}</p>
      <ul className="license-summary">
        {[...byLicense.entries()]
          .sort((a, b) => b[1] - a[1])
          .map(([license, n]) => (
            <li key={license}>
              {license}: {n}
            </li>
          ))}
      </ul>
      <details className="license-list">
        <summary>{t("ui.help.licenses_show")}</summary>
        <ul>
          {LICENSED.map((c) => (
            <li key={`${c.name}@${c.version}`}>
              {c.name} {c.version} — {c.license}
            </li>
          ))}
        </ul>
      </details>
    </article>
  );
}

export function Help() {
  const { t, i18n } = useTranslation();
  useAppShortcuts();
  const [topic, setTopic] = useState<Topic>(initialTopic);
  useEffect(() => {
    const un = listen<string>("xs://help", (e) => {
      const next = e.payload;
      if (next === "about" || (TOPICS as readonly string[]).includes(next)) setTopic(next as Topic);
    });
    return () => void un.then((f) => f());
  }, []);
  const all: Topic[] = [...TOPICS, "about"];
  return (
    <div className="prefs help">
      <nav aria-label={t("ui.help.topics")}>
        <div className="nav-list" onKeyDown={arrowFocus("vertical")}>
          {all.map((id) => (
            <button
              key={id}
              type="button"
              aria-current={id === topic ? "page" : undefined}
              onClick={() => setTopic(id)}
            >
              <span>{id === "about" ? t("ui.help.about") : helpPage(i18n.language, id).title}</span>
            </button>
          ))}
        </div>
        <div className="nav-footer">
          <LanguageToggle />
        </div>
      </nav>
      <main key={topic} className="section-enter">
        {topic === "about" ? <About /> : <Page topic={topic} />}
      </main>
    </div>
  );
}

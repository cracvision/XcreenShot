// Preferences > AI (SMT-039): turning cloud AI on (after saying what is sent and to
// whom), the API key kept in the system keychain, and the models.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { isCommandError, run } from "../lib/commands";
import { useApp } from "../store";
import { SettingRow } from "./SettingRow";

interface KeyStatus {
  saved: boolean;
  hint?: string;
}

function message(e: unknown): string {
  return isCommandError(e) ? e.message : String(e);
}

function Consent({ onAccept, onCancel }: { onAccept: () => void; onCancel: () => void }) {
  const { t } = useTranslation();
  return (
    <Dialog
      title={t("ui.ai.consent_title")}
      onCancel={onCancel}
      actions={
        <>
          <button type="button" className="btn primary" onClick={onAccept}>
            {t("ui.ai.consent_accept")}
          </button>
          <button type="button" className="btn" onClick={onCancel}>
            {t("ui.ai.cancel")}
          </button>
        </>
      }
    >
      <ul className="ai-consent">
        <li>{t("ui.ai.consent_what")}</li>
        <li>{t("ui.ai.consent_who")}</li>
        <li>{t("ui.ai.consent_cost")}</li>
        <li>{t("ui.ai.consent_rest")}</li>
      </ul>
    </Dialog>
  );
}

function KeyBlock() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<KeyStatus | null>(null);
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState("");
  const [note, setNote] = useState<{ ok: boolean; text: string } | null>(null);
  const [testing, setTesting] = useState(false);

  useEffect(() => {
    run<KeyStatus>("ai.key_status")
      .then(setStatus)
      .catch((e) => setNote({ ok: false, text: message(e) }));
  }, []);

  const save = async () => {
    setNote(null);
    try {
      setStatus(await run<KeyStatus>("ai.key_set", { key: draft }));
      setDraft("");
      setEditing(false);
    } catch (e) {
      setNote({ ok: false, text: message(e) });
    }
  };

  const remove = async () => {
    setNote(null);
    try {
      setStatus(await run<KeyStatus>("ai.key_clear"));
    } catch (e) {
      setNote({ ok: false, text: message(e) });
    }
  };

  const test = async () => {
    setNote(null);
    setTesting(true);
    try {
      const r = await run<{ model: string }>("ai.key_test");
      const name = t(`settings.ai_model.options.${r.model}`).replace(/\s*\(.*\)$/, "");
      setNote({ ok: true, text: t("ui.ai.test_ok", { model: name }) });
    } catch (e) {
      setNote({ ok: false, text: message(e) });
    } finally {
      setTesting(false);
    }
  };

  const showInput = !status?.saved || editing;
  return (
    <div className="setting" data-key="ai.key">
      <label className="label" htmlFor="ai-key">
        {t("ui.ai.key")}
      </label>
      <div className="control ai-key">
        {showInput ? (
          <>
            <input
              id="ai-key"
              type="password"
              autoComplete="off"
              spellCheck={false}
              placeholder={t("ui.ai.key_placeholder")}
              value={draft}
              aria-describedby="ai-key-desc"
              onChange={(e) => setDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && draft.trim()) void save();
              }}
            />
            <button
              type="button"
              className="btn"
              disabled={!draft.trim()}
              onClick={() => void save()}
            >
              {t("ui.ai.save")}
            </button>
            {editing && (
              <button type="button" className="btn" onClick={() => setEditing(false)}>
                {t("ui.ai.cancel")}
              </button>
            )}
          </>
        ) : (
          <>
            <span id="ai-key">{t("ui.ai.saved", { hint: status?.hint ?? "" })}</span>
            <button type="button" className="btn" onClick={() => setEditing(true)}>
              {t("ui.ai.change")}
            </button>
            <button type="button" className="btn" onClick={() => void remove()}>
              {t("ui.ai.remove")}
            </button>
          </>
        )}
      </div>
      <div className="description" id="ai-key-desc">
        {t("ui.ai.key_hint")}
        <div className="row ai-key-actions">
          <button
            type="button"
            className="btn"
            disabled={!status?.saved || testing}
            onClick={() => void test()}
          >
            {testing ? t("ui.ai.testing") : t("ui.ai.test")}
          </button>
          <button type="button" className="btn link" onClick={() => void run("ai.open_console")}>
            {t("ui.ai.get_key")}
          </button>
        </div>
        {note && (
          <div className={note.ok ? "ok-text" : "error-text"} role={note.ok ? "status" : "alert"}>
            {note.text}
          </div>
        )}
      </div>
    </div>
  );
}

interface McpConfig {
  config: string;
  claudeCode: string;
}

/** The other way (SMT-043): a Claude app connects to XcreenShot over MCP (APP-029). */
function McpBlock() {
  const { t } = useTranslation();
  const [note, setNote] = useState<{ ok: boolean; text: string } | null>(null);
  const copy = async (which: "config" | "claudeCode") => {
    setNote(null);
    try {
      const c = await run<McpConfig>("app.mcp_config");
      await navigator.clipboard.writeText(c[which]);
      setNote({
        ok: true,
        text: t(which === "config" ? "ui.ai.mcp_copied" : "ui.ai.mcp_copied_code"),
      });
    } catch (e) {
      setNote({ ok: false, text: message(e) });
    }
  };
  return (
    <>
      <h2>{t("ui.ai.mcp_title")}</h2>
      <p className="section-intro">{t("ui.ai.mcp_body")}</p>
      <div className="row">
        <button type="button" className="btn" onClick={() => void copy("config")}>
          {t("ui.ai.mcp_copy")}
        </button>
        <button type="button" className="btn" onClick={() => void copy("claudeCode")}>
          {t("ui.ai.mcp_copy_code")}
        </button>
      </div>
      {note && (
        <p className={note.ok ? "ok-text" : "error-text"} role={note.ok ? "status" : "alert"}>
          {note.text}
        </p>
      )}
    </>
  );
}

export function AiSection({ highlightKey }: { highlightKey: string | null }) {
  const { t } = useTranslation();
  const settings = useApp((s) => s.settings);
  const [asking, setAsking] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const enabled = settings.find((s) => s.key === "ai.enabled");
  const models = settings.filter((s) => s.key === "ai.model" || s.key === "ai.namingModel");

  const turn = async (on: boolean) => {
    setError(null);
    try {
      await run("settings.set", { key: "ai.enabled", value: on });
    } catch (e) {
      setError(message(e));
    }
  };

  return (
    <>
      <p className="section-intro">{t("ui.ai.intro")}</p>
      {enabled && (
        <div
          className={`setting${highlightKey === enabled.key ? " highlight" : ""}`}
          data-key={enabled.key}
        >
          <label className="label" htmlFor="setting-ai.enabled">
            {enabled.label}
          </label>
          <div className="control">
            <input
              id="setting-ai.enabled"
              type="checkbox"
              role="switch"
              className="switch"
              checked={enabled.value === true}
              disabled={enabled.locked}
              aria-describedby="setting-ai.enabled-desc"
              onChange={(e) => (e.target.checked ? setAsking(true) : void turn(false))}
            />
          </div>
          <div className="description" id="setting-ai.enabled-desc">
            {enabled.description}
            {enabled.locked && <div className="locked">🔒 {t("ui.prefs.locked")}</div>}
            {error && (
              <div className="error-text" role="alert">
                {error}
              </div>
            )}
          </div>
        </div>
      )}
      <KeyBlock />
      {models.map((s) => (
        <SettingRow key={s.key} setting={s} highlighted={highlightKey === s.key} />
      ))}
      <McpBlock />
      {asking && (
        <Consent
          onAccept={() => {
            setAsking(false);
            void turn(true);
          }}
          onCancel={() => setAsking(false)}
        />
      )}
    </>
  );
}

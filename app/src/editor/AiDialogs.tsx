// Ask the AI and Suggest name (SMT-040, SMT-041), and what to do when cloud AI is not
// set up yet (SMT-039).
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { useApp } from "../store";
import {
  ask,
  closeAi,
  copyAnswer,
  insertAnswer,
  newConversation,
  openAiPreferences,
  rename,
  stop,
  suggestName,
  useAi,
  type Turn,
} from "./aiState";
import { useEditor } from "./state";

const QUICK = ["describe", "alt", "summary", "poem"] as const;

function Setup() {
  const { t } = useTranslation();
  const message = useAi((s) => s.setupMessage);
  return (
    <Dialog
      title={t("ui.ai.not_ready_title")}
      onCancel={closeAi}
      actions={
        <>
          <button type="button" className="btn primary" onClick={openAiPreferences}>
            {t("ui.ai.open_prefs")}
          </button>
          <button type="button" className="btn" onClick={closeAi}>
            {t("ui.common.close")}
          </button>
        </>
      }
    >
      <p>{message}</p>
    </Dialog>
  );
}

function Answer({ turn, last }: { turn: Turn; last: boolean }) {
  const { t } = useTranslation();
  return (
    <div className="ai-turn">
      <div className="ai-q">
        <span className="ai-who">{t("ui.ai.you")}</span> {turn.prompt}
      </div>
      <div className="ai-a">
        <span className="ai-who">{t("ui.ai.assistant")}</span>
        <div className="ai-text">
          {turn.answer}
          {turn.stopped && <span className="muted"> {t("ui.ai.stopped")}</span>}
        </div>
      </div>
      <div className="row ai-turn-actions">
        <button type="button" className="btn small" onClick={() => void copyAnswer(turn.answer)}>
          {t("ui.ai.copy")}
        </button>
        <button
          type="button"
          className={`btn small${last ? " primary" : ""}`}
          onClick={() => void insertAnswer(turn.answer)}
        >
          {t("ui.ai.insert")}
        </button>
      </div>
    </div>
  );
}

function Ask() {
  const { t } = useTranslation();
  const turns = useAi((s) => s.turns);
  const pending = useAi((s) => s.pending);
  const usage = useAi((s) => s.usage);
  const error = useAi((s) => s.error);
  const doc = useEditor((s) => s.doc);
  const model = useApp(
    (s) => (s.settings.find((x) => x.key === "ai.model")?.value as string) ?? "claude-opus-5-5",
  );
  const [prompt, setPrompt] = useState("");
  const input = useRef<HTMLTextAreaElement>(null);
  const log = useRef<HTMLDivElement>(null);

  useEffect(() => {
    input.current?.focus();
  }, []);
  // The newest text stays in view while it arrives.
  useEffect(() => {
    log.current?.scrollTo({ top: log.current.scrollHeight });
  }, [turns.length, pending?.answer]);

  const send = () => {
    if (!prompt.trim() || pending) return;
    void ask(prompt);
    setPrompt("");
  };
  // "Claude Opus 5.5", without the note on the choice.
  const modelName = t(`settings.ai_model.options.${model}`).replace(/\s*\(.*\)$/, "");

  return (
    <Dialog
      title={t("ui.ai.ask_title")}
      onCancel={closeAi}
      actions={
        <>
          {pending ? (
            <button type="button" className="btn primary" onClick={stop}>
              {t("ui.ai.stop")}
            </button>
          ) : (
            <button type="button" className="btn primary" disabled={!prompt.trim()} onClick={send}>
              {t("ui.ai.send")}
            </button>
          )}
          {turns.length > 0 && !pending && (
            <button type="button" className="btn" onClick={newConversation}>
              {t("ui.ai.new")}
            </button>
          )}
          <button type="button" className="btn" onClick={closeAi}>
            {t("ui.common.close")}
          </button>
        </>
      }
    >
      <div className="ai-dialog">
        <p className="muted ai-sending">
          {t("ui.ai.sending", {
            width: doc?.canvas.width ?? 0,
            height: doc?.canvas.height ?? 0,
            model: modelName,
          })}
        </p>
        {(turns.length > 0 || pending) && (
          <div className="ai-log" ref={log} aria-live="polite">
            {turns.map((turn, i) => (
              <Answer key={i} turn={turn} last={i === turns.length - 1 && !pending} />
            ))}
            {pending && (
              <div className="ai-turn">
                <div className="ai-q">
                  <span className="ai-who">{t("ui.ai.you")}</span> {pending.prompt}
                </div>
                <div className="ai-a">
                  <span className="ai-who">{t("ui.ai.assistant")}</span>
                  <div className="ai-text">{pending.answer || t("ui.ai.thinking")}</div>
                </div>
              </div>
            )}
          </div>
        )}
        {error && (
          <p className="error-text" role="alert">
            {error}
          </p>
        )}
        <div className="row ai-quick" role="group" aria-label={t("ui.ai.quick")}>
          {QUICK.map((q) => (
            <button
              key={q}
              type="button"
              className="btn small"
              disabled={!!pending}
              onClick={() => {
                setPrompt(t(`ui.ai.prompt_${q}`));
                input.current?.focus();
              }}
            >
              {t(`ui.ai.quick_${q}`)}
            </button>
          ))}
        </div>
        <label className="ai-label" htmlFor="ai-prompt">
          {turns.length > 0 ? t("ui.ai.more") : t("ui.ai.prompt_label")}
        </label>
        <textarea
          id="ai-prompt"
          ref={input}
          rows={3}
          value={prompt}
          placeholder={t("ui.ai.prompt_placeholder")}
          onChange={(e) => setPrompt(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
              e.preventDefault();
              send();
            }
          }}
        />
        {usage && (
          <p className="muted ai-usage">
            {t("ui.ai.usage", { input: usage.inputTokens, output: usage.outputTokens })}
          </p>
        )}
      </div>
    </Dialog>
  );
}

function Name() {
  const { t } = useTranslation();
  const name = useAi((s) => s.name);
  const naming = useAi((s) => s.naming);
  const error = useAi((s) => s.error);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (!naming) input.current?.select();
  }, [naming]);
  return (
    <Dialog
      title={t("ui.ai.name_title")}
      onCancel={closeAi}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            disabled={naming || !name.trim()}
            onClick={() => void rename()}
          >
            {t("ui.ai.rename")}
          </button>
          <button
            type="button"
            className="btn"
            disabled={naming}
            onClick={() => void suggestName()}
          >
            {t("ui.ai.another")}
          </button>
          <button type="button" className="btn" onClick={closeAi}>
            {t("ui.ai.cancel")}
          </button>
        </>
      }
    >
      <label className="ai-label" htmlFor="ai-name">
        {t("ui.ai.name_label")}
      </label>
      <input
        id="ai-name"
        ref={input}
        className="ai-name"
        value={naming ? "" : name}
        placeholder={naming ? t("ui.ai.name_asking") : ""}
        disabled={naming}
        aria-busy={naming}
        onChange={(e) => useAi.setState({ name: e.target.value })}
        onKeyDown={(e) => {
          if (e.key === "Enter" && name.trim() && !naming) void rename();
        }}
      />
      {error && (
        <p className="error-text" role="alert">
          {error}
        </p>
      )}
    </Dialog>
  );
}

export function AiDialogs() {
  const open = useAi((s) => s.open);
  if (open === "setup") return <Setup />;
  if (open === "ask") return <Ask />;
  if (open === "name") return <Name />;
  return null;
}

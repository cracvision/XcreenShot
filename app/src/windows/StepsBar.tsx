// The step recorder's bar (SMT-023): the step count, which flashes when a step is
// added; Pause / Resume, Undo last step, Finish and Cancel, which asks first when there
// are steps. It never shows in the steps' pictures, and clicks on it never make a step.
// The notice that opens a recording (SMT-022) is here too.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";

export interface StepsState {
  status: "notice" | "recording" | "paused" | "idle";
  steps: number;
  confirm?: boolean;
  diskFull?: boolean;
  manySteps?: boolean;
  cancelKey?: string;
}

function useSteps(): StepsState {
  const [state, setState] = useState<StepsState>({ status: "notice", steps: 0 });
  useEffect(() => {
    document.documentElement.classList.add("launcher-root");
    void run<StepsState>("smart.steps_status")
      .then(setState)
      .catch(() => undefined);
    const off = listen<StepsState>("xs://steps", (e) => setState(e.payload));
    return () => void off.then((f) => f());
  }, []);
  return state;
}

const act = (command: string, params?: Record<string, unknown>) =>
  void run(command, params).catch(() => undefined);

export function StepsBar() {
  const { t } = useTranslation();
  const state = useSteps();
  const paused = state.status === "paused";
  const count = t("ui.stepsbar.steps", { count: state.steps });

  if (state.confirm) {
    const question = t("ui.stepsbar.confirm", { count: state.steps });
    return (
      <div className="recbar glass" role="alertdialog" aria-label={question}>
        <span className="recbar-ask">{question}</span>
        <button
          type="button"
          className="btn small danger"
          onClick={() => act("smart.steps_cancel", { confirm: true })}
        >
          {t("ui.stepsbar.discard")}
        </button>
        <button
          type="button"
          className="btn small"
          autoFocus
          onClick={() => act("smart.steps_keep")}
        >
          {t("ui.stepsbar.keep")}
        </button>
      </div>
    );
  }

  return (
    <div
      className="recbar glass"
      role="toolbar"
      aria-label={t("ui.stepsbar.title")}
      onKeyDown={(e) => {
        if (e.key !== "Escape") return;
        e.preventDefault();
        act("smart.steps_cancel");
      }}
    >
      <span className="recbar-grip" data-tauri-drag-region aria-hidden="true">
        ⋮⋮
      </span>
      <span
        className={`recbar-dot${state.status === "recording" ? " on" : ""}`}
        role="img"
        aria-label={paused ? t("ui.stepsbar.paused") : t("ui.stepsbar.title")}
      />
      {/* A new key at each step: its flash plays again (SMT-023). */}
      <span
        key={state.steps}
        className={`stepsbar-count${state.steps > 0 ? " flash" : ""}`}
        aria-live="polite"
      >
        {count}
      </span>
      {paused && (
        <span className={`recbar-size${state.diskFull ? " low" : ""}`}>
          {state.diskFull ? t("ui.stepsbar.disk_full") : t("ui.stepsbar.paused")}
        </span>
      )}
      {state.manySteps && !paused && (
        <span className="recbar-size" title={t("ui.stepsbar.many")}>
          {t("ui.stepsbar.many")}
        </span>
      )}
      <span className="recbar-fill" data-tauri-drag-region />
      <button
        type="button"
        className="btn small"
        onClick={() => act(paused ? "smart.steps_resume" : "smart.steps_pause")}
      >
        {paused ? t("ui.stepsbar.resume") : t("ui.stepsbar.pause")}
      </button>
      <button
        type="button"
        className="btn small"
        disabled={state.steps === 0}
        title={t("ui.stepsbar.undo")}
        onClick={() => act("smart.steps_undo")}
      >
        {t("ui.stepsbar.undo")}
      </button>
      <button type="button" className="btn small primary" onClick={() => act("smart.steps_stop")}>
        {t("ui.stepsbar.finish")}
      </button>
      <button type="button" className="btn small" onClick={() => act("smart.steps_cancel")}>
        {t("ui.stepsbar.cancel")}
      </button>
    </div>
  );
}

/** "Recording steps: every click makes a step" for 2 s (SMT-022); a click closes it. */
export function StepsNotice() {
  const { t } = useTranslation();
  const state = useSteps();
  const key =
    state.cancelKey === "shiftEsc" ? "Shift+Esc" : state.cancelKey === "none" ? "" : "Esc";
  return (
    <button
      type="button"
      className="stepsnote glass"
      onClick={() => act("smart.steps_notice_closed")}
    >
      <strong>{t("ui.stepsbar.notice")}</strong>
      {key && <span className="muted">{t("ui.stepsbar.notice_cancel", { key })}</span>}
    </button>
  );
}

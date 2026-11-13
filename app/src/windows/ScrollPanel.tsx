// The scrolling capture's panel (CAP-034, CAP-035): beside the area, never in the
// capture. It shows the joined image as it grows and its size, with Done (Enter) and
// Cancel (Esc); an automatic capture shows its progress with Stop here. The keys work
// while the app being scrolled keeps the focus: XcreenShot listens to them itself.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { isCommandError, run } from "../lib/commands";
import { xsdocUrl } from "../editor/state";

interface ScrollState {
  running: boolean;
  auto?: boolean;
  started?: boolean;
  width?: number;
  height?: number;
  full?: boolean;
  /** Changes each time there is a new preview. */
  preview?: number;
}

const errorMessage = (e: unknown) => (isCommandError(e) ? e.message : String(e));

export function ScrollPanel() {
  const { t } = useTranslation();
  const [state, setState] = useState<ScrollState>({ running: true });
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    void run<ScrollState>("capture.scroll_state").then(setState);
    const un = listen<ScrollState>("xs://scroll", (e) => setState(e.payload));
    return () => void un.then((f) => f());
  }, []);

  const act = (action: "start" | "done" | "cancel") => {
    setProblem(null);
    run("capture.scroll_action", { action }).catch((e: unknown) => setProblem(errorMessage(e)));
  };

  const auto = state.auto === true;
  const started = state.started !== false;
  const hint = !started
    ? t("ui.scroll.ask_hint")
    : auto
      ? t("ui.scroll.auto_hint")
      : t("ui.scroll.manual_hint");
  const size =
    state.width && state.height
      ? t("ui.scroll.size", { width: state.width, height: state.height })
      : "";

  return (
    <div className="scroll-panel">
      <p className="scroll-hint" role="status">
        {hint}
      </p>
      {!auto && (
        <div className="scroll-preview" aria-label={t("ui.scroll.preview")} role="img">
          {state.preview ? (
            <img src={xsdocUrl(`scroll/preview?v=${state.preview}`)} alt="" draggable={false} />
          ) : null}
        </div>
      )}
      <p className="scroll-size" aria-live="polite">
        {size}
        {state.full && <span className="scroll-full"> · {t("ui.scroll.full")}</span>}
      </p>
      {auto && (
        <div className="scroll-progress" aria-hidden="true">
          <span />
        </div>
      )}
      {problem && (
        <p className="scroll-problem" role="alert">
          {problem}
        </p>
      )}
      <div className="scroll-actions">
        <button type="button" className="btn" onClick={() => act("cancel")}>
          {t("ui.scroll.cancel")} <kbd>Esc</kbd>
        </button>
        {started ? (
          <button type="button" className="btn primary" onClick={() => act("done")}>
            {auto ? t("ui.scroll.stop_here") : t("ui.scroll.done")} <kbd>Enter</kbd>
          </button>
        ) : (
          <button type="button" className="btn primary" onClick={() => act("start")}>
            {t("ui.scroll.start")}
          </button>
        )}
      </div>
    </div>
  );
}

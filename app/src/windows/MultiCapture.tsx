// Multi-capture panel (CAP-067): the captures of the session, to remove or insert in
// a document (EDT-078). Clicks on it never capture; Esc and Finish end capturing, and
// the panel stays so the captures can still be inserted. Closing it ends the session.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { isCommandError, run } from "../lib/commands";
import { xsdocUrl } from "../editor/state";

const errorMessage = (e: unknown) => (isCommandError(e) ? e.message : String(e));

interface Item {
  id: string;
  path: string;
  width: number;
  height: number;
}

interface MultiState {
  capturing: boolean;
  items: Item[];
}

export function MultiCapture() {
  const { t } = useTranslation();
  const [state, setState] = useState<MultiState>({ capturing: false, items: [] });
  const [selected, setSelected] = useState<string[]>([]);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    void run<MultiState>("capture.multi_state").then(setState);
    const un = listen<MultiState>("xs://multicapture", (e) => setState(e.payload));
    return () => void un.then((f) => f());
  }, []);

  // What is no longer in the panel cannot stay selected.
  const ids = state.items.map((i) => i.id);
  const chosen = selected.filter((id) => ids.includes(id));

  const toggle = (id: string) =>
    setSelected((s) => (s.includes(id) ? s.filter((x) => x !== id) : [...s, id]));

  const act = (promise: Promise<unknown>) => {
    setProblem(null);
    promise.catch((e: unknown) => setProblem(errorMessage(e)));
  };

  const insert = () =>
    act(
      (async () => {
        // In the order they were taken, whatever the order they were picked.
        const paths = state.items.filter((i) => chosen.includes(i.id)).map((i) => i.path);
        await run("document.insert_images", { paths });
        await run("editor.show");
        setSelected([]);
      })(),
    );

  const remove = (id: string) => act(run("capture.multi_remove", { ids: [id] }));

  return (
    <div className="multi">
      <p className="multi-hint" role="status">
        {state.capturing ? t("ui.multi.hint") : t("ui.multi.stopped")}
      </p>
      {state.items.length === 0 ? (
        <p className="multi-empty muted">{t("ui.multi.empty")}</p>
      ) : (
        <ul className="multi-grid" aria-label={t("ui.multi.captures")}>
          {state.items.map((item, n) => {
            const on = chosen.includes(item.id);
            const name = t("ui.multi.capture_n", {
              n: n + 1,
              width: item.width,
              height: item.height,
            });
            return (
              <li key={item.id} className="multi-item">
                <button
                  type="button"
                  className="multi-thumb"
                  aria-pressed={on}
                  aria-label={name}
                  title={name}
                  onClick={() => toggle(item.id)}
                >
                  <img src={xsdocUrl(`multi/${item.id}`)} alt="" draggable={false} />
                </button>
                <button
                  type="button"
                  className="multi-remove"
                  aria-label={t("ui.multi.remove", { name })}
                  title={t("ui.multi.remove_hint")}
                  onClick={() => remove(item.id)}
                >
                  ×
                </button>
              </li>
            );
          })}
        </ul>
      )}
      {problem && (
        <p className="multi-problem" role="alert">
          {problem}
        </p>
      )}
      <div className="multi-actions">
        <button
          type="button"
          className="btn"
          disabled={state.items.length === 0}
          onClick={() => setSelected(chosen.length === ids.length ? [] : ids)}
        >
          {chosen.length === ids.length && ids.length > 0 ? t("ui.multi.none") : t("ui.multi.all")}
        </button>
        <button
          type="button"
          className="btn"
          disabled={state.items.length === 0}
          onClick={() => act(run("capture.multi_clear"))}
        >
          {t("ui.multi.clear")}
        </button>
        <button
          type="button"
          className="btn primary"
          disabled={chosen.length === 0}
          onClick={insert}
        >
          {chosen.length === 0
            ? t("ui.multi.insert_none")
            : t("ui.multi.insert", { count: chosen.length })}
        </button>
      </div>
      <div className="multi-actions">
        {state.capturing ? (
          <button type="button" className="btn" onClick={() => act(run("capture.multi_stop"))}>
            {t("ui.multi.finish")}
          </button>
        ) : (
          <>
            <button type="button" className="btn" onClick={() => act(run("capture.multi_start"))}>
              {t("ui.multi.resume")}
            </button>
            <button
              type="button"
              className="btn"
              onClick={() => act(run("capture.multi_stop", { close: true }))}
            >
              {t("ui.multi.close")}
            </button>
          </>
        )}
      </div>
    </div>
  );
}

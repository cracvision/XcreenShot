// The results window of Extract text (SMT-006): the text to review and edit, Copy all
// (with its format and without, so each app pastes what it takes), Copy and close,
// Without format, Save as text, another language and joined lines. Esc closes it; the
// text and its zone are forgotten then (SMT-001).
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { save } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Menu, type MenuItem } from "../components/Menu";
import { isCommandError, run } from "../lib/commands";
import { useAppShortcuts } from "../lib/appShortcuts";
import { useApp, useSetting } from "../store";
import { textTips, type TextResultData } from "./textResultState";
import { QrCodeList } from "../components/QrCodeList";

interface Languages {
  languages: { code: string; name: string; tag: string | null; available: boolean }[];
  settingsUri: string;
}

export function TextResult({ id }: { id: string }) {
  const { t } = useTranslation();
  useAppShortcuts();
  const [result, setResult] = useState<TextResultData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState("");
  const [join, setJoin] = useState(false);
  const [langs, setLangs] = useState<Languages | null>(null);
  const plain = useSetting("smart.extractText.plain", false);
  const spellcheck = useSetting("editor.spellcheck", true);
  const rich = useRef<HTMLDivElement>(null);
  const area = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    run<TextResultData>("smart.text_result", { id })
      .then(setResult)
      .catch((e: unknown) => setError(isCommandError(e) ? e.message : String(e)));
    run<Languages>("smart.ocr_languages")
      .then(setLangs)
      .catch(() => undefined);
    const win = getCurrentWebviewWindow();
    // Closed in any way: the text and the zone go.
    const off = win.onCloseRequested(() => {
      void run("smart.text_close", { id }).catch(() => undefined);
    });
    return () => void off.then((f) => f());
  }, [id]);

  // The text shown changes with the format and the joined lines; edits start over.
  const html = result ? (join ? result.htmlJoined : result.html) : "";
  const text = result ? (join ? result.textJoined : result.text) : "";
  useEffect(() => {
    if (rich.current) rich.current.innerHTML = html;
  }, [html, plain]);

  const current = useCallback((): { text: string; html: string | null } => {
    if (plain) return { text: area.current?.value ?? text, html: null };
    const el = rich.current;
    return { text: el?.innerText ?? text, html: el?.innerHTML ?? html };
  }, [plain, text, html]);

  const close = useCallback(() => void getCurrentWebviewWindow().close(), []);

  const copyAll = useCallback(
    async (thenClose = false) => {
      const c = current();
      try {
        await run("smart.copy_text", { text: c.text, html: c.html });
        if (thenClose) close();
        else setStatus(t("ui.extract.copied"));
      } catch (e) {
        setStatus(isCommandError(e) ? e.message : String(e));
      }
    },
    [current, close, t],
  );

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        close();
      } else if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        void copyAll(true);
      } else if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.code === "KeyC") {
        e.preventDefault();
        void copyAll();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [close, copyAll]);

  const saveText = async () => {
    const path = await save({
      defaultPath: `${t("ui.extract.file_name")}.txt`,
      filters: [{ name: t("ui.extract.file_type"), extensions: ["txt"] }],
    });
    if (!path) return;
    try {
      await run("smart.save_text", { path, text: current().text });
      setStatus(t("ui.extract.saved"));
    } catch (e) {
      setStatus(isCommandError(e) ? e.message : String(e));
    }
  };

  const again = async (language: string) => {
    setStatus(t("ui.extract.reading"));
    try {
      setResult(await run<TextResultData>("smart.text_again", { id, language }));
      setStatus("");
    } catch (e) {
      setStatus(isCommandError(e) ? e.message : String(e));
    }
  };

  const languageItems: MenuItem[] = [
    { id: "auto", label: t("ui.extract.auto"), onSelect: () => void again("auto") },
    ...(langs?.languages ?? []).map((l) => ({
      id: l.code,
      label: l.available ? l.name : `${l.name} · ${t("ui.extract.not_here")}`,
      disabled: !l.available,
      title: t("ui.extract.not_here_help"),
      onSelect: () => void again(l.code),
    })),
  ];
  if (langs?.languages.some((l) => !l.available))
    languageItems.push({
      id: "add",
      label: t("ui.extract.add_language"),
      separator: true,
      onSelect: () => void run("smart.language_settings").catch(() => undefined),
    });

  if (error)
    return (
      <div className="text-result">
        <p className="panel-status" role="alert">
          {error}
        </p>
      </div>
    );
  if (!result) return <div className="text-result" />;

  return (
    <div className="text-result">
      {result.found ? (
        plain ? (
          <textarea
            ref={area}
            key={`${join}`}
            className="text-result-area"
            aria-label={t("ui.extract.text_label")}
            spellCheck={spellcheck}
            defaultValue={text}
            autoFocus
          />
        ) : (
          <div
            ref={rich}
            className="text-result-area rich"
            role="textbox"
            aria-multiline="true"
            aria-label={t("ui.extract.text_label")}
            contentEditable
            suppressContentEditableWarning
            spellCheck={spellcheck}
            tabIndex={0}
          />
        )
      ) : (
        <div className="text-result-none" role="status">
          <h1>{t("ui.extract.none_title")}</h1>
          <ul>
            {textTips(t).map((tip) => (
              <li key={tip}>{tip}</li>
            ))}
          </ul>
        </div>
      )}
      {result.codes && result.codes.length > 0 && (
        <section className="text-result-codes" aria-label={t("ui.qr.codes_title")}>
          <h2>{t("ui.qr.codes_title")}</h2>
          <QrCodeList codes={result.codes} button="btn small" primary="btn primary small" />
        </section>
      )}
      <div className="text-result-bar">
        {result.found && (
          <>
            <button type="button" className="btn primary" onClick={() => void copyAll(true)}>
              {t("ui.extract.copy_close")}
            </button>
            <button type="button" className="btn" onClick={() => void copyAll()}>
              {t("ui.extract.copy_all")}
            </button>
            <label className="panel-check">
              <input
                type="checkbox"
                checked={plain}
                onChange={(e) =>
                  void run("settings.set", {
                    key: "smart.extractText.plain",
                    value: e.target.checked,
                  }).then(() => useApp.getState().refreshSettings())
                }
              />
              {t("ui.extract.plain")}
            </label>
          </>
        )}
        {!result.found && (
          <button
            type="button"
            className="btn primary"
            onClick={() => {
              void run("smart.extract_text", { source: "screen" }).catch(() => undefined);
              close();
            }}
          >
            {t("ui.extract.repeat")}
          </button>
        )}
        <span className="grow" />
        <span className="muted text-result-status" role="status">
          {status ||
            (result.found
              ? t("ui.extract.words", { count: result.words, language: result.language })
              : "")}
        </span>
        <Menu
          triggerClass="btn icon"
          align="end"
          side="top"
          chevron={false}
          ariaLabel={t("ui.extract.more")}
          title={t("ui.extract.more")}
          label={<span aria-hidden="true">⋯</span>}
          items={[
            {
              id: "save",
              label: t("ui.extract.save_txt"),
              disabled: !result.found,
              onSelect: () => void saveText(),
            },
            {
              id: "language",
              label: t("ui.extract.other_language"),
              submenu: languageItems,
              onSelect: () => undefined,
            },
            {
              id: "join",
              label: t("ui.extract.join_lines"),
              checked: join,
              disabled: !result.found,
              onSelect: () => setJoin(!join),
            },
          ]}
        />
      </div>
    </div>
  );
}

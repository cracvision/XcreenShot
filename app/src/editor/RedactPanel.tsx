// The auto redact panel (SMT-010 to SMT-015): categories and the user's terms with how
// many matches each has, those redacted by hand, the style of the redactions, Next /
// Previous and Done. It takes the place of the side panel's tabs while it is open. Every
// control runs a command; the canvas's words are clicked in `RedactOverlay`.
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { useIndicator } from "../components/useIndicator";
import { selectTool } from "./actions";
import { ColorPicker } from "./ColorPicker";
import { Icon } from "./icons";
import { NumberField } from "./Fields";
import {
  acknowledgeReminder,
  CATEGORIES,
  clearRedactions,
  dismissNotice,
  finishRedact,
  matchToTerms,
  removeCurrent,
  setStyle,
  setTerms,
  stepRedaction,
  termName,
  toggleCategory,
  undoMatches,
  useRedact,
  type RedactType,
  type Term,
} from "./redactState";

const TYPES: RedactType[] = ["solid", "blur", "pixelate"];

function Count({ n }: { n: number | undefined }) {
  return <span className="redact-count">{n ?? 0}</span>;
}

/** What "all matches" did, with Undo and Add to My terms (SMT-015). */
function MatchNotice() {
  const { t } = useTranslation();
  const notice = useRedact((s) => s.notice);
  if (!notice) return null;
  const message =
    notice.count === 0
      ? t("ui.redact.no_matches")
      : notice.remove
        ? t("ui.redact.unmatched", { count: notice.count })
        : t("ui.redact.matched", { count: notice.count });
  return (
    <div className="redact-notice" role="status">
      <p>{message}</p>
      <button
        type="button"
        className="btn icon small redact-notice-close"
        aria-label={t("ui.redact.dismiss")}
        title={t("ui.redact.dismiss")}
        onClick={dismissNotice}
      >
        <Icon name="close" />
      </button>
      <div className="redact-notice-actions">
        {notice.count > 0 && (
          <button type="button" className="btn small" onClick={() => void undoMatches()}>
            {t("ui.redact.undo")}
          </button>
        )}
        {!notice.remove && (
          <button type="button" className="btn small" onClick={() => void matchToTerms()}>
            {t("ui.redact.to_terms")}
          </button>
        )}
      </div>
    </div>
  );
}

function TermRows() {
  const { t } = useTranslation();
  const terms = useRedact((s) => s.terms);
  const counts = useRedact((s) => s.result?.counts);
  const invalid = useRedact((s) => s.result?.invalidPatterns);
  const [words, setWords] = useState("");
  const [exact, setExact] = useState(false);
  const [patternName, setPatternName] = useState("");
  const [pattern, setPattern] = useState("");
  const change = (i: number, next: Term | null) =>
    void setTerms(
      next ? terms.map((x, j) => (j === i ? next : x)) : terms.filter((_, j) => j !== i),
    );
  return (
    <>
      {terms.length > 0 && (
        <ul className="redact-list" aria-label={t("ui.redact.my_terms")}>
          {terms.map((term, i) => {
            const name = termName(term);
            const bad = invalid?.includes(name) ?? false;
            return (
              <li key={`${i}-${name}`} className={bad ? "invalid" : undefined}>
                <label className="panel-check">
                  <input
                    type="checkbox"
                    checked={term.on !== false}
                    aria-invalid={bad || undefined}
                    onChange={(e) => change(i, { ...term, on: e.target.checked })}
                  />
                  <span className="redact-name">
                    {name}
                    <small className="muted">
                      {term.pattern !== undefined
                        ? t("ui.redact.pattern")
                        : term.exact
                          ? t("ui.redact.exact")
                          : t("ui.redact.words")}
                    </small>
                  </span>
                </label>
                {bad ? (
                  <span className="redact-bad" role="status">
                    {t("ui.redact.invalid_pattern")}
                  </span>
                ) : (
                  <Count n={counts?.[`term:${name}`]} />
                )}
                <button
                  type="button"
                  className="btn icon small"
                  aria-label={t("ui.redact.remove_term", { name })}
                  title={t("ui.redact.remove_term", { name })}
                  onClick={() => change(i, null)}
                >
                  ×
                </button>
              </li>
            );
          })}
        </ul>
      )}
      <form
        className="redact-add"
        onSubmit={(e) => {
          e.preventDefault();
          if (!words.trim()) return;
          void setTerms([...terms, { text: words.trim(), exact, on: true }]);
          setWords("");
        }}
      >
        <input
          type="text"
          value={words}
          placeholder={t("ui.redact.add_words")}
          aria-label={t("ui.redact.add_words")}
          onChange={(e) => setWords(e.target.value)}
        />
        <button type="submit" className="btn small" disabled={!words.trim()}>
          {t("ui.redact.add")}
        </button>
        <label className="panel-check">
          <input type="checkbox" checked={exact} onChange={(e) => setExact(e.target.checked)} />
          {t("ui.redact.exact")}
        </label>
      </form>
      <form
        className="redact-add"
        onSubmit={(e) => {
          e.preventDefault();
          if (!pattern.trim()) return;
          const name = patternName.trim();
          void setTerms([
            ...terms,
            { ...(name ? { name } : {}), pattern: pattern.trim(), on: true },
          ]);
          setPattern("");
          setPatternName("");
        }}
      >
        <input
          type="text"
          className="wide"
          value={patternName}
          placeholder={t("ui.redact.pattern_name")}
          aria-label={t("ui.redact.pattern_name")}
          onChange={(e) => setPatternName(e.target.value)}
        />
        <input
          type="text"
          className="mono"
          value={pattern}
          placeholder="PO-\d{6}"
          aria-label={t("ui.redact.add_pattern")}
          spellCheck={false}
          onChange={(e) => setPattern(e.target.value)}
        />
        <button type="submit" className="btn small" disabled={!pattern.trim()}>
          {t("ui.redact.add")}
        </button>
      </form>
    </>
  );
}

function StyleRows() {
  const { t } = useTranslation();
  const style = useRedact((s) => s.style);
  const seg = useRef<HTMLDivElement>(null);
  useIndicator(seg, style.type);
  return (
    <>
      <div className="prop-field">
        <span id="redact-type">{t("ui.redact.type")}</span>
        <div className="seg small" role="radiogroup" aria-labelledby="redact-type" ref={seg}>
          <span className="seg-indicator" aria-hidden="true" />
          {TYPES.map((type) => (
            <button
              key={type}
              type="button"
              role="radio"
              aria-checked={style.type === type}
              onClick={() => style.type !== type && void setStyle({ type })}
            >
              {t(`ui.editor.blur.${type}`)}
            </button>
          ))}
        </div>
      </div>
      {style.type === "solid" && (
        <ColorPicker
          label={t("ui.editor.prop.color")}
          value={style.color}
          onCommit={(color) => color && void setStyle({ color: color.slice(0, 7) })}
        />
      )}
      <NumberField
        label={t("ui.redact.margin")}
        value={style.margin}
        min={0}
        max={10}
        step={1}
        unit="px"
        onCommit={(margin) => void setStyle({ margin })}
      />
    </>
  );
}

function Reminder() {
  const { t } = useTranslation();
  const open = useRedact((s) => s.reminder);
  const [hide, setHide] = useState(false);
  if (!open) return null;
  return (
    <Dialog
      title={t("ui.redact.reminder_title")}
      onCancel={() => useRedact.setState({ reminder: false })}
      actions={
        <>
          <button
            type="button"
            className="btn"
            onClick={() => {
              useRedact.setState({ reminder: false });
              void setStyle({ type: "solid" });
            }}
          >
            {t("ui.redact.use_solid")}
          </button>
          <button type="button" className="btn primary" onClick={() => acknowledgeReminder(hide)}>
            {t("ui.redact.done")}
          </button>
        </>
      }
    >
      <p>{t("ui.redact.reminder_body")}</p>
      <label className="panel-check">
        <input type="checkbox" checked={hide} onChange={(e) => setHide(e.target.checked)} />
        {t("ui.editor.dont_ask_again")}
      </label>
    </Dialog>
  );
}

export function RedactPanel() {
  const { t } = useTranslation();
  const result = useRedact((s) => s.result);
  const busy = useRedact((s) => s.busy);
  const slow = useRedact((s) => s.slow);
  const categories = useRedact((s) => s.categories);
  const index = useRedact((s) => s.index);
  const items = result?.items ?? [];
  const nothing = result !== null && !busy && result.redactions === 0;
  return (
    <section className="redact-panel" aria-label={t("ui.redact.title")} aria-busy={busy}>
      <h2 className="side-title">{t("ui.redact.title")}</h2>
      {slow && (
        <p className="redact-progress" role="status">
          {t("ui.redact.analyzing")}
          <progress aria-label={t("ui.redact.analyzing")} />
        </p>
      )}
      {result && !result.textRead && <p className="muted hint">{t("ui.redact.faces_only")}</p>}
      {nothing && (
        <div className="redact-empty" role="status">
          <p>{t("ui.redact.nothing")}</p>
          {!result.textRead && (
            <button type="button" className="btn small" onClick={() => selectTool("blur")}>
              {t("ui.redact.add_by_hand")}
            </button>
          )}
        </div>
      )}
      {result?.textRead && <p className="muted hint">{t("ui.redact.click_hint")}</p>}
      <MatchNotice />

      <h3 className="side-subtitle">{t("ui.redact.categories")}</h3>
      <ul className="redact-list">
        {CATEGORIES.map((c) => (
          <li key={c}>
            <label className="panel-check">
              <input
                type="checkbox"
                checked={categories.includes(c)}
                onChange={() => void toggleCategory(c)}
              />
              <span className="redact-name">{t(`ui.redact.cat.${c}`)}</span>
            </label>
            <Count n={result?.counts[c]} />
          </li>
        ))}
        {(result?.counts.manual ?? 0) > 0 && (
          <li className="redact-manual">
            <span className="redact-name">{t("ui.redact.cat.manual")}</span>
            <Count n={result?.counts.manual} />
          </li>
        )}
      </ul>

      <h3 className="side-subtitle">{t("ui.redact.my_terms")}</h3>
      <TermRows />

      <h3 className="side-subtitle">{t("ui.redact.style")}</h3>
      <StyleRows />

      <div className="redact-nav" role="group" aria-label={t("ui.redact.review")}>
        <button
          type="button"
          className="btn small"
          disabled={items.length === 0}
          onClick={() => stepRedaction(-1)}
        >
          {t("ui.redact.previous")}
        </button>
        <span className="muted" aria-live="polite">
          {items.length === 0
            ? t("ui.redact.none")
            : index < 0
              ? t("ui.redact.total", { count: items.length })
              : t("ui.redact.position", { n: index + 1, count: items.length })}
        </span>
        <button
          type="button"
          className="btn small"
          disabled={items.length === 0}
          onClick={() => stepRedaction(1)}
        >
          {t("ui.redact.next")}
        </button>
        <button
          type="button"
          className="btn small redact-remove"
          disabled={index < 0 || busy}
          title={t("ui.redact.remove_current")}
          aria-label={t("ui.redact.remove_current")}
          onClick={() => void removeCurrent()}
        >
          {t("ui.redact.remove")}
        </button>
      </div>
      <div className="redact-actions">
        <button
          type="button"
          className="btn"
          disabled={items.length === 0 || busy}
          onClick={() => void clearRedactions()}
        >
          {t("ui.redact.remove_all")}
        </button>
        <button type="button" className="btn primary" onClick={() => void finishRedact()}>
          {t("ui.redact.done")}
        </button>
      </div>
      <p className="muted hint">{t("ui.redact.private")}</p>
      <Reminder />
    </section>
  );
}

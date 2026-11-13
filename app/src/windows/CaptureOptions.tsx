// The options of the capture panel's tabs (CAP-002, CAP-010), shared by the panel and
// the preset editor (CAP-060).
import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import type { PanelSettings, ScrollProps } from "../lib/commands";
import { tenStep } from "../lib/numberKeys";

const SELECTIONS = ["region", "window", "fullscreen"] as const;
/** The Image tab also scrolls (CAP-034, CAP-035) and extracts text (SMT-005). */
const IMAGE_SELECTIONS = [...SELECTIONS, "scrolling", "text"] as const;

interface Common {
  selection: (typeof IMAGE_SELECTIONS)[number];
  copy: boolean;
  cursor: boolean;
  open: "editor" | "none";
}

/** Controls shared by both tabs, in the CAP-002 order (Selection … Capture cursor).
 * With Extract text, Open in and Capture cursor do not apply: off, with the reason. */
function CommonOptions<T extends Common>({
  value,
  onChange,
  selections = SELECTIONS,
}: {
  value: T;
  onChange: (patch: Partial<Common>) => void;
  selections?: readonly Common["selection"][];
}) {
  const { t } = useTranslation();
  const id = useId();
  const text = value.selection === "text";
  // A scrolling capture has no pointer (CAP-002).
  const noCursor = text || value.selection === "scrolling";
  return (
    <>
      <label className="panel-field" htmlFor={`${id}-sel`}>
        <span>{t("ui.panel.selection")}</span>
        <select
          id={`${id}-sel`}
          value={value.selection}
          onChange={(e) => onChange({ selection: e.target.value as Common["selection"] })}
        >
          {selections.map((s) => (
            <option key={s} value={s}>
              {t(`ui.panel.sel_${s}`)}
            </option>
          ))}
        </select>
      </label>
      <label
        className="panel-field"
        htmlFor={`${id}-open`}
        title={text ? t("ui.panel.text_no_open") : undefined}
      >
        <span>{t("ui.panel.open_in")}</span>
        <select
          id={`${id}-open`}
          disabled={text}
          value={value.open}
          onChange={(e) => onChange({ open: e.target.value as Common["open"] })}
        >
          <option value="editor">{t("ui.panel.open_editor")}</option>
          <option value="none">{t("ui.panel.open_none")}</option>
        </select>
      </label>
      <label className="panel-check">
        <input
          type="checkbox"
          checked={value.copy}
          onChange={(e) => onChange({ copy: e.target.checked })}
        />
        {text ? t("ui.panel.copy_text") : t("ui.panel.copy")}
      </label>
      {/* The pointer arrives as its own object, to move or delete (CAP-047). */}
      <label
        className={`panel-check${noCursor ? " disabled" : ""}`}
        title={
          text
            ? t("ui.panel.text_no_cursor")
            : noCursor
              ? t("ui.panel.scroll_no_cursor")
              : t("ui.panel.cursor_help")
        }
      >
        <input
          type="checkbox"
          disabled={noCursor}
          checked={value.cursor && !noCursor}
          onChange={(e) => onChange({ cursor: e.target.checked })}
        />
        {t("ui.panel.cursor")}
      </label>
    </>
  );
}

export function UniversalOptions({
  value,
  onChange,
}: {
  value: PanelSettings["universal"];
  onChange: (patch: Partial<PanelSettings["universal"]>) => void;
}) {
  const { t } = useTranslation();
  return (
    <>
      {/* Its list has no Extract text, so no patch carries it. */}
      <CommonOptions
        value={value}
        onChange={(p) => onChange(p as Partial<PanelSettings["universal"]>)}
      />
      <label className="panel-check">
        <input
          type="checkbox"
          checked={value.delay5}
          onChange={(e) => onChange({ delay5: e.target.checked })}
        />
        {t("ui.panel.delay5")}
      </label>
    </>
  );
}

export function ImageOptions({
  value,
  onChange,
}: {
  value: PanelSettings["image"];
  onChange: (patch: Partial<PanelSettings["image"]>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  // What is being typed; the saved value shows otherwise.
  const [draft, setDraft] = useState<string | null>(null);
  const commitSeconds = () => {
    const n = Math.round(Number(draft));
    if (draft !== null && Number.isFinite(n) && n >= 1 && n <= 60) onChange({ delay: n });
    setDraft(null);
  };
  return (
    <>
      <CommonOptions value={value} onChange={onChange} selections={IMAGE_SELECTIONS} />
      {value.selection === "scrolling" && (
        <ScrollOptions
          value={value.scroll}
          onChange={(s) => onChange({ scroll: { ...value.scroll, ...s } })}
        />
      )}
      <div className="panel-row">
        <label className="panel-field" htmlFor={`${id}-timing`}>
          <span>{t("ui.panel.timing")}</span>
          <select
            id={`${id}-timing`}
            value={value.timing}
            onChange={(e) => onChange({ timing: e.target.value as "none" | "delay" })}
          >
            <option value="none">{t("ui.panel.timing_none")}</option>
            <option value="delay">{t("ui.panel.timing_delay")}</option>
          </select>
        </label>
        {value.timing === "delay" && (
          <label className="panel-field narrow" htmlFor={`${id}-sec`}>
            <span>{t("ui.panel.seconds")}</span>
            <input
              id={`${id}-sec`}
              type="number"
              min={1}
              max={60}
              step={1}
              value={draft ?? String(value.delay)}
              title={t("ui.panel.seconds_range")}
              onChange={(e) => setDraft(e.target.value)}
              onBlur={commitSeconds}
              onKeyDown={(e) => {
                tenStep(e, (n) => setDraft(String(n)));
                if (e.key === "Enter") commitSeconds();
              }}
            />
          </label>
        )}
      </div>
    </>
  );
}

/** The properties of Selection = Scrolling (CAP-034, CAP-035): manual or automatic, and
 * how an automatic one moves. */
function ScrollOptions({
  value,
  onChange,
}: {
  value: ScrollProps;
  onChange: (patch: Partial<ScrollProps>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  const auto = value.type === "auto";
  const [wait, setWait] = useState<string | null>(null);
  const commitWait = () => {
    const n = Math.round(Number(wait) * 10) / 10;
    if (wait !== null && Number.isFinite(n) && n >= 0 && n <= 5) onChange({ wait: n });
    setWait(null);
  };
  return (
    <fieldset className="panel-props">
      <label className="panel-field" htmlFor={`${id}-type`}>
        <span>{t("ui.panel.scroll_type")}</span>
        <select
          id={`${id}-type`}
          value={value.type}
          onChange={(e) => onChange({ type: e.target.value as ScrollProps["type"] })}
        >
          <option value="manual">{t("ui.panel.scroll_manual")}</option>
          <option value="auto">{t("ui.panel.scroll_auto")}</option>
        </select>
      </label>
      {auto && (
        <>
          <label className="panel-field" htmlFor={`${id}-dir`}>
            <span>{t("ui.panel.scroll_direction")}</span>
            <select
              id={`${id}-dir`}
              value={value.direction}
              onChange={(e) => onChange({ direction: e.target.value as ScrollProps["direction"] })}
            >
              <option value="vertical">{t("ui.panel.scroll_vertical")}</option>
              <option value="horizontal">{t("ui.panel.scroll_horizontal")}</option>
              <option value="both">{t("ui.panel.scroll_both")}</option>
            </select>
          </label>
          <label className="panel-field" htmlFor={`${id}-from`}>
            <span>{t("ui.panel.scroll_start_from")}</span>
            <select
              id={`${id}-from`}
              value={value.startFrom}
              onChange={(e) => onChange({ startFrom: e.target.value as ScrollProps["startFrom"] })}
            >
              <option value="start">{t("ui.panel.scroll_from_start")}</option>
              <option value="current">{t("ui.panel.scroll_from_current")}</option>
            </select>
          </label>
          <label className="panel-field narrow" htmlFor={`${id}-wait`}>
            <span>{t("ui.panel.scroll_wait")}</span>
            <input
              id={`${id}-wait`}
              type="number"
              min={0}
              max={5}
              step={0.1}
              value={wait ?? String(value.wait)}
              title={t("ui.panel.scroll_wait_range")}
              onChange={(e) => setWait(e.target.value)}
              onBlur={commitWait}
              onKeyDown={(e) => {
                if (e.key === "Enter") commitWait();
              }}
            />
          </label>
          <label className="panel-check">
            <input
              type="checkbox"
              checked={value.restore}
              onChange={(e) => onChange({ restore: e.target.checked })}
            />
            {t("ui.panel.scroll_restore")}
          </label>
        </>
      )}
    </fieldset>
  );
}

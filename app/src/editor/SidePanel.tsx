// Side panel (EDT-005) with the Properties of the selected objects or of the active
// tool (EDT-014). Saved styles arrive in M2: the Styles tab says so.
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { CAPS, LINE_STYLES, type ArrowStyle } from "./geometry";
import { useEditor, usePanel, type ArrowObject, type DocDetails } from "./state";
import { useDraft } from "./useDraft";

/** Colors of the default theme "Base" (EDT-017) and a few basics until the full color
 * selector (EDT-019, build 4b). */
const SWATCHES = [
  "#F2545B",
  "#2D7FF9",
  "#F5A524",
  "#1F2430",
  "#FFFFFF",
  "#16A34A",
  "#9333EA",
  "#000000",
];

function useCommit(): (changes: Partial<ArrowStyle>) => void {
  const { active, selection, showError } = useEditor.getState();
  return (changes) => {
    // EDT-014: with objects selected the change goes to them (and becomes the tool's
    // style); without, it changes the tool's style.
    const p =
      active && selection.length > 0
        ? run("document.update_object", {
            documentId: active,
            objectIds: selection,
            set: { style: changes },
          })
        : run("editor.set_style", { tool: "arrow", style: changes });
    p.catch(showError);
  };
}

/** Slider with its number field. The value is committed when the gesture ends, so a
 * drag is one undo step (EDT-092). Arrow keys step 1, Shift+arrows 10 (EDT-013). */
function NumberField(props: {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  unit: string;
  onCommit: (v: number) => void;
}) {
  const { label, value, min, max, step, unit, onCommit } = props;
  const [draft, setDraft] = useDraft(String(value));
  const [sliding, setSliding] = useState<number | null>(null);
  const id = `f-${label}`;
  const commit = (v: number) => {
    const clamped = Math.min(max, Math.max(min, v));
    if (Number.isFinite(clamped) && clamped !== value) onCommit(clamped);
  };
  const typed = () => {
    const v = Number(draft.replace(",", "."));
    if (Number.isFinite(v)) commit(v);
    setDraft(null);
  };
  return (
    <div className="prop-field">
      <label htmlFor={id}>{label}</label>
      <div className="prop-number">
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={sliding ?? value}
          aria-label={label}
          onChange={(e) => setSliding(Number(e.target.value))}
          onPointerUp={() => {
            if (sliding !== null) commit(sliding);
            setSliding(null);
          }}
          onKeyUp={() => {
            if (sliding !== null) commit(sliding);
            setSliding(null);
          }}
        />
        <input
          id={id}
          type="text"
          inputMode="decimal"
          value={sliding !== null ? String(sliding) : draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={typed}
          onKeyDown={(e) => {
            if (e.key === "Enter") typed();
            if (e.key === "ArrowUp" || e.key === "ArrowDown") {
              e.preventDefault();
              const d = (e.key === "ArrowUp" ? 1 : -1) * (e.shiftKey ? 10 : 1);
              commit(value + d);
            }
          }}
        />
        <span className="muted">{unit}</span>
      </div>
    </div>
  );
}

function ColorField({ value, onCommit }: { value: string; onCommit: (v: string) => void }) {
  const { t } = useTranslation();
  const [draft, setDraft] = useDraft(value);
  const typed = () => {
    const v = draft.trim().toUpperCase();
    const hex = v.startsWith("#") ? v : `#${v}`;
    if (/^#[0-9A-F]{6}([0-9A-F]{2})?$/.test(hex)) {
      if (hex !== value.toUpperCase()) onCommit(hex);
    }
    setDraft(null);
  };
  return (
    <div className="prop-field">
      <span id="color-label">{t("ui.editor.prop.color")}</span>
      <div className="swatches" role="group" aria-labelledby="color-label">
        {SWATCHES.map((c) => (
          <button
            key={c}
            type="button"
            className="swatch"
            style={{ background: c }}
            aria-pressed={c.toUpperCase() === value.toUpperCase()}
            aria-label={c}
            title={c}
            onClick={() => onCommit(c)}
          />
        ))}
      </div>
      <input
        className="hex"
        type="text"
        value={draft}
        aria-label={t("ui.editor.prop.hex")}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={typed}
        onKeyDown={(e) => e.key === "Enter" && typed()}
      />
    </div>
  );
}

function ArrowProperties({ style }: { style: ArrowStyle }) {
  const { t } = useTranslation();
  const commit = useCommit();
  const capOptions = CAPS.map((c) => (
    <option key={c} value={c}>
      {t(`ui.editor.cap.${c}`)}
    </option>
  ));
  return (
    <>
      <ColorField value={style.color} onCommit={(color) => commit({ color })} />
      <NumberField
        label={t("ui.editor.prop.width")}
        value={style.width}
        min={1}
        max={100}
        step={0.5}
        unit="px"
        onCommit={(width) => commit({ width })}
      />
      <div className="prop-field">
        <label htmlFor="start-cap">{t("ui.editor.prop.start_cap")}</label>
        <select
          id="start-cap"
          value={style.startCap}
          onChange={(e) => commit({ startCap: e.target.value as ArrowStyle["startCap"] })}
        >
          {capOptions}
        </select>
      </div>
      <div className="prop-field">
        <label htmlFor="end-cap">{t("ui.editor.prop.end_cap")}</label>
        <select
          id="end-cap"
          value={style.endCap}
          onChange={(e) => commit({ endCap: e.target.value as ArrowStyle["endCap"] })}
        >
          {capOptions}
        </select>
      </div>
      <div className="prop-field">
        <label htmlFor="line-style">{t("ui.editor.prop.line_style")}</label>
        <select
          id="line-style"
          value={style.lineStyle}
          onChange={(e) => commit({ lineStyle: e.target.value as ArrowStyle["lineStyle"] })}
        >
          {LINE_STYLES.map((l) => (
            <option key={l} value={l}>
              {t(`ui.editor.line.${l}`)}
            </option>
          ))}
        </select>
      </div>
      <NumberField
        label={t("ui.editor.prop.opacity")}
        value={style.opacity}
        min={0}
        max={100}
        step={1}
        unit="%"
        onCommit={(opacity) => commit({ opacity })}
      />
    </>
  );
}

function DocumentProperties({ doc }: { doc: DocDetails }) {
  const { t } = useTranslation();
  return (
    <dl className="doc-props">
      <dt>{t("ui.editor.prop.size")}</dt>
      <dd>
        {doc.canvas.width} × {doc.canvas.height} px
      </dd>
      <dt>{t("ui.editor.prop.resolution")}</dt>
      <dd>{t("ui.editor.dpi", { dpi: Math.round(doc.dpi) })}</dd>
      <dt>{t("ui.editor.prop.canvas_color")}</dt>
      <dd>{doc.canvas.color ?? t("ui.editor.transparent")}</dd>
      <dt>{t("ui.editor.prop.file")}</dt>
      <dd className="path">{doc.path ?? "—"}</dd>
    </dl>
  );
}

export const PANEL_MIN = 260;
export const PANEL_MAX = 480;

/** Drag the left edge to resize the panel (EDT-005). */
function ResizeEdge() {
  const { t } = useTranslation();
  const width = usePanel((s) => s.width);
  const set = usePanel((s) => s.set);
  // A ref, not state: pointer moves arrive faster than renders.
  const start = useRef<{ x: number; width: number } | null>(null);
  const clamp = (w: number) => Math.min(PANEL_MAX, Math.max(PANEL_MIN, w));
  return (
    <div
      className="panel-edge"
      role="separator"
      aria-orientation="vertical"
      aria-label={t("ui.editor.panel_width")}
      aria-valuemin={PANEL_MIN}
      aria-valuemax={PANEL_MAX}
      aria-valuenow={width}
      tabIndex={0}
      onPointerDown={(e) => {
        e.currentTarget.setPointerCapture(e.pointerId);
        start.current = { x: e.clientX, width };
      }}
      onPointerMove={(e) => {
        const s = start.current;
        if (s) set({ width: clamp(s.width + s.x - e.clientX) });
      }}
      onPointerUp={() => (start.current = null)}
      onKeyDown={(e) => {
        const step = e.shiftKey ? 40 : 10;
        if (e.key === "ArrowLeft") set({ width: clamp(width + step) });
        if (e.key === "ArrowRight") set({ width: clamp(width - step) });
      }}
    />
  );
}

export function SidePanel() {
  const { t } = useTranslation();
  const visible = usePanel((s) => s.visible);
  const width = usePanel((s) => s.width);
  const tab = usePanel((s) => s.tab);
  const setPanel = usePanel((s) => s.set);
  const setTab = (tab: "props" | "styles") => setPanel({ tab });
  const doc = useEditor((s) => s.doc);
  const tool = useEditor((s) => s.tool);
  const styles = useEditor((s) => s.styles);
  const selection = useEditor((s) => s.selection);

  const selected = (doc?.objects ?? []).filter(
    (o): o is ArrowObject => o.type === "arrow" && selection.includes(o.id),
  );
  const first = selected[0];

  let title: string;
  let body: React.ReactNode;
  if (first) {
    title = t("ui.editor.props_selected", { count: selected.length });
    body = <ArrowProperties style={first.style} />;
  } else if (tool === "arrow" && styles) {
    title = t("ui.editor.props_tool", { tool: t("ui.editor.tool.arrow") });
    body = <ArrowProperties style={styles.arrow} />;
  } else if (doc) {
    title = t("ui.editor.props_document");
    body = <DocumentProperties doc={doc} />;
  } else {
    title = t("ui.editor.props_document");
    body = <p className="muted">{t("ui.editor.no_document")}</p>;
  }

  if (!visible) return null;
  return (
    <aside className="side-panel" style={{ width }} aria-label={t("ui.editor.side_panel")}>
      <ResizeEdge />
      <div className="panel-tabs" role="tablist">
        <button
          type="button"
          role="tab"
          aria-selected={tab === "styles"}
          onClick={() => setTab("styles")}
        >
          {t("ui.editor.tab_styles")}
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={tab === "props"}
          onClick={() => setTab("props")}
        >
          {t("ui.editor.tab_properties")}
        </button>
      </div>
      <div className="side-body" role="tabpanel">
        {tab === "styles" ? (
          <p className="muted">{t("ui.editor.styles_soon")}</p>
        ) : (
          <section aria-label={title}>
            <h2 className="side-title">{title}</h2>
            {body}
          </section>
        )}
      </div>
    </aside>
  );
}

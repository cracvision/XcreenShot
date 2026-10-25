// Side panel (EDT-005) with the Properties of the selected objects, of the text being
// typed or of the active tool (EDT-014). Saved styles arrive in M2: the Styles tab says so.
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { applyCrop, loadFonts, resetCrop } from "./actions";
import { ColorPicker } from "./ColorPicker";
import { CAPS, LINE_STYLES, type Cap, type LineStyle } from "./geometry";
import { Icon, type IconName } from "./icons";
import {
  useEditor,
  usePanel,
  type BlurType,
  type DocDetails,
  type ShapeKind,
  type TextAlign,
  type TextStyle,
} from "./state";
import { useDraft } from "./useDraft";

/** Slider with its number field. The value is committed when the gesture ends, so a
 * drag is one undo step (EDT-092). Arrow keys step 1, Shift+arrows 10 (EDT-013). */
function NumberField(props: {
  label: string;
  /** `undefined` when the selected objects differ (Mixed, EDT-014). */
  value: number | undefined;
  min: number;
  max: number;
  step: number;
  unit: string;
  onCommit: (v: number) => void;
}) {
  const { label, min, max, step, unit, onCommit } = props;
  const { t } = useTranslation();
  const mixed = props.value === undefined;
  const value = props.value ?? min;
  const [draft, setDraft] = useDraft(mixed ? "" : String(value));
  const [sliding, setSliding] = useState<number | null>(null);
  const id = `f-${label}`;
  const commit = (v: number) => {
    const clamped = Math.min(max, Math.max(min, v));
    if (Number.isFinite(clamped) && (mixed || clamped !== value)) onCommit(clamped);
  };
  const typed = () => {
    const v = Number(draft.replace(",", "."));
    if (draft.trim() !== "" && Number.isFinite(v)) commit(v);
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
          placeholder={mixed ? t("ui.editor.mixed") : ""}
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

/** What the Properties tab edits: the text being typed, the selected objects, or the
 * active tool's style (EDT-014). */
interface Target {
  kind: StyledKind;
  /** One style per object (or the tool's alone). */
  styles: Record<string, unknown>[];
  commit: (changes: Record<string, unknown>) => void;
}

type StyledKind = "arrow" | "text" | "shape" | "blur";
const STYLED: StyledKind[] = ["arrow", "text", "shape", "blur"];

function useTarget(): Target | null {
  const doc = useEditor((s) => s.doc);
  const tool = useEditor((s) => s.tool);
  const styles = useEditor((s) => s.styles);
  const selection = useEditor((s) => s.selection);
  const textEdit = useEditor((s) => s.textEdit);
  const { active, showError } = useEditor.getState();
  if (!styles) return null;
  const fail = (p: Promise<unknown>) => void p.catch(showError);

  if (textEdit) {
    return {
      kind: "text",
      styles: [textEdit.style as unknown as Record<string, unknown>],
      commit: (changes) => {
        const next = { ...textEdit.style, ...changes } as TextStyle;
        useEditor.setState({ textEdit: { ...textEdit, style: next } });
        fail(
          textEdit.id && active
            ? run("document.update_object", {
                documentId: active,
                objectId: textEdit.id,
                set: { style: changes },
              })
            : run("editor.set_style", { tool: "text", style: changes }),
        );
      },
    };
  }
  const selected = (doc?.objects ?? []).filter((o) => selection.includes(o.id));
  const kinds = [...new Set(selected.map((o) => o.type))];
  const kind = kinds[0];
  if (selected.length > 0 && kinds.length === 1 && STYLED.includes(kind as StyledKind)) {
    return {
      kind: kind as StyledKind,
      styles: selected.map((o) =>
        "style" in o ? (o.style as unknown as Record<string, unknown>) : {},
      ),
      // EDT-014: the change goes to the selected objects and becomes the tool's style.
      commit: (changes) =>
        fail(
          run("document.update_object", {
            documentId: active,
            objectIds: selection,
            set: { style: changes },
          }),
        ),
    };
  }
  if (selected.length === 0 && STYLED.includes(tool as StyledKind)) {
    const k = tool as StyledKind;
    return {
      kind: k,
      styles: [styles[k] as unknown as Record<string, unknown>],
      commit: (changes) => fail(run("editor.set_style", { tool: k, style: changes })),
    };
  }
  return null;
}

/** The value of `key` when every style agrees, else `undefined` (shown as Mixed). */
function common<T>(styles: Record<string, unknown>[], key: string): T | undefined {
  const first = styles[0]?.[key];
  return styles.every((s) => s[key] === first) ? (first as T) : undefined;
}

function SelectField<T extends string>(props: {
  label: string;
  value: T | undefined;
  options: { value: T; label: string; title?: string }[];
  onCommit: (v: T) => void;
}) {
  const { t } = useTranslation();
  const id = `sel-${props.label}`;
  return (
    <div className="prop-field">
      <label htmlFor={id}>{props.label}</label>
      <select
        id={id}
        value={props.value ?? ""}
        onChange={(e) => props.onCommit(e.target.value as T)}
      >
        {props.value === undefined && <option value="">{t("ui.editor.mixed")}</option>}
        {props.options.map((o) => (
          <option key={o.value} value={o.value} title={o.title}>
            {o.label}
          </option>
        ))}
      </select>
    </div>
  );
}

function Properties({ target }: { target: Target }) {
  const { t } = useTranslation();
  const fonts = useEditor((s) => s.fonts);
  const { kind, styles, commit } = target;
  const v = <T,>(key: string) => common<T>(styles, key);
  const num = (key: string) => v<number>(key);
  const opacity = (
    <NumberField
      label={t("ui.editor.prop.opacity")}
      value={num("opacity")}
      min={0}
      max={100}
      step={1}
      unit="%"
      onCommit={(opacity) => commit({ opacity })}
    />
  );
  const lineStyle = (
    <SelectField
      label={t("ui.editor.prop.line_style")}
      value={v<LineStyle>("lineStyle")}
      options={LINE_STYLES.map((l) => ({ value: l, label: t(`ui.editor.line.${l}`) }))}
      onCommit={(lineStyle) => commit({ lineStyle })}
    />
  );
  const shapeType = (
    <SelectField
      label={t("ui.editor.prop.shape_type")}
      value={v<ShapeKind>("shapeType")}
      options={(["rectangle", "ellipse"] as const).map((s) => ({
        value: s,
        label: t(`ui.editor.shape.${s}`),
      }))}
      onCommit={(shapeType) => commit({ shapeType })}
    />
  );
  switch (kind) {
    case "arrow": {
      const caps = CAPS.map((c) => ({ value: c, label: t(`ui.editor.cap.${c}`) }));
      return (
        <>
          <ColorPicker
            label={t("ui.editor.prop.color")}
            value={v<string>("color") ?? "#000000"}
            mixed={v<string>("color") === undefined}
            onCommit={(color) => color && commit({ color })}
          />
          <NumberField
            label={t("ui.editor.prop.width")}
            value={num("width")}
            min={1}
            max={100}
            step={0.5}
            unit="px"
            onCommit={(width) => commit({ width })}
          />
          <SelectField
            label={t("ui.editor.prop.start_cap")}
            value={v<Cap>("startCap")}
            options={caps}
            onCommit={(startCap) => commit({ startCap })}
          />
          <SelectField
            label={t("ui.editor.prop.end_cap")}
            value={v<Cap>("endCap")}
            options={caps}
            onCommit={(endCap) => commit({ endCap })}
          />
          {lineStyle}
          {opacity}
        </>
      );
    }
    case "text": {
      const align = v<TextAlign>("align");
      const toggle = (key: "bold" | "italic", icon: IconName) => (
        <button
          type="button"
          className="btn icon"
          aria-pressed={v<boolean>(key) === true}
          aria-label={t(`ui.editor.prop.${key}`)}
          title={t(`ui.editor.prop.${key}`)}
          onClick={() => commit({ [key]: v<boolean>(key) !== true })}
        >
          <Icon name={icon} />
        </button>
      );
      return (
        <>
          <div className="prop-field">
            <label htmlFor="font-field">{t("ui.editor.prop.font")}</label>
            <FontField
              value={v<string>("font")}
              fonts={fonts}
              onCommit={(font) => commit({ font })}
            />
          </div>
          <NumberField
            label={t("ui.editor.prop.font_size")}
            value={num("size")}
            min={1}
            max={1000}
            step={0.5}
            unit="pt"
            onCommit={(size) => commit({ size })}
          />
          <div className="prop-field">
            <span id="style-label">{t("ui.editor.prop.text_style")}</span>
            <div className="prop-buttons" role="group" aria-labelledby="style-label">
              {toggle("bold", "bold")}
              {toggle("italic", "italic")}
            </div>
          </div>
          <ColorPicker
            label={t("ui.editor.prop.color")}
            value={v<string>("color") ?? "#000000"}
            mixed={v<string>("color") === undefined}
            onCommit={(color) => color && commit({ color })}
          />
          <div className="prop-field">
            <span id="align-label">{t("ui.editor.prop.align")}</span>
            <div className="prop-buttons" role="radiogroup" aria-labelledby="align-label">
              {(["left", "center", "right", "justify"] as const).map((a) => (
                <button
                  key={a}
                  type="button"
                  role="radio"
                  className="btn icon"
                  aria-checked={align === a}
                  aria-label={t(`ui.editor.align.${a}`)}
                  title={t(`ui.editor.align.${a}`)}
                  onClick={() => commit({ align: a })}
                >
                  <Icon name={ALIGN_ICONS[a]} />
                </button>
              ))}
            </div>
          </div>
          {opacity}
        </>
      );
    }
    case "shape":
      return (
        <>
          {shapeType}
          <ColorPicker
            label={t("ui.editor.prop.fill")}
            value={v<string | null>("fill") ?? null}
            mixed={v<string | null>("fill") === undefined}
            allowTransparent
            onCommit={(fill) => commit({ fill })}
          />
          <ColorPicker
            label={t("ui.editor.prop.outline")}
            value={v<string>("outline") ?? "#000000"}
            mixed={v<string>("outline") === undefined}
            onCommit={(outline) => outline && commit({ outline })}
          />
          <NumberField
            label={t("ui.editor.prop.outline_width")}
            value={num("outlineWidth")}
            min={0}
            max={100}
            step={0.5}
            unit="px"
            onCommit={(outlineWidth) => commit({ outlineWidth })}
          />
          {lineStyle}
          {opacity}
        </>
      );
    case "blur": {
      const blurType = v<BlurType>("blurType");
      return (
        <>
          <SelectField
            label={t("ui.editor.prop.blur_type")}
            value={blurType}
            options={(["blur", "pixelate", "solid"] as const).map((b) => ({
              value: b,
              label: t(`ui.editor.blur.${b}`),
              ...(b === "solid" ? { title: t("ui.editor.blur.solid_hint") } : {}),
            }))}
            onCommit={(blurType) => commit({ blurType })}
          />
          {blurType === "solid" ? (
            <ColorPicker
              label={t("ui.editor.prop.color")}
              value={v<string>("color") ?? "#000000"}
              mixed={v<string>("color") === undefined}
              onCommit={(color) => color && commit({ color })}
            />
          ) : (
            <NumberField
              label={t("ui.editor.prop.intensity")}
              value={num("intensity")}
              min={1}
              max={100}
              step={1}
              unit=""
              onCommit={(intensity) => commit({ intensity })}
            />
          )}
          {shapeType}
          <p className="muted hint">{t("ui.editor.blur.security")}</p>
        </>
      );
    }
  }
}

const ALIGN_ICONS: Record<TextAlign, IconName> = {
  left: "alignLeft",
  center: "alignCenter",
  right: "alignRight",
  justify: "alignJustify",
};

/** Installed fonts with search (EDT-026): a field that suggests as you type. */
function FontField(props: {
  value: string | undefined;
  fonts: string[];
  onCommit: (v: string) => void;
}) {
  const { t } = useTranslation();
  const [draft, setDraft] = useDraft(props.value ?? "");
  const typed = () => {
    const name = draft.trim();
    if (name && name !== props.value) props.onCommit(name);
    setDraft(null);
  };
  return (
    <>
      <input
        id="font-field"
        list="font-list"
        value={draft}
        placeholder={props.value === undefined ? t("ui.editor.mixed") : ""}
        onFocus={() => void loadFonts()}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={typed}
        onKeyDown={(e) => e.key === "Enter" && typed()}
      />
      <datalist id="font-list">
        {props.fonts.map((f) => (
          <option key={f} value={f} />
        ))}
      </datalist>
    </>
  );
}

/** While Crop is active (EDT-049): the size of the box, Crop and Cancel. */
function CropProperties() {
  const { t } = useTranslation();
  const crop = useEditor((s) => s.cropRect);
  const doc = useEditor((s) => s.doc);
  const rect = crop ?? (doc ? { x: 0, y: 0, w: doc.canvas.width, h: doc.canvas.height } : null);
  if (!rect) return null;
  return (
    <>
      <p className="muted">{t("ui.editor.crop_help")}</p>
      <dl className="doc-props">
        <dt>{t("ui.editor.prop.size")}</dt>
        <dd>
          {Math.round(rect.w)} × {Math.round(rect.h)} px
        </dd>
      </dl>
      <div className="row">
        <button type="button" className="btn primary" onClick={() => void applyCrop()}>
          {t("ui.editor.crop_apply")}
        </button>
        <button type="button" className="btn" onClick={resetCrop}>
          {t("ui.editor.cancel")}
        </button>
      </div>
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
  const selection = useEditor((s) => s.selection);
  const editing = useEditor((s) => s.textEdit !== null);
  const target = useTarget();

  let title: string;
  let body: React.ReactNode;
  if (target && (selection.length > 0 || editing)) {
    const count = target.styles.length;
    title = t("ui.editor.props_selection", {
      what:
        count > 1 ? t("ui.editor.objects_count", { count }) : t(`ui.editor.tool.${target.kind}`),
    });
    body = <Properties target={target} />;
  } else if (target) {
    title = t("ui.editor.props_tool", { tool: t(`ui.editor.tool.${target.kind}`) });
    body = <Properties target={target} />;
  } else if (doc && tool === "crop") {
    title = t("ui.editor.tool.crop");
    body = <CropProperties />;
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

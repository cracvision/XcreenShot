// Side panel (EDT-005) with the Properties of the selected objects, of the text being
// typed or of the active tool (EDT-014). Saved styles arrive in M2: the Styles tab says so.
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useIndicator } from "../components/useIndicator";
import { run } from "../lib/commands";
import { useSetting } from "../store";
import { applyCrop, loadFonts, resetCrop } from "./actions";
import { ColorPicker } from "./ColorPicker";
import { FillPicker } from "./FillPicker";
import { Field, PositionSize } from "./PositionSize";
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
  STEP_SEQUENCES,
  STEP_SHAPES,
  type StepSequence,
  type StepShapeKind,
  CALLOUT_SHAPES,
  type CalloutShapeKind,
  type TailStyle,
  type VAlign,
  HIGHLIGHT_SHAPES,
  type HighlightShapeKind,
  PEN_TIPS,
  type PenTip,
  LENS_SHAPES,
  type LensShape,
  SPOT_SHAPES,
  type SpotShape,
  SHAPE_KINDS,
  SIMPLE_KINDS,
  type SimpleKind,
  type GapFill,
  type Fill,
  type SelectStyle,
  type Shadow,
  WITH_SHADOW,
} from "./state";
import { useDraft } from "./useDraft";
import { stepSequence } from "./actions";
import { selectPixels } from "./pixelSelection";
import { StampBrowser } from "./StampBrowser";
import { StylesTab } from "./StylesTab";

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
          onKeyDown={(e) => {
            // Shift + arrows: ten steps at once (EDT-013); alone, the slider's own step.
            const dir = { ArrowUp: 1, ArrowRight: 1, ArrowDown: -1, ArrowLeft: -1 }[e.key];
            if (!e.shiftKey || dir === undefined) return;
            e.preventDefault();
            commit((sliding ?? value) + dir * step * 10);
            setSliding(null);
          }}
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

type StyledKind =
  | "arrow"
  | "line"
  | "text"
  | "shape"
  | "blur"
  | "image"
  | "step"
  | "callout"
  | "highlight"
  | "pen"
  | "magnify"
  | "spotlight"
  | "stamp"
  | "simplify"
  | "fill"
  | "eraser"
  | "select"
  | "wand";
/** Tools that change pixels (EDT-047, EDT-048): their own style shows even with a
 * selection, which says only which floating image the eraser works on. */
const PIXEL_TOOLS: StyledKind[] = ["fill", "eraser", "select", "wand"];
const STYLED: StyledKind[] = [
  "arrow",
  "line",
  "text",
  "shape",
  "blur",
  "image",
  "step",
  "callout",
  "highlight",
  "pen",
  "magnify",
  "spotlight",
  "stamp",
  "simplify",
  "fill",
  "eraser",
  "select",
  "wand",
];

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
  if (PIXEL_TOOLS.includes(tool as StyledKind)) {
    const k = tool as "fill" | "eraser" | "select" | "wand";
    return {
      kind: k,
      styles: [styles[k] as unknown as Record<string, unknown>],
      commit: (changes) => fail(run("editor.set_style", { tool: k, style: changes })),
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
  // A tool's own style; there is no image tool, images are inserted (EDT-078).
  if (selected.length === 0 && STYLED.includes(tool as StyledKind)) {
    const k = tool as Exclude<StyledKind, "image">;
    return {
      kind: k,
      styles: [styles[k] as unknown as Record<string, unknown>],
      commit: (changes) => fail(run("editor.set_style", { tool: k, style: changes })),
    };
  }
  return null;
}

/** Several objects of different kinds: the properties they all have, changed in all of
 * them at once (EDT-080, EDT-014). For now their opacity. */
function SharedProperties() {
  const { t } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const selection = useEditor((s) => s.selection);
  const { active, showError } = useEditor.getState();
  const styles = (doc?.objects ?? [])
    .filter((o) => selection.includes(o.id))
    .map((o) => ("style" in o ? (o.style as unknown as Record<string, unknown>) : {}));
  if (styles.length === 0 || !styles.every((s) => typeof s.opacity === "number")) return null;
  return (
    <NumberField
      label={t("ui.editor.prop.opacity")}
      value={common<number>(styles, "opacity")}
      min={0}
      max={100}
      step={1}
      unit="%"
      onCommit={(opacity) =>
        void run("document.update_object", {
          documentId: active,
          objectIds: selection,
          set: { style: { opacity } },
        }).catch(showError)
      }
    />
  );
}

/** The value of `key` when every style agrees, else `undefined` (shown as Mixed). */
function common<T>(styles: Record<string, unknown>[], key: string): T | undefined {
  const first = styles[0]?.[key];
  // Objects (a gradient, a shadow) are the same when their values are.
  const text = JSON.stringify(first ?? null);
  return styles.every((s) => s[key] === first || JSON.stringify(s[key] ?? null) === text)
    ? (first as T)
    : undefined;
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
  const shadowed = (WITH_SHADOW as readonly string[]).includes(target.kind);
  const shadows = target.styles.map((s) => (s.shadow ?? null) as Shadow | null);
  const first = JSON.stringify(shadows[0] ?? null);
  const mixed = shadows.some((s) => JSON.stringify(s) !== first);
  return (
    <>
      <KindProperties target={target} />
      {shadowed && <ShadowSection value={shadows[0]} mixed={mixed} commit={target.commit} />}
    </>
  );
}

function KindProperties({ target }: { target: Target }) {
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
  // What fills the place selected pixels leave (EDT-045).
  const gapField = (
    <SelectField
      label={t("ui.editor.pixel.gap_fill")}
      value={v<GapFill>("gapFill")}
      options={(["smart", "transparent", "canvas"] as const).map((g) => ({
        value: g,
        label: t(`ui.editor.pixel.gap.${g}`),
      }))}
      onCommit={(gapFill) => commit({ gapFill })}
    />
  );
  // Smart fill: Fast or Photo (EDT-045).
  const qualityField = v<GapFill>("gapFill") === "smart" && (
    <>
      <SelectField
        label={t("ui.editor.pixel.quality")}
        value={v<"fast" | "photo">("quality")}
        options={(["fast", "photo"] as const).map((q) => ({
          value: q,
          label: t(`ui.editor.pixel.quality_${q}`),
        }))}
        onCommit={(quality) => commit({ quality })}
      />
      <p className="muted hint">{t("ui.editor.pixel.quality_help")}</p>
    </>
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
      options={(kind === "shape" ? SHAPE_KINDS : (["rectangle", "ellipse"] as const)).map((s) => ({
        value: s,
        label: t(`ui.editor.shape.${s}`),
      }))}
      onCommit={(shapeType) => commit({ shapeType })}
    />
  );
  // Font, size, bold and italic, color and alignment: text and callouts (EDT-026, EDT-028).
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
  const textBlock = (colorKey: "color" | "textColor") => (
    <>
      <div className="prop-field">
        <label htmlFor="font-field">{t("ui.editor.prop.font")}</label>
        <FontField value={v<string>("font")} fonts={fonts} onCommit={(font) => commit({ font })} />
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
        label={t(colorKey === "color" ? "ui.editor.prop.color" : "ui.editor.prop.text_color")}
        value={v<string>(colorKey) ?? "#000000"}
        mixed={v<string>(colorKey) === undefined}
        onCommit={(color) => color && commit({ [colorKey]: color })}
      />
      <div className="prop-field">
        <span id="align-label">{t("ui.editor.prop.align")}</span>
        <AlignChoice value={align} onChange={(a) => commit({ align: a })} />
      </div>
    </>
  );
  switch (kind) {
    case "arrow":
    case "line": {
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
    case "text":
      return (
        <>
          {textBlock("color")}
          {opacity}
        </>
      );
    case "callout":
      return (
        <CalloutProperties
          v={v}
          num={num}
          commit={commit}
          lineStyle={lineStyle}
          text={textBlock("textColor")}
          opacity={opacity}
        />
      );
    case "shape":
      return (
        <>
          {shapeType}
          {/* Rounded rectangle: its corners, in all the selected ones (EDT-030). */}
          {v<ShapeKind>("shapeType") === "roundedRectangle" && (
            <NumberField
              label={t("ui.editor.prop.corner_radius")}
              value={num("cornerRadius")}
              min={0}
              max={500}
              step={1}
              unit="px"
              onCommit={(cornerRadius) => commit({ cornerRadius })}
            />
          )}
          <FillPicker
            label={t("ui.editor.prop.fill")}
            value={v<Fill | null>("fill") ?? null}
            mixed={v<Fill | null>("fill") === undefined}
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
    case "image":
      return opacity;
    case "simplify":
      return <SimplifyProperties v={v} num={num} commit={commit} opacity={opacity} />;
    case "stamp":
      // A stamp's color (its own, or another for monochrome ones) and opacity (EDT-031).
      return (
        <>
          <ColorPicker
            label={t("ui.editor.prop.color")}
            value={v<string | null>("color") ?? "#3A3F4B"}
            mixed={v<string | null>("color") === undefined}
            onCommit={(color) => color && commit({ color })}
          />
          <button type="button" className="btn small" onClick={() => commit({ color: null })}>
            {t("ui.editor.stamps.own_color")}
          </button>
          {opacity}
        </>
      );
    case "spotlight":
      // Spotlight (EDT-039): the outside's color, opacity, blur and soft edge, shared.
      return (
        <>
          <SelectField
            label={t("ui.editor.prop.spot_shape")}
            value={v<SpotShape>("shape")}
            options={SPOT_SHAPES.map((s) => ({ value: s, label: t(`ui.editor.spot_shape.${s}`) }))}
            onCommit={(shape) => commit({ shape })}
          />
          <ColorPicker
            label={t("ui.editor.prop.outside_color")}
            value={v<string>("color") ?? "#000000"}
            mixed={v<string>("color") === undefined}
            onCommit={(color) => color && commit({ color })}
          />
          {opacity}
          <NumberField
            label={t("ui.editor.prop.outside_blur")}
            value={num("blur")}
            min={0}
            max={100}
            step={1}
            unit=""
            onCommit={(blur) => commit({ blur })}
          />
          <NumberField
            label={t("ui.editor.prop.feather")}
            value={num("feather")}
            min={0}
            max={50}
            step={1}
            unit="px"
            onCommit={(feather) => commit({ feather })}
          />
        </>
      );
    case "magnify":
      // Magnify (EDT-038): shape, zoom, outline and connectors.
      return (
        <>
          <SelectField
            label={t("ui.editor.prop.lens_shape")}
            value={v<LensShape>("shape")}
            options={LENS_SHAPES.map((s) => ({ value: s, label: t(`ui.editor.lens_shape.${s}`) }))}
            onCommit={(shape) => commit({ shape })}
          />
          <NumberField
            label={t("ui.editor.prop.zoom")}
            value={num("zoom")}
            min={110}
            max={800}
            step={10}
            unit="%"
            onCommit={(zoom) => commit({ zoom })}
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
            max={20}
            step={0.5}
            unit="px"
            onCommit={(outlineWidth) => commit({ outlineWidth })}
          />
          <label className="panel-check">
            <input
              type="checkbox"
              checked={v<boolean>("connectors") !== false}
              onChange={(e) => commit({ connectors: e.target.checked })}
            />
            {t("ui.editor.prop.connectors")}
          </label>
          {opacity}
        </>
      );
    case "pen":
      // Pen (EDT-037): color, width, tip, smoothing; editable is how new strokes are made.
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
            step={1}
            unit="px"
            onCommit={(width) => commit({ width })}
          />
          <SelectField
            label={t("ui.editor.prop.pen_tip")}
            value={v<PenTip>("tip")}
            options={PEN_TIPS.map((s) => ({ value: s, label: t(`ui.editor.pen_tip.${s}`) }))}
            onCommit={(tip) => commit({ tip })}
          />
          <NumberField
            label={t("ui.editor.prop.smooth")}
            value={num("smooth")}
            min={0}
            max={100}
            step={1}
            unit="%"
            onCommit={(smooth) => commit({ smooth })}
          />
          <label className="panel-check">
            <input
              type="checkbox"
              checked={v<boolean>("antialias") !== false}
              onChange={(e) => commit({ antialias: e.target.checked })}
            />
            {t("ui.editor.prop.antialias")}
          </label>
          <label className="panel-check">
            <input
              type="checkbox"
              checked={v<boolean>("editable") !== false}
              onChange={(e) => commit({ editable: e.target.checked })}
            />
            {t("ui.editor.prop.editable")}
          </label>
          {opacity}
        </>
      );
    case "highlight":
      // Highlighter (EDT-036): color, opacity and shape.
      return (
        <>
          <SelectField
            label={t("ui.editor.prop.highlight_shape")}
            value={v<HighlightShapeKind>("shape")}
            options={HIGHLIGHT_SHAPES.map((s) => ({
              value: s,
              label: t(`ui.editor.highlight_shape.${s}`),
            }))}
            onCommit={(shape) => commit({ shape })}
          />
          <ColorPicker
            label={t("ui.editor.prop.color")}
            value={v<string>("color") ?? "#FFE14D"}
            mixed={v<string>("color") === undefined}
            onCommit={(color) => color && commit({ color })}
          />
          {opacity}
        </>
      );
    case "step":
      return <StepProperties v={v} num={num} commit={commit} opacity={opacity} />;
    case "fill":
      // Fill (EDT-047): a color or Transparent, how alike, and everywhere or touching.
      return (
        <>
          <FillPicker
            label={t("ui.editor.pixel.fill_color")}
            value={v<Fill | null>("color") ?? null}
            allowTransparent
            onCommit={(color) => commit({ color })}
          />
          <NumberField
            label={t("ui.editor.pixel.tolerance")}
            value={num("tolerance")}
            min={0}
            max={100}
            step={1}
            unit="%"
            onCommit={(tolerance) => commit({ tolerance })}
          />
          {opacity}
          <label className="panel-check" title={t("ui.editor.pixel.global_help")}>
            <input
              type="checkbox"
              checked={v<boolean>("global") === true}
              onChange={(e) => commit({ global: e.target.checked })}
            />
            {t("ui.editor.pixel.global")}
          </label>
        </>
      );
    case "select": {
      // Selection (EDT-043): its shape, Snap to object, edges and the gap (EDT-045).
      const shape = v<SelectStyle["shape"]>("shape");
      return (
        <>
          <SelectField
            label={t("ui.editor.pixel.sel_shape")}
            value={shape}
            options={(["rectangle", "ellipse", "polygon", "freehand"] as const).map((s) => ({
              value: s,
              label: t(`ui.editor.pixel.${s}`),
            }))}
            onCommit={(shape) => commit({ shape })}
          />
          {shape === "polygon" && <p className="muted hint">{t("ui.editor.pixel.polygon_help")}</p>}
          {(shape === "rectangle" || shape === "ellipse") && (
            <label className="panel-check">
              <input
                type="checkbox"
                checked={v<boolean>("snap") === true}
                onChange={(e) => commit({ snap: e.target.checked })}
              />
              {t("ui.editor.pixel.snap")}
            </label>
          )}
          {(shape === "rectangle" || shape === "ellipse") && v<boolean>("snap") === true && (
            <NumberField
              label={t("ui.editor.pixel.level")}
              value={num("level")}
              min={0}
              max={100}
              step={1}
              unit=""
              onCommit={(level) => commit({ level })}
            />
          )}
          <label className="panel-check">
            <input
              type="checkbox"
              checked={v<boolean>("antialias") !== false}
              onChange={(e) => commit({ antialias: e.target.checked })}
            />
            {t("ui.editor.pixel.antialias")}
          </label>
          <NumberField
            label={t("ui.editor.pixel.feather")}
            value={num("feather")}
            min={0}
            max={50}
            step={1}
            unit="px"
            onCommit={(feather) => commit({ feather })}
          />
          {gapField}
          {qualityField}
          <SelectionBox />
        </>
      );
    }
    case "wand":
      // Magic wand (EDT-046): how alike, everywhere or touching, and the gap.
      return (
        <>
          <NumberField
            label={t("ui.editor.pixel.tolerance")}
            value={num("tolerance")}
            min={0}
            max={100}
            step={1}
            unit="%"
            onCommit={(tolerance) => commit({ tolerance })}
          />
          <label className="panel-check" title={t("ui.editor.pixel.global_help")}>
            <input
              type="checkbox"
              checked={v<boolean>("global") === true}
              onChange={(e) => commit({ global: e.target.checked })}
            />
            {t("ui.editor.pixel.global")}
          </label>
          {gapField}
          {qualityField}
        </>
      );
    case "eraser":
      // Eraser (EDT-048): thickness, tip, how much it erases and smooth edges.
      return (
        <>
          <NumberField
            label={t("ui.editor.pixel.eraser_width")}
            value={num("width")}
            min={1}
            max={200}
            step={1}
            unit="px"
            onCommit={(width) => commit({ width })}
          />
          <SelectField
            label={t("ui.editor.pixel.eraser_shape")}
            value={v<"round" | "square">("shape")}
            options={(["round", "square"] as const).map((s) => ({
              value: s,
              label: t(`ui.editor.pixel.${s}`),
            }))}
            onCommit={(shape) => commit({ shape })}
          />
          {opacity}
          <label className="panel-check">
            <input
              type="checkbox"
              checked={v<boolean>("antialias") !== false}
              onChange={(e) => commit({ antialias: e.target.checked })}
            />
            {t("ui.editor.pixel.antialias")}
          </label>
        </>
      );
  }
}

/** The nine cells of the shadow grid (EDT-021): row and column, -1 to 1. */
const SHADOW_CELLS: [number, number][] = [
  [-1, -1],
  [-1, 0],
  [-1, 1],
  [0, -1],
  [0, 0],
  [0, 1],
  [1, -1],
  [1, 0],
  [1, 1],
];

/** The grid cell a shadow is in: its direction, or the center when it has none. */
function shadowCell(s: Shadow | null | undefined): string | null {
  if (!s) return null;
  if (s.distance === 0) return "0,0";
  const a = (Math.round(s.angle / 45) * 45) % 360;
  const col = Math.round(Math.cos((a * Math.PI) / 180));
  const row = Math.round(Math.sin((a * Math.PI) / 180));
  return `${row},${col}`;
}

/** Shadow (EDT-021): a 3 × 3 grid of directions and, folded, its settings. */
function ShadowSection(props: {
  value: Shadow | null | undefined;
  mixed: boolean;
  commit: (changes: Record<string, unknown>) => void;
}) {
  const { t } = useTranslation();
  const { value, mixed, commit } = props;
  const cell = mixed ? null : shadowCell(value);
  const current: Shadow = value ?? DEFAULT_SHADOW;
  const set = (changes: Partial<Shadow>) => commit({ shadow: { ...current, ...changes } });
  const pick = (row: number, col: number) => {
    const key = `${row},${col}`;
    // The active cell again: no shadow.
    if (key === cell) return commit({ shadow: null });
    if (row === 0 && col === 0) return set({ distance: 0 });
    const angle = (((Math.atan2(row, col) * 180) / Math.PI + 360) % 360) as number;
    set({ angle: Math.round(angle), distance: current.distance || DEFAULT_SHADOW.distance });
  };
  return (
    <fieldset className="shadow-section">
      <legend>{t("ui.editor.shadow.title")}</legend>
      <div className="shadow-grid" role="group" aria-label={t("ui.editor.shadow.direction")}>
        {SHADOW_CELLS.map(([row, col]) => (
          <button
            key={`${row},${col}`}
            type="button"
            className="shadow-cell"
            aria-pressed={cell === `${row},${col}`}
            aria-label={t(`ui.editor.shadow.cell.${row + 1}${col + 1}`)}
            title={t(`ui.editor.shadow.cell.${row + 1}${col + 1}`)}
            onClick={() => pick(row, col)}
          >
            <span className="shadow-dot" style={{ translate: `${col * 4}px ${row * 4}px` }} />
          </button>
        ))}
      </div>
      {value && !mixed && (
        <details className="shadow-more">
          <summary>{t("ui.editor.shadow.advanced")}</summary>
          <ColorPicker
            label={t("ui.editor.prop.color")}
            value={current.color}
            onCommit={(color) => color && set({ color })}
          />
          <NumberField
            label={t("ui.editor.shadow.angle")}
            value={current.angle}
            min={0}
            max={359}
            step={1}
            unit="°"
            onCommit={(angle) => set({ angle })}
          />
          <NumberField
            label={t("ui.editor.shadow.distance")}
            value={current.distance}
            min={0}
            max={50}
            step={1}
            unit="px"
            onCommit={(distance) => set({ distance })}
          />
          <NumberField
            label={t("ui.editor.prop.opacity")}
            value={current.opacity}
            min={0}
            max={100}
            step={1}
            unit="%"
            onCommit={(opacity) => set({ opacity })}
          />
          <NumberField
            label={t("ui.editor.shadow.blur")}
            value={current.blur}
            min={0}
            max={50}
            step={1}
            unit="px"
            onCommit={(blur) => set({ blur })}
          />
        </details>
      )}
    </fieldset>
  );
}

const DEFAULT_SHADOW: Shadow = { color: "#000000", angle: 45, distance: 6, opacity: 50, blur: 8 };

/** A rectangle or ellipse selection in numbers (EDT-043, EDT-086): typing a size
 * selects exactly that. */
function SelectionBox() {
  const { t } = useTranslation();
  const sel = useEditor((s) => s.doc?.pixelSelection ?? null);
  const style = useEditor((s) => s.styles?.select);
  const shape = sel?.shape;
  if (!sel || !shape) return null;
  const r = shape.rect;
  const set = (k: "x" | "y" | "w" | "h", value: number) =>
    void selectPixels({
      shape: shape.type,
      rect: { ...r, [k]: k === "w" || k === "h" ? Math.max(1, value) : value },
      antialias: style?.antialias ?? true,
      feather: style?.feather ?? 0,
      ...(sel.target ? { objectId: sel.target } : {}),
    });
  return (
    <fieldset className="pos-size">
      <legend>{t("ui.editor.pixel.selection")}</legend>
      <div className="pos-grid">
        <Field label={t("ui.editor.pixel.sel_x")} value={r.x} onApply={(v) => set("x", v)} />
        <Field label={t("ui.editor.pixel.sel_y")} value={r.y} onApply={(v) => set("y", v)} />
        <Field label={t("ui.editor.pixel.sel_w")} value={r.w} onApply={(v) => set("w", v)} />
        <Field label={t("ui.editor.pixel.sel_h")} value={r.h} onApply={(v) => set("h", v)} />
      </div>
    </fieldset>
  );
}

/** Callout (EDT-028): body, outline, corners, tails, text and Fit to text. */
function CalloutProperties(props: {
  v: <T>(key: string) => T | undefined;
  num: (key: string) => number | undefined;
  commit: (changes: Record<string, unknown>) => void;
  lineStyle: React.ReactNode;
  text: React.ReactNode;
  opacity: React.ReactNode;
}) {
  const { t } = useTranslation();
  const { v, num, commit, lineStyle, text, opacity } = props;
  const check = (key: "shading" | "fitText") => (
    <label className="panel-check">
      <input
        type="checkbox"
        checked={v<boolean>(key) === true}
        onChange={(e) => commit({ [key]: e.target.checked })}
      />
      {t(`ui.editor.prop.${key === "fitText" ? "fit_text" : "shading"}`)}
    </label>
  );
  return (
    <>
      <SelectField
        label={t("ui.editor.prop.callout_shape")}
        value={v<CalloutShapeKind>("shape")}
        options={CALLOUT_SHAPES.map((s) => ({
          value: s,
          label: t(`ui.editor.callout_shape.${s}`),
        }))}
        onCommit={(shape) => commit({ shape })}
      />
      <FillPicker
        label={t("ui.editor.prop.fill")}
        value={v<Fill | null>("fill") ?? null}
        mixed={v<Fill | null>("fill") === undefined}
        allowTransparent
        onCommit={(fill) => commit({ fill })}
      />
      {check("shading")}
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
        max={20}
        step={0.5}
        unit="px"
        onCommit={(outlineWidth) => commit({ outlineWidth })}
      />
      {lineStyle}
      {v<CalloutShapeKind>("shape") === "roundedRectangle" && (
        <NumberField
          label={t("ui.editor.prop.corner_radius")}
          value={num("cornerRadius")}
          min={0}
          max={200}
          step={1}
          unit="px"
          onCommit={(cornerRadius) => commit({ cornerRadius })}
        />
      )}
      <SelectField
        label={t("ui.editor.prop.tail_style")}
        value={v<TailStyle>("tailStyle")}
        options={(["straight", "curved", "arrow"] as const).map((s) => ({
          value: s,
          label: t(`ui.editor.tail.${s}`),
        }))}
        onCommit={(tailStyle) => commit({ tailStyle })}
      />
      <NumberField
        label={t("ui.editor.prop.tail_width")}
        value={num("tailWidth")}
        min={2}
        max={200}
        step={1}
        unit="px"
        onCommit={(tailWidth) => commit({ tailWidth })}
      />
      {text}
      <SelectField
        label={t("ui.editor.prop.valign")}
        value={v<VAlign>("valign")}
        options={(["top", "middle", "bottom"] as const).map((s) => ({
          value: s,
          label: t(`ui.editor.valign.${s}`),
        }))}
        onCommit={(valign) => commit({ valign })}
      />
      <NumberField
        label={t("ui.editor.prop.padding")}
        value={num("padding")}
        min={0}
        max={100}
        step={1}
        unit="px"
        onCommit={(padding) => commit({ padding })}
      />
      {check("fitText")}
      {opacity}
    </>
  );
}

/** Simplify (EDT-041): the shape, its palette's colors, text lines, and how the original
 * shows under the shapes; palettes are saved with the tool. */
function SimplifyProperties(props: {
  v: <T>(key: string) => T | undefined;
  num: (key: string) => number | undefined;
  commit: (changes: Record<string, unknown>) => void;
  opacity: React.ReactNode;
}) {
  const { t } = useTranslation();
  const { v, num, commit, opacity } = props;
  const tool = useEditor((s) => s.styles?.simplify);
  const doc = useEditor((s) => s.doc);
  const lockOnExit = useSetting("editor.simplifyLockOnExit", false);
  const kind = v<SimpleKind>("kind");
  const palettes = tool?.palettes ?? [];
  const palette = palettes.find((p) => p.name === tool?.palette) ?? palettes[0];
  const setTool = (style: Record<string, unknown>) =>
    void run("editor.set_style", { tool: "simplify", style }).catch(() => undefined);
  const view = doc?.simplify ?? { showOriginal: true, opacity: 100 };
  const setView = (change: Record<string, unknown>) =>
    doc &&
    void run("document.simplify_view", { documentId: doc.documentId, ...change }).catch(
      () => undefined,
    );
  const saveNew = () => {
    // The palette in use plus the color chosen now, as a new palette.
    const fill = v<string>("fill");
    const colors = [...(palette?.colors ?? [])];
    if (fill && !colors.includes(fill)) colors.unshift(fill);
    let n = palettes.length + 1;
    while (palettes.some((p) => p.name === t("ui.editor.simplify.palette_n", { n }))) n++;
    const name = t("ui.editor.simplify.palette_n", { n });
    setTool({ palettes: [...palettes, { name, colors }], palette: name });
  };
  return (
    <>
      <SelectField
        label={t("ui.editor.prop.simple_kind")}
        value={kind}
        options={SIMPLE_KINDS.map((k) => ({ value: k, label: t(`ui.editor.simple_kind.${k}`) }))}
        onCommit={(kind) => commit({ kind })}
      />
      <SelectField
        label={t("ui.editor.simplify.palette")}
        value={palette?.name}
        options={palettes.map((p) => ({ value: p.name, label: p.name }))}
        onCommit={(name) => {
          // New shapes take the chosen palette's colors.
          const first = palettes.find((p) => p.name === name)?.colors[0];
          setTool({ palette: name, ...(first ? { fill: first } : {}) });
        }}
      />
      <div className="simplify-swatches" role="group" aria-label={t("ui.editor.simplify.palette")}>
        {palette?.colors.map((c) => (
          <button
            key={c}
            type="button"
            className="swatch"
            style={{ background: c }}
            title={c}
            aria-label={c}
            aria-pressed={v<string>("fill") === c}
            onClick={() => commit({ fill: c })}
          />
        ))}
      </div>
      <button type="button" className="btn small" onClick={saveNew}>
        {t("ui.editor.simplify.save_palette")}
      </button>
      <ColorPicker
        label={t("ui.editor.prop.fill")}
        value={v<string>("fill") ?? "#B4BAC4"}
        mixed={v<string>("fill") === undefined}
        onCommit={(fill) => fill && commit({ fill })}
      />
      {(kind === "textLine" || kind === "paragraph") && (
        <>
          <ColorPicker
            label={t("ui.editor.prop.background")}
            value={v<string | null>("background") ?? null}
            mixed={v<string | null>("background") === undefined}
            allowTransparent
            onCommit={(background) => commit({ background })}
          />
          <NumberField
            label={t("ui.editor.prop.text_height")}
            value={num("textHeight")}
            min={2}
            max={200}
            step={1}
            unit="px"
            onCommit={(textHeight) => commit({ textHeight })}
          />
        </>
      )}
      {kind === "paragraph" && (
        <NumberField
          label={t("ui.editor.prop.line_spacing")}
          value={num("lineSpacing")}
          min={1}
          max={5}
          step={0.1}
          unit="×"
          onCommit={(lineSpacing) => commit({ lineSpacing })}
        />
      )}
      {opacity}
      {doc && (
        <>
          <label className="panel-check">
            <input
              type="checkbox"
              checked={view.showOriginal}
              onChange={(e) => setView({ showOriginal: e.target.checked })}
            />
            {t("ui.editor.simplify.show_original")}
          </label>
          {view.showOriginal && (
            <NumberField
              label={t("ui.editor.simplify.original_opacity")}
              value={view.opacity}
              min={0}
              max={100}
              step={1}
              unit="%"
              onCommit={(opacity) => setView({ opacity })}
            />
          )}
        </>
      )}
      <label className="panel-check">
        <input
          type="checkbox"
          checked={lockOnExit}
          onChange={(e) =>
            void run("settings.set", {
              key: "editor.simplifyLockOnExit",
              value: e.target.checked,
            }).catch(() => undefined)
          }
        />
        {t("ui.editor.simplify.lock_on_exit")}
      </label>
    </>
  );
}

/** Step (EDT-035): shape, colors, size, sequence; Restart sequence and Renumber. */
function StepProperties(props: {
  v: <T>(key: string) => T | undefined;
  num: (key: string) => number | undefined;
  commit: (changes: Record<string, unknown>) => void;
  opacity: React.ReactNode;
}) {
  const { t } = useTranslation();
  const { v, num, commit, opacity } = props;
  const sequence = v<StepSequence>("sequence");
  const next = useEditor((s) => (sequence ? s.doc?.stepNext?.[sequence] : undefined));
  return (
    <>
      <SelectField
        label={t("ui.editor.prop.step_shape")}
        value={v<StepShapeKind>("shape")}
        options={STEP_SHAPES.map((s) => ({ value: s, label: t(`ui.editor.step_shape.${s}`) }))}
        onCommit={(shape) => commit({ shape })}
      />
      <FillPicker
        label={t("ui.editor.prop.fill")}
        value={v<Fill | null>("fill") ?? null}
        mixed={v<Fill | null>("fill") === undefined}
        allowTransparent
        onCommit={(fill) => commit({ fill })}
      />
      <ColorPicker
        label={t("ui.editor.prop.text_color")}
        value={v<string>("textColor") ?? "#FFFFFF"}
        mixed={v<string>("textColor") === undefined}
        onCommit={(textColor) => textColor && commit({ textColor })}
      />
      <ColorPicker
        label={t("ui.editor.prop.outline")}
        value={v<string>("outline") ?? "#FFFFFF"}
        mixed={v<string>("outline") === undefined}
        onCommit={(outline) => outline && commit({ outline })}
      />
      <NumberField
        label={t("ui.editor.prop.outline_width")}
        value={num("outlineWidth")}
        min={0}
        max={10}
        step={0.5}
        unit="px"
        onCommit={(outlineWidth) => commit({ outlineWidth })}
      />
      <NumberField
        label={t("ui.editor.prop.step_size")}
        value={num("size")}
        min={16}
        max={256}
        step={1}
        unit="px"
        onCommit={(size) => commit({ size })}
      />
      <SelectField
        label={t("ui.editor.prop.sequence")}
        value={sequence}
        options={STEP_SEQUENCES.map((s) => ({ value: s, label: t(`ui.editor.sequence.${s}`) }))}
        onCommit={(sequence) => commit({ sequence })}
      />
      {opacity}
      <div className="prop-field">
        {next && <span className="muted">{t("ui.editor.step_next", { value: next })}</span>}
        <div className="step-actions">
          <button
            type="button"
            className="btn small"
            onClick={() => void stepSequence("restart", sequence)}
          >
            {t("ui.editor.step_restart")}
          </button>
          <button
            type="button"
            className="btn small"
            onClick={() => void stepSequence("renumber", sequence)}
          >
            {t("menu.step_renumber")}
          </button>
        </div>
      </div>
    </>
  );
}

const ALIGN_ICONS: Record<TextAlign, IconName> = {
  left: "alignLeft",
  center: "alignCenter",
  right: "alignRight",
  justify: "alignJustify",
};

/** Text alignment: a pill in the logo's gradient springs to the chosen one (EDT-026). */
function AlignChoice({
  value,
  onChange,
}: {
  value: TextAlign | undefined;
  onChange: (a: TextAlign) => void;
}) {
  const { t } = useTranslation();
  const group = useRef<HTMLDivElement>(null);
  useIndicator(group, value);
  return (
    <div
      className="prop-buttons seg brand"
      role="radiogroup"
      aria-labelledby="align-label"
      ref={group}
    >
      <span className="seg-indicator" aria-hidden="true" />
      {(["left", "center", "right", "justify"] as const).map((a) => (
        <button
          key={a}
          type="button"
          role="radio"
          className="btn icon"
          aria-checked={value === a}
          aria-label={t(`ui.editor.align.${a}`)}
          title={t(`ui.editor.align.${a}`)}
          onClick={() => onChange(a)}
        >
          <Icon name={ALIGN_ICONS[a]} />
        </button>
      ))}
    </div>
  );
}

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
  const pixelTool = target !== null && PIXEL_TOOLS.includes(target.kind);
  if (target && !pixelTool && (selection.length > 0 || editing)) {
    const count = target.styles.length;
    title = t("ui.editor.props_selection", {
      what:
        count > 1 ? t("ui.editor.objects_count", { count }) : t(`ui.editor.tool.${target.kind}`),
    });
    body = (
      <>
        <Properties target={target} />
        {/* Where it is and how big, in numbers (EDT-086). */}
        {!editing && <PositionSize />}
      </>
    );
  } else if (doc && !pixelTool && selection.length > 1) {
    // Objects of several kinds: what they share, where they are (EDT-080).
    title = t("ui.editor.props_selection", {
      what: t("ui.editor.objects_count", { count: selection.length }),
    });
    body = (
      <>
        <SharedProperties />
        {!editing && <PositionSize />}
      </>
    );
  } else if (target) {
    // A pixel tool changes the image, not the next objects (EDT-047, EDT-048).
    title = pixelTool
      ? t(`ui.editor.tool.${target.kind}`)
      : t("ui.editor.props_tool", { tool: t(`ui.editor.tool.${target.kind}`) });
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
      <PanelTabs tab={tab} setTab={setTab} />
      <div className="side-body" role="tabpanel">
        {tab === "styles" ? (
          <>
            <StylesTab />
            {(tool === "stamp" || target?.kind === "stamp") && <StampBrowser />}
          </>
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

/** Styles and Properties, with a highlight that springs between them. */
function PanelTabs({ tab, setTab }: { tab: string; setTab: (tab: "styles" | "props") => void }) {
  const { t } = useTranslation();
  const tabs = useRef<HTMLDivElement>(null);
  useIndicator(tabs, tab);
  return (
    <div className="panel-tabs seg" role="tablist" ref={tabs}>
      <span className="seg-indicator" aria-hidden="true" />
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
  );
}

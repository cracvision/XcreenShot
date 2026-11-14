// Quick edit bar (EDT-006): next to the one selected object, its most used properties,
// its saved styles and ⋯ (the Properties tab). It is HTML over the canvas, never part
// of the image. Each change is one update_object, as in Properties.
import { useEffect, useState, type MouseEvent } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { CAPS, LINE_STYLES } from "./geometry";
import { Icon } from "./icons";
import { useContextMenu } from "./menus";
import { parseHex } from "./color";
import {
  CALLOUT_SHAPES,
  SHAPE_KINDS,
  STEP_SHAPES,
  usePanel,
  useEditor,
  type DocObject,
} from "./state";

const BAR_HEIGHT = 38;

interface Saved {
  id: string;
  name?: string;
}

function Swatch(props: { label: string; value: string | null; onPick: (c: string) => void }) {
  // A native color well: the full selector stays in Properties (EDT-019).
  const value = props.value && parseHex(props.value) ? props.value.slice(0, 7) : "#000000";
  return (
    <label className="qb-swatch" title={props.label}>
      <span className="sr-only">{props.label}</span>
      <input
        type="color"
        value={value}
        onChange={(e) => props.onPick(e.target.value.toUpperCase())}
      />
      <span className="qb-chip" style={{ background: props.value ?? "transparent" }} />
    </label>
  );
}

function Pick<T extends string>(props: {
  label: string;
  value: T;
  options: readonly T[];
  name: (v: T) => string;
  onPick: (v: T) => void;
}) {
  return (
    <select
      className="qb-select"
      aria-label={props.label}
      title={props.label}
      value={props.value}
      onChange={(e) => props.onPick(e.target.value as T)}
    >
      {props.options.map((o) => (
        <option key={o} value={o}>
          {props.name(o)}
        </option>
      ))}
    </select>
  );
}

function Num(props: {
  label: string;
  value: number;
  min: number;
  max: number;
  onPick: (v: number) => void;
}) {
  const [draft, setDraft] = useState<string | null>(null);
  return (
    <input
      className="qb-num"
      type="text"
      inputMode="numeric"
      aria-label={props.label}
      title={props.label}
      value={draft ?? String(Math.round(props.value * 10) / 10)}
      onChange={(e) => setDraft(e.target.value)}
      onBlur={() => {
        const v = Number(draft);
        if (draft !== null && Number.isFinite(v))
          props.onPick(Math.max(props.min, Math.min(props.max, v)));
        setDraft(null);
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter") (e.target as HTMLInputElement).blur();
        e.stopPropagation();
      }}
    />
  );
}

/**
 * The bar for `object`, placed over (or, without room, under) its box `rect` in the
 * canvas area's own pixels.
 */
export function QuickBar({
  object,
  rect,
  areaWidth,
  areaTop = 0,
}: {
  object: DocObject;
  rect: { x: number; y: number; w: number; h: number };
  areaWidth: number;
  /** Where the visible part of the canvas starts (below the floating toolbar). */
  areaTop?: number;
}) {
  const { t } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const fonts = useEditor((s) => s.fonts);
  const [saved, setSaved] = useState<Saved[]>([]);
  const kind = object.type;
  useEffect(() => {
    let alive = true;
    void run<{ tools: Record<string, Saved[]> }>("editor.styles")
      .then((r) => alive && setSaved(r.tools[kind] ?? []))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [kind]);
  if (!doc) return null;
  const style = ("style" in object ? object.style : {}) as unknown as Record<string, unknown>;
  const set = (changes: Record<string, unknown>) =>
    void run("document.update_object", {
      documentId: doc.documentId,
      objectId: object.id,
      set: { style: changes },
    }).catch((e) => useEditor.getState().showError(e));
  const str = (k: string) => (typeof style[k] === "string" ? (style[k] as string) : "");
  const num = (k: string) => (typeof style[k] === "number" ? (style[k] as number) : 0);
  const fillOf = (k: string) => (typeof style[k] === "string" ? (style[k] as string) : null);

  let fields: React.ReactNode;
  switch (kind) {
    case "arrow":
    case "line":
      fields = (
        <>
          <Swatch
            label={t("ui.editor.prop.color")}
            value={str("color")}
            onPick={(color) => set({ color })}
          />
          <Num
            label={t("ui.editor.prop.width")}
            value={num("width")}
            min={1}
            max={100}
            onPick={(width) => set({ width })}
          />
          {kind === "arrow" && (
            <Pick
              label={t("ui.editor.prop.end_cap")}
              value={str("endCap") as (typeof CAPS)[number]}
              options={CAPS}
              name={(c) => t(`ui.editor.cap.${c}`)}
              onPick={(endCap) => set({ endCap })}
            />
          )}
          <Pick
            label={t("ui.editor.prop.line_style")}
            value={str("lineStyle") as (typeof LINE_STYLES)[number]}
            options={LINE_STYLES}
            name={(l) => t(`ui.editor.line.${l}`)}
            onPick={(lineStyle) => set({ lineStyle })}
          />
        </>
      );
      break;
    case "text":
      fields = (
        <>
          <select
            className="qb-select qb-font"
            aria-label={t("ui.editor.prop.font")}
            value={str("font")}
            onChange={(e) => set({ font: e.target.value })}
          >
            {(fonts.includes(str("font")) ? fonts : [str("font"), ...fonts]).map((f) => (
              <option key={f} value={f}>
                {f}
              </option>
            ))}
          </select>
          <Num
            label={t("ui.editor.prop.font_size")}
            value={num("size")}
            min={4}
            max={400}
            onPick={(size) => set({ size })}
          />
          <button
            type="button"
            className="qb-toggle"
            aria-pressed={style.bold === true}
            aria-label={t("ui.editor.prop.bold")}
            title={t("ui.editor.prop.bold")}
            onClick={() => set({ bold: style.bold !== true })}
          >
            <b>B</b>
          </button>
          <Swatch
            label={t("ui.editor.prop.color")}
            value={str("color")}
            onPick={(color) => set({ color })}
          />
        </>
      );
      break;
    case "callout":
      fields = (
        <>
          <Pick
            label={t("ui.editor.prop.callout_shape")}
            value={str("shape") as (typeof CALLOUT_SHAPES)[number]}
            options={CALLOUT_SHAPES}
            name={(s) => t(`ui.editor.callout_shape.${s}`)}
            onPick={(shape) => set({ shape })}
          />
          <Swatch
            label={t("ui.editor.prop.fill")}
            value={fillOf("fill")}
            onPick={(fill) => set({ fill })}
          />
          <Swatch
            label={t("ui.editor.prop.outline")}
            value={str("outline")}
            onPick={(outline) => set({ outline })}
          />
          <Num
            label={t("ui.editor.prop.font_size")}
            value={num("size")}
            min={4}
            max={400}
            onPick={(size) => set({ size })}
          />
        </>
      );
      break;
    case "shape":
      fields = (
        <>
          <Pick
            label={t("ui.editor.prop.shape_type")}
            value={str("shapeType") as (typeof SHAPE_KINDS)[number]}
            options={SHAPE_KINDS}
            name={(s) => t(`ui.editor.shape.${s}`)}
            onPick={(shapeType) => set({ shapeType })}
          />
          <Swatch
            label={t("ui.editor.prop.fill")}
            value={fillOf("fill")}
            onPick={(fill) => set({ fill })}
          />
          <Swatch
            label={t("ui.editor.prop.outline")}
            value={str("outline")}
            onPick={(outline) => set({ outline })}
          />
          <Num
            label={t("ui.editor.prop.outline_width")}
            value={num("outlineWidth")}
            min={0}
            max={100}
            onPick={(outlineWidth) => set({ outlineWidth })}
          />
        </>
      );
      break;
    case "step":
      fields = (
        <>
          <Swatch
            label={t("ui.editor.prop.fill")}
            value={fillOf("fill")}
            onPick={(fill) => set({ fill })}
          />
          <Pick
            label={t("ui.editor.prop.step_shape")}
            value={str("shape") as (typeof STEP_SHAPES)[number]}
            options={STEP_SHAPES}
            name={(s) => t(`ui.editor.step_shape.${s}`)}
            onPick={(shape) => set({ shape })}
          />
          <input
            className="qb-num"
            type="text"
            aria-label={t("ui.editor.quick.value")}
            title={t("ui.editor.quick.value")}
            defaultValue={
              "value" in object ? String((object as { value?: unknown }).value ?? "") : ""
            }
            onKeyDown={(e) => e.stopPropagation()}
            onBlur={(e) =>
              void run("document.update_object", {
                documentId: doc.documentId,
                objectId: object.id,
                set: { value: e.target.value },
              }).catch((err) => useEditor.getState().showError(err))
            }
          />
        </>
      );
      break;
    case "blur":
      fields = (
        <>
          <Pick
            label={t("ui.editor.prop.blur_type")}
            value={str("blurType") as "blur" | "pixelate" | "solid"}
            options={["blur", "pixelate", "solid"] as const}
            name={(b) => t(`ui.editor.blur.${b}`)}
            onPick={(blurType) => set({ blurType })}
          />
          <Num
            label={t("ui.editor.prop.intensity")}
            value={num("intensity")}
            min={1}
            max={100}
            onPick={(intensity) => set({ intensity })}
          />
        </>
      );
      break;
    default:
      return null;
  }

  const stylesMenu = (e: MouseEvent<HTMLButtonElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    useContextMenu.setState({
      open: true,
      x: r.left,
      y: r.bottom + 4,
      label: t("ui.editor.quick.styles"),
      items: saved.length
        ? saved.map((s, i) => ({
            id: `qb-style-${s.id}`,
            label: s.name ?? t("ui.editor.quick.style_n", { n: i + 1 }),
            onSelect: () =>
              void run("editor.style", {
                action: "apply",
                tool: kind,
                styleId: s.id,
                documentId: doc.documentId,
                objectIds: [object.id],
              }).catch((err) => useEditor.getState().showError(err)),
          }))
        : [
            {
              id: "qb-none",
              label: t("ui.editor.quick.no_styles"),
              disabled: true,
              onSelect: () => undefined,
            },
          ],
    });
  };

  // Over the object, or under it without room; inside the area left to right.
  const width = 360;
  const above = rect.y - BAR_HEIGHT - 10;
  const top = above >= areaTop + 4 ? above : rect.y + rect.h + 10;
  const left = Math.max(4, Math.min(areaWidth - width - 4, rect.x + rect.w / 2 - width / 2));
  return (
    <div
      className="quick-bar"
      role="toolbar"
      aria-label={t("ui.editor.quick.label")}
      style={{ left, top }}
      onPointerDown={(e) => e.stopPropagation()}
    >
      {fields}
      <button type="button" className="qb-button" onClick={stylesMenu} aria-haspopup="menu">
        {t("ui.editor.quick.styles")} ▾
      </button>
      <button
        type="button"
        className="qb-button"
        aria-label={t("ui.editor.quick.more")}
        title={t("ui.editor.quick.more")}
        onClick={() => usePanel.getState().set({ visible: true, tab: "props" })}
      >
        <Icon name="more" />
      </button>
    </div>
  );
}

// Properties > Position and size (EDT-086): X, Y, width, height (with a proportion lock)
// and angle of a selected object; start, end and length of an arrow; X and Y of several
// selected objects. Fields take numbers or `+10`, `-5`, `*2`, `/2`; Enter or leaving the
// field applies, Esc goes back.
import { useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { arrowFocus } from "../components/arrowFocus";
import { run } from "../lib/commands";
import { useSetting } from "../store";
import { arrangeSelection } from "./actions";
import { Icon } from "./icons";
import { extentPoints, boxOf, reboxed } from "./objects";
import { evaluate, keepRatio } from "./positionMath";
import { isSegment, useEditor, type DocObject } from "./state";

const round = (v: number) => Math.round(v * 100) / 100;

const ALIGN_ACTIONS = [
  "align_left",
  "align_hcenter",
  "align_right",
  "align_top",
  "align_vcenter",
  "align_bottom",
  "distribute_h",
  "distribute_v",
  "equal_width",
  "equal_height",
] as const;

/** One field: shows `value`; what is typed is evaluated on it when applied. */
function Field({
  label,
  value,
  onApply,
  disabled = false,
}: {
  label: string;
  value: number;
  onApply: (v: number) => void;
  disabled?: boolean;
}) {
  const [draft, setDraft] = useState<string | null>(null);
  const apply = () => {
    if (draft === null) return;
    const v = evaluate(draft, value);
    setDraft(null);
    // Not a number: the field goes back to its value (EDT-086).
    if (v !== null && v !== value) onApply(v);
  };
  const onKey = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      apply();
    } else if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      setDraft(null);
    }
  };
  return (
    <label className="pos-field">
      <span>{label}</span>
      <input
        type="text"
        inputMode="decimal"
        value={draft ?? String(round(value))}
        disabled={disabled}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={apply}
        onKeyDown={onKey}
      />
    </label>
  );
}

export function PositionSize() {
  const { t } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const selection = useEditor((s) => s.selection);
  const showError = useEditor((s) => s.showError);
  const [locked, setLocked] = useState(false);
  const scaleStrokes = useSetting("editor.scaleStrokes", true);
  if (!doc || selection.length === 0) return null;
  const objects = doc.objects.filter((o) => selection.includes(o.id));
  if (objects.length === 0) return null;
  const update = (o: DocObject, set: Record<string, unknown>) =>
    run("document.update_object", { documentId: doc.documentId, objectId: o.id, set }).catch(
      showError,
    );

  // Several: where their common box is; moving it moves them all (EDT-079).
  if (objects.length > 1) {
    const pts = objects.flatMap(extentPoints);
    const x = Math.min(...pts.map((p) => p[0]));
    const y = Math.min(...pts.map((p) => p[1]));
    const w = Math.max(...pts.map((p) => p[0])) - x;
    const h = Math.max(...pts.map((p) => p[1])) - y;
    const ids = objects.map((o) => o.id);
    const move = (dx: number, dy: number) =>
      run("document.update_object", {
        documentId: doc.documentId,
        objectIds: ids,
        move: [dx, dy],
      }).catch(showError);
    // A new size of their common box stretches them all from its top left (EDT-080).
    const resize = (which: "w" | "h", v: number) => {
      const value = Math.max(1, v);
      const size = locked
        ? keepRatio(w, h, which, value)
        : which === "w"
          ? { w: value, h }
          : { w, h: value };
      run("document.transform", {
        documentId: doc.documentId,
        objectIds: ids,
        scale: { from: [x, y, w, h], to: [x, y, size.w, size.h] },
        strokes: scaleStrokes,
      }).catch(showError);
    };
    return (
      <>
        <fieldset className="pos-size">
          <legend>{t("ui.editor.pos.title")}</legend>
          <div className="pos-grid">
            <Field label="X" value={x} onApply={(v) => void move(v - x, 0)} />
            <Field label="Y" value={y} onApply={(v) => void move(0, v - y)} />
            <Field label={t("ui.editor.pos.width")} value={w} onApply={(v) => resize("w", v)} />
            <Field
              label={t("ui.editor.pos.height")}
              value={h}
              disabled={h < 1}
              onApply={(v) => resize("h", v)}
            />
            <label className="pos-lock">
              <input
                type="checkbox"
                checked={locked}
                onChange={(e) => setLocked(e.target.checked)}
              />
              <span>{t("ui.editor.pos.lock")}</span>
            </label>
            {/* Whether line widths and text sizes follow (EDT-080). */}
            <label className="pos-lock">
              <input
                type="checkbox"
                checked={scaleStrokes}
                onChange={(e) =>
                  void run("settings.set", {
                    key: "editor.scaleStrokes",
                    value: e.target.checked,
                  }).catch(showError)
                }
              />
              <span>{t("ui.editor.scale_strokes")}</span>
            </label>
          </div>
        </fieldset>
        {/* Align and distribute the selection (EDT-083). */}
        <fieldset className="pos-size">
          <legend>{t("ui.editor.align_title")}</legend>
          <div
            className="align-buttons"
            role="toolbar"
            aria-label={t("ui.editor.align_title")}
            onKeyDown={arrowFocus("horizontal")}
          >
            {ALIGN_ACTIONS.map((a) => (
              <button
                key={a}
                type="button"
                className="btn icon small"
                title={t(`menu.${a}`).replace("&", "")}
                aria-label={t(`menu.${a}`).replace("&", "")}
                disabled={a.startsWith("distribute") && objects.length < 3}
                onClick={() => arrangeSelection(a)}
              >
                <Icon name={a} />
              </button>
            ))}
          </div>
        </fieldset>
      </>
    );
  }

  const o = objects[0];
  if (!o) return null;
  if (isSegment(o)) {
    const [fx, fy] = o.from;
    const [tx, ty] = o.to;
    const length = Math.hypot(tx - fx, ty - fy);
    const setEnd = (end: "from" | "to", point: [number, number]) =>
      void update(o, { [end]: point });
    return (
      <fieldset className="pos-size">
        <legend>{t("ui.editor.pos.title")}</legend>
        <div className="pos-grid">
          <Field
            label={t("ui.editor.pos.start_x")}
            value={fx}
            onApply={(v) => setEnd("from", [v, fy])}
          />
          <Field
            label={t("ui.editor.pos.start_y")}
            value={fy}
            onApply={(v) => setEnd("from", [fx, v])}
          />
          <Field
            label={t("ui.editor.pos.end_x")}
            value={tx}
            onApply={(v) => setEnd("to", [v, ty])}
          />
          <Field
            label={t("ui.editor.pos.end_y")}
            value={ty}
            onApply={(v) => setEnd("to", [tx, v])}
          />
          <Field
            label={t("ui.editor.pos.length")}
            value={length}
            onApply={(v) => {
              // Same direction, new length: the end moves (EDT-086).
              if (length === 0) return;
              const k = v / length;
              setEnd("to", [fx + (tx - fx) * k, fy + (ty - fy) * k]);
            }}
          />
        </div>
      </fieldset>
    );
  }

  const box = boxOf(o);
  if (!box) return null;
  const apply = (changes: Partial<typeof box>) => {
    const [, set] = reboxed(o, { ...box, ...changes });
    void update(o, set);
  };
  const size = (which: "w" | "h", v: number) => {
    const value = Math.max(1, v);
    apply(locked ? keepRatio(box.w, box.h, which, value) : { [which]: value });
  };
  return (
    <fieldset className="pos-size">
      <legend>{t("ui.editor.pos.title")}</legend>
      <div className="pos-grid">
        <Field label="X" value={box.x} onApply={(v) => apply({ x: v })} />
        <Field label="Y" value={box.y} onApply={(v) => apply({ y: v })} />
        <Field label={t("ui.editor.pos.width")} value={box.w} onApply={(v) => size("w", v)} />
        {/* A text's height follows its lines (EDT-026). */}
        <Field
          label={t("ui.editor.pos.height")}
          value={box.h}
          disabled={o.type === "text"}
          onApply={(v) => size("h", v)}
        />
        <Field
          label={t("ui.editor.pos.angle")}
          value={box.rotation}
          onApply={(v) => apply({ rotation: Math.max(-180, Math.min(180, v)) })}
        />
        <label className="pos-lock">
          <input type="checkbox" checked={locked} onChange={(e) => setLocked(e.target.checked)} />
          <span>{t("ui.editor.pos.lock")}</span>
        </label>
      </div>
    </fieldset>
  );
}

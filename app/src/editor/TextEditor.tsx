// Typing a text box (EDT-026). While typing, a field drawn by the browser stands in for
// the text Rust draws; Esc or a click outside ends it.
import { useEffect, useRef, type CSSProperties } from "react";
import { useTranslation } from "react-i18next";
import { finishTextEdit } from "./actions";
import type { Point } from "./geometry";
import type { TextEdit } from "./state";
import { LINE_SPACING } from "./textMetrics";

interface Props {
  edit: TextEdit;
  /** Canvas point → position in the canvas area, in CSS pixels. */
  toScreen: (p: Point) => [number, number];
  /** CSS pixels per image pixel. */
  scale: number;
  dpi: number;
}

export function TextEditor({ edit, toScreen, scale, dpi }: Props) {
  const { t } = useTranslation();
  const ref = useRef<HTMLTextAreaElement>(null);
  const done = useRef(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.focus();
    // The cursor goes to the end of the text (EDT-042).
    el.setSelectionRange(el.value.length, el.value.length);
  }, []);

  const finish = () => {
    if (done.current) return;
    done.current = true;
    void finishTextEdit(ref.current?.value ?? "");
  };

  const { style } = edit;
  const [left, top] = toScreen(edit.at);
  const px = ((style.size * dpi) / 72) * scale;
  const css: CSSProperties & { fieldSizing?: string } = {
    left,
    top,
    fontFamily: `"${style.font}", sans-serif`,
    fontSize: px,
    fontWeight: style.bold ? 700 : 400,
    fontStyle: style.italic ? "italic" : "normal",
    lineHeight: LINE_SPACING,
    color: style.color,
    textAlign: style.align,
    transform: `rotate(${edit.rotation}deg)`,
    whiteSpace: edit.width ? "pre-wrap" : "pre",
    ...(edit.width
      ? { width: edit.width * scale, fieldSizing: "content" }
      : { fieldSizing: "content", minWidth: Math.max(px * 0.6, 8) }),
  };

  return (
    <textarea
      ref={ref}
      className="text-editor"
      style={css}
      defaultValue={edit.text}
      aria-label={t("ui.editor.text_field")}
      spellCheck={false}
      onBlur={finish}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          e.stopPropagation();
          finish();
        }
      }}
    />
  );
}

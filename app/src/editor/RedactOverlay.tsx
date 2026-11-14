// Redacting by hand while auto redact's panel is open (SMT-014, SMT-015): the words read
// on the canvas light up under the pointer. A click redacts a word, Shift+click or a
// drag the words between (one redaction per line), a double click the whole line; a
// click on a redaction removes it. The right-click menu redacts or uncovers every place
// of the same text. From the keyboard, the panel does the same: My terms adds, and
// Next / Previous with Remove takes away.
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { MenuItem } from "../components/Menu";
import type { Point } from "./geometry";
import { useContextMenu } from "./menus";
import { redactMatches, redactRects, removeRedactions, useRedact } from "./redactState";
import {
  between,
  covered,
  lineRect,
  spanRects,
  textUnder,
  type At,
  type Rect,
} from "./redactWords";
import { isOn } from "../features";
import { editText } from "./editText";

/** How long a click waits for a second one before redacting a single word. */
const DOUBLE_CLICK_MS = 300;

export function RedactOverlay({
  documentId,
  toScreen,
  scale,
  insets,
}: {
  documentId: string;
  toScreen: (p: Point) => [number, number];
  scale: number;
  /** The part of the canvas area left visible by the floating bars. */
  insets: { top: number; right: number; bottom: number; left: number };
}) {
  const { t } = useTranslation();
  const open = useRedact((s) => s.documentId === documentId);
  const lines = useRedact((s) => s.lines);
  const items = useRedact((s) => s.result?.items);
  const [anchor, setAnchor] = useState<At | null>(null);
  const [span, setSpan] = useState<{ from: At; to: At } | null>(null);
  const dragging = useRef(false);
  const single = useRef<number | undefined>(undefined);

  // A drag ends wherever the button is let go.
  useEffect(() => {
    const up = () => {
      if (!dragging.current) return;
      dragging.current = false;
      setSpan((s) => {
        if (s && (s.from.line !== s.to.line || s.from.word !== s.to.word)) {
          void redactRects(spanRects(useRedact.getState().lines, s.from, s.to));
        }
        return null;
      });
    };
    window.addEventListener("pointerup", up);
    return () => {
      window.removeEventListener("pointerup", up);
      window.clearTimeout(single.current);
    };
  }, []);

  if (!open) return null;
  const redactions = items ?? [];
  const boxes = redactions.map((i) => i.rect);
  const box = (r: Rect) => {
    const [x, y] = toScreen([r[0], r[1]]);
    return {
      left: x - insets.left,
      top: y - insets.top,
      width: r[2] * scale,
      height: r[3] * scale,
    };
  };
  const menu = (x: number, y: number, entries: MenuItem[]) =>
    useContextMenu.setState({ open: true, x, y, label: t("ui.redact.title"), items: entries });

  return (
    <div
      className="redact-overlay"
      aria-hidden="true"
      style={{ top: insets.top, right: insets.right, bottom: insets.bottom, left: insets.left }}
    >
      {lines.map((line, li) =>
        line.words.map((w, wi) => {
          const at = { line: li, word: wi };
          if (covered(w.rect, boxes)) return null;
          const picked = span !== null && between(at, span.from, span.to);
          return (
            <span
              key={`${li}-${wi}`}
              className={`redact-word${picked ? " picked" : ""}`}
              style={box(w.rect)}
              title={t("ui.redact.click_word")}
              onPointerDown={(e) => {
                if (e.button !== 0) return;
                e.stopPropagation();
                if (e.shiftKey && anchor) {
                  window.clearTimeout(single.current);
                  void redactRects(spanRects(lines, anchor, at));
                  return;
                }
                setAnchor(at);
                dragging.current = true;
                setSpan({ from: at, to: at });
              }}
              onPointerEnter={() => {
                if (dragging.current) setSpan((s) => (s ? { ...s, to: at } : s));
              }}
              onClick={(e) => {
                if (e.shiftKey) return;
                // A double click redacts the line: wait for it before redacting the word.
                window.clearTimeout(single.current);
                single.current = window.setTimeout(
                  () => void redactRects([w.rect]),
                  DOUBLE_CLICK_MS,
                );
              }}
              onDoubleClick={(e) => {
                e.stopPropagation();
                window.clearTimeout(single.current);
                const whole = lineRect(lines, li);
                if (whole) void redactRects([whole]);
              }}
              onContextMenu={(e) => {
                e.preventDefault();
                e.stopPropagation();
                menu(e.clientX, e.clientY, [
                  {
                    id: "redact",
                    label: t("ui.redact.redact_word"),
                    onSelect: () => void redactRects([w.rect]),
                  },
                  {
                    id: "all",
                    label: t("ui.redact.redact_all"),
                    onSelect: () => void redactMatches(w.text, false),
                  },
                ]);
              }}
            />
          );
        }),
      )}
      {redactions.map((i) => (
        <span
          key={i.objectId}
          className="redact-box"
          style={box(i.rect)}
          title={t("ui.redact.click_box")}
          onPointerDown={(e) => e.stopPropagation()}
          onDoubleClick={(e) => e.stopPropagation()}
          onClick={() => {
            window.clearTimeout(single.current);
            void removeRedactions([i.objectId]);
          }}
          onContextMenu={(e) => {
            e.preventDefault();
            e.stopPropagation();
            const text = textUnder(lines, i.rect);
            menu(e.clientX, e.clientY, [
              {
                id: "remove",
                label: t("ui.redact.remove"),
                onSelect: () => void removeRedactions([i.objectId]),
              },
              {
                id: "all",
                label: t("ui.redact.redact_all"),
                disabled: !text,
                separator: true,
                onSelect: () => void redactMatches(text, false),
              },
              {
                id: "none",
                label: t("ui.redact.remove_all_matches"),
                disabled: !text,
                onSelect: () => void redactMatches(text, true),
              },
              // The text below, no longer redacted, as a Text object (SMT-009).
              ...(isOn("editText")
                ? [
                    {
                      id: "edit_text",
                      label: t("ui.edit_text.menu"),
                      separator: true,
                      disabled: !text,
                      onSelect: () =>
                        void removeRedactions([i.objectId]).then(() =>
                          editText(i.source ?? i.rect, true),
                        ),
                    },
                  ]
                : []),
            ]);
          }}
        />
      ))}
    </div>
  );
}

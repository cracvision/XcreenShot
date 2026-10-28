// What a screen reader hears in the editor (EDT-013): the selected object with its
// position and size, and changes of tool and zoom. A polite live region reads them.
import i18next from "i18next";
import { useEffect, useState } from "react";
import { extentPoints } from "./objects";
import { useEditor, type DocObject } from "./state";

const t = (key: string, vars: Record<string, unknown> = {}) => i18next.t(key, vars);

/** "Text: Step 1, at 320, 140, 200 by 80 pixels". */
export function describe(o: DocObject): string {
  const kind = t(`ui.editor.tool.${o.type}`);
  const what = o.type === "text" ? `${kind}: ${o.text.split("\n")[0] ?? ""}` : kind;
  // Where it is and how big, from its corners (an arrow: its two ends).
  const points = extentPoints(o);
  if (points.length === 0) return what;
  const xs = points.map((p) => p[0]);
  const ys = points.map((p) => p[1]);
  const [x, y] = [Math.min(...xs), Math.min(...ys)];
  const r = Math.round;
  return t("ui.editor.a11y.object", {
    what,
    x: r(x),
    y: r(y),
    w: r(Math.max(...xs) - x),
    h: r(Math.max(...ys) - y),
  });
}

/** The text of the live region: the last thing worth hearing. */
export function useAnnouncement(): string {
  const [text, setText] = useState("");
  useEffect(() => {
    let zoom: number | undefined;
    return useEditor.subscribe((s, before) => {
      const id = s.selection.length === 1 ? s.selection[0] : undefined;
      const o = id ? s.doc?.objects.find((x) => x.id === id) : undefined;
      const was = before.selection.length === 1 ? before.selection[0] : undefined;
      if (o && (id !== was || s.doc !== before.doc)) {
        if (id !== was) setText(describe(o));
      } else if (s.selection.length > 1 && s.selection.length !== before.selection.length) {
        setText(t("ui.editor.objects_count", { count: s.selection.length }));
      }
      if (s.tool !== before.tool) {
        setText(t("ui.editor.a11y.tool", { tool: t(`ui.editor.tool.${s.tool}`) }));
      }
      const z = s.active ? s.views[s.active]?.zoom : undefined;
      if (z !== undefined && zoom !== undefined && Math.round(z) !== Math.round(zoom)) {
        setText(t("ui.editor.a11y.zoom", { zoom: Math.round(z) }));
      }
      zoom = z;
    });
  }, []);
  return text;
}

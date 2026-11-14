// Live measurement (EDT-094), while the Measure tool is chosen: under the pointer, how
// far what is there goes to each side; with Alt and an object selected, the space to
// the object under the pointer; a drag lays a ruler with its length, angle and ΔX/ΔY.
// Right-click on a ruler: Make a dimension (a line with its label). Nothing here is
// part of the image: it is drawn over the canvas and never exported.
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import type { Point } from "./geometry";
import {
  dimensionObjects,
  edges,
  gap,
  rulerDetails,
  rulerOf,
  type Box,
  type Pixels,
} from "./measure";
import { useContextMenu } from "./menus";
import { boxOf } from "./objects";
import { useEditor, xsdocUrl, type DocObject } from "./state";

/** Pictures larger than this are not read for the edges (memory). */
const MAX_PIXELS = 40_000_000;

/** The document's final picture, read once per version, and where the canvas's top
 * left is in it (objects and effects can stick out of the canvas). */
function usePixels(documentId: string, version: number): { px: Pixels; origin: Point } | null {
  const [got, setGot] = useState<{ px: Pixels; origin: Point } | null>(null);
  useEffect(() => {
    let gone = false;
    void (async () => {
      try {
        const r = await fetch(xsdocUrl(`${documentId}/final?v=${version}`));
        const [ox, oy] = (r.headers.get("X-Origin") ?? "0,0").split(",").map(Number);
        const bitmap = await createImageBitmap(await r.blob());
        if (gone || bitmap.width * bitmap.height > MAX_PIXELS) return;
        const c = document.createElement("canvas");
        c.width = bitmap.width;
        c.height = bitmap.height;
        const ctx = c.getContext("2d", { willReadFrequently: true });
        if (!ctx) return;
        ctx.drawImage(bitmap, 0, 0);
        const d = ctx.getImageData(0, 0, c.width, c.height);
        setGot({
          px: { width: d.width, height: d.height, data: d.data },
          origin: [ox || 0, oy || 0],
        });
      } catch {
        // Without the picture the ruler still works.
      }
    })();
    return () => {
      gone = true;
    };
  }, [documentId, version]);
  return got;
}

function topObjectAt(objects: DocObject[], p: Point, skip: Set<string>): Box | null {
  for (let i = objects.length - 1; i >= 0; i--) {
    const o = objects[i];
    if (!o || skip.has(o.id)) continue;
    const b = boxOf(o);
    if (b && p[0] >= b.x && p[0] <= b.x + b.w && p[1] >= b.y && p[1] <= b.y + b.h) return b;
  }
  return null;
}

export function MeasureOverlay({
  documentId,
  version,
  toScreen,
  toCanvas,
  insets,
}: {
  documentId: string;
  version: number;
  toScreen: (p: Point) => [number, number];
  toCanvas: (s: { x: number; y: number }) => Point;
  insets: { top: number; right: number; bottom: number; left: number };
}) {
  const { t } = useTranslation();
  const objects = useEditor((s) => s.doc?.objects ?? []);
  const selection = useEditor((s) => s.selection);
  const canvas = useEditor((s) => s.doc?.canvas ?? { width: 0, height: 0 });
  const pixels = usePixels(documentId, version);
  const [at, setAt] = useState<Point | null>(null);
  const [alt, setAlt] = useState(false);
  const [ruler, setRuler] = useState<{ from: Point; to: Point } | null>(null);
  const dragging = useRef(false);
  const ref = useRef<HTMLDivElement>(null);

  // Esc drops the ruler; Alt switches to the space between objects.
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      setAlt(e.altKey);
      if (e.type === "keydown" && e.key === "Escape") setRuler(null);
    };
    window.addEventListener("keydown", key);
    window.addEventListener("keyup", key);
    return () => {
      window.removeEventListener("keydown", key);
      window.removeEventListener("keyup", key);
    };
  }, []);

  const local = (e: React.PointerEvent | React.MouseEvent) => {
    const r = ref.current?.parentElement?.getBoundingClientRect();
    return toCanvas({ x: e.clientX - (r?.left ?? 0), y: e.clientY - (r?.top ?? 0) });
  };
  const screen = (p: Point) => {
    const [x, y] = toScreen(p);
    return [x - insets.left, y - insets.top] as const;
  };
  const snapped = (from: Point, to: Point, shift: boolean): Point => {
    if (!shift) return to;
    // Shift: straight across, down or at 45°.
    const dx = to[0] - from[0];
    const dy = to[1] - from[1];
    const step = Math.PI / 4;
    const a = Math.round(Math.atan2(dy, dx) / step) * step;
    const len = Math.hypot(dx, dy);
    return [from[0] + Math.round(Math.cos(a) * len), from[1] + Math.round(Math.sin(a) * len)];
  };

  const elements: React.ReactNode[] = [];
  const label = (x: number, y: number, text: string, key: string) =>
    elements.push(
      <span key={key} className="measure-label" style={{ left: x, top: y }}>
        {text}
      </span>,
    );
  const line = (a: Point, b: Point, key: string, ends = true) => {
    const [x1, y1] = screen(a);
    const [x2, y2] = screen(b);
    elements.push(
      <svg key={key} className="measure-svg" aria-hidden="true">
        <line x1={x1} y1={y1} x2={x2} y2={y2} />
        {ends && (
          <>
            <circle cx={x1} cy={y1} r={2.5} />
            <circle cx={x2} cy={y2} r={2.5} />
          </>
        )}
      </svg>,
    );
  };

  const selected = objects.filter((o) => selection.includes(o.id));
  const selBox = selected.length === 1 && selected[0] ? boxOf(selected[0]) : null;
  if (ruler) {
    line(ruler.from, ruler.to, "ruler");
    const [mx, my] = screen([(ruler.from[0] + ruler.to[0]) / 2, (ruler.from[1] + ruler.to[1]) / 2]);
    label(mx, my, rulerDetails(ruler.from, ruler.to), "ruler-label");
  } else if (at && alt && selBox) {
    // The space between the selected object and the one under the pointer.
    const other = topObjectAt(objects, at, new Set(selection));
    if (other) {
      const g = gap(selBox, other);
      if (g.dx !== null) {
        const y =
          Math.max(selBox.y, other.y) +
          (Math.min(selBox.y + selBox.h, other.y + other.h) - Math.max(selBox.y, other.y)) / 2;
        const midY = Number.isFinite(y) ? y : other.y + other.h / 2;
        const x1 = other.x >= selBox.x + selBox.w ? selBox.x + selBox.w : other.x + other.w;
        line([x1, midY], [x1 + g.dx, midY], "gap-x");
        const [lx, ly] = screen([x1 + g.dx / 2, midY]);
        label(lx, ly, `${g.dx} px`, "gap-x-label");
      }
      if (g.dy !== null) {
        const x =
          Math.max(selBox.x, other.x) +
          (Math.min(selBox.x + selBox.w, other.x + other.w) - Math.max(selBox.x, other.x)) / 2;
        const y1 = other.y >= selBox.y + selBox.h ? selBox.y + selBox.h : other.y + other.h;
        line([x, y1], [x, y1 + g.dy], "gap-y");
        const [lx, ly] = screen([x, y1 + g.dy / 2]);
        label(lx, ly, `${g.dy} px`, "gap-y-label");
      }
    }
  } else if (at && pixels) {
    // How far what is under the pointer goes to each side.
    const p: Point = [at[0] + pixels.origin[0], at[1] + pixels.origin[1]];
    const e = edges(pixels.px, p[0], p[1]);
    if (e) {
      const [x, y] = [Math.floor(at[0]) + 0.5, Math.floor(at[1]) + 0.5];
      line([x - e.left - 0.5, y], [x + e.right + 0.5, y], "h", false);
      line([x, y - e.up - 0.5], [x, y + e.down + 0.5], "v", false);
      const put = (q: Point, n: number, k: string) => {
        if (n <= 0) return;
        const [lx, ly] = screen(q);
        label(lx, ly, `${n}`, k);
      };
      put([x - e.left / 2, y], e.left, "l");
      put([x + e.right / 2, y], e.right, "r");
      put([x, y - e.up / 2], e.up, "u");
      put([x, y + e.down / 2], e.down, "d");
      const [sx, sy] = screen([x, y]);
      label(sx + 64, sy - 14, `${e.left + e.right + 1} × ${e.up + e.down + 1} px`, "size");
    }
  }

  return (
    <div
      ref={ref}
      className="measure-overlay"
      role="img"
      aria-label={t("ui.measure.label")}
      style={{ top: insets.top, right: insets.right, bottom: insets.bottom, left: insets.left }}
      onPointerDown={(e) => {
        if (e.button !== 0) return;
        e.stopPropagation();
        e.currentTarget.setPointerCapture(e.pointerId);
        const p = local(e);
        dragging.current = true;
        setRuler({ from: p, to: p });
      }}
      onPointerMove={(e) => {
        e.stopPropagation();
        const p = local(e);
        setAt(p);
        setAlt(e.altKey);
        if (dragging.current)
          setRuler((r) => (r ? { from: r.from, to: snapped(r.from, p, e.shiftKey) } : r));
      }}
      onPointerUp={(e) => {
        e.stopPropagation();
        dragging.current = false;
        // A click without a drag only clears the ruler.
        setRuler((r) => (r && rulerOf(r.from, r.to).length >= 2 ? r : null));
      }}
      onPointerLeave={() => setAt(null)}
      onContextMenu={(e) => {
        e.preventDefault();
        e.stopPropagation();
        if (!ruler) return;
        const r = ruler;
        useContextMenu.setState({
          open: true,
          x: e.clientX,
          y: e.clientY,
          label: t("ui.measure.label"),
          items: [
            {
              id: "dimension",
              label: t("ui.measure.dimension"),
              onSelect: () => {
                setRuler(null);
                void run("document.annotate", {
                  documentId,
                  objects: dimensionObjects(r.from, r.to, canvas.width, canvas.height),
                }).catch(() => undefined);
              },
            },
            { id: "clear", label: t("ui.measure.clear"), onSelect: () => setRuler(null) },
          ],
        });
      }}
    >
      {elements}
    </div>
  );
}

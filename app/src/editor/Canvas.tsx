// The editor canvas (EDT-007, EDT-022, EDT-024, EDT-026, EDT-029, EDT-040, EDT-042,
// EDT-049). Konva draws; the pointer is handled here in canvas pixels; every change is
// one command when the gesture ends.
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Circle, Group, Image as KImage, Layer, Line, Rect, Stage } from "react-konva";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { setting, useSetting } from "../store";
import {
  applyCrop,
  duplicateSelection,
  startCalloutEdit,
  startNewText,
  startTextEdit,
} from "./actions";
import { canvasMenu, lockedMenu, menuPoint } from "./menus";
import {
  center,
  contains,
  corners,
  dragRect,
  handlePoint,
  resize,
  RESIZE_HANDLES,
  rotateTo,
  toLocal,
  toWorld,
  type Box,
  type Handle,
  type Rect as RectBox,
} from "./boxes";
import { arrowGeometry, constrain45, type Point } from "./geometry";
import { useObjectImages } from "./objectImages";
import {
  ArrowShape,
  BlurShape,
  CalloutShape,
  HighlightShape,
  ImageShape,
  ShapeShape,
  StepShape,
  TextShape,
} from "./ObjectShapes";
import { boxOf, extentPoints, hits, inBand, moved, reboxed } from "./objects";
import {
  docUrl,
  takeZoomRequest,
  useEditor,
  usePointer,
  isSegment,
  type CalloutObject,
  type DocDetails,
  type DocObject,
} from "./state";
import { TextEditor } from "./TextEditor";
import { Icon } from "./icons";
import { expand, unitOf } from "./groups";
import { boundsOf, normalized, transformed, type TransformOp } from "./transform";
import { hasRadius, MAX_TAILS } from "./callouts";
import {
  fitZoom,
  imagePoint,
  layout,
  openingZoom,
  originalZoom,
  stepZoom,
  viewFromScroll,
  zoomAround,
  type Size,
  type View,
} from "./view";

/** Movement below this (CSS px) is a click, not a drag: a click draws nothing (EDT-024). */
const DRAG_THRESHOLD = 3;
/** Handle size, pointer reach and the turning handle's stem, in CSS px (EDT-022). */
const HANDLE = 11;
const HIT = 6;
const STEM = 24;
/** Selection outline and handles: the logo's electric blue. */
const ACCENT = "#0064fb";

type BoxHandle = Exclude<Handle, "rotate">;

type ArrowObj = Extract<DocObject, { type: "arrow" | "line" }>;

/** What a press on a handle grabs: the crop box, an arrow end or an object's handle. */
type HandleHit =
  | { crop: BoxHandle }
  | { object: ArrowObj; end: "from" | "to" }
  | { object: DocObject; handle: Handle }
  // A handle of the box around several selected objects (EDT-080).
  | { multi: Handle }
  // A callout's tail tip, its + (a new tail), its corner radius, its anchor (EDT-028).
  | { tail: CalloutObject; index: number | null }
  | { radius: CalloutObject }
  | { anchor: CalloutObject };

type Drag =
  | {
      kind: "draw";
      tool: "arrow" | "line" | "shape" | "blur" | "text" | "step" | "callout" | "highlight";
      from: Point;
      to: Point;
      startX: number;
      startY: number;
      square: boolean;
      fromCenter: boolean;
      /** Let go without moving: a click (a step goes there). */
      click?: boolean;
    }
  | { kind: "move"; ids: string[]; start: Point; delta: Point; moved: boolean; copy?: boolean }
  // A selection box drawn with Move over an empty place (EDT-079).
  | { kind: "band"; start: Point; end: Point; touch: boolean; add: boolean }
  | { kind: "end"; id: string; end: "from" | "to"; other: Point; point: Point }
  | { kind: "resize"; id: string; handle: BoxHandle; start: Box; box: Box }
  | { kind: "rotate"; id: string; start: Box; box: Box }
  // Several objects stretched or turned with their common box (EDT-080).
  | { kind: "mresize"; handle: BoxHandle; start: Box; box: Box }
  | { kind: "mrotate"; start: Box; angle: number }
  // A callout's tail tip (index null: a new one), radius or body alone (EDT-028).
  | { kind: "tail"; id: string; index: number | null; point: Point; start: Point }
  | { kind: "radius"; id: string; box: Box; value: number }
  | { kind: "anchor"; id: string; start: Point; delta: Point }
  | { kind: "crop"; mode: "new" | "move" | BoxHandle; start: Point; from: RectBox; rect: RectBox }
  | { kind: "pan"; x: number; y: number; left: number; top: number };

function useBaseImage(id: string, key: number): HTMLImageElement | null {
  const [img, setImg] = useState<HTMLImageElement | null>(null);
  useEffect(() => {
    let alive = true;
    const el = new window.Image();
    // Readable pixels, for the canvas color picker (EDT-117 `I`).
    el.crossOrigin = "anonymous";
    el.onload = () => {
      if (!alive) return;
      setImg(el);
      // Once painted, the shell logs how long the capture took to show (APP-049).
      requestAnimationFrame(() =>
        requestAnimationFrame(() => void invoke("xs_editor_painted").catch(() => undefined)),
      );
    };
    el.src = docUrl(id, "base", key);
    return () => {
      alive = false;
    };
  }, [id, key]);
  // Another document starts empty; the same one keeps showing until the new view loads.
  useEffect(() => () => setImg(null), [id]);
  return img;
}

function useDevicePixelRatio(): number {
  const [dpr, setDpr] = useState(window.devicePixelRatio || 1);
  useEffect(() => {
    const mq = window.matchMedia(`(resolution: ${dpr}dppx)`);
    const update = () => setDpr(window.devicePixelRatio || 1);
    mq.addEventListener("change", update);
    return () => mq.removeEventListener("change", update);
  }, [dpr]);
  return dpr;
}

/** A small number that changes when `text` does: a URL version. */
function hashOf(text: string): number {
  let h = 2166136261;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

/** Light and mid gray squares of a fixed screen size for transparent areas (EDT-010). */
const checker = (() => {
  const c = document.createElement("canvas");
  c.width = c.height = 16;
  const g = c.getContext("2d");
  if (g) {
    g.fillStyle = "#e6e6e6";
    g.fillRect(0, 0, 16, 16);
    g.fillStyle = "#c8c8c8";
    g.fillRect(0, 0, 8, 8);
    g.fillRect(8, 8, 8, 8);
  }
  return c;
})();

/** Handles a selected object shows (EDT-022): text changes only its width. */
function handlesOf(o: DocObject): Handle[] {
  if (o.type === "text") return ["w", "e", "rotate"];
  // A step stays square: its corners (EDT-035).
  if (o.type === "step") return ["nw", "ne", "se", "sw", "rotate"];
  if (o.type === "callout") return [...RESIZE_HANDLES, "rotate"];
  if (o.type === "shape" || o.type === "blur" || o.type === "image" || o.type === "highlight")
    return [...RESIZE_HANDLES, "rotate"];
  return [];
}

/** Where a callout's + is: beside the middle of its right side (EDT-028). */
function plusAt(box: Box, scale: number): Point {
  return toWorld(box, [box.w / 2 + 18 / scale, 0]);
}

/** Where a callout's radius handle is: on its top side, as far in as the radius. */
function radiusAt(box: Box, radius: number, scale: number): Point {
  const r = Math.max(radius, 14 / scale);
  return toWorld(box, [-box.w / 2 + r, -box.h / 2 + 10 / scale]);
}

const clampRect = (r: RectBox, bounds: RectBox): RectBox => {
  const x0 = Math.max(r.x, bounds.x);
  const y0 = Math.max(r.y, bounds.y);
  const x1 = Math.min(r.x + r.w, bounds.x + bounds.w);
  const y1 = Math.min(r.y + r.h, bounds.y + bounds.h);
  return { x: x0, y: y0, w: Math.max(1, x1 - x0), h: Math.max(1, y1 - y0) };
};

interface Props {
  doc: DocDetails;
}

export function Canvas({ doc }: Props) {
  const { t } = useTranslation();
  const tool = useEditor((s) => s.tool);
  const styles = useEditor((s) => s.styles);
  const selection = useEditor((s) => s.selection);
  const storedView = useEditor((s) => s.views[doc.documentId]);
  const zoomRequest = useEditor((s) => s.zoomRequest);
  const textEdit = useEditor((s) => s.textEdit);
  const cropState = useEditor((s) => s.cropRect);
  const autoExpand = useSetting("editor.autoExpandCanvas", true);
  const scaleStrokes = useSetting("editor.scaleStrokes", true);
  const { setSelection, setView, showError } = useEditor.getState();
  // Another document: no group is entered (EDT-082).
  useEffect(() => {
    useEditor.setState({ entered: [] });
  }, [doc.documentId]);

  const scrollRef = useRef<HTMLDivElement>(null);
  const [viewport, setViewport] = useState<Size | null>(null);
  const dpr = useDevicePixelRatio();
  // With flattened objects the image shown changes with them (EDT-091).
  const flatKey = useMemo(
    () => (doc.flattened?.length ? hashOf(JSON.stringify(doc.flattened)) : 0),
    [doc.flattened],
  );
  const base = useBaseImage(doc.documentId, flatKey);
  const baseAt = { x: doc.baseView?.x ?? 0, y: doc.baseView?.y ?? 0 };
  const objectImages = useObjectImages(doc.documentId, doc.objects, doc.version);
  const [drag, setDrag] = useState<Drag | null>(null);
  const [space, setSpace] = useState(false);
  const [hover, setHover] = useState<string | null>(null);
  /** What a finished gesture showed, until the core reports the change. */
  const [pending, setPending] = useState<{ version: number; object: DocObject } | null>(null);

  const canvasSize: Size = { width: doc.canvas.width, height: doc.canvas.height };
  const cropping = tool === "crop";
  const cropRect: RectBox | null = cropping
    ? (cropState ?? { x: 0, y: 0, w: canvasSize.width, h: canvasSize.height })
    : null;
  // What the scroll area covers, in canvas pixels: the canvas, or while cropping the
  // whole original image and the canvas grown past it, so the crop can grow back
  // (EDT-050, EDT-071).
  const ext: RectBox = cropping
    ? (() => {
        const x0 = Math.min(0, -doc.crop.x);
        const y0 = Math.min(0, -doc.crop.y);
        const x1 = Math.max(canvasSize.width, doc.base.width - doc.crop.x);
        const y1 = Math.max(canvasSize.height, doc.base.height - doc.crop.y);
        return { x: x0, y: y0, w: x1 - x0, h: y1 - y0 };
      })()
    : { x: 0, y: 0, w: canvasSize.width, h: canvasSize.height };
  const extSize: Size = { width: ext.w, height: ext.h };

  useLayoutEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const measure = () => setViewport({ width: el.clientWidth, height: el.clientHeight });
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // First view of a document: this session's, the remembered one, or the opening zoom.
  const view: View | null = useMemo(() => {
    if (storedView) return storedView;
    if (!viewport) return null;
    // EDT-007, EDT-118: the remembered view, and shrinking big images to fit.
    const remembered = setting("editor.rememberView", true) ? doc.view : null;
    if (remembered?.zoom && remembered.x !== undefined && remembered.y !== undefined)
      return { zoom: remembered.zoom, cx: remembered.x, cy: remembered.y };
    return {
      zoom: setting("editor.shrinkToFit", true) ? openingZoom(canvasSize, viewport, dpr) : 100,
      cx: canvasSize.width / 2,
      cy: canvasSize.height / 2,
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [storedView, viewport, doc.documentId]);

  // The layout works in the covered area's own coordinates.
  const lay =
    view && viewport
      ? layout(
          { zoom: view.zoom, cx: view.cx - ext.x, cy: view.cy - ext.y },
          extSize,
          viewport,
          dpr,
        )
      : null;

  useEffect(() => {
    if (view && !storedView) setView(doc.documentId, view);
  }, [view, storedView, doc.documentId, setView]);

  const settingScroll = useRef(false);
  useLayoutEffect(() => {
    const el = scrollRef.current;
    if (!el || !lay) return;
    if (
      Math.abs(el.scrollLeft - lay.scrollLeft) > 0.5 ||
      Math.abs(el.scrollTop - lay.scrollTop) > 0.5
    ) {
      settingScroll.current = true;
      el.scrollLeft = lay.scrollLeft;
      el.scrollTop = lay.scrollTop;
    }
  });

  useEffect(() => {
    if (!view || !viewport) return;
    const timer = setTimeout(() => {
      run("editor.view", {
        documentId: doc.documentId,
        zoom: view.zoom,
        x: view.cx,
        y: view.cy,
        viewportWidth: viewport.width,
        viewportHeight: viewport.height,
        dpr,
      }).catch(() => undefined);
    }, 400);
    return () => clearTimeout(timer);
  }, [view, viewport, dpr, doc.documentId]);

  /** Canvas point → position in the canvas area (CSS px). */
  const toScreen = (p: Point): [number, number] =>
    lay
      ? [
          lay.originX - lay.scrollLeft + (p[0] - ext.x) * lay.scale,
          lay.originY - lay.scrollTop + (p[1] - ext.y) * lay.scale,
        ]
      : [0, 0];
  const toCanvas = (s: { x: number; y: number }): Point => {
    if (!lay) return [0, 0];
    const [x, y] = imagePoint(lay, s);
    return [x + ext.x, y + ext.y];
  };

  const applyZoom = (zoom: number, at?: { x: number; y: number }) => {
    if (!view || !viewport) return;
    const p = at ?? { x: viewport.width / 2, y: viewport.height / 2 };
    const v = zoomAround(
      { zoom: view.zoom, cx: view.cx - ext.x, cy: view.cy - ext.y },
      zoom,
      p,
      extSize,
      viewport,
      dpr,
    );
    setView(doc.documentId, { zoom: v.zoom, cx: v.cx + ext.x, cy: v.cy + ext.y });
  };

  useEffect(() => {
    if (!zoomRequest || !view || !viewport) return;
    const request = takeZoomRequest(doc.documentId);
    if (!request || (request.mode === "percent" && request.value === null)) return;
    switch (request.mode) {
      case "fit":
        setView(doc.documentId, {
          zoom: fitZoom(extSize, viewport, dpr),
          cx: ext.x + ext.w / 2,
          cy: ext.y + ext.h / 2,
        });
        break;
      case "original":
        applyZoom(originalZoom(doc.metadata.display?.scale ?? dpr, dpr));
        break;
      case "in":
      case "out":
        applyZoom(stepZoom(view.zoom, request.mode === "in"));
        break;
      default:
        if (request.value !== null) applyZoom(request.value);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [zoomRequest, view, viewport]);

  useEffect(() => {
    const typing = (e: KeyboardEvent) =>
      e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement;
    const down = (e: KeyboardEvent) => {
      if (e.code === "Space" && !typing(e)) {
        e.preventDefault();
        setSpace(true);
      }
    };
    const up = (e: KeyboardEvent) => {
      if (e.code === "Space") setSpace(false);
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
  }, []);

  // Objects as the canvas shows them: with the gesture in progress applied.
  const multiOp: TransformOp | null =
    drag?.kind === "mresize"
      ? { kind: "scale", from: drag.start, to: drag.box, strokes: scaleStrokes }
      : drag?.kind === "mrotate"
        ? { kind: "rotate", center: center(drag.start), degrees: drag.angle }
        : null;
  const shown: DocObject[] = doc.objects.map((o) => {
    if (pending?.version === doc.version && pending.object.id === o.id) return pending.object;
    if (multiOp) return selection.includes(o.id) ? transformed(o, multiOp) : o;
    if (drag?.kind === "move")
      return drag.ids.includes(o.id) ? moved(o, drag.delta[0], drag.delta[1])[0] : o;
    if (!drag || !("id" in drag) || drag.id !== o.id) return o;
    switch (drag.kind) {
      case "end":
        return isSegment(o) ? { ...o, [drag.end]: drag.point } : o;
      case "resize":
      case "rotate":
        return reboxed(o, drag.box)[0];
      case "tail": {
        if (o.type !== "callout") return o;
        const i = drag.index;
        const tails =
          i === null
            ? [...o.tails, drag.point]
            : o.tails.map((t, k): Point => (k === i ? drag.point : t));
        return { ...o, tails };
      }
      case "radius":
        return o.type === "callout" ? { ...o, style: { ...o.style, cornerRadius: drag.value } } : o;
      case "anchor":
        // The body moves; the tails stay where they point (EDT-028).
        return o.type === "callout"
          ? {
              ...o,
              rect: { ...o.rect, x: o.rect.x + drag.delta[0], y: o.rect.y + drag.delta[1] },
            }
          : o;
    }
  });
  const committed = new Map(doc.objects.map((o) => [o.id, o]));
  const plusPoint = (box: Box) => plusAt(box, lay?.scale ?? 1);
  const radiusPoint = (box: Box, radius: number) => radiusAt(box, radius, lay?.scale ?? 1);
  /** A selected callout's tails, +, radius and anchor (EDT-028). */
  const calloutHandles = (o: CalloutObject, box: Box) => {
    const c = toScreen(center(box));
    const [px, py] = toScreen(plusPoint(box));
    const dragging = drag?.kind === "tail" && drag.id === o.id;
    return (
      <>
        {dragging &&
          o.tails.map((t, i) => (
            <Line key={`tl${i}`} points={[...c, ...toScreen(t)]} stroke={ACCENT} dash={[4, 3]} />
          ))}
        {o.tails.map((t, i) => {
          const [x, y] = toScreen(t);
          return (
            <Rect
              key={`t${i}`}
              x={x}
              y={y}
              width={HANDLE}
              height={HANDLE}
              offsetX={HANDLE / 2}
              offsetY={HANDLE / 2}
              rotation={45}
              fill={ACCENT}
              stroke="#ffffff"
              strokeWidth={1.5}
            />
          );
        })}
        {o.tails.length < MAX_TAILS && (
          <Group x={px} y={py}>
            <Circle radius={HANDLE / 2 + 1} fill="#ffffff" stroke={ACCENT} strokeWidth={1.5} />
            <Line points={[-3.5, 0, 3.5, 0]} stroke={ACCENT} strokeWidth={1.5} />
            <Line points={[0, -3.5, 0, 3.5]} stroke={ACCENT} strokeWidth={1.5} />
          </Group>
        )}
        {hasRadius(o) &&
          (() => {
            const [x, y] = toScreen(radiusPoint(box, o.style.cornerRadius));
            return (
              <Rect
                x={x}
                y={y}
                width={HANDLE - 3}
                height={HANDLE - 3}
                offsetX={(HANDLE - 3) / 2}
                offsetY={(HANDLE - 3) / 2}
                rotation={45 + box.rotation}
                fill="#ffffff"
                stroke={ACCENT}
                strokeWidth={1.5}
              />
            );
          })()}
        <Circle x={c[0]} y={c[1]} radius={3.5} fill={ACCENT} stroke="#ffffff" strokeWidth={1} />
      </>
    );
  };
  /** The box around several selected objects, as they are stored (EDT-080). */
  const multiBox = (): Box | null => {
    if (selection.length < 2) return null;
    const all = boundsOf(doc.objects.filter((o) => selection.includes(o.id)));
    return all ? { ...all, rotation: 0 } : null;
  };

  const local = (e: React.PointerEvent | React.WheelEvent | React.MouseEvent) => {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };
  const near = (a: Point, s: { x: number; y: number }) => {
    const [x, y] = toScreen(a);
    return Math.hypot(x - s.x, y - s.y) <= HANDLE / 2 + 2;
  };

  /** The handle under screen point `s`, on a selected object or the crop box. */
  const hitHandle = (s: { x: number; y: number }): HandleHit | null => {
    if (!lay) return null;
    if (cropRect) {
      const box: Box = { ...cropRect, rotation: 0 };
      for (const h of RESIZE_HANDLES)
        if (near(handlePoint(box, h, 0), s)) return { crop: h as BoxHandle };
      return null;
    }
    if (selection.length > 1) {
      const all = multiBox();
      if (!all) return null;
      for (const h of [...RESIZE_HANDLES, "rotate" as const])
        if (near(handlePoint(all, h, STEM / lay.scale), s)) return { multi: h };
      return null;
    }
    if (selection.length !== 1) return null;
    const o = shown.find((x) => x.id === selection[0]);
    if (!o) return null;
    if (o.type === "callout" && !o.locked) {
      const box = boxOf(o);
      if (box) {
        for (let i = o.tails.length - 1; i >= 0; i--) {
          const t = o.tails[i];
          if (t && near(t, s)) return { tail: o, index: i };
        }
        if (o.tails.length < MAX_TAILS && near(plusPoint(box), s)) return { tail: o, index: null };
        if (hasRadius(o) && near(radiusPoint(box, o.style.cornerRadius), s)) return { radius: o };
        if (near(center(box), s)) return { anchor: o };
      }
    }
    if (isSegment(o)) {
      for (const end of ["to", "from"] as const) if (near(o[end], s)) return { object: o, end };
      return null;
    }
    const box = boxOf(o);
    if (!box) return null;
    for (const h of handlesOf(o))
      if (near(handlePoint(box, h, STEM / lay.scale), s)) return { object: o, handle: h };
    return null;
  };

  /** The top object under `p`; locked ones only with `locked` (EDT-085). */
  const hitObject = (p: Point, locked = false): DocObject | null => {
    if (!lay) return null;
    for (const o of [...shown].reverse())
      if ((locked || !o.locked) && hits(o, p, HIT / lay.scale)) return o;
    return null;
  };

  const onPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!lay || !scrollRef.current) return;
    (e.currentTarget as HTMLElement).focus();
    const s = local(e);
    if (e.button === 1 || (e.button === 0 && space)) {
      e.currentTarget.setPointerCapture(e.pointerId);
      setDrag({
        kind: "pan",
        x: e.clientX,
        y: e.clientY,
        left: scrollRef.current.scrollLeft,
        top: scrollRef.current.scrollTop,
      });
      return;
    }
    if (e.button !== 0) return;
    const p = toCanvas(s);
    const capture = () => e.currentTarget.setPointerCapture(e.pointerId);
    const handle = hitHandle(s);

    if (cropRect) {
      capture();
      // Inside the box it moves, unless it covers everything (then a new box is drawn).
      const all =
        cropRect.x <= ext.x && cropRect.y <= ext.y && cropRect.w >= ext.w && cropRect.h >= ext.h;
      const mode: "new" | "move" | BoxHandle =
        handle && "crop" in handle
          ? handle.crop
          : !all && contains({ ...cropRect, rotation: 0 }, p)
            ? "move"
            : "new";
      setDrag({ kind: "crop", mode, start: p, from: cropRect, rect: cropRect });
      return;
    }
    if (handle && "multi" in handle) {
      const start = multiBox();
      if (start) {
        capture();
        setDrag(
          handle.multi === "rotate"
            ? { kind: "mrotate", start, angle: 0 }
            : { kind: "mresize", handle: handle.multi, start, box: start },
        );
        return;
      }
    }
    if (handle && "tail" in handle) {
      capture();
      const c = handle.tail;
      const at = handle.index === null ? p : (c.tails[handle.index] ?? p);
      setDrag({ kind: "tail", id: c.id, index: handle.index, point: at, start: p });
      return;
    }
    if (handle && "radius" in handle) {
      const box = boxOf(handle.radius);
      if (box) {
        capture();
        setDrag({
          kind: "radius",
          id: handle.radius.id,
          box,
          value: handle.radius.style.cornerRadius,
        });
        return;
      }
    }
    if (handle && "anchor" in handle) {
      capture();
      setDrag({ kind: "anchor", id: handle.anchor.id, start: p, delta: [0, 0] });
      return;
    }
    if (handle && "end" in handle) {
      capture();
      const a = handle.object;
      setDrag({
        kind: "end",
        id: a.id,
        end: handle.end,
        other: handle.end === "to" ? a.from : a.to,
        point: a[handle.end],
      });
      return;
    }
    if (handle && "handle" in handle) {
      const box = boxOf(handle.object);
      if (box) {
        capture();
        setDrag(
          handle.handle === "rotate"
            ? { kind: "rotate", id: handle.object.id, start: box, box }
            : { kind: "resize", id: handle.object.id, handle: handle.handle, start: box, box },
        );
        return;
      }
    }
    // EDT-022: a click on an object selects it, except with Alt (draw over it).
    const hit = e.altKey && tool !== "move" ? null : hitObject(p);
    // Shift or Ctrl/Cmd adds an object to the selection, or takes it out (EDT-079).
    const adding = e.shiftKey || e.ctrlKey || e.metaKey;
    // A click leaves the entered group when it lands outside it (EDT-082).
    const entered = useEditor.getState().entered;
    const inside = hit ? unitOf(hit, entered) !== undefined : false;
    if (entered.length > 0 && (!hit || !inside)) useEditor.setState({ entered: [] });
    const level = inside ? entered : [];
    // A click takes the object with the rest of its group (EDT-082).
    const unit = hit ? expand(doc.objects, [hit.id], level) : [];
    if (hit && adding) {
      setSelection(
        selection.includes(hit.id)
          ? selection.filter((x) => !unit.includes(x))
          : [...selection, ...unit.filter((x) => !selection.includes(x))],
      );
      return;
    }
    if (hit) {
      // Dragging one of several selected objects moves them all.
      const ids = selection.includes(hit.id) ? selection : unit;
      if (!selection.includes(hit.id)) setSelection(unit);
      capture();
      // With Move, Alt+drag leaves a copy where it is let go (EDT-088).
      const copy = e.altKey && tool === "move";
      setDrag({ kind: "move", ids, start: p, delta: [0, 0], moved: false, copy });
      return;
    }
    if (tool === "move") {
      // An empty place: a selection box (EDT-079).
      capture();
      if (!adding) setSelection([]);
      setDrag({ kind: "band", start: p, end: p, touch: e.altKey, add: adding });
      return;
    }
    if (
      tool === "arrow" ||
      tool === "line" ||
      tool === "shape" ||
      tool === "blur" ||
      tool === "text" ||
      tool === "step" ||
      tool === "callout" ||
      tool === "highlight"
    ) {
      capture();
      setDrag({
        kind: "draw",
        tool,
        from: p,
        to: p,
        startX: s.x,
        startY: s.y,
        square: e.shiftKey,
        fromCenter: e.altKey,
      });
      return;
    }
    setSelection([]);
  };

  /**
   * The gesture after the pointer reached canvas point `p` with these keys. Used while
   * moving and again on release, so the last position counts even when the release
   * comes before the screen caught up with the moves.
   */
  const step = (
    d: Exclude<Drag, { kind: "pan" }>,
    e: { shiftKey: boolean; altKey: boolean },
    p: Point,
  ): Drag => {
    switch (d.kind) {
      case "draw":
        return {
          ...d,
          to: (d.tool === "arrow" || d.tool === "line") && e.shiftKey ? constrain45(d.from, p) : p,
          square: e.shiftKey,
          fromCenter: e.altKey,
        };
      case "end":
        return { ...d, point: e.shiftKey ? constrain45(d.other, p) : p };
      case "band":
        return { ...d, end: p, touch: e.altKey };
      case "move": {
        let dx = p[0] - d.start[0];
        let dy = p[1] - d.start[1];
        // Shift keeps the move horizontal or vertical (EDT-117).
        if (e.shiftKey) {
          if (Math.abs(dx) > Math.abs(dy)) dy = 0;
          else dx = 0;
        }
        const isMoved = d.moved || Math.hypot(dx, dy) * (lay?.scale ?? 1) >= DRAG_THRESHOLD;
        return { ...d, delta: isMoved ? [dx, dy] : [0, 0], moved: isMoved };
      }
      case "resize": {
        // An image keeps its proportions from a corner; Shift frees them (EDT-078).
        const kind = doc.objects.find((o) => o.id === d.id)?.type;
        const image = kind === "image";
        const corner = d.handle.length === 2;
        const keepRatio = kind === "step" || (image && corner ? !e.shiftKey : e.shiftKey);
        return { ...d, box: resize(d.start, d.handle, p, { keepRatio, fromCenter: e.altKey }) };
      }
      case "rotate":
        return { ...d, box: { ...d.start, rotation: rotateTo(d.start, p, e.shiftKey) } };
      case "mresize": {
        // From a corner the proportions stay, as with pictures; Shift frees them.
        const keepRatio = d.handle.length === 2 ? !e.shiftKey : e.shiftKey;
        return { ...d, box: resize(d.start, d.handle, p, { keepRatio, fromCenter: e.altKey }) };
      }
      case "mrotate":
        return { ...d, angle: normalized(rotateTo(d.start, p, e.shiftKey)) };
      case "tail":
        return { ...d, point: p };
      case "radius": {
        const [lx] = toLocal(d.box, p);
        const most = Math.min(200, d.box.w / 2, d.box.h / 2);
        return { ...d, value: Math.round(Math.max(0, Math.min(most, lx + d.box.w / 2))) };
      }
      case "anchor": {
        let dx = p[0] - d.start[0];
        let dy = p[1] - d.start[1];
        if (e.shiftKey) {
          if (Math.abs(dx) > Math.abs(dy)) dy = 0;
          else dx = 0;
        }
        return { ...d, delta: [dx, dy] };
      }
      case "crop": {
        const bounds = ext;
        let rect: RectBox;
        if (d.mode === "new") rect = dragRect(d.start, p, e.shiftKey, e.altKey);
        else if (d.mode === "move") {
          const dx = Math.min(
            Math.max(p[0] - d.start[0], bounds.x - d.from.x),
            bounds.x + bounds.w - d.from.x - d.from.w,
          );
          const dy = Math.min(
            Math.max(p[1] - d.start[1], bounds.y - d.from.y),
            bounds.y + bounds.h - d.from.y - d.from.h,
          );
          rect = { ...d.from, x: d.from.x + dx, y: d.from.y + dy };
        } else {
          // Shift keeps the current proportion (EDT-049).
          rect = resize({ ...d.from, rotation: 0 }, d.mode, p, { keepRatio: e.shiftKey });
        }
        return { ...d, rect: clampRect(rect, bounds) };
      }
    }
  };

  const onPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!lay) return;
    const s = local(e);
    const p = toCanvas(s);
    usePointer.setState({ point: p, screen: s });
    if (!drag) {
      const h = hitHandle(s);
      const handleName = h
        ? "crop" in h
          ? h.crop
          : "multi" in h
            ? h.multi
            : "end" in h || "tail" in h || "radius" in h
              ? "end"
              : "anchor" in h
                ? "anchor"
                : h.handle
        : null;
      setHover(
        handleName ??
          (cropRect ? null : hitObject(p) ? "object" : hitObject(p, true) ? "locked" : null),
      );
      return;
    }
    if (drag.kind === "pan") {
      if (scrollRef.current) {
        scrollRef.current.scrollLeft = drag.left - (e.clientX - drag.x);
        scrollRef.current.scrollTop = drag.top - (e.clientY - drag.y);
      }
      return;
    }
    setDrag(step(drag, e, p));
  };

  const update = async (o: DocObject, set: Record<string, unknown>) => {
    setPending({ version: doc.version, object: o });
    await run("document.update_object", { documentId: doc.documentId, objectId: o.id, set });
  };

  const finish = async (d: Drag) => {
    const id = doc.documentId;
    try {
      if (d.kind === "draw") {
        const object =
          d.tool === "arrow" || d.tool === "line"
            ? { type: d.tool, from: d.from, to: d.to }
            : d.tool === "step"
              ? // A step is square; a click places it at its size (EDT-035).
                d.click
                ? { type: "step", at: d.from }
                : { type: "step", rect: dragRect(d.from, d.to, true, d.fromCenter) }
              : d.tool === "callout"
                ? // A callout: a click makes one of a usual size (EDT-028).
                  {
                    type: "callout",
                    text: "",
                    rect: d.click
                      ? { x: d.from[0], y: d.from[1], w: 200, h: 60 }
                      : dragRect(d.from, d.to, d.square, d.fromCenter),
                  }
                : { type: d.tool, rect: dragRect(d.from, d.to, d.square, d.fromCenter) };
        const r = await run<{ ids: string[] }>("document.annotate", {
          documentId: id,
          objects: [object],
        });
        // Then the text is typed in it.
        if (d.tool === "callout" && r.ids[0] && styles) {
          startCalloutEdit({
            id: r.ids[0],
            type: "callout",
            rect: (object as { rect: RectBox }).rect,
            rotation: 0,
            text: "",
            tails: [],
            style: styles.callout,
          });
          return;
        }
        // The new object stays selected to adjust it (EDT-022, editor.autoSelectAfterDraw).
        if (r.ids[0] && setting("editor.autoSelectAfterDraw", true)) setSelection([r.ids[0]]);
      } else if (d.kind === "move" && d.moved && d.copy) {
        await duplicateSelection(d.delta);
      } else if (d.kind === "move" && d.moved) {
        const one = d.ids.length === 1 ? committed.get(d.ids[0] ?? "") : undefined;
        if (one) {
          const [next, set] = moved(one, d.delta[0], d.delta[1]);
          await update(next, set);
        } else {
          // Several: one command and one undo step (EDT-079).
          await run("document.update_object", {
            documentId: id,
            objectIds: d.ids,
            move: d.delta,
          });
        }
      } else if (d.kind === "band") {
        const r = dragRect(d.start, d.end, false, false);
        if (r.w * scale < DRAG_THRESHOLD && r.h * scale < DRAG_THRESHOLD) return;
        const boxed = doc.objects
          .filter((o) => !o.locked && inBand(o, r, d.touch))
          .map((o) => o.id);
        // Whole groups (EDT-082).
        const inside = expand(doc.objects, boxed, []);
        setSelection(d.add ? [...new Set([...selection, ...inside])] : inside);
      } else if (d.kind === "end") {
        const o = committed.get(d.id);
        if (o && isSegment(o)) await update({ ...o, [d.end]: d.point }, { [d.end]: d.point });
      } else if (d.kind === "resize" || d.kind === "rotate") {
        const o = committed.get(d.id);
        if (o) {
          const [next, set] = reboxed(o, d.box);
          await update(next, set);
        }
      } else if (d.kind === "tail") {
        const c = committed.get(d.id);
        const box = c ? boxOf(c) : null;
        if (c?.type !== "callout" || !box) return;
        // A click on + adds a tail beside the body.
        const click = Math.hypot(d.point[0] - d.start[0], d.point[1] - d.start[1]) * scale < 3;
        const point: Point =
          d.index === null && click ? toWorld(box, [box.w / 2 + 50, box.h / 2 + 30]) : d.point;
        // A tip let go inside the body takes that tail away.
        const inside = contains(box, point);
        let tails: Point[];
        if (d.index === null) {
          if (inside) return;
          tails = [...c.tails, point];
        } else if (inside) tails = c.tails.filter((_, k) => k !== d.index);
        else tails = c.tails.map((t, k): Point => (k === d.index ? point : t));
        await run("document.update_object", { documentId: id, objectId: d.id, set: { tails } });
      } else if (d.kind === "radius") {
        await run("document.update_object", {
          documentId: id,
          objectId: d.id,
          set: { style: { cornerRadius: d.value } },
        });
      } else if (d.kind === "anchor") {
        const c = committed.get(d.id);
        if (c?.type !== "callout" || (d.delta[0] === 0 && d.delta[1] === 0)) return;
        const rect = { ...c.rect, x: c.rect.x + d.delta[0], y: c.rect.y + d.delta[1] };
        await run("document.update_object", { documentId: id, objectId: d.id, set: { rect } });
      } else if (d.kind === "mresize") {
        const { start, box } = d;
        if (start.w === box.w && start.h === box.h && start.x === box.x && start.y === box.y)
          return;
        await run("document.transform", {
          documentId: id,
          objectIds: selection,
          scale: { from: [start.x, start.y, start.w, start.h], to: [box.x, box.y, box.w, box.h] },
          strokes: scaleStrokes,
        });
      } else if (d.kind === "mrotate") {
        if (d.angle === 0) return;
        await run("document.transform", {
          documentId: id,
          objectIds: selection,
          rotate: d.angle,
          around: center(d.start),
        });
      } else if (d.kind === "crop") {
        useEditor.setState({ cropRect: d.rect });
      }
    } catch (err) {
      setPending(null);
      showError(err);
    }
  };

  const onPointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    const last = drag;
    setDrag(null);
    if (!last || last.kind === "pan") return;
    // Where the pointer was let go, not where the last drawn frame had it.
    const d = step(last, e, toCanvas(local(e)));
    if (d.kind === "pan") return;
    if (d.kind === "draw") {
      const s = local(e);
      const click = Math.hypot(s.x - d.startX, s.y - d.startY) < DRAG_THRESHOLD;
      // Text: a click makes a box that grows, a drag a box of that width (EDT-026).
      if (d.tool === "text") {
        if (click) startNewText(d.from, null);
        else {
          const r = dragRect(d.from, d.to, false, false);
          startNewText([r.x, r.y], Math.max(r.w, 1));
        }
        return;
      }
      if (click && d.tool !== "step" && d.tool !== "callout") return;
      if (d.tool === "step" || d.tool === "callout") {
        void finish({ ...d, click });
        return;
      }
    }
    if (d.kind === "crop" && d.mode === "new") {
      const s = local(e);
      const [sx, sy] = toScreen(d.start);
      if (Math.hypot(s.x - sx, s.y - sy) < DRAG_THRESHOLD) return;
    }
    void finish(d);
  };

  // Right click: the menu of the object under the pointer, which it selects, or of the
  // image (EDT-002).
  const onContextMenu = (e: React.MouseEvent<HTMLDivElement>) => {
    e.preventDefault();
    if (!lay || cropping) return;
    const p = toCanvas(local(e));
    const hit = hitObject(p);
    // A locked object offers Unlock (EDT-085).
    const locked = hit ? null : hitObject(p, true);
    if (locked) {
      lockedMenu(locked.id, menuPoint(e));
      return;
    }
    if (hit && !selection.includes(hit.id)) setSelection([hit.id]);
    // On a flattened place: Unflatten (EDT-091), the newest layer there.
    const layer = hit
      ? undefined
      : [...(doc.flattened ?? [])]
          .reverse()
          .find(
            (l) =>
              l.bounds &&
              p[0] >= l.bounds[0] &&
              p[0] <= l.bounds[2] &&
              p[1] >= l.bounds[1] &&
              p[1] <= l.bounds[3],
          );
    canvasMenu(hit !== null, menuPoint(e), layer, hit?.type === "step" ? hit : undefined);
  };

  const onDoubleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const p = toCanvas(local(e));
    // On a group: enter it, and select what was clicked inside (EDT-082).
    {
      const hit = hitObject(p);
      const { entered } = useEditor.getState();
      const unit = hit ? unitOf(hit, entered) : undefined;
      if (hit && typeof unit === "string" && !cropRect) {
        const next = [...entered, unit];
        useEditor.setState({ entered: next });
        setSelection(expand(doc.objects, [hit.id], next));
        return;
      }
    }
    if (cropRect) {
      // Double click inside the crop box applies it (EDT-049).
      if (contains({ ...cropRect, rotation: 0 }, p)) void applyCrop();
      return;
    }
    const hit = hitObject(p);
    if (hit?.type === "text") startTextEdit(hit);
    // Edit value (EDT-035).
    if (hit?.type === "step") useEditor.setState({ stepEdit: { id: hit.id, value: hit.value } });
    // Types into a callout (EDT-028).
    if (hit?.type === "callout") startCalloutEdit(hit);
  };

  const onWheel = (e: React.WheelEvent<HTMLDivElement>) => {
    if (!view || !scrollRef.current) return;
    if (e.ctrlKey || e.metaKey) {
      const factor = Math.exp(-e.deltaY * 0.0025);
      applyZoom(view.zoom * factor, local(e));
      return;
    }
    const horizontal = e.shiftKey && e.deltaX === 0;
    scrollRef.current.scrollBy({
      left: horizontal ? e.deltaY : e.deltaX,
      top: horizontal ? 0 : e.deltaY,
    });
  };

  const onScroll = () => {
    const el = scrollRef.current;
    if (!el || !lay || !view || !viewport) return;
    if (settingScroll.current) {
      settingScroll.current = false;
      return;
    }
    const v = viewFromScroll(view.zoom, lay, el.scrollLeft, el.scrollTop, viewport);
    setView(doc.documentId, { zoom: v.zoom, cx: v.cx + ext.x, cy: v.cy + ext.y });
  };

  useEffect(() => {
    const el = scrollRef.current?.parentElement;
    if (!el) return;
    const stop = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) e.preventDefault();
    };
    el.addEventListener("wheel", stop, { passive: false });
    return () => el.removeEventListener("wheel", stop);
  }, []);

  const resizeCursor: Record<string, string> = {
    nw: "nwse-resize",
    se: "nwse-resize",
    ne: "nesw-resize",
    sw: "nesw-resize",
    n: "ns-resize",
    s: "ns-resize",
    e: "ew-resize",
    w: "ew-resize",
    rotate: "grab",
    end: "pointer",
    anchor: "move",
  };
  const cursor =
    drag?.kind === "pan"
      ? "grabbing"
      : space
        ? "grab"
        : hover === "locked"
          ? "not-allowed"
          : hover && resizeCursor[hover]
            ? resizeCursor[hover]
            : hover === "object" || drag?.kind === "move"
              ? "move"
              : tool === "text"
                ? "text"
                : tool === "move"
                  ? "default"
                  : "crosshair";

  const drawing = drag?.kind === "draw" && styles ? drag : null;
  const lockAt = usePointer((s) => (hover === "locked" ? s.screen : null));
  const scale = lay?.scale ?? 1;
  const editingId = textEdit?.id ?? null;
  const shownCrop = drag?.kind === "crop" ? drag.rect : cropRect;

  // While a gesture takes an object past the edge it shows whole, unless a crop left it
  // outside or it is a blur, which only works on the image (EDT-071).
  const free = new Set<string>();
  if (autoExpand) {
    const outside = new Set(doc.outside ?? []);
    const ids = [
      ...(drag?.kind === "move" ? drag.ids : []),
      ...(multiOp ? selection : []),
      drag && "id" in drag ? drag.id : null,
      pending?.version === doc.version ? pending.object.id : null,
    ];
    for (const id of ids) {
      const o = id ? doc.objects.find((x) => x.id === id) : undefined;
      if (o && o.type !== "blur" && !outside.has(o.id)) free.add(o.id);
    }
  }

  const clip = { clipX: ext.x, clipY: ext.y, clipWidth: ext.w, clipHeight: ext.h };

  const objectNode = (o: DocObject) => {
    switch (o.type) {
      case "arrow":
      case "line":
        return (
          <ArrowShape
            key={o.id}
            g={arrowGeometry(o.from, o.to, o.style)}
            style={o.style}
            scale={scale}
          />
        );
      case "shape":
        return <ShapeShape key={o.id} o={o} scale={scale} />;
      case "highlight":
        return <HighlightShape key={o.id} o={o} />;
      case "blur": {
        const c = committed.get(o.id);
        return (
          <BlurShape
            key={o.id}
            o={o}
            committed={c?.type === "blur" ? c : o}
            image={objectImages.get(o.id)}
            canvas={canvasSize}
          />
        );
      }
      case "text":
        return o.id === editingId ? null : (
          <TextShape key={o.id} o={o} image={objectImages.get(o.id)} />
        );
      case "image":
        return <ImageShape key={o.id} o={o} image={objectImages.get(o.id)} />;
      case "callout": {
        const c = committed.get(o.id);
        return (
          <CalloutShape
            key={o.id}
            o={o}
            committed={c?.type === "callout" ? c : o}
            image={objectImages.get(o.id)}
          />
        );
      }
      case "step": {
        const c = committed.get(o.id);
        return (
          <StepShape
            key={o.id}
            o={o}
            committed={c?.type === "step" ? c : o}
            image={objectImages.get(o.id)}
          />
        );
      }
      default:
        return null;
    }
  };

  const dashedBox = (kind: "blur" | "text" | "step" | "callout") => {
    if (drawing?.tool !== kind) return null;
    const r = dragRect(
      drawing.from,
      drawing.to,
      drawing.square || kind === "step",
      drawing.fromCenter,
    );
    return (
      <Rect
        x={r.x}
        y={r.y}
        width={r.w}
        height={r.h}
        stroke={ACCENT}
        strokeWidth={1.5 / scale}
        dash={[6 / scale, 4 / scale]}
        {...(kind === "blur" ? { fill: "rgba(128,128,128,0.35)" } : {})}
      />
    );
  };
  const previews = (
    <>
      {drawing && styles && (drawing.tool === "arrow" || drawing.tool === "line") && (
        <ArrowShape
          g={arrowGeometry(drawing.from, drawing.to, styles[drawing.tool])}
          style={styles[drawing.tool]}
          scale={scale}
        />
      )}
      {drawing && styles && drawing.tool === "shape" && (
        <ShapeShape
          o={{
            id: "new",
            type: "shape",
            rect: dragRect(drawing.from, drawing.to, drawing.square, drawing.fromCenter),
            rotation: 0,
            style: styles.shape,
          }}
          scale={scale}
        />
      )}
      {dashedBox("text")}
      {dashedBox("step")}
      {dashedBox("callout")}
      {drawing && styles && drawing.tool === "highlight" && (
        <HighlightShape
          o={{
            id: "new",
            type: "highlight",
            rect: dragRect(drawing.from, drawing.to, drawing.square, drawing.fromCenter),
            rotation: 0,
            style: styles.highlight,
          }}
        />
      )}
    </>
  );
  const blurPreview = dashedBox("blur");

  return (
    <div className="canvas-area">
      {/* Over a locked object, a small lock by the pointer (EDT-085). */}
      {hover === "locked" && lockAt && (
        <span
          className="lock-badge"
          style={{ left: lockAt.x + 14, top: lockAt.y + 14 }}
          aria-hidden="true"
        >
          <Icon name="lock" />
        </span>
      )}
      <div className="canvas-scroll" ref={scrollRef} onScroll={onScroll}>
        {lay && <div style={{ width: lay.contentWidth, height: lay.contentHeight }} />}
      </div>
      {lay && viewport && (
        <div
          className="canvas-stage"
          style={{ width: viewport.width, height: viewport.height, cursor }}
          tabIndex={0}
          role="application"
          aria-label={t("ui.editor.canvas_label", { name: doc.name })}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={() => setDrag(null)}
          onPointerLeave={() => usePointer.setState({ point: null, screen: null })}
          onDoubleClick={onDoubleClick}
          onContextMenu={onContextMenu}
          onWheel={onWheel}
          onAuxClick={(e) => e.preventDefault()}
        >
          <Stage width={viewport.width} height={viewport.height} listening={false}>
            <Layer imageSmoothingEnabled={view !== null && view.zoom < 100}>
              <Group
                x={lay.originX - lay.scrollLeft - ext.x * scale}
                y={lay.originY - lay.scrollTop - ext.y * scale}
                scaleX={scale}
                scaleY={scale}
              >
                <Rect
                  x={ext.x}
                  y={ext.y}
                  width={ext.w}
                  height={ext.h}
                  fill="#ffffff"
                  shadowColor="black"
                  shadowBlur={12 / scale}
                  shadowOpacity={0.25}
                />
                <Group {...clip}>
                  <Rect
                    x={ext.x}
                    y={ext.y}
                    width={ext.w}
                    height={ext.h}
                    fillPatternImage={checker as unknown as HTMLImageElement}
                    fillPatternScaleX={1 / scale}
                    fillPatternScaleY={1 / scale}
                  />
                  {doc.canvas.color && (
                    <Rect
                      width={canvasSize.width}
                      height={canvasSize.height}
                      fill={doc.canvas.color}
                    />
                  )}
                  {base && (
                    <KImage image={base} x={baseAt.x - doc.crop.x} y={baseAt.y - doc.crop.y} />
                  )}
                </Group>
                {/* Objects are cut at the edge, except the one being drawn or dragged:
                    it shows whole and the canvas grows to fit it when the gesture ends
                    (EDT-071). Each is cut on its own to keep their order. */}
                {shown.map((o) =>
                  free.has(o.id) ? (
                    objectNode(o)
                  ) : (
                    <Group key={o.id} {...clip}>
                      {objectNode(o)}
                    </Group>
                  ),
                )}
                {autoExpand ? previews : <Group {...clip}>{previews}</Group>}
                <Group {...clip}>{blurPreview}</Group>
                {shownCrop && (
                  <Group>
                    {/* Outside the crop the image is darkened (EDT-049). */}
                    <Rect
                      x={ext.x}
                      y={ext.y}
                      width={ext.w}
                      height={shownCrop.y - ext.y}
                      fill="rgba(0,0,0,0.55)"
                    />
                    <Rect
                      x={ext.x}
                      y={shownCrop.y + shownCrop.h}
                      width={ext.w}
                      height={ext.y + ext.h - shownCrop.y - shownCrop.h}
                      fill="rgba(0,0,0,0.55)"
                    />
                    <Rect
                      x={ext.x}
                      y={shownCrop.y}
                      width={shownCrop.x - ext.x}
                      height={shownCrop.h}
                      fill="rgba(0,0,0,0.55)"
                    />
                    <Rect
                      x={shownCrop.x + shownCrop.w}
                      y={shownCrop.y}
                      width={ext.x + ext.w - shownCrop.x - shownCrop.w}
                      height={shownCrop.h}
                      fill="rgba(0,0,0,0.55)"
                    />
                  </Group>
                )}
              </Group>
            </Layer>
            <Layer>
              {shownCrop && (
                <Group>
                  <Line
                    points={corners({ ...shownCrop, rotation: 0 }).flatMap(toScreen)}
                    closed
                    stroke="#ffffff"
                    strokeWidth={1.5}
                  />
                  {RESIZE_HANDLES.map((h) => {
                    const [x, y] = toScreen(handlePoint({ ...shownCrop, rotation: 0 }, h, 0));
                    return (
                      <Rect
                        key={h}
                        x={x}
                        y={y}
                        width={HANDLE}
                        height={HANDLE}
                        offsetX={HANDLE / 2}
                        offsetY={HANDLE / 2}
                        fill="#ffffff"
                        stroke={ACCENT}
                        strokeWidth={1.5}
                      />
                    );
                  })}
                </Group>
              )}
              {drag?.kind === "band" && (
                <Line
                  points={corners({
                    ...dragRect(drag.start, drag.end, false, false),
                    rotation: 0,
                  }).flatMap(toScreen)}
                  closed
                  stroke={ACCENT}
                  strokeWidth={1}
                  dash={[4, 3]}
                  fill="rgba(0, 100, 251, 0.08)"
                />
              )}
              {/* Several selected: a thin outline each and a common box (EDT-079). */}
              {!shownCrop &&
                selection.length > 1 &&
                (() => {
                  const marks = shown.filter((o) => selection.includes(o.id));
                  const pts = marks.flatMap(extentPoints);
                  if (pts.length === 0) return null;
                  const xs = pts.map((q) => q[0]);
                  const ys = pts.map((q) => q[1]);
                  // While turning, the box turns with them (EDT-080).
                  const all: Box =
                    drag?.kind === "mrotate"
                      ? { ...drag.start, rotation: drag.angle }
                      : drag?.kind === "mresize"
                        ? drag.box
                        : {
                            x: Math.min(...xs),
                            y: Math.min(...ys),
                            w: Math.max(...xs) - Math.min(...xs),
                            h: Math.max(...ys) - Math.min(...ys),
                            rotation: 0,
                          };
                  const [tx, ty] = toScreen(handlePoint(all, "n", 0));
                  const [rx, ry] = toScreen(handlePoint(all, "rotate", STEM / scale));
                  return (
                    <Group>
                      {marks.map((o) => (
                        <Line
                          key={o.id}
                          points={
                            isSegment(o)
                              ? [...toScreen(o.from), ...toScreen(o.to)]
                              : (() => {
                                  const box = boxOf(o);
                                  return box ? corners(box).flatMap(toScreen) : [];
                                })()
                          }
                          closed={!isSegment(o)}
                          stroke={ACCENT}
                          strokeWidth={1}
                        />
                      ))}
                      <Line
                        points={corners(all).flatMap(toScreen)}
                        closed
                        stroke={ACCENT}
                        strokeWidth={1}
                        dash={[4, 3]}
                      />
                      <Line points={[tx, ty, rx, ry]} stroke={ACCENT} strokeWidth={1} />
                      <Circle
                        x={rx}
                        y={ry}
                        radius={HANDLE / 2}
                        fill="#ffffff"
                        stroke={ACCENT}
                        strokeWidth={1.5}
                      />
                      {RESIZE_HANDLES.map((h) => {
                        const [x, y] = toScreen(handlePoint(all, h, 0));
                        return (
                          <Rect
                            key={h}
                            x={x}
                            y={y}
                            width={HANDLE - 2}
                            height={HANDLE - 2}
                            offsetX={(HANDLE - 2) / 2}
                            offsetY={(HANDLE - 2) / 2}
                            rotation={all.rotation}
                            fill="#ffffff"
                            stroke={ACCENT}
                            strokeWidth={1.5}
                          />
                        );
                      })}
                    </Group>
                  );
                })()}
              {!shownCrop &&
                selection.length === 1 &&
                shown
                  .filter((o) => selection.includes(o.id) && o.id !== editingId)
                  .map((o) => {
                    if (isSegment(o)) {
                      return (["from", "to"] as const).map((end) => {
                        const [x, y] = toScreen(o[end]);
                        return (
                          <Rect
                            key={`${o.id}-${end}`}
                            x={x}
                            y={y}
                            width={HANDLE}
                            height={HANDLE}
                            offsetX={HANDLE / 2}
                            offsetY={HANDLE / 2}
                            rotation={45}
                            fill={ACCENT}
                            stroke="#ffffff"
                            strokeWidth={1.5}
                          />
                        );
                      });
                    }
                    const box = boxOf(o);
                    if (!box) return null;
                    const outline = corners(box).flatMap(toScreen);
                    const [tx, ty] = toScreen(handlePoint(box, "n", 0));
                    const [rx, ry] = toScreen(handlePoint(box, "rotate", STEM / scale));
                    const list = handlesOf(o);
                    return (
                      <Group key={o.id}>
                        <Line
                          points={outline}
                          closed
                          stroke={ACCENT}
                          strokeWidth={1}
                          dash={[4, 3]}
                        />
                        {list.includes("rotate") && (
                          <>
                            <Line points={[tx, ty, rx, ry]} stroke={ACCENT} strokeWidth={1} />
                            <Circle
                              x={rx}
                              y={ry}
                              radius={HANDLE / 2}
                              fill="#ffffff"
                              stroke={ACCENT}
                              strokeWidth={1.5}
                            />
                          </>
                        )}
                        {list
                          .filter((h) => h !== "rotate")
                          .map((h) => {
                            const [x, y] = toScreen(handlePoint(box, h, 0));
                            return (
                              <Rect
                                key={h}
                                x={x}
                                y={y}
                                width={HANDLE - 2}
                                height={HANDLE - 2}
                                offsetX={(HANDLE - 2) / 2}
                                offsetY={(HANDLE - 2) / 2}
                                rotation={box.rotation}
                                fill="#ffffff"
                                stroke={ACCENT}
                                strokeWidth={1.5}
                              />
                            );
                          })}
                        {o.type === "callout" && calloutHandles(o, box)}
                      </Group>
                    );
                  })}
            </Layer>
          </Stage>
        </div>
      )}
      {textEdit && lay && (
        <TextEditor edit={textEdit} toScreen={toScreen} scale={scale} dpi={doc.dpi} />
      )}
    </div>
  );
}

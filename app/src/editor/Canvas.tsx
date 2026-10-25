// The editor canvas (EDT-007, EDT-022, EDT-024, EDT-026, EDT-029, EDT-040, EDT-042,
// EDT-049). Konva draws; the pointer is handled here in canvas pixels; every change is
// one command when the gesture ends.
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Circle, Group, Image as KImage, Layer, Line, Rect, Stage } from "react-konva";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { applyCrop, startNewText, startTextEdit } from "./actions";
import {
  contains,
  corners,
  dragRect,
  handlePoint,
  resize,
  RESIZE_HANDLES,
  rotateTo,
  type Box,
  type Handle,
  type Rect as RectBox,
} from "./boxes";
import { arrowGeometry, constrain45, type Point } from "./geometry";
import { useObjectImages } from "./objectImages";
import { ArrowShape, BlurShape, ShapeShape, TextShape } from "./ObjectShapes";
import { boxOf, hits, moved, reboxed } from "./objects";
import {
  docUrl,
  takeZoomRequest,
  useEditor,
  usePointer,
  type DocDetails,
  type DocObject,
} from "./state";
import { TextEditor } from "./TextEditor";
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
const ACCENT = "#0b7469";

type BoxHandle = Exclude<Handle, "rotate">;

type ArrowObj = Extract<DocObject, { type: "arrow" }>;

/** What a press on a handle grabs: the crop box, an arrow end or an object's handle. */
type HandleHit =
  | { crop: BoxHandle }
  | { object: ArrowObj; end: "from" | "to" }
  | { object: DocObject; handle: Handle };

type Drag =
  | {
      kind: "draw";
      tool: "arrow" | "shape" | "blur" | "text";
      from: Point;
      to: Point;
      startX: number;
      startY: number;
      square: boolean;
      fromCenter: boolean;
    }
  | { kind: "move"; id: string; start: Point; delta: Point; moved: boolean }
  | { kind: "end"; id: string; end: "from" | "to"; other: Point; point: Point }
  | { kind: "resize"; id: string; handle: BoxHandle; start: Box; box: Box }
  | { kind: "rotate"; id: string; start: Box; box: Box }
  | { kind: "crop"; mode: "new" | "move" | BoxHandle; start: Point; from: RectBox; rect: RectBox }
  | { kind: "pan"; x: number; y: number; left: number; top: number };

function useBaseImage(id: string): HTMLImageElement | null {
  const [img, setImg] = useState<HTMLImageElement | null>(null);
  useEffect(() => {
    let alive = true;
    const el = new window.Image();
    // Readable pixels, for the canvas color picker (EDT-117 `I`).
    el.crossOrigin = "anonymous";
    el.onload = () => alive && setImg(el);
    el.src = docUrl(id, "base");
    return () => {
      alive = false;
      setImg(null);
    };
  }, [id]);
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
  if (o.type === "shape" || o.type === "blur") return [...RESIZE_HANDLES, "rotate"];
  return [];
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
  const { setSelection, setView, showError } = useEditor.getState();

  const scrollRef = useRef<HTMLDivElement>(null);
  const [viewport, setViewport] = useState<Size | null>(null);
  const dpr = useDevicePixelRatio();
  const base = useBaseImage(doc.documentId);
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
  // whole original image, so the crop can grow back (EDT-050).
  const ext: RectBox = cropping
    ? { x: -doc.crop.x, y: -doc.crop.y, w: doc.base.width, h: doc.base.height }
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
    if (doc.view?.zoom && doc.view.x !== undefined && doc.view.y !== undefined)
      return { zoom: doc.view.zoom, cx: doc.view.x, cy: doc.view.y };
    return {
      zoom: openingZoom(canvasSize, viewport, dpr),
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
  const shown: DocObject[] = doc.objects.map((o) => {
    if (pending?.version === doc.version && pending.object.id === o.id) return pending.object;
    if (!drag || !("id" in drag) || drag.id !== o.id) return o;
    switch (drag.kind) {
      case "move":
        return moved(o, drag.delta[0], drag.delta[1])[0];
      case "end":
        return o.type === "arrow" ? { ...o, [drag.end]: drag.point } : o;
      case "resize":
      case "rotate":
        return reboxed(o, drag.box)[0];
    }
  });
  const committed = new Map(doc.objects.map((o) => [o.id, o]));

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
    if (selection.length !== 1) return null;
    const o = shown.find((x) => x.id === selection[0]);
    if (!o) return null;
    if (o.type === "arrow") {
      for (const end of ["to", "from"] as const) if (near(o[end], s)) return { object: o, end };
      return null;
    }
    const box = boxOf(o);
    if (!box) return null;
    for (const h of handlesOf(o))
      if (near(handlePoint(box, h, STEM / lay.scale), s)) return { object: o, handle: h };
    return null;
  };

  const hitObject = (p: Point): DocObject | null => {
    if (!lay) return null;
    for (const o of [...shown].reverse()) if (hits(o, p, HIT / lay.scale)) return o;
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
    if (hit) {
      setSelection([hit.id]);
      capture();
      setDrag({ kind: "move", id: hit.id, start: p, delta: [0, 0], moved: false });
      return;
    }
    if (tool === "arrow" || tool === "shape" || tool === "blur" || tool === "text") {
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

  const onPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!lay) return;
    const s = local(e);
    const p = toCanvas(s);
    usePointer.setState({ point: p, screen: s });
    if (!drag) {
      const h = hitHandle(s);
      const handleName = h ? ("crop" in h ? h.crop : "end" in h ? "end" : h.handle) : null;
      setHover(handleName ?? (!cropRect && hitObject(p) ? "object" : null));
      return;
    }
    switch (drag.kind) {
      case "pan":
        if (scrollRef.current) {
          scrollRef.current.scrollLeft = drag.left - (e.clientX - drag.x);
          scrollRef.current.scrollTop = drag.top - (e.clientY - drag.y);
        }
        break;
      case "draw":
        setDrag({
          ...drag,
          to: drag.tool === "arrow" && e.shiftKey ? constrain45(drag.from, p) : p,
          square: e.shiftKey,
          fromCenter: e.altKey,
        });
        break;
      case "end":
        setDrag({ ...drag, point: e.shiftKey ? constrain45(drag.other, p) : p });
        break;
      case "move": {
        let dx = p[0] - drag.start[0];
        let dy = p[1] - drag.start[1];
        // Shift keeps the move horizontal or vertical (EDT-117).
        if (e.shiftKey) {
          if (Math.abs(dx) > Math.abs(dy)) dy = 0;
          else dx = 0;
        }
        const isMoved = drag.moved || Math.hypot(dx, dy) * lay.scale >= DRAG_THRESHOLD;
        setDrag({ ...drag, delta: isMoved ? [dx, dy] : [0, 0], moved: isMoved });
        break;
      }
      case "resize":
        setDrag({
          ...drag,
          box: resize(drag.start, drag.handle, p, { keepRatio: e.shiftKey, fromCenter: e.altKey }),
        });
        break;
      case "rotate":
        setDrag({ ...drag, box: { ...drag.start, rotation: rotateTo(drag.start, p, e.shiftKey) } });
        break;
      case "crop": {
        const bounds = ext;
        let rect: RectBox;
        if (drag.mode === "new") rect = dragRect(drag.start, p, e.shiftKey, e.altKey);
        else if (drag.mode === "move") {
          const dx = Math.min(
            Math.max(p[0] - drag.start[0], bounds.x - drag.from.x),
            bounds.x + bounds.w - drag.from.x - drag.from.w,
          );
          const dy = Math.min(
            Math.max(p[1] - drag.start[1], bounds.y - drag.from.y),
            bounds.y + bounds.h - drag.from.y - drag.from.h,
          );
          rect = { ...drag.from, x: drag.from.x + dx, y: drag.from.y + dy };
        } else {
          // Shift keeps the current proportion (EDT-049).
          rect = resize({ ...drag.from, rotation: 0 }, drag.mode, p, { keepRatio: e.shiftKey });
        }
        setDrag({ ...drag, rect: clampRect(rect, bounds) });
        break;
      }
    }
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
          d.tool === "arrow"
            ? { type: "arrow", from: d.from, to: d.to }
            : { type: d.tool, rect: dragRect(d.from, d.to, d.square, d.fromCenter) };
        const r = await run<{ ids: string[] }>("document.annotate", {
          documentId: id,
          objects: [object],
        });
        // The new object stays selected to adjust it (EDT-022, editor.autoSelectAfterDraw).
        if (r.ids[0]) setSelection([r.ids[0]]);
      } else if (d.kind === "move" && d.moved) {
        const o = committed.get(d.id);
        if (o) {
          const [next, set] = moved(o, d.delta[0], d.delta[1]);
          await update(next, set);
        }
      } else if (d.kind === "end") {
        const o = committed.get(d.id);
        if (o?.type === "arrow") await update({ ...o, [d.end]: d.point }, { [d.end]: d.point });
      } else if (d.kind === "resize" || d.kind === "rotate") {
        const o = committed.get(d.id);
        if (o) {
          const [next, set] = reboxed(o, d.box);
          await update(next, set);
        }
      } else if (d.kind === "crop") {
        useEditor.setState({ cropRect: d.rect });
      }
    } catch (err) {
      setPending(null);
      showError(err);
    }
  };

  const onPointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    const d = drag;
    setDrag(null);
    if (!d || d.kind === "pan") return;
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
      if (click) return;
    }
    if (d.kind === "crop" && d.mode === "new") {
      const s = local(e);
      const [sx, sy] = toScreen(d.start);
      if (Math.hypot(s.x - sx, s.y - sy) < DRAG_THRESHOLD) return;
    }
    void finish(d);
  };

  const onDoubleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const p = toCanvas(local(e));
    if (cropRect) {
      // Double click inside the crop box applies it (EDT-049).
      if (contains({ ...cropRect, rotation: 0 }, p)) void applyCrop();
      return;
    }
    const hit = hitObject(p);
    if (hit?.type === "text") startTextEdit(hit);
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
  };
  const cursor =
    drag?.kind === "pan"
      ? "grabbing"
      : space
        ? "grab"
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
  const scale = lay?.scale ?? 1;
  const editingId = textEdit?.id ?? null;
  const shownCrop = drag?.kind === "crop" ? drag.rect : cropRect;

  return (
    <div className="canvas-area">
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
                <Group
                  {...(cropping
                    ? {}
                    : {
                        clipX: 0,
                        clipY: 0,
                        clipWidth: canvasSize.width,
                        clipHeight: canvasSize.height,
                      })}
                >
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
                  {base && <KImage image={base} x={-doc.crop.x} y={-doc.crop.y} />}
                  {shown.map((o) => {
                    switch (o.type) {
                      case "arrow":
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
                      default:
                        return null;
                    }
                  })}
                  {drawing && styles && drawing.tool === "arrow" && (
                    <ArrowShape
                      g={arrowGeometry(drawing.from, drawing.to, styles.arrow)}
                      style={styles.arrow}
                      scale={scale}
                    />
                  )}
                  {drawing && styles && drawing.tool === "shape" && (
                    <ShapeShape
                      o={{
                        id: "new",
                        type: "shape",
                        rect: dragRect(
                          drawing.from,
                          drawing.to,
                          drawing.square,
                          drawing.fromCenter,
                        ),
                        rotation: 0,
                        style: styles.shape,
                      }}
                      scale={scale}
                    />
                  )}
                  {drawing && (drawing.tool === "blur" || drawing.tool === "text") && (
                    <Rect
                      {...(() => {
                        const r = dragRect(
                          drawing.from,
                          drawing.to,
                          drawing.square,
                          drawing.fromCenter,
                        );
                        return { x: r.x, y: r.y, width: r.w, height: r.h };
                      })()}
                      stroke={ACCENT}
                      strokeWidth={1.5 / scale}
                      dash={[6 / scale, 4 / scale]}
                      {...(drawing.tool === "blur" ? { fill: "rgba(128,128,128,0.35)" } : {})}
                    />
                  )}
                </Group>
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
              {!shownCrop &&
                shown
                  .filter((o) => selection.includes(o.id) && o.id !== editingId)
                  .map((o) => {
                    if (o.type === "arrow") {
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

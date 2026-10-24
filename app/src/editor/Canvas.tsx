// The editor canvas (EDT-007, EDT-022, EDT-024, EDT-042). Konva draws; the pointer
// is handled here in image pixels; every change is one command when the gesture ends.
import Konva from "konva";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Circle, Group, Image as KImage, Layer, Line, Rect, Stage } from "react-konva";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import {
  arrowGeometry,
  constrain45,
  distanceToSegment,
  type ArrowStyle,
  type Geometry,
  type Point,
} from "./geometry";
import { docUrl, useEditor, usePointer, type ArrowObject, type DocDetails } from "./state";
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
/** Handle size and hit distance, in CSS px (EDT-022). */
const HANDLE = 11;
const HIT = 6;

type Drag =
  | { kind: "draw"; from: Point; to: Point; startX: number; startY: number }
  | { kind: "move"; id: string; from: Point; to: Point; start: Point; delta: Point; moved: boolean }
  | { kind: "end"; id: string; end: "from" | "to"; other: Point; point: Point }
  | { kind: "pan"; x: number; y: number; left: number; top: number };

function isArrow(o: { type: string }): o is ArrowObject {
  return o.type === "arrow";
}

function useBaseImage(id: string): HTMLImageElement | null {
  const [img, setImg] = useState<HTMLImageElement | null>(null);
  useEffect(() => {
    let alive = true;
    const el = new window.Image();
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

function ArrowShape({ g, style, scale }: { g: Geometry; style: ArrowStyle; scale: number }) {
  const ref = useRef<Konva.Group>(null);
  const translucent = style.opacity < 100;
  // A translucent arrow is drawn whole and blended once, like the exported image.
  useEffect(() => {
    const node = ref.current;
    if (!node) return;
    if (translucent) node.cache({ pixelRatio: Math.max(1, scale * window.devicePixelRatio) });
    else node.clearCache();
  });
  const color = style.color;
  return (
    <Group ref={ref} opacity={style.opacity / 100} listening={false}>
      {g.shaft && (
        <Line
          points={g.shaft}
          stroke={color}
          strokeWidth={g.width}
          lineCap="butt"
          lineJoin="round"
          {...(g.dash ? { dash: g.dash } : {})}
        />
      )}
      {g.caps.map((s, i) =>
        s.kind === "circle" ? (
          <Circle
            key={i}
            x={s.cx}
            y={s.cy}
            radius={s.r}
            {...(s.fill ? { fill: color } : { stroke: color, strokeWidth: s.strokeWidth })}
          />
        ) : (
          <Line
            key={i}
            points={s.points}
            closed={s.closed}
            lineCap="butt"
            lineJoin="round"
            {...(s.fill
              ? { fill: color, strokeEnabled: false }
              : { stroke: color, strokeWidth: s.strokeWidth })}
          />
        ),
      )}
    </Group>
  );
}

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
  const { setSelection, setView, showError } = useEditor.getState();

  const scrollRef = useRef<HTMLDivElement>(null);
  const [viewport, setViewport] = useState<Size | null>(null);
  const dpr = useDevicePixelRatio();
  const base = useBaseImage(doc.documentId);
  const [drag, setDrag] = useState<Drag | null>(null);
  const [space, setSpace] = useState(false);
  const [hover, setHover] = useState<"object" | "handle" | null>(null);
  /** What a finished gesture showed, until the core reports the change. */
  const [pending, setPending] = useState<{ version: number; arrow: ArrowObject } | null>(null);

  const image: Size = { width: doc.canvas.width, height: doc.canvas.height };

  // Size of the visible area, without scroll bars.
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
      zoom: openingZoom(image, viewport, dpr),
      cx: image.width / 2,
      cy: image.height / 2,
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [storedView, viewport, doc.documentId]);

  const lay = view && viewport ? layout(view, image, viewport, dpr) : null;

  // The first view becomes this document's view, so the status bar shows its zoom.
  useEffect(() => {
    if (view && !storedView) setView(doc.documentId, view);
  }, [view, storedView, doc.documentId, setView]);

  // Keep the scroll bars where the view says.
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

  // Tell the core the view, so it is remembered (EDT-007).
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

  const applyZoom = (zoom: number, at?: { x: number; y: number }) => {
    if (!view || !viewport) return;
    const p = at ?? { x: viewport.width / 2, y: viewport.height / 2 };
    setView(doc.documentId, zoomAround(view, zoom, p, image, viewport, dpr));
  };

  // Zoom asked by a command, a key or the status bar.
  const lastRequest = useRef(0);
  useEffect(() => {
    if (!zoomRequest || zoomRequest.seq === lastRequest.current || !view || !viewport) return;
    // Only a percentage needs a value; in and out step from the current zoom.
    if (zoomRequest.mode === "percent" && zoomRequest.value === null) return;
    lastRequest.current = zoomRequest.seq;
    switch (zoomRequest.mode) {
      case "fit":
        setView(doc.documentId, {
          zoom: fitZoom(image, viewport, dpr),
          cx: image.width / 2,
          cy: image.height / 2,
        });
        break;
      case "original":
        applyZoom(originalZoom(doc.metadata.display?.scale ?? dpr, dpr));
        break;
      case "in":
      case "out":
        applyZoom(stepZoom(view.zoom, zoomRequest.mode === "in"));
        break;
      default:
        if (zoomRequest.value !== null) applyZoom(zoomRequest.value);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [zoomRequest, view, viewport]);

  // Space held = pan (EDT-117).
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

  const arrows = doc.objects.filter(isArrow).map((a) => {
    // Until the core reports the change, the arrow stays where the gesture left it.
    if (pending?.version === doc.version && pending.arrow.id === a.id) return pending.arrow;
    if (drag?.kind === "move" && drag.id === a.id)
      return {
        ...a,
        from: [a.from[0] + drag.delta[0], a.from[1] + drag.delta[1]] as Point,
        to: [a.to[0] + drag.delta[0], a.to[1] + drag.delta[1]] as Point,
      };
    if (drag?.kind === "end" && drag.id === a.id) return { ...a, [drag.end]: drag.point };
    return a;
  });

  const local = (e: React.PointerEvent | React.WheelEvent) => {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };

  const hitHandle = (p: { x: number; y: number }) => {
    if (!lay) return null;
    for (const a of arrows) {
      if (!selection.includes(a.id)) continue;
      for (const end of ["to", "from"] as const) {
        const pt = a[end];
        const sx = lay.originX - lay.scrollLeft + pt[0] * lay.scale;
        const sy = lay.originY - lay.scrollTop + pt[1] * lay.scale;
        if (Math.hypot(sx - p.x, sy - p.y) <= HANDLE / 2 + 2) return { arrow: a, end };
      }
    }
    return null;
  };

  const hitObject = (ip: Point): ArrowObject | null => {
    if (!lay) return null;
    // Topmost first.
    for (const a of [...arrows].reverse()) {
      const reach = Math.max(a.style.width / 2, HIT / lay.scale);
      if (distanceToSegment(ip, a.from, a.to) <= reach) return a;
    }
    return null;
  };

  const onPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!lay || !scrollRef.current) return;
    (e.currentTarget as HTMLElement).focus();
    const p = local(e);
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
    const ip = imagePoint(lay, p);
    const handle = hitHandle(p);
    if (handle) {
      e.currentTarget.setPointerCapture(e.pointerId);
      setDrag({
        kind: "end",
        id: handle.arrow.id,
        end: handle.end,
        other: handle.end === "to" ? handle.arrow.from : handle.arrow.to,
        point: handle.arrow[handle.end],
      });
      return;
    }
    // EDT-022: a click on an object selects it, except with Alt (draw over it).
    const hit = e.altKey && tool !== "move" ? null : hitObject(ip);
    if (hit) {
      setSelection([hit.id]);
      e.currentTarget.setPointerCapture(e.pointerId);
      setDrag({
        kind: "move",
        id: hit.id,
        from: hit.from,
        to: hit.to,
        start: ip,
        delta: [0, 0],
        moved: false,
      });
      return;
    }
    if (tool === "arrow") {
      e.currentTarget.setPointerCapture(e.pointerId);
      setDrag({ kind: "draw", from: ip, to: ip, startX: p.x, startY: p.y });
      return;
    }
    setSelection([]);
  };

  const onPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!lay) return;
    const p = local(e);
    const ip = imagePoint(lay, p);
    usePointer.setState({ point: ip });
    if (!drag) {
      setHover(hitHandle(p) ? "handle" : hitObject(ip) ? "object" : null);
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
        setDrag({ ...drag, to: e.shiftKey ? constrain45(drag.from, ip) : ip });
        break;
      case "end":
        setDrag({ ...drag, point: e.shiftKey ? constrain45(drag.other, ip) : ip });
        break;
      case "move": {
        let dx = ip[0] - drag.start[0];
        let dy = ip[1] - drag.start[1];
        // Shift keeps the move horizontal or vertical (EDT-117).
        if (e.shiftKey) {
          if (Math.abs(dx) > Math.abs(dy)) dy = 0;
          else dx = 0;
        }
        const moved = drag.moved || Math.hypot(dx, dy) * lay.scale >= DRAG_THRESHOLD;
        setDrag({ ...drag, delta: moved ? [dx, dy] : [0, 0], moved });
        break;
      }
    }
  };

  const finish = async (d: Drag) => {
    const id = doc.documentId;
    try {
      if (d.kind === "draw") {
        const r = await run<{ ids: string[] }>("document.annotate", {
          documentId: id,
          objects: [{ type: "arrow", from: d.from, to: d.to }],
        });
        // The new arrow stays selected to adjust it (EDT-022, editor.autoSelectAfterDraw).
        if (r.ids[0]) setSelection([r.ids[0]]);
      } else if (d.kind === "move" && d.moved) {
        const a = arrows.find((x) => x.id === d.id);
        if (a) setPending({ version: doc.version, arrow: a });
        await run("document.update_object", {
          documentId: id,
          objectId: d.id,
          set: {
            from: [d.from[0] + d.delta[0], d.from[1] + d.delta[1]],
            to: [d.to[0] + d.delta[0], d.to[1] + d.delta[1]],
          },
        });
      } else if (d.kind === "end") {
        const a = arrows.find((x) => x.id === d.id);
        if (a) setPending({ version: doc.version, arrow: a });
        await run("document.update_object", {
          documentId: id,
          objectId: d.id,
          set: { [d.end]: d.point },
        });
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
      const p = local(e);
      if (Math.hypot(p.x - d.startX, p.y - d.startY) < DRAG_THRESHOLD) return;
    }
    void finish(d);
  };

  const onWheel = (e: React.WheelEvent<HTMLDivElement>) => {
    if (!view || !scrollRef.current) return;
    if (e.ctrlKey || e.metaKey) {
      // Ctrl+wheel and pinch zoom around the pointer (EDT-007).
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
    setView(doc.documentId, viewFromScroll(view.zoom, lay, el.scrollLeft, el.scrollTop, viewport));
  };

  // While the editor wants native pinch on macOS too, block the page zoom.
  useEffect(() => {
    const el = scrollRef.current?.parentElement;
    if (!el) return;
    const stop = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) e.preventDefault();
    };
    el.addEventListener("wheel", stop, { passive: false });
    return () => el.removeEventListener("wheel", stop);
  }, []);

  const cursor =
    drag?.kind === "pan"
      ? "grabbing"
      : space
        ? "grab"
        : hover === "handle"
          ? "pointer"
          : hover === "object" || drag?.kind === "move"
            ? "move"
            : tool === "arrow"
              ? "crosshair"
              : "default";

  const drawing =
    drag?.kind === "draw" && styles ? arrowGeometry(drag.from, drag.to, styles.arrow) : null;

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
          onPointerLeave={() => usePointer.setState({ point: null })}
          onWheel={onWheel}
          onAuxClick={(e) => e.preventDefault()}
        >
          <Stage width={viewport.width} height={viewport.height} listening={false}>
            <Layer imageSmoothingEnabled={view !== null && view.zoom < 100}>
              <Group
                x={lay.originX - lay.scrollLeft}
                y={lay.originY - lay.scrollTop}
                scaleX={lay.scale}
                scaleY={lay.scale}
              >
                <Rect
                  width={image.width}
                  height={image.height}
                  fill="#ffffff"
                  shadowColor="black"
                  shadowBlur={12 / lay.scale}
                  shadowOpacity={0.25}
                />
                <Group clipX={0} clipY={0} clipWidth={image.width} clipHeight={image.height}>
                  <Rect
                    width={image.width}
                    height={image.height}
                    fillPatternImage={checker as unknown as HTMLImageElement}
                    fillPatternScaleX={1 / lay.scale}
                    fillPatternScaleY={1 / lay.scale}
                  />
                  {doc.canvas.color && (
                    <Rect width={image.width} height={image.height} fill={doc.canvas.color} />
                  )}
                  {base && <KImage image={base} x={-doc.crop.x} y={-doc.crop.y} />}
                  {arrows.map((a) => (
                    <ArrowShape
                      key={a.id}
                      g={arrowGeometry(a.from, a.to, a.style)}
                      style={a.style}
                      scale={lay.scale}
                    />
                  ))}
                  {drawing && styles && (
                    <ArrowShape g={drawing} style={styles.arrow} scale={lay.scale} />
                  )}
                </Group>
              </Group>
            </Layer>
            <Layer>
              {arrows
                .filter((a) => selection.includes(a.id))
                .flatMap((a) =>
                  (["from", "to"] as const).map((end) => (
                    <Rect
                      key={`${a.id}-${end}`}
                      x={lay.originX - lay.scrollLeft + a[end][0] * lay.scale}
                      y={lay.originY - lay.scrollTop + a[end][1] * lay.scale}
                      width={HANDLE}
                      height={HANDLE}
                      offsetX={HANDLE / 2}
                      offsetY={HANDLE / 2}
                      rotation={45}
                      fill="#0b7469"
                      stroke="#ffffff"
                      strokeWidth={1.5}
                    />
                  )),
                )}
            </Layer>
          </Stage>
        </div>
      )}
    </div>
  );
}

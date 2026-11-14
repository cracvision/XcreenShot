// The editor canvas (EDT-007, EDT-022, EDT-024, EDT-026, EDT-029, EDT-040, EDT-042,
// EDT-049). Konva draws; the pointer is handled here in canvas pixels; every change is
// one command when the gesture ends.
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import {
  Circle,
  Ellipse,
  Group,
  Image as KImage,
  Layer,
  Line,
  Rect,
  Shape,
  Stage,
  Text,
} from "react-konva";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { setting, settingValue, useApp, useSetting } from "../store";
import {
  applyCrop,
  duplicateSelection,
  startCalloutEdit,
  startNewText,
  startTextEdit,
} from "./actions";
import { canvasMenu, lockedMenu, menuPoint, selectionMenu, zoneMenu } from "./menus";
import { captureIntoZone, templateCommand } from "./templateActions";
import {
  bands,
  dividerAt,
  snapOffset,
  templateOf,
  useDropPoint,
  useZones,
  zoneAt,
  zonesOf,
} from "./templateUi";
import {
  insideSelection,
  modeOf,
  selectionAction,
  selectPixels,
  type SelectMode,
} from "./pixelSelection";
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
import { useObjectImages, useShadowImages } from "./objectImages";
import { useFinalPreview } from "./finalPreview";
import { sampleUrl, STYLED_TOOLS } from "./themesUi";
import {
  ArrowShape,
  BlurShape,
  CalloutShape,
  HighlightShape,
  ImageShape,
  MagnifyShape,
  PenPreview,
  PenShape,
  ShapeShape,
  SpotlightShape,
  StampShape,
  StepShape,
  TextShape,
  ZoneShape,
} from "./ObjectShapes";
import { boxOf, extentPoints, hits, inBand, moved, reboxed } from "./objects";
import {
  docUrl,
  takeZoomRequest,
  useEditor,
  usePointer,
  isSegment,
  type CalloutObject,
  type MagnifyObject,
  type ShapeObject,
  type DocDetails,
  type DocObject,
  type ZoneObject,
} from "./state";
import { RedactOverlay } from "./RedactOverlay";
import { TextEditor } from "./TextEditor";
import { Icon } from "./icons";
import { expand, unitOf } from "./groups";
import { boundsOf, normalized, transformed, type TransformOp } from "./transform";
import { snapMove, snapPoint, type Guide, type Spacing } from "./snap";
import { QuickBar } from "./QuickBar";
import { CanvasBars } from "./CanvasBars";
import type { Insets } from "./insets";
import { QUICK_KINDS } from "./quickKinds";
import { hasRadius, MAX_TAILS } from "./callouts";
import { corners as polyCorners, fromPoints as polyFromPoints, refit, sideAt } from "./polygon";
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
import { separatedText } from "./separateState";
import { openQr } from "./qrState";
import { isOn } from "../features";

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
type ImageObj = Extract<DocObject, { type: "image" }>;

/** What a press on a handle grabs: the crop box, an arrow end or an object's handle. */
type HandleHit =
  | { crop: BoxHandle }
  | { object: ArrowObj; end: "from" | "to" }
  | { object: DocObject; handle: Handle }
  // A handle of the box around several selected objects (EDT-080).
  | { multi: Handle }
  // A callout's tail tip, its + (a new tail), its corner radius, its anchor (EDT-028).
  | { tail: CalloutObject; index: number | null }
  | { radius: CalloutObject | ShapeObject | ImageObj }
  // A polygon's corner (EDT-030).
  | { vertex: ShapeObject; index: number }
  | { anchor: CalloutObject }
  | { zone: MagnifyObject }
  // The canvas's edges and its rounded corners' rhombus (EDT-070, EDT-075).
  | { canvas: BoxHandle }
  | { canvasRadius: true };

type Drag =
  | {
      kind: "draw";
      tool:
        | "arrow"
        | "line"
        | "shape"
        | "blur"
        | "text"
        | "step"
        | "callout"
        | "highlight"
        | "magnify"
        | "spotlight"
        | "stamp"
        | "simplify";
      from: Point;
      to: Point;
      startX: number;
      startY: number;
      square: boolean;
      fromCenter: boolean;
      /** Let go without moving: a click (a step goes there). */
      click?: boolean;
    }
  | {
      kind: "move";
      ids: string[];
      start: Point;
      delta: Point;
      moved: boolean;
      copy?: boolean;
      /** Snapping lines and equal spaces shown while moving (EDT-084). */
      guides?: Guide[];
      spacings?: Spacing[];
    }
  // A selection box drawn with Move over an empty place (EDT-079).
  | { kind: "band"; start: Point; end: Point; touch: boolean; add: boolean }
  | { kind: "end"; id: string; end: "from" | "to"; other: Point; point: Point }
  | { kind: "resize"; id: string; handle: BoxHandle; start: Box; box: Box; guides?: Guide[] }
  | { kind: "rotate"; id: string; start: Box; box: Box }
  // Several objects stretched or turned with their common box (EDT-080).
  | { kind: "mresize"; handle: BoxHandle; start: Box; box: Box; guides?: Guide[] }
  | { kind: "mrotate"; start: Box; angle: number }
  // A callout's tail tip (index null: a new one), radius or body alone (EDT-028).
  | { kind: "tail"; id: string; index: number | null; point: Point; start: Point }
  | { kind: "radius"; id: string; box: Box; value: number }
  | { kind: "vertex"; id: string; index: number; point: Point }
  | { kind: "anchor"; id: string; start: Point; delta: Point }
  // A lens's zone, moved alone (EDT-038).
  | { kind: "zone"; id: string; start: Point; delta: Point }
  | { kind: "crop"; mode: "new" | "move" | BoxHandle; start: Point; from: RectBox; rect: RectBox }
  | { kind: "pan"; x: number; y: number; left: number; top: number }
  // A freehand stroke being drawn (EDT-037); Shift keeps it straight.
  | { kind: "pen"; points: Point[]; straight: boolean }
  // An eraser stroke (EDT-048), on the image or on the floating image `target`.
  | { kind: "erase"; points: Point[]; straight: boolean; target: string | null }
  // A pixel selection being drawn (EDT-043), on the image or a floating image.
  | {
      kind: "psel";
      shape: "rectangle" | "ellipse" | "freehand";
      from: Point;
      to: Point;
      points: Point[];
      mode: SelectMode;
      target: string | null;
      square: boolean;
      startX: number;
      startY: number;
    }
  // Selected pixels being dragged (EDT-044).
  | { kind: "pxmove"; start: Point; delta: Point; moved: boolean }
  // The canvas resized by a handle (EDT-070), or its corners rounded (EDT-075).
  | { kind: "canvas"; handle: BoxHandle; start: Box; box: Box; fromCenter: boolean }
  | { kind: "cradius"; value: number; startX: number; startY: number }
  // A strip being marked with Cut out (EDT-052); Shift turns its direction.
  | { kind: "cut"; start: Point; end: Point; vertical: boolean }
  // A template's zone (EDT-098): its picture taken to another zone, or moved inside it
  // while framing; a section's divider dragged (EDT-099).
  | { kind: "zswap"; id: string; start: Point; to: Point }
  | { kind: "zframe"; id: string; start: Point; from: [number, number]; delta: Point }
  | { kind: "divider"; id: string; start: Point; height: number; delta: number };

/** What stays put when the canvas is resized by a handle (EDT-070). */
const ANCHOR_OF: Record<BoxHandle, string> = {
  e: "left",
  w: "right",
  n: "bottom",
  s: "top",
  ne: "bottomLeft",
  nw: "bottomRight",
  se: "topLeft",
  sw: "topRight",
};

/** How far the corners' rhombus sits from the top-right corner at radius 0, in screen
 * pixels (EDT-075). */
const CORNER_GAP = 16;

/** Snapping distance in screen pixels, whatever the zoom (EDT-084). */
const SNAP = 6;

/** Thicker guides with the system's increased contrast (APP-045). */
const highContrast = (): boolean =>
  typeof window !== "undefined" &&
  Boolean(window.matchMedia?.("(prefers-contrast: more), (forced-colors: active)").matches);

/** The size or radius by the pointer while a handle is dragged. */
function DragLabel({ x, y, text }: { x: number; y: number; text: string }) {
  const w = text.length * 7 + 14;
  return (
    <Group x={x + 14} y={y + 14} listening={false}>
      <Rect width={w} height={22} cornerRadius={5} fill="rgba(16, 16, 28, 0.88)" />
      <Text x={7} y={5} text={text} fontSize={12} fill="#ffffff" />
    </Group>
  );
}

/** The eraser's tip around the pointer, in screen pixels: dark and light, so it shows
 * on any image (EDT-048). */
function EraserOutline({
  at,
  size,
  square,
}: {
  at: { x: number; y: number };
  size: number;
  square: boolean;
}) {
  const r = Math.max(size / 2, 1);
  return (
    <Group x={at.x} y={at.y} listening={false}>
      {[
        { stroke: "#000000", width: 3, opacity: 0.45 },
        { stroke: "#ffffff", width: 1, opacity: 1 },
      ].map((l) =>
        square ? (
          <Rect
            key={l.stroke}
            x={-r}
            y={-r}
            width={r * 2}
            height={r * 2}
            stroke={l.stroke}
            strokeWidth={l.width}
            opacity={l.opacity}
          />
        ) : (
          <Circle
            key={l.stroke}
            radius={r}
            stroke={l.stroke}
            strokeWidth={l.width}
            opacity={l.opacity}
          />
        ),
      )}
    </Group>
  );
}

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
  // A lens: its corners change its zoom (EDT-038).
  if (o.type === "magnify") return ["nw", "ne", "se", "sw"];
  if (
    o.type === "shape" ||
    o.type === "blur" ||
    o.type === "image" ||
    o.type === "highlight" ||
    o.type === "spotlight" ||
    o.type === "stamp" ||
    o.type === "simplify" ||
    o.type === "qr"
  )
    return [...RESIZE_HANDLES, "rotate"];
  return [];
}

/** A polygon being drawn (EDT-030): its corners so far and a line to the pointer. */
function PolygonDraft({ points, scale }: { points: Point[]; scale: number }) {
  const at = usePointer((s) => s.point);
  const line = at ? [...points, at] : points;
  return (
    <Line
      points={line.flat()}
      stroke={ACCENT}
      strokeWidth={1.5 / scale}
      dash={[6 / scale, 4 / scale]}
      listening={false}
    />
  );
}

/** `r` on whole pixels. */
function wholePixels(r: { x: number; y: number; w: number; h: number }) {
  const x = Math.round(r.x);
  const y = Math.round(r.y);
  return { x, y, w: Math.round(r.x + r.w) - x, h: Math.round(r.y + r.h) - y };
}

/** A pixel selection being drawn (EDT-043): dashed like the polygon draft. */
function SelectionDraft({ d, scale }: { d: Extract<Drag, { kind: "psel" }>; scale: number }) {
  const look = {
    stroke: ACCENT,
    strokeWidth: 1.5 / scale,
    dash: [6 / scale, 4 / scale],
    listening: false,
  };
  if (d.shape === "freehand") return <Line points={d.points.flat()} {...look} />;
  const r = dragRect(d.from, d.to, d.square, false);
  return d.shape === "ellipse" ? (
    <Ellipse x={r.x + r.w / 2} y={r.y + r.h / 2} radiusX={r.w / 2} radiusY={r.h / 2} {...look} />
  ) : (
    <Rect x={r.x} y={r.y} width={r.w} height={r.h} {...look} />
  );
}

/** Whether a lens sits away from its zone (EDT-038). */
function lensAway(o: MagnifyObject): boolean {
  const z = o.source;
  return Math.hypot(o.at[0] - (z.x + z.w / 2), o.at[1] - (z.y + z.h / 2)) > 1;
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
  /** How much of the canvas area the floating bars cover (Cristal): the image is fitted,
   * centered and scrolled in the rest, and passes beneath them. */
  insets: Insets;
}

export function Canvas({ doc, insets }: Props) {
  const { t } = useTranslation();
  const tool = useEditor((s) => s.tool);
  const styles = useEditor((s) => s.styles);
  const selection = useEditor((s) => s.selection);
  const storedView = useEditor((s) => s.views[doc.documentId]);
  const zoomRequest = useEditor((s) => s.zoomRequest);
  const textEdit = useEditor((s) => s.textEdit);
  const cropState = useEditor((s) => s.cropRect);
  const autoExpand = useSetting("editor.autoExpandCanvas", true);
  const snapOn = useSetting("editor.snap", true);
  const quickOn = useSetting("editor.quickEditBar", true);
  // The pixel grid (EDT-008) and the workspace color (EDT-010).
  const gridOn = useSetting("editor.pixelGrid", true);
  const gridFrom = useApp((s) => {
    const v = settingValue(s.settings, "editor.pixelGridMinZoom");
    return typeof v === "number" ? v : 600;
  });
  const workspace = useApp((s) => {
    const v = settingValue(s.settings, "editor.workspaceColor");
    return typeof v === "string" && /^#[0-9a-f]{6}([0-9a-f]{2})?$/i.test(v) ? v : null;
  });
  const scaleStrokes = useSetting("editor.scaleStrokes", true);
  const { setSelection, setView, showError } = useEditor.getState();
  // Another document: no group is entered (EDT-082).
  useEffect(() => {
    useEditor.setState({ entered: [] });
  }, [doc.documentId]);

  const scrollRef = useRef<HTMLDivElement>(null);
  const stageRef = useRef<HTMLDivElement>(null);
  const areaRef = useRef<HTMLDivElement>(null);
  // The whole canvas area, and the part of it the floating bars leave visible.
  const [area, setArea] = useState<Size | null>(null);
  const { top: insetTop, right: insetRight, bottom: insetBottom, left: insetLeft } = insets;
  const viewport: Size | null = useMemo(
    () =>
      area
        ? {
            width: Math.max(1, area.width - insetLeft - insetRight),
            height: Math.max(1, area.height - insetTop - insetBottom),
          }
        : null,
    [area, insetTop, insetRight, insetBottom, insetLeft],
  );
  const dpr = useDevicePixelRatio();
  // With flattened objects, edits of pixels or new original pixels (resized, turned,
  // flipped, cut out) the image shown changes with them (EDT-091, ADR 0005, ADR 0006).
  const flatKey = useMemo(
    () =>
      doc.flattened?.length || doc.patches?.length || doc.baseImage
        ? hashOf(JSON.stringify([doc.flattened ?? [], doc.patches ?? [], doc.baseImage ?? null]))
        : 0,
    [doc.flattened, doc.patches, doc.baseImage],
  );
  const base = useBaseImage(doc.documentId, flatKey);
  const baseAt = { x: doc.baseView?.x ?? 0, y: doc.baseView?.y ?? 0 };
  const objectImages = useObjectImages(doc.documentId, doc.objects, doc.version);
  // Templates (EDT-095 to EDT-099): the zone clicked last, the one being framed.
  const activeZone = useZones((s) => s.active);
  const framing = useZones((s) => s.framing);
  const dropAt = useDropPoint((s) => s.at);
  const template = templateOf(doc);
  useEffect(() => () => useZones.setState({ pointAt: null, active: null, framing: null }), []);
  const shadowImages = useShadowImages(doc.documentId, doc.objects);
  // The image with its effects over the canvas, unless something is being changed
  // (EDT-053): then the editable layers show.
  const finalView = useFinalPreview(doc);
  const [drag, setDrag] = useState<Drag | null>(null);
  const [space, setSpace] = useState(false);
  const [hover, setHover] = useState<string | null>(null);
  /** A polygon being drawn, a corner per click (EDT-030). Another tool or document
   * drops it. */
  const [polyDraft, setPolyDraft] = useState<{ doc: string; points: Point[] } | null>(null);
  const poly =
    polyDraft && polyDraft.doc === doc.documentId && tool === "shape" ? polyDraft.points : null;
  const setPoly = (points: Point[] | null) =>
    setPolyDraft(points ? { doc: doc.documentId, points } : null);
  const closePolygon = async (pts: Point[]) => {
    setPoly(null);
    // A double click also clicked twice on its corner.
    const kept = pts.filter((p, i) => {
      const prev = pts[i - 1];
      return !prev || Math.hypot(p[0] - prev[0], p[1] - prev[1]) > 0.5;
    });
    if (kept.length < 3) return;
    const { rect, points } = polyFromPoints(kept);
    try {
      const r = await run<{ ids: string[] }>("document.annotate", {
        documentId: doc.documentId,
        objects: [{ type: "shape", rect, points, style: { shapeType: "polygon" } }],
      });
      if (r.ids[0] && setting("editor.autoSelectAfterDraw", true)) setSelection([r.ids[0]]);
    } catch (err) {
      showError(err);
    }
  };
  /** A polygon selection being drawn (EDT-043), a corner per click. */
  const [pselDraft, setPselDraft] = useState<{
    doc: string;
    points: Point[];
    mode: SelectMode;
    target: string | null;
  } | null>(null);
  const pPoly =
    pselDraft &&
    pselDraft.doc === doc.documentId &&
    tool === "select" &&
    styles?.select.shape === "polygon"
      ? pselDraft
      : null;
  const closePixelPolygon = (d: { points: Point[]; mode: SelectMode; target: string | null }) => {
    setPselDraft(null);
    const kept = d.points.filter((p, i) => {
      const prev = d.points[i - 1];
      return !prev || Math.hypot(p[0] - prev[0], p[1] - prev[1]) > 0.5;
    });
    if (kept.length < 3) return;
    void selectPixels({
      shape: "polygon",
      points: kept,
      mode: d.mode,
      antialias: styles?.select.antialias ?? true,
      feather: styles?.select.feather ?? 0,
      ...(d.target ? { objectId: d.target } : {}),
    });
  };
  useEffect(() => {
    if (!pPoly) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Enter") closePixelPolygon(pPoly);
      else if (e.key === "Backspace")
        setPselDraft(
          pPoly.points.length > 1 ? { ...pPoly, points: pPoly.points.slice(0, -1) } : null,
        );
      else if (e.key === "Escape") setPselDraft(null);
      else return;
      e.preventDefault();
      e.stopPropagation();
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pPoly]);
  // The marching ants move, unless motion is reduced (EDT-043).
  const hasPixelSel = Boolean(doc.pixelSelection);
  const [antPhase, setAntPhase] = useState(0);
  useEffect(() => {
    if (!hasPixelSel || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const timer = window.setInterval(() => setAntPhase((v) => (v + 1) % 8), 120);
    return () => window.clearInterval(timer);
  }, [hasPixelSel]);
  // Enter closes it, Backspace takes the last corner away, Esc cancels it.
  useEffect(() => {
    if (!poly) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Enter") void closePolygon(poly);
      else if (e.key === "Backspace") setPoly(poly.length > 1 ? poly.slice(0, -1) : null);
      else if (e.key === "Escape") setPoly(null);
      else return;
      e.preventDefault();
      e.stopPropagation();
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [poly]);

  /** What a finished gesture showed, until the core reports the change. */
  const [pending, setPending] = useState<{ version: number; object: DocObject } | null>(null);
  // Once the core reports another version it is done: Undo can bring the old version
  // number back, and the gesture must not show again.
  if (pending && pending.version !== doc.version) setPending(null);

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
    const el = areaRef.current;
    if (!el) return;
    const measure = () =>
      setArea((a) =>
        a && a.width === el.clientWidth && a.height === el.clientHeight
          ? a
          : { width: el.clientWidth, height: el.clientHeight },
      );
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

  // Until the user zooms or scrolls, the opening zoom follows the visible part: a bar that
  // appears with the document (the tray, with the first capture) must not cover the image.
  const opening = useRef<View | null>(null);
  useEffect(() => {
    if (view && !storedView) {
      opening.current = view;
      setView(doc.documentId, view);
    }
  }, [view, storedView, doc.documentId, setView]);
  useEffect(() => {
    if (!viewport || !storedView || storedView !== opening.current) return;
    if (setting("editor.rememberView", true) && doc.view?.zoom) return;
    if (!setting("editor.shrinkToFit", true)) return;
    const zoom = openingZoom(canvasSize, viewport, dpr);
    if (Math.abs(zoom - storedView.zoom) < 0.01) return;
    const next = { zoom, cx: canvasSize.width / 2, cy: canvasSize.height / 2 };
    opening.current = next;
    setView(doc.documentId, next);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [viewport?.width, viewport?.height]);

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

  /** Canvas point → position in the canvas area (CSS px). The visible part starts at the
   * insets: the layout works inside it. */
  const toScreen = (p: Point): [number, number] =>
    lay
      ? [
          insetLeft + lay.originX - lay.scrollLeft + (p[0] - ext.x) * lay.scale,
          insetTop + lay.originY - lay.scrollTop + (p[1] - ext.y) * lay.scale,
        ]
      : [0, 0];
  const toCanvas = (s: { x: number; y: number }): Point => {
    if (!lay) return [0, 0];
    const [x, y] = imagePoint(lay, { x: s.x - insetLeft, y: s.y - insetTop });
    return [x + ext.x, y + ext.y];
  };
  // Files dropped on a zone fill it (EDT-098): the editor asks where a window point is.
  useEffect(() => {
    useZones.setState({
      pointAt: (cx, cy) => {
        const r = stageRef.current?.getBoundingClientRect();
        return r ? toCanvas({ x: cx - r.left, y: cy - r.top }) : null;
      },
    });
  });
  /** Where a zone's picture is being framed to, held to its center and edges. */
  const zoneOffset = (
    z: ZoneObject,
    image: HTMLImageElement,
    d: { from: [number, number]; delta: Point },
  ): [number, number] =>
    snapOffset(
      z,
      { w: image.naturalWidth || image.width, h: image.naturalHeight || image.height },
      [d.from[0] + d.delta[0], d.from[1] + d.delta[1]],
      SNAP / (lay?.scale ?? 1),
    );

  /** Zoom keeping `at` (a point in the canvas area) still; the visible part's center by
   * default. */
  const applyZoom = (zoom: number, at?: { x: number; y: number }) => {
    if (!view || !viewport) return;
    const p = at
      ? { x: at.x - insetLeft, y: at.y - insetTop }
      : { x: viewport.width / 2, y: viewport.height / 2 };
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
        if (o.type === "shape") return { ...o, style: { ...o.style, cornerRadius: drag.value } };
        return o.type === "callout" ? { ...o, style: { ...o.style, cornerRadius: drag.value } } : o;
      case "vertex": {
        // The corner follows the pointer; the box grows or shrinks around them all.
        const box = boxOf(o);
        if (o.type !== "shape" || !box || !o.points) return o;
        const pts = polyCorners(box, o.points).map((q, k): Point =>
          k === drag.index ? drag.point : q,
        );
        const { rect, points } = refit(box, pts);
        return { ...o, rect, points };
      }
      case "anchor":
        // The body moves; the tails stay where they point (EDT-028). A lens moves
        // away from its zone (EDT-038).
        if (o.type === "magnify")
          return { ...o, at: [o.at[0] + drag.delta[0], o.at[1] + drag.delta[1]] };
        return o.type === "callout"
          ? {
              ...o,
              rect: { ...o.rect, x: o.rect.x + drag.delta[0], y: o.rect.y + drag.delta[1] },
            }
          : o;
      case "zone":
        return o.type === "magnify"
          ? {
              ...o,
              source: {
                ...o.source,
                x: o.source.x + drag.delta[0],
                y: o.source.y + drag.delta[1],
              },
            }
          : o;
      default:
        // A template's zone keeps its place while its picture moves (EDT-098).
        return o;
    }
  });
  const committed = new Map(doc.objects.map((o) => [o.id, o]));
  const plusPoint = (box: Box) => plusAt(box, lay?.scale ?? 1);
  const radiusPoint = (box: Box, radius: number) => radiusAt(box, radius, lay?.scale ?? 1);
  // With Move and nothing selected, the canvas has handles to resize it and a rhombus
  // for its rounded corners (EDT-070, EDT-075).
  // A template's size comes from its sections (EDT-099): no canvas handles there.
  const canvasEdit =
    tool === "move" &&
    selection.length === 0 &&
    !doc.pixelSelection &&
    !cropping &&
    !textEdit &&
    !template;
  const canvasBox: Box = { x: 0, y: 0, w: canvasSize.width, h: canvasSize.height, rotation: 0 };
  /** The rhombus for radius `r`: on the diagonal from the top-right corner. */
  const cornerPoint = (r: number): Point => {
    const o = r + CORNER_GAP / (lay?.scale ?? 1);
    return [canvasSize.width - o, o];
  };
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
  /** A rounded rectangle's radius and a polygon's corners (EDT-030). */
  const shapeHandles = (o: ShapeObject, box: Box) => {
    const diamond = (q: Point, key: string, filled: boolean) => {
      const [x, y] = toScreen(q);
      return (
        <Rect
          key={key}
          x={x}
          y={y}
          width={HANDLE - 2}
          height={HANDLE - 2}
          offsetX={(HANDLE - 2) / 2}
          offsetY={(HANDLE - 2) / 2}
          rotation={45 + box.rotation}
          fill={filled ? ACCENT : "#ffffff"}
          stroke={filled ? "#ffffff" : ACCENT}
          strokeWidth={1.5}
        />
      );
    };
    return (
      <>
        {o.style.shapeType === "roundedRectangle" &&
          diamond(radiusPoint(box, o.style.cornerRadius), "radius", false)}
        {o.style.shapeType === "polygon" &&
          o.points &&
          polyCorners(box, o.points).map((q, i) => diamond(q, `v${i}`, true))}
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
    if (canvasEdit) {
      if (near(cornerPoint(doc.canvas.cornerRadius), s)) return { canvasRadius: true };
      for (const h of RESIZE_HANDLES)
        if (near(handlePoint(canvasBox, h, 0), s)) return { canvas: h as BoxHandle };
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
    // A lens's zone has a handle of its own (EDT-038).
    // Only once the lens sits elsewhere: before, a drag from its middle moves the lens.
    if (o.type === "magnify" && !o.locked && lensAway(o)) {
      const z = o.source;
      if (near([z.x + z.w / 2, z.y + z.h / 2], s)) return { zone: o };
    }
    // A polygon's corners and a rounded rectangle's radius (EDT-030).
    if (o.type === "shape" && !o.locked) {
      const box = boxOf(o);
      if (box && o.style.shapeType === "polygon" && o.points) {
        const pts = polyCorners(box, o.points);
        for (let i = pts.length - 1; i >= 0; i--) {
          const q = pts[i];
          if (q && near(q, s)) return { vertex: o, index: i };
        }
      }
      if (box && o.style.shapeType === "roundedRectangle")
        if (near(radiusPoint(box, o.style.cornerRadius), s)) return { radius: o };
    }
    // A floating image's rounded corners (EDT-075).
    if (o.type === "image" && !o.locked) {
      const box = boxOf(o);
      if (box && near(radiusPoint(box, o.style.cornerRadius ?? 0), s)) return { radius: o };
    }
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
    // A press outside the zone being framed ends the framing (EDT-098).
    const framed = useZones.getState().framing;
    if (framed && zoneAt(doc.objects, p)?.id !== framed) useZones.setState({ framing: null });
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
    // Cut out (EDT-052): a strip of the whole width (or height) of the image.
    if (tool === "cutout" && styles) {
      capture();
      const vertical = (styles.cutout.direction === "vertical") !== e.shiftKey;
      setDrag({ kind: "cut", start: p, end: p, vertical });
      return;
    }
    // Pixel tools (EDT-047, EDT-048; ADR 0005) work on the image, not on objects.
    if (tool === "fill" && styles) {
      // A click on a floating image fills it; on any other object, nothing.
      const hit = hitObject(p, true);
      if (hit && (hit.type !== "image" || hit.locked)) return;
      const f = styles.fill;
      void run("document.fill", {
        documentId: doc.documentId,
        at: p,
        color: f.color,
        tolerance: f.tolerance,
        opacity: f.opacity,
        global: f.global,
        ...(hit ? { objectId: hit.id } : {}),
      }).catch(showError);
      return;
    }
    if (tool === "eraser") {
      // On the selected floating image, if one is; otherwise on the image.
      const only = selection.length === 1 ? shown.find((o) => o.id === selection[0]) : undefined;
      const target = only?.type === "image" && !only.locked ? only.id : null;
      capture();
      setDrag({ kind: "erase", points: [p], straight: false, target });
      return;
    }
    if (tool === "select" && styles) {
      const sel = doc.pixelSelection ?? null;
      const mode = modeOf(sel !== null, e);
      // A drag inside the selection moves its pixels (EDT-044).
      if (sel && mode === "replace" && !pPoly && insideSelection(sel, p)) {
        capture();
        setDrag({ kind: "pxmove", start: p, delta: [0, 0], moved: false });
        return;
      }
      // On a floating image, its pixels; anywhere else, the image's.
      const hit = hitObject(p);
      const target = hit?.type === "image" && !hit.locked ? hit.id : null;
      const shape = styles.select.shape;
      if (shape === "polygon") {
        const draft = pPoly?.points ?? [];
        const first = draft[0];
        if (pPoly && draft.length >= 3 && first && near(first, s)) {
          closePixelPolygon(pPoly);
          return;
        }
        const last = draft[draft.length - 1];
        setPselDraft({
          doc: doc.documentId,
          points: [...draft, e.shiftKey && last ? constrain45(last, p) : p],
          mode: pPoly?.mode ?? mode,
          target: pPoly?.target ?? target,
        });
        return;
      }
      capture();
      setDrag({
        kind: "psel",
        shape,
        from: p,
        to: p,
        points: [p],
        mode,
        target,
        square: false,
        startX: s.x,
        startY: s.y,
      });
      return;
    }
    if (tool === "wand" && styles) {
      // A click selects the color there (EDT-046); Shift adds, Alt subtracts.
      const hit = hitObject(p);
      const target = hit?.type === "image" && !hit.locked ? hit.id : null;
      void selectPixels({
        shape: "wand",
        at: p,
        tolerance: styles.wand.tolerance,
        global: styles.wand.global,
        mode: modeOf(Boolean(doc.pixelSelection), e),
        ...(target ? { objectId: target } : {}),
      });
      return;
    }
    if (handle && "canvas" in handle) {
      capture();
      setDrag({
        kind: "canvas",
        handle: handle.canvas,
        start: canvasBox,
        box: canvasBox,
        fromCenter: e.altKey,
      });
      return;
    }
    if (handle && "canvasRadius" in handle) {
      capture();
      setDrag({ kind: "cradius", value: doc.canvas.cornerRadius, startX: s.x, startY: s.y });
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
          value: handle.radius.style.cornerRadius ?? 0,
        });
        return;
      }
    }
    if (handle && "vertex" in handle) {
      const o = handle.vertex;
      const box = boxOf(o);
      // Alt+click takes a corner away, down to three (EDT-030).
      if (e.altKey) {
        if (box && o.points && o.points.length > 3) {
          const pts = polyCorners(box, o.points).filter((_, k) => k !== handle.index);
          void run("document.update_object", {
            documentId: doc.documentId,
            objectId: o.id,
            set: refit(box, pts),
          }).catch(showError);
        }
        return;
      }
      const at = box && o.points ? polyCorners(box, o.points)[handle.index] : undefined;
      capture();
      setDrag({ kind: "vertex", id: o.id, index: handle.index, point: at ?? p });
      return;
    }
    if (handle && "zone" in handle) {
      capture();
      setDrag({ kind: "zone", id: handle.zone.id, start: p, delta: [0, 0] });
      return;
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
    // Alt+click on a side of the selected polygon adds a corner there (EDT-030).
    if (e.altKey && selection.length === 1) {
      const sel = shown.find((o) => o.id === selection[0]);
      const box = sel ? boxOf(sel) : null;
      if (sel?.type === "shape" && sel.style.shapeType === "polygon" && sel.points && box) {
        const pts = polyCorners(box, sel.points);
        const side = sideAt(pts, p, (HANDLE / 2 + 2) / lay.scale);
        if (side >= 0) {
          const next = [...pts.slice(0, side + 1), p, ...pts.slice(side + 1)];
          void run("document.update_object", {
            documentId: doc.documentId,
            objectId: sel.id,
            set: refit(box, next),
          }).catch(showError);
          return;
        }
      }
    }
    // A polygon is drawn a corner per click (EDT-030).
    if (tool === "shape" && styles?.shape.shapeType === "polygon") {
      const draft = poly ?? [];
      const first = draft[0];
      if (draft.length >= 3 && first && near(first, s)) {
        void closePolygon(draft);
        return;
      }
      const last = draft[draft.length - 1];
      setPoly([...draft, e.shiftKey && last ? constrain45(last, p) : p]);
      return;
    }
    // The divider under a template's section makes it taller or shorter (EDT-099).
    if (
      template &&
      (tool === "move" || tool === "favorites") &&
      !e.altKey &&
      p[0] >= 0 &&
      p[0] <= canvasSize.width
    ) {
      const section = dividerAt(template, p[1], HIT / lay.scale);
      if (section) {
        capture();
        setDrag({ kind: "divider", id: section.id, start: p, height: section.height, delta: 0 });
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
    // Dragging a lens alone moves it away from its zone (EDT-038).
    if (
      hit?.type === "magnify" &&
      !adding &&
      (selection.length <= 1 || !selection.includes(hit.id))
    ) {
      setSelection([hit.id]);
      capture();
      setDrag({ kind: "anchor", id: hit.id, start: p, delta: [0, 0] });
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
    // A template's zone (EDT-098): a click makes it the one Paste fills; a drag takes
    // its picture to another zone; while framing, a drag moves the picture inside it.
    if ((tool === "move" || tool === "favorites") && !adding) {
      const z = zoneAt(doc.objects, p);
      if (z) {
        setSelection([]);
        useZones.setState({ active: z.id });
        if (z.image) {
          capture();
          setDrag(
            useZones.getState().framing === z.id
              ? { kind: "zframe", id: z.id, start: p, from: z.offset ?? [0, 0], delta: [0, 0] }
              : { kind: "zswap", id: z.id, start: p, to: p },
          );
        }
        return;
      }
    }
    if (tool === "pen") {
      capture();
      setDrag({ kind: "pen", points: [p], straight: e.shiftKey });
      return;
    }
    if (tool === "move" || tool === "favorites") {
      // An empty place: a selection box (EDT-079).
      capture();
      useZones.setState({ active: null });
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
      tool === "highlight" ||
      tool === "magnify" ||
      tool === "spotlight" ||
      tool === "stamp" ||
      tool === "simplify"
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
  /** The boxes of the objects not in `except`, to snap to (EDT-084). */
  const snapTargets = (except: string[]): RectBox[] =>
    doc.objects
      .filter((o) => !except.includes(o.id))
      .map((o) => boundsOf([o]))
      .filter((r): r is RectBox => r !== null);
  const canvasRect: RectBox = { x: 0, y: 0, w: canvasSize.width, h: canvasSize.height };
  /** Snapping applies unless it is off or Ctrl (Cmd) is held (EDT-084). */
  const snapping = (e: { ctrlKey?: boolean; metaKey?: boolean }) =>
    snapOn && !e.ctrlKey && !e.metaKey;
  /** A resize handle's point snapped to the lines near it (EDT-084). */
  const snapHandle = (
    start: Box,
    handle: BoxHandle,
    p: Point,
    e: { ctrlKey?: boolean; metaKey?: boolean },
    except: string[],
  ): { point: Point; guides: Guide[] } => {
    if (!snapping(e) || start.rotation !== 0) return { point: p, guides: [] };
    return snapPoint(
      p,
      { x: /[ew]/.test(handle), y: /[ns]/.test(handle) },
      canvasRect,
      snapTargets(except),
      SNAP / (lay?.scale ?? 1),
    );
  };

  const step = (
    d: Exclude<Drag, { kind: "pan" }>,
    e: { shiftKey: boolean; altKey: boolean; ctrlKey?: boolean; metaKey?: boolean },
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
      case "psel": {
        if (d.shape !== "freehand") return { ...d, to: p, square: e.shiftKey && d.mode !== "add" };
        const last = d.points[d.points.length - 1] ?? p;
        const far = Math.hypot(p[0] - last[0], p[1] - last[1]) * (lay?.scale ?? 1) >= 2;
        return far ? { ...d, to: p, points: [...d.points, p] } : { ...d, to: p };
      }
      case "pxmove": {
        let dx = p[0] - d.start[0];
        let dy = p[1] - d.start[1];
        // Shift keeps the move horizontal or vertical.
        if (e.shiftKey) {
          if (Math.abs(dx) > Math.abs(dy)) dy = 0;
          else dx = 0;
        }
        const isMoved = d.moved || Math.hypot(dx, dy) * (lay?.scale ?? 1) >= DRAG_THRESHOLD;
        return { ...d, delta: isMoved ? [Math.round(dx), Math.round(dy)] : [0, 0], moved: isMoved };
      }
      case "erase":
      case "pen": {
        const first = d.points[0] ?? p;
        // Shift: a straight line, level, upright or at 45° (EDT-037).
        if (e.shiftKey) return { ...d, points: [first, constrain45(first, p)], straight: true };
        const last = d.points[d.points.length - 1] ?? p;
        const far = Math.hypot(p[0] - last[0], p[1] - last[1]) * (lay?.scale ?? 1) >= 1;
        return far ? { ...d, points: [...d.points, p], straight: false } : d;
      }
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
        if (!isMoved) return { ...d, delta: [0, 0], moved: false, guides: [], spacings: [] };
        const start = snapping(e)
          ? boundsOf(doc.objects.filter((o) => d.ids.includes(o.id)))
          : null;
        if (!start) return { ...d, delta: [dx, dy], moved: true, guides: [], spacings: [] };
        // Edges, centers and equal spaces (EDT-084); Shift keeps its axis.
        const r = snapMove(start, dx, dy, canvasRect, snapTargets(d.ids), SNAP / (lay?.scale ?? 1));
        const free = (axis: "x" | "y") => !e.shiftKey || (axis === "x" ? dy === 0 : dx === 0);
        return {
          ...d,
          delta: [free("x") ? r.dx : dx, free("y") ? r.dy : dy],
          moved: true,
          guides: r.guides.filter((g) => free(g.axis)),
          spacings: r.spacings.filter((s) => free(s.axis)),
        };
      }
      case "resize": {
        // An image keeps its proportions from a corner; Shift frees them (EDT-078).
        const kind = doc.objects.find((o) => o.id === d.id)?.type;
        const image = kind === "image";
        const corner = d.handle.length === 2;
        // A stamp keeps its proportions like a picture; Shift lets it stretch (EDT-031).
        const picture = image || kind === "stamp";
        const keepRatio = kind === "step" || (picture && corner ? !e.shiftKey : e.shiftKey);
        const s = snapHandle(d.start, d.handle, p, e, [d.id]);
        return {
          ...d,
          box: resize(d.start, d.handle, s.point, { keepRatio, fromCenter: e.altKey }),
          guides: s.guides,
        };
      }
      case "rotate":
        return { ...d, box: { ...d.start, rotation: rotateTo(d.start, p, e.shiftKey) } };
      case "mresize": {
        // From a corner the proportions stay, as with pictures; Shift frees them.
        const keepRatio = d.handle.length === 2 ? !e.shiftKey : e.shiftKey;
        const s = snapHandle(d.start, d.handle, p, e, selection);
        return {
          ...d,
          box: resize(d.start, d.handle, s.point, { keepRatio, fromCenter: e.altKey }),
          guides: s.guides,
        };
      }
      case "mrotate":
        return { ...d, angle: normalized(rotateTo(d.start, p, e.shiftKey)) };
      case "tail":
        return { ...d, point: p };
      case "vertex":
        return { ...d, point: p };
      case "canvas":
        // Alt from the center, Shift keeps the proportion (EDT-070).
        return {
          ...d,
          fromCenter: e.altKey,
          box: resize(d.start, d.handle, p, { keepRatio: e.shiftKey, fromCenter: e.altKey }),
        };
      case "cut":
        return {
          ...d,
          end: p,
          vertical: (styles?.cutout.direction === "vertical") !== e.shiftKey,
        };
      case "cradius": {
        // Along the diagonal; at most half the shorter side (EDT-075).
        const most = Math.floor(Math.min(canvasSize.width, canvasSize.height) / 2);
        const along = (canvasSize.width - p[0] + p[1]) / 2 - CORNER_GAP / (lay?.scale ?? 1);
        return { ...d, value: Math.round(Math.max(0, Math.min(most, along))) };
      }
      case "radius": {
        const [lx] = toLocal(d.box, p);
        const most = Math.min(200, d.box.w / 2, d.box.h / 2);
        return { ...d, value: Math.round(Math.max(0, Math.min(most, lx + d.box.w / 2))) };
      }
      case "anchor":
      case "zone": {
        let dx = p[0] - d.start[0];
        let dy = p[1] - d.start[1];
        if (e.shiftKey) {
          if (Math.abs(dx) > Math.abs(dy)) dy = 0;
          else dx = 0;
        }
        return { ...d, delta: [dx, dy] };
      }
      case "zswap":
        return { ...d, to: p };
      case "zframe":
        return { ...d, delta: [p[0] - d.start[0], p[1] - d.start[1]] };
      case "divider":
        return { ...d, delta: p[1] - d.start[1] };
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
    if (
      !drag &&
      tool === "select" &&
      doc.pixelSelection &&
      insideSelection(doc.pixelSelection, p)
    ) {
      setHover("pxsel");
      return;
    }
    if (!drag) {
      const h = hitHandle(s);
      const moving = (tool === "move" || tool === "favorites") && !cropRect;
      if (
        !h &&
        moving &&
        template &&
        p[0] >= 0 &&
        p[0] <= canvasSize.width &&
        dividerAt(template, p[1], HIT / lay.scale)
      ) {
        setHover("s");
        return;
      }
      const handleName = h
        ? "crop" in h
          ? h.crop
          : "canvas" in h
            ? h.canvas
            : "multi" in h
              ? h.multi
              : "end" in h || "tail" in h || "radius" in h || "vertex" in h || "canvasRadius" in h
                ? "end"
                : "anchor" in h || "zone" in h
                  ? "anchor"
                  : h.handle
        : null;
      setHover(
        handleName ??
          (cropRect
            ? null
            : hitObject(p)
              ? "object"
              : hitObject(p, true)
                ? "locked"
                : moving && zoneAt(doc.objects, p)?.image
                  ? "object"
                  : null),
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
      if (d.kind === "zswap") {
        const target = zoneAt(doc.objects, d.to);
        if (target && target.id !== d.id)
          await run("document.template", {
            documentId: id,
            action: "swap_zones",
            zone: d.id,
            with: target.id,
          });
        return;
      }
      if (d.kind === "zframe") {
        const z = doc.objects.find((o): o is ZoneObject => o.id === d.id && o.type === "zone");
        const image = objectImages.get(d.id);
        if (!z || !image || (d.delta[0] === 0 && d.delta[1] === 0)) return;
        await run("document.template", {
          documentId: id,
          action: "frame_zone",
          zone: d.id,
          offset: zoneOffset(z, image, d),
        });
        return;
      }
      if (d.kind === "divider") {
        const height = Math.max(40, Math.round(d.height + d.delta));
        if (height !== d.height)
          await run("document.template", {
            documentId: id,
            action: "resize_section",
            sectionId: d.id,
            height,
          });
        return;
      }
      if (d.kind === "erase") {
        const er = styles?.eraser;
        if (!er) return;
        await run("document.erase", {
          documentId: id,
          points: d.points,
          width: er.width,
          shape: er.shape,
          opacity: er.opacity,
          antialias: er.antialias,
          ...(d.target ? { objectId: d.target } : {}),
        });
        return;
      }
      if (d.kind === "pen") {
        const pen = styles?.pen;
        const r = await run<{ ids: string[] }>("document.annotate", {
          documentId: id,
          objects: [{ type: "pen", points: d.points }],
        });
        // A stroke painted into the image is no object to select.
        if (r.ids[0] && pen?.editable !== false && setting("editor.autoSelectAfterDraw", true))
          setSelection([r.ids[0]]);
        return;
      }
      if (d.kind === "draw") {
        const object =
          d.tool === "arrow" || d.tool === "line"
            ? { type: d.tool, from: d.from, to: d.to }
            : d.tool === "step"
              ? // A step is square; a click places it at its size (EDT-035).
                d.click
                ? { type: "step", at: d.from }
                : { type: "step", rect: dragRect(d.from, d.to, true, d.fromCenter) }
              : d.tool === "stamp"
                ? // A click places it at its usual size; a drag, at the height dragged.
                  d.click
                  ? { type: "stamp", at: d.from }
                  : { type: "stamp", rect: dragRect(d.from, d.to, d.square, d.fromCenter) }
                : d.tool === "magnify"
                  ? // A lens over the zone dragged; a click, over a small square (EDT-038).
                    {
                      type: "magnify",
                      source: d.click
                        ? { x: d.from[0] - 30, y: d.from[1] - 30, w: 60, h: 60 }
                        : dragRect(d.from, d.to, d.square, d.fromCenter),
                    }
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
      } else if (d.kind === "vertex") {
        const o = committed.get(d.id);
        const box = o ? boxOf(o) : null;
        if (o?.type !== "shape" || !box || !o.points) return;
        const pts = polyCorners(box, o.points).map((q, k): Point => (k === d.index ? d.point : q));
        await run("document.update_object", {
          documentId: id,
          objectId: d.id,
          set: refit(box, pts),
        });
      } else if (d.kind === "canvas") {
        const width = Math.round(d.box.w);
        const height = Math.round(d.box.h);
        if (width === canvasSize.width && height === canvasSize.height) return;
        await run("document.canvas_size", {
          documentId: id,
          width,
          height,
          anchor: d.fromCenter ? "center" : ANCHOR_OF[d.handle],
        });
      } else if (d.kind === "cut") {
        const c = styles?.cutout;
        if (!c) return;
        const axis = d.vertical ? 0 : 1;
        const r = await run<{ warnings?: string[] }>("document.cut_out", {
          documentId: id,
          direction: d.vertical ? "vertical" : "horizontal",
          from: Math.round(d.start[axis]),
          to: Math.round(d.end[axis]),
          mode: c.mode,
          join: c.join,
          gap: c.gap,
        });
        if (r.warnings?.[0]) useEditor.getState().showToast(r.warnings[0]);
      } else if (d.kind === "cradius") {
        if (d.value === doc.canvas.cornerRadius) return;
        await run("document.canvas", { documentId: id, cornerRadius: d.value });
      } else if (d.kind === "radius") {
        await run("document.update_object", {
          documentId: id,
          objectId: d.id,
          set: { style: { cornerRadius: d.value } },
        });
      } else if (
        d.kind === "zone" ||
        (d.kind === "anchor" && committed.get(d.id)?.type === "magnify")
      ) {
        const c = committed.get(d.id);
        if (c?.type !== "magnify" || (d.delta[0] === 0 && d.delta[1] === 0)) return;
        const set =
          d.kind === "zone"
            ? { source: { ...c.source, x: c.source.x + d.delta[0], y: c.source.y + d.delta[1] } }
            : { at: [c.at[0] + d.delta[0], c.at[1] + d.delta[1]] };
        await run("document.update_object", { documentId: id, objectId: d.id, set });
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
    if (d.kind === "psel") {
      const s = local(e);
      const st = styles?.select;
      if (Math.hypot(s.x - d.startX, s.y - d.startY) < DRAG_THRESHOLD) {
        // A click outside the selection drops it (EDT-043).
        if (doc.pixelSelection && d.mode === "replace") void selectPixels({ shape: "none" });
        return;
      }
      const common = {
        mode: d.mode,
        antialias: st?.antialias ?? true,
        feather: st?.feather ?? 0,
        ...(d.target ? { objectId: d.target } : {}),
      };
      if (d.shape === "freehand")
        void selectPixels({ shape: "freehand", points: d.points, ...common });
      else
        void selectPixels({
          shape: d.shape,
          // Whole pixels, as the selection is (EDT-043).
          rect: wholePixels(dragRect(d.from, d.to, d.square, false)),
          snap: st?.snap ?? false,
          level: st?.level ?? 50,
          ...common,
        });
      return;
    }
    if (d.kind === "pxmove") {
      if (d.moved && (d.delta[0] !== 0 || d.delta[1] !== 0))
        void selectionAction("move", { offset: d.delta });
      return;
    }
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
      const clickable = ["step", "callout", "magnify", "stamp"].includes(d.tool);
      if (click && !clickable) return;
      if (clickable) {
        void finish({ ...d, click });
        return;
      }
    }
    if (d.kind === "cut") {
      // A click marks nothing (EDT-052).
      const size = Math.abs(d.vertical ? d.end[0] - d.start[0] : d.end[1] - d.start[1]);
      if (size * (lay?.scale ?? 1) < DRAG_THRESHOLD) return;
      void finish(d);
      return;
    }
    if (d.kind === "cradius") {
      // A click rounds the corners 20 px (EDT-075).
      const s = local(e);
      const click = Math.hypot(s.x - d.startX, s.y - d.startY) < DRAG_THRESHOLD;
      void finish(click ? { ...d, value: 20 } : d);
      return;
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
    // Inside a pixel selection: what can be done with it (EDT-044).
    const sel = doc.pixelSelection;
    if (sel && insideSelection(sel, p)) {
      selectionMenu(menuPoint(e), !sel.target);
      return;
    }
    const hit = hitObject(p);
    // A locked object offers Unlock (EDT-085).
    const locked = hit ? null : hitObject(p, true);
    if (locked) {
      lockedMenu(locked.id, menuPoint(e));
      return;
    }
    // A template's zone: its own menu (EDT-098).
    const zone = hit ? null : zoneAt(doc.objects, p);
    if (zone) {
      useZones.setState({ active: zone.id });
      zoneMenu(zone, menuPoint(e));
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
    canvasMenu(
      hit !== null,
      menuPoint(e),
      layer,
      hit?.type === "step" ? hit : undefined,
      hit?.type === "image" && !hit.locked ? hit.id : undefined,
      hit?.type,
    );
  };

  const onDoubleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const p = toCanvas(local(e));
    if (pPoly) {
      if (pPoly.points.length >= 3) closePixelPolygon(pPoly);
      return;
    }
    if (poly) {
      if (poly.length >= 3) void closePolygon(poly);
      return;
    }
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
    // On a zone's picture: framing (EDT-098).
    if ((tool === "move" || tool === "favorites") && !cropRect && !hitObject(p)) {
      const z = zoneAt(doc.objects, p);
      if (z?.image) {
        useZones.setState({ framing: z.id, active: z.id });
        return;
      }
    }
    if (cropRect) {
      // Double click inside the crop box applies it (EDT-049).
      if (contains({ ...cropRect, rotation: 0 }, p)) void applyCrop();
      return;
    }
    const hit = hitObject(p);
    // A QR code: its panel, to change it (SMT-036).
    if (hit?.type === "qr" && isOn("qr")) {
      openQr(hit);
      return;
    }
    if (hit?.type === "image" && hit.separated?.text) {
      void separatedText(hit.id);
      return;
    }
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
        : tool === "eraser"
          ? "none"
          : tool === "fill" || tool === "wand"
            ? "crosshair"
            : tool === "select"
              ? hover === "pxsel" || drag?.kind === "pxmove"
                ? "move"
                : "crosshair"
              : hover === "locked"
                ? "not-allowed"
                : hover && resizeCursor[hover]
                  ? resizeCursor[hover]
                  : hover === "object" || drag?.kind === "move"
                    ? "move"
                    : tool === "text"
                      ? "text"
                      : tool === "move" || tool === "favorites"
                        ? "default"
                        : "crosshair";

  const drawing = drag?.kind === "draw" && styles ? drag : null;
  const lockAt = usePointer((s) => (hover === "locked" ? s.screen : null));
  const fillProgress = useEditor((s) => s.fillProgress);
  // Crop and Cut out show the real image, without the effects (EDT-053).
  const showFx =
    Boolean(finalView) && !drag && !textEdit && tool !== "crop" && tool !== "cutout" && !cropRect;
  // A sample of the current style by the pointer, when asked for (EDT-015).
  const previewOn = useSetting("editor.cursorStylePreview", false);
  const previewTool = (STYLED_TOOLS as readonly string[]).includes(tool) ? tool : null;
  const previewAt = usePointer((s) => (previewOn && previewTool && !drag ? s.screen : null));
  const previewStyle =
    previewTool && styles ? (styles as unknown as Record<string, unknown>)[previewTool] : null;
  const stylePreview =
    previewOn && previewTool && previewStyle
      ? sampleUrl(previewTool, "current", previewStyle)
      : null;
  // The eraser's outline follows the pointer at its real size (EDT-048).
  const eraserAt = usePointer((s) => (tool === "eraser" && !space ? s.screen : null));
  const erasing = drag?.kind === "erase" && styles ? drag : null;
  const erasePreview = erasing && styles && (
    <PenPreview
      points={erasing.points}
      color={doc.canvas.color ?? "#ffffff"}
      width={styles.eraser.width}
      opacity={styles.eraser.opacity}
      square={styles.eraser.shape === "square"}
    />
  );
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

  // An object's shadow, as Rust draws it (EDT-021). It follows a move; during other
  // gestures it waits for the new one.
  const shadowNode = (o: DocObject) => {
    const committed = doc.objects.find((x) => x.id === o.id);
    const at = committed?.shadowAt;
    const image = shadowImages.get(o.id);
    if (!at || !image) return null;
    const moving = drag?.kind === "move" && drag.ids.includes(o.id) ? drag.delta : null;
    if (o !== committed && !moving) return null;
    return (
      <KImage
        image={image}
        x={at[0] + (moving?.[0] ?? 0)}
        y={at[1] + (moving?.[1] ?? 0)}
        width={at[2]}
        height={at[3]}
        listening={false}
      />
    );
  };
  // Zones marked on screen (EDT-098, EDT-099): the one Paste fills, the one being
  // framed, where a dragged picture would go; a section's divider being dragged.
  const zoneMark = (z: ZoneObject, key: string, width: number, dashed: boolean) => {
    const [x0, y0] = toScreen([z.rect.x, z.rect.y]);
    const [x1, y1] = toScreen([z.rect.x + z.rect.w, z.rect.y + z.rect.h]);
    return (
      <Rect
        key={key}
        x={x0}
        y={y0}
        width={x1 - x0}
        height={y1 - y0}
        stroke={ACCENT}
        strokeWidth={width}
        {...(dashed ? { dash: [6, 4] } : {})}
        listening={false}
      />
    );
  };
  const zoneMarks = (() => {
    if (cropRect) return null;
    const marks: React.ReactNode[] = [];
    // Where dragged tray items would land (TRY-014).
    if (dropAt) {
      const [x, y] = toScreen(dropAt);
      if (
        dropAt[0] >= 0 &&
        dropAt[1] >= 0 &&
        dropAt[0] <= canvasSize.width &&
        dropAt[1] <= canvasSize.height
      )
        marks.push(
          <Circle
            key="drop"
            x={x}
            y={y}
            radius={18}
            stroke={ACCENT}
            strokeWidth={2}
            dash={[5, 4]}
            listening={false}
          />,
          <Line
            key="drop-h"
            points={[x - 8, y, x + 8, y]}
            stroke={ACCENT}
            strokeWidth={2}
            listening={false}
          />,
          <Line
            key="drop-v"
            points={[x, y - 8, x, y + 8]}
            stroke={ACCENT}
            strokeWidth={2}
            listening={false}
          />,
        );
    }
    const byId = (id: string | null) =>
      id ? (zonesOf(doc.objects).find((z) => z.id === id) ?? null) : null;
    const active = byId(activeZone);
    if (active && !drag) marks.push(zoneMark(active, "active", 2, false));
    const framed = byId(framing);
    if (framed) marks.push(zoneMark(framed, "framing", 2, true));
    if (drag?.kind === "zswap") {
      const target = zoneAt(doc.objects, drag.to);
      if (target && target.id !== drag.id) marks.push(zoneMark(target, "target", 3, false));
    }
    if (drag?.kind === "divider" && template) {
      const band = bands(template).find((b) => b.section.id === drag.id);
      if (band) {
        const height = Math.max(40, Math.round(drag.height + drag.delta));
        const [x0, y] = toScreen([0, band.top + height]);
        const [x1] = toScreen([canvasSize.width, 0]);
        marks.push(
          <Line
            key="divider"
            points={[x0, y, x1, y]}
            stroke={ACCENT}
            strokeWidth={2}
            dash={[6, 4]}
            listening={false}
          />,
          <DragLabel key="divider-label" x={x1 - 60} y={y - 24} text={`${height} px`} />,
        );
      }
    }
    return marks;
  })();

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
      case "simplify": {
        // A simplified shape as Rust draws it, like a stamp (EDT-041).
        const c = committed.get(o.id);
        return (
          <StampShape
            key={o.id}
            o={o}
            committed={c?.type === "simplify" ? c : o}
            image={objectImages.get(o.id)}
          />
        );
      }
      case "stamp": {
        const c = committed.get(o.id);
        return (
          <StampShape
            key={o.id}
            o={o}
            committed={c?.type === "stamp" ? c : o}
            image={objectImages.get(o.id)}
          />
        );
      }
      // A QR code as Rust draws it, at its size (SMT-036).
      case "qr": {
        const c = committed.get(o.id);
        return (
          <StampShape
            key={o.id}
            o={o}
            committed={c?.type === "qr" ? c : o}
            image={objectImages.get(o.id)}
          />
        );
      }
      case "spotlight": {
        const c = committed.get(o.id);
        return (
          <SpotlightShape
            key={o.id}
            o={o}
            committed={c?.type === "spotlight" ? c : o}
            image={objectImages.get(o.id)}
          />
        );
      }
      case "magnify": {
        const c = committed.get(o.id);
        return (
          <MagnifyShape
            key={o.id}
            o={o}
            committed={c?.type === "magnify" ? c : o}
            image={objectImages.get(o.id)}
          />
        );
      }
      case "pen": {
        const c = committed.get(o.id);
        return (
          <PenShape
            key={o.id}
            o={o}
            committed={c?.type === "pen" ? c : o}
            image={objectImages.get(o.id)}
          />
        );
      }
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
      case "zone": {
        const image = objectImages.get(o.id);
        const framed = drag?.kind === "zframe" && drag.id === o.id ? drag : null;
        return (
          <ZoneShape
            key={o.id}
            o={o}
            image={image}
            scale={scale}
            offset={framed && image ? zoneOffset(o, image, framed) : undefined}
            dim={drag?.kind === "zswap" && drag.id === o.id}
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

  const dashedBox = (
    kind: "blur" | "text" | "step" | "callout" | "magnify" | "spotlight" | "stamp" | "simplify",
  ) => {
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
      {poly && <PolygonDraft points={poly} scale={scale} />}
      {pPoly && <PolygonDraft points={pPoly.points} scale={scale} />}
      {drag?.kind === "psel" && <SelectionDraft d={drag} scale={scale} />}
      {dashedBox("step")}
      {dashedBox("callout")}
      {dashedBox("magnify")}
      {dashedBox("spotlight")}
      {dashedBox("stamp")}
      {dashedBox("simplify")}
      {erasing?.target && erasePreview}
      {drag?.kind === "pen" && styles && (
        <PenPreview
          points={drag.points}
          color={styles.pen.color}
          width={styles.pen.width}
          opacity={styles.pen.opacity}
          square={styles.pen.tip === "square"}
        />
      )}
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
  // The pixel selection's marching ants (EDT-043); while its pixels move, they go
  // with them over the image (EDT-044).
  const pixelSel = doc.pixelSelection ?? null;
  const pxMove = drag?.kind === "pxmove" ? drag : null;
  const pixelAnts = pixelSel && (
    <Group x={pxMove?.delta[0] ?? 0} y={pxMove?.delta[1] ?? 0} listening={false}>
      {pxMove?.moved && base && !pixelSel.target && (
        <Group
          clipFunc={(ctx) => {
            for (const loop of pixelSel.outline) {
              loop.forEach((pt, i) =>
                i === 0 ? ctx.moveTo(pt[0], pt[1]) : ctx.lineTo(pt[0], pt[1]),
              );
              ctx.closePath();
            }
          }}
        >
          <KImage image={base} x={baseAt.x - doc.crop.x} y={baseAt.y - doc.crop.y} />
        </Group>
      )}
      {pixelSel.outline.map((loop, i) => (
        <Group key={i}>
          <Line points={loop.flat()} closed stroke="#000000" strokeWidth={1 / scale} />
          <Line
            points={loop.flat()}
            closed
            stroke="#ffffff"
            strokeWidth={1 / scale}
            dash={[4 / scale, 4 / scale]}
            dashOffset={antPhase / scale}
          />
        </Group>
      ))}
    </Group>
  );

  return (
    <div
      className="canvas-area"
      ref={areaRef}
      style={{
        backgroundColor: workspace ?? undefined,
        // The dot grid moves with the canvas when it scrolls.
        backgroundPosition: lay
          ? `${insetLeft - lay.scrollLeft}px ${insetTop - lay.scrollTop}px`
          : undefined,
      }}
    >
      {/* Over a locked object, a small lock by the pointer (EDT-085). */}
      {stylePreview && previewAt && (
        // The current style next to the pointer (EDT-015 `editor.cursorStylePreview`).
        <img
          className="style-preview"
          src={stylePreview}
          alt=""
          aria-hidden="true"
          style={{ left: previewAt.x + 18, top: previewAt.y + 18 }}
        />
      )}
      {fillProgress !== null && (
        // A long Photo fill (EDT-045): how far it is, and Esc to stop it.
        <div className="fill-progress" role="status">
          <span>{t("ui.editor.pixel.filling")}</span>
          <progress max={1} value={fillProgress} aria-label={t("ui.editor.pixel.filling")} />
          <span className="muted">{t("ui.editor.pixel.esc_cancels")}</span>
        </div>
      )}
      {lay &&
        !cropRect &&
        zonesOf(doc.objects).map((z) => {
          const [x0, y0] = toScreen([z.rect.x, z.rect.y]);
          const [x1, y1] = toScreen([z.rect.x + z.rect.w, z.rect.y + z.rect.h]);
          if (!z.image)
            return (
              <div
                key={z.id}
                className="zone-empty"
                style={{ left: x0, top: y0, width: x1 - x0, height: y1 - y0 }}
              >
                {x1 - x0 > 160 && y1 - y0 > 80 && <span>{t("ui.editor.template.drop_here")}</span>}
                <button
                  type="button"
                  className="btn small"
                  onClick={() => void captureIntoZone(z.id)}
                >
                  <Icon name="capture" />
                  {t("ui.editor.template.capture")}
                </button>
              </div>
            );
          if (framing !== z.id) return null;
          const frame = (params: Record<string, unknown>) =>
            void templateCommand("frame_zone", { zone: z.id, ...params });
          return (
            <div
              key={z.id}
              className="zone-frame-bar"
              role="toolbar"
              aria-label={t("ui.editor.template.frame")}
              style={{ left: Math.max(4, x0), top: Math.max(4, y0 - 44) }}
            >
              <button
                type="button"
                className="btn icon small"
                aria-label={t("ui.editor.template.zoom_out")}
                title={t("ui.editor.template.zoom_out")}
                onClick={() => frame({ zoomBy: -1 })}
              >
                <Icon name="minus" />
              </button>
              <span className="zone-zoom">{Math.round((z.zoom ?? 1) * 100)} %</span>
              <button
                type="button"
                className="btn icon small"
                aria-label={t("ui.editor.template.zoom_in")}
                title={t("ui.editor.template.zoom_in")}
                onClick={() => frame({ zoomBy: 1 })}
              >
                <Icon name="plus" />
              </button>
              <button type="button" className="btn small" onClick={() => frame({ reset: true })}>
                {t("ui.editor.template.reset_frame")}
              </button>
              <button
                type="button"
                className="btn small primary"
                onClick={() => useZones.setState({ framing: null })}
              >
                {t("ui.editor.template.done")}
              </button>
            </div>
          );
        })}
      {(() => {
        // The quick edit bar by the one selected object, not while it changes (EDT-006).
        if (!quickOn || drag || textEdit || !lay || !viewport || selection.length !== 1)
          return null;
        const o = doc.objects.find((x) => x.id === selection[0]);
        if (!o || o.locked || !(QUICK_KINDS as readonly string[]).includes(o.type)) return null;
        const b = boundsOf([o]);
        if (!b) return null;
        const [x0, y0] = toScreen([b.x, b.y]);
        const [x1, y1] = toScreen([b.x + b.w, b.y + b.h]);
        return (
          <QuickBar
            key={o.id}
            object={o}
            rect={{ x: x0, y: y0, w: x1 - x0, h: y1 - y0 }}
            areaWidth={insetLeft + viewport.width}
            areaTop={insetTop}
          />
        );
      })()}
      {hover === "locked" && lockAt && (
        <span
          className="lock-badge"
          style={{ left: lockAt.x + 14, top: lockAt.y + 14 }}
          aria-hidden="true"
        >
          <Icon name="lock" />
        </span>
      )}
      <div
        className="canvas-scroll"
        ref={scrollRef}
        onScroll={onScroll}
        style={{ top: insetTop, right: insetRight, bottom: insetBottom, left: insetLeft }}
      >
        {lay && <div style={{ width: lay.contentWidth, height: lay.contentHeight }} />}
      </div>
      {lay && viewport && area && (
        <CanvasBars
          scroller={scrollRef}
          content={{ width: lay.contentWidth, height: lay.contentHeight }}
          visible={viewport}
          scroll={{ left: lay.scrollLeft, top: lay.scrollTop }}
          insets={insets}
        />
      )}
      {lay && viewport && area && (
        <div
          className="canvas-stage"
          ref={stageRef}
          style={{ width: area.width, height: area.height, cursor }}
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
          <Stage width={area.width} height={area.height} listening={false}>
            <Layer imageSmoothingEnabled={view !== null && view.zoom < 100}>
              <Group
                x={insetLeft + lay.originX - lay.scrollLeft - ext.x * scale}
                y={insetTop + lay.originY - lay.scrollTop - ext.y * scale}
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
                  visible={!showFx}
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
                    visible={!showFx}
                  />
                  {doc.canvas.color && !showFx && (
                    <Rect
                      width={canvasSize.width}
                      height={canvasSize.height}
                      fill={doc.canvas.color}
                    />
                  )}
                  {/* Simplify: a plain background, the original dimmed or hidden (EDT-041). */}
                  {doc.simplify && !doc.canvas.color && (
                    <Rect width={canvasSize.width} height={canvasSize.height} fill="#ffffff" />
                  )}
                  {base && !showFx && (
                    <KImage
                      image={base}
                      x={baseAt.x - doc.crop.x}
                      y={baseAt.y - doc.crop.y}
                      opacity={
                        doc.simplify
                          ? doc.simplify.showOriginal
                            ? doc.simplify.opacity / 100
                            : 0
                          : 1
                      }
                    />
                  )}
                  {/* Erasing the image: under the objects, which stay (EDT-048). */}
                  {erasing && !erasing.target && erasePreview}
                </Group>
                {/* Objects are cut at the edge, except the one being drawn or dragged:
                    it shows whole and the canvas grows to fit it when the gesture ends
                    (EDT-071). Each is cut on its own to keep their order. */}
                {showFx && finalView && (
                  <KImage
                    image={finalView.image}
                    x={-finalView.origin[0]}
                    y={-finalView.origin[1]}
                    listening={false}
                  />
                )}
                {!showFx &&
                  shown.map((o) =>
                    free.has(o.id) ? (
                      <Group key={o.id}>
                        {shadowNode(o)}
                        {objectNode(o)}
                      </Group>
                    ) : (
                      <Group key={o.id} {...clip}>
                        {shadowNode(o)}
                        {objectNode(o)}
                      </Group>
                    ),
                  )}
                {autoExpand ? previews : <Group {...clip}>{previews}</Group>}
                <Group {...clip}>{blurPreview}</Group>
                {pixelAnts}
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
              {zoneMarks}
              {eraserAt && styles && (
                <EraserOutline
                  at={eraserAt}
                  size={styles.eraser.width * scale}
                  square={styles.eraser.shape === "square"}
                />
              )}
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
              {gridOn &&
                lay &&
                viewport &&
                view &&
                view.zoom >= gridFrom &&
                (() => {
                  // One physical pixel between the pixels of the image, inverting what is
                  // below so it shows on any color; never part of the image (EDT-008).
                  const [x0, y0] = toCanvas({ x: 0, y: 0 });
                  const [x1, y1] = toCanvas({
                    x: insetLeft + viewport.width + insetRight,
                    y: insetTop + viewport.height + insetBottom,
                  });
                  const left = Math.max(0, Math.floor(x0));
                  const top = Math.max(0, Math.floor(y0));
                  const right = Math.min(canvasSize.width, Math.ceil(x1));
                  const bottom = Math.min(canvasSize.height, Math.ceil(y1));
                  if (right <= left || bottom <= top) return null;
                  const [sx0, sy0] = toScreen([left, top]);
                  const [sx1, sy1] = toScreen([right, bottom]);
                  return (
                    <Shape
                      listening={false}
                      globalCompositeOperation="difference"
                      sceneFunc={(ctx) => {
                        const c = ctx._context;
                        c.beginPath();
                        for (let x = left; x <= right; x++) {
                          const sx = toScreen([x, 0])[0];
                          c.moveTo(sx, sy0);
                          c.lineTo(sx, sy1);
                        }
                        for (let y = top; y <= bottom; y++) {
                          const sy = toScreen([0, y])[1];
                          c.moveTo(sx0, sy);
                          c.lineTo(sx1, sy);
                        }
                        c.lineWidth = 1 / dpr;
                        c.strokeStyle = "rgba(255, 255, 255, 0.45)";
                        c.stroke();
                      }}
                    />
                  );
                })()}
              {canvasEdit &&
                lay &&
                (() => {
                  // The canvas's handles and rhombus (EDT-070, EDT-075).
                  const box = drag?.kind === "canvas" ? drag.box : canvasBox;
                  const r = drag?.kind === "cradius" ? drag.value : doc.canvas.cornerRadius;
                  const [rx, ry] = toScreen(cornerPoint(r));
                  const [x0, y0] = toScreen([box.x, box.y]);
                  return (
                    <Group listening={false}>
                      {drag?.kind === "canvas" && (
                        <Line
                          points={corners(box).flatMap(toScreen)}
                          closed
                          stroke={ACCENT}
                          strokeWidth={1.5}
                          dash={[5, 4]}
                        />
                      )}
                      {drag?.kind === "cradius" && (
                        <Rect
                          x={x0}
                          y={y0}
                          width={box.w * lay.scale}
                          height={box.h * lay.scale}
                          cornerRadius={r * lay.scale}
                          stroke={ACCENT}
                          strokeWidth={1.5}
                          dash={[5, 4]}
                        />
                      )}
                      {drag?.kind !== "cradius" &&
                        RESIZE_HANDLES.map((h) => {
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
                              fill="#ffffff"
                              stroke={ACCENT}
                              strokeWidth={1.5}
                            />
                          );
                        })}
                      {drag?.kind !== "canvas" && (
                        <Rect
                          x={rx}
                          y={ry}
                          width={HANDLE - 2}
                          height={HANDLE - 2}
                          offsetX={(HANDLE - 2) / 2}
                          offsetY={(HANDLE - 2) / 2}
                          rotation={45}
                          fill={r > 0 ? ACCENT : "#ffffff"}
                          stroke={r > 0 ? "#ffffff" : ACCENT}
                          strokeWidth={1.5}
                        />
                      )}
                      {drag?.kind === "canvas" && (
                        <DragLabel
                          {...(() => {
                            const [x, y] = toScreen(handlePoint(box, drag.handle, 0));
                            return { x, y };
                          })()}
                          text={`${Math.round(box.w)} × ${Math.round(box.h)} px`}
                        />
                      )}
                      {drag?.kind === "cradius" && <DragLabel x={rx} y={ry} text={`${r} px`} />}
                    </Group>
                  );
                })()}
              {(drag?.kind === "move" || drag?.kind === "resize" || drag?.kind === "mresize") &&
                (() => {
                  // Snapping guides and equal spaces (EDT-084).
                  const width = highContrast() ? 2 : 1;
                  const guides = drag.guides ?? [];
                  const spacings = drag.kind === "move" ? (drag.spacings ?? []) : [];
                  return (
                    <Group listening={false}>
                      {guides.map((g, i) => (
                        <Line
                          key={`g${i}`}
                          points={
                            g.axis === "x"
                              ? [...toScreen([g.at, g.from]), ...toScreen([g.at, g.to])]
                              : [...toScreen([g.from, g.at]), ...toScreen([g.to, g.at])]
                          }
                          stroke={ACCENT}
                          strokeWidth={width}
                        />
                      ))}
                      {spacings.map((s, i) => {
                        const a =
                          s.axis === "x" ? toScreen([s.a, s.cross]) : toScreen([s.cross, s.a]);
                        const b =
                          s.axis === "x" ? toScreen([s.b, s.cross]) : toScreen([s.cross, s.b]);
                        const label = `${Math.round(Math.abs(s.b - s.a))}`;
                        return (
                          <Group key={`s${i}`}>
                            <Line
                              points={[...a, ...b]}
                              stroke={ACCENT}
                              strokeWidth={width}
                              dash={[3, 2]}
                            />
                            <DragLabel
                              x={(a[0] + b[0]) / 2 - 20}
                              y={(a[1] + b[1]) / 2 - 24}
                              text={label}
                            />
                          </Group>
                        );
                      })}
                    </Group>
                  );
                })()}
              {drag?.kind === "cut" &&
                styles &&
                (() => {
                  // The strip being marked, across the whole image, and its size (EDT-052).
                  const axis = drag.vertical ? 0 : 1;
                  const a = Math.round(Math.min(drag.start[axis], drag.end[axis]));
                  const b = Math.round(Math.max(drag.start[axis], drag.end[axis]));
                  const box: RectBox = drag.vertical
                    ? { x: a, y: 0, w: b - a, h: canvasSize.height }
                    : { x: 0, y: a, w: canvasSize.width, h: b - a };
                  const adding = styles.cutout.mode === "add";
                  const [lx, ly] = toScreen(drag.end);
                  return (
                    <Group listening={false}>
                      <Line
                        points={corners({ ...box, rotation: 0 }).flatMap(toScreen)}
                        closed
                        stroke={adding ? ACCENT : "#FC1B39"}
                        strokeWidth={1.5}
                        dash={[6, 4]}
                        fill={adding ? "rgba(0, 100, 251, 0.16)" : "rgba(252, 27, 57, 0.22)"}
                      />
                      <DragLabel x={lx} y={ly} text={`${b - a} px`} />
                    </Group>
                  );
                })()}
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
                    // A stroke: a dashed box around it, to show it is selected (EDT-037).
                    if (o.type === "pen") {
                      const all = boundsOf([o]);
                      if (!all) return null;
                      const pad = o.style.width / 2 + 2 / scale;
                      return (
                        <Line
                          key={o.id}
                          points={corners({
                            x: all.x - pad,
                            y: all.y - pad,
                            w: all.w + 2 * pad,
                            h: all.h + 2 * pad,
                            rotation: 0,
                          }).flatMap(toScreen)}
                          closed
                          stroke={ACCENT}
                          strokeWidth={1}
                          dash={[4, 3]}
                        />
                      );
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
                        {o.type === "shape" && shapeHandles(o, box)}
                        {o.type === "image" &&
                          !o.locked &&
                          (() => {
                            // Its rounded corners' rhombus (EDT-075).
                            const r =
                              drag?.kind === "radius" && drag.id === o.id
                                ? drag.value
                                : (o.style.cornerRadius ?? 0);
                            const [x, y] = toScreen(radiusPoint(box, r));
                            return (
                              <Rect
                                x={x}
                                y={y}
                                width={HANDLE - 2}
                                height={HANDLE - 2}
                                offsetX={(HANDLE - 2) / 2}
                                offsetY={(HANDLE - 2) / 2}
                                rotation={45 + box.rotation}
                                fill={r > 0 ? ACCENT : "#ffffff"}
                                stroke={r > 0 ? "#ffffff" : ACCENT}
                                strokeWidth={1.5}
                              />
                            );
                          })()}
                        {o.type === "magnify" &&
                          lensAway(o) &&
                          (() => {
                            // The zone: a dashed outline and its own handle (EDT-038).
                            const z = o.source;
                            const [zx, zy] = toScreen([z.x + z.w / 2, z.y + z.h / 2]);
                            return (
                              <>
                                <Line
                                  points={corners({ ...z, rotation: 0 }).flatMap(toScreen)}
                                  closed
                                  stroke={ACCENT}
                                  strokeWidth={1}
                                  dash={[4, 3]}
                                />
                                <Rect
                                  x={zx}
                                  y={zy}
                                  width={HANDLE}
                                  height={HANDLE}
                                  offsetX={HANDLE / 2}
                                  offsetY={HANDLE / 2}
                                  fill={ACCENT}
                                  stroke="#ffffff"
                                  strokeWidth={1.5}
                                />
                              </>
                            );
                          })()}
                      </Group>
                    );
                  })}
            </Layer>
          </Stage>
          {/* Auto redact's panel is open: the words read can be clicked (SMT-014). */}
          <RedactOverlay
            documentId={doc.documentId}
            toScreen={toScreen}
            scale={scale}
            insets={{ top: insetTop, right: insetRight, bottom: insetBottom, left: insetLeft }}
          />
        </div>
      )}
      {/* The keyboard focus ring, around the part left visible: the edges of the canvas
          are under the floating bars. */}
      <div
        className="canvas-focus"
        aria-hidden="true"
        style={{ top: insetTop, right: insetRight, bottom: insetBottom, left: insetLeft }}
      />
      {textEdit && lay && (
        <TextEditor edit={textEdit} toScreen={toScreen} scale={scale} dpi={doc.dpi} />
      )}
    </div>
  );
}

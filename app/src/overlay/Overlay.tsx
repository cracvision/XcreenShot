// On-screen selection over the frozen screen (CAP-009, CAP-011, CAP-013, CAP-015,
// CAP-016, CAP-017, CAP-018, CAP-021, CAP-025, CAP-027). One instance per monitor
// window. Everything is computed in physical desktop pixels; the canvases use the
// monitor's physical size so the frozen image and the pixel grid are exact.
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { QuickBar } from "./QuickAnnotate";
import { closes, ellipsePoints, extend, SCISSORS, type FreehandShape } from "./freehand";
import {
  COLORS,
  drawMarks,
  firstColor,
  hitMark,
  markBounds,
  markFromDrag,
  moveMark,
  toObjects,
  TOOLS,
  type Mark,
  type Tool,
} from "./quickMarks";
import { setLanguage } from "../i18n";
import { tenStep } from "../lib/numberKeys";
import {
  bounding,
  clampInside,
  contains,
  fromPoints,
  intersect,
  lockAspect,
  nextZoom,
  placeBar,
  placeMagnifier,
  ratioFromKeys,
  snapToGuides,
  GUIDE_SIZES,
  resizeByHandle,
  toHex,
  type Handle,
  type Pt,
  type Rect,
} from "./geometry";

type Mode = "region" | "window" | "universal" | "scrolling" | "multi" | "menu";
type Phase = "aiming" | "dragging" | "adjusting";

interface Monitor {
  id: number;
  name: string;
  rect: Rect;
  scale: number;
  primary: boolean;
}

interface Win {
  id: number;
  rect: Rect;
  title: string;
  app: string;
}

interface Start {
  seq: number;
  index: number;
  lang: "en" | "es";
  nativeBackground: boolean;
  request: {
    mode: Mode;
    monitors: Monitor[];
    windows: Win[];
    pointer: [number, number] | null;
    autoDetect: boolean;
    showMagnifier: boolean;
    magnifierZoom: number;
    /** Universal with Selection = Window: windows only, no dragging (CAP-010). */
    windowsOnly?: boolean;
    /** Universal with Selection = Full screen: start adjusting a whole monitor (CAP-010). */
    startFullscreen?: boolean;
    /** Thin lines, no dimming, no label (CAP-014). */
    simpleCrosshair?: boolean;
    /** The compatibility profile's keys on Windows (CAP-066). */
    compatibility?: boolean;
    /** Show "H: help" in the hint, the first times (CAP-020). */
    helpHint?: boolean;
    /** Region with a fixed size (CAP-026): a rectangle of this size follows the pointer. */
    fixedSize?: { width: number; height: number };
    /** Menu capture (CAP-031): the open menus found, outermost first. */
    menus?: Rect[];
    /** Quick annotation (CAP-057): after selecting, mark the area here and copy it. */
    quickAnnotate?: boolean;
    /** Freehand (CAP-032): select this shape instead of a rectangle. */
    freehand?: FreehandShape;
    /** Its first tool: Arrow, or the last one used (CAP-057). */
    quickTool?: Tool;
    /** The active theme's colors for the marks (CAP-057, EDT-017). */
    quickColors?: string[];
  };
}

/** Smallest area a scrolling capture accepts (CAP-034), in physical pixels. */
const SCROLL_MIN = 50;

/** Scrolling is offered from Universal and Region (CAP-034, CAP-035). */
const scrollable = (start: Start, sel: Rect | null) =>
  start.request.mode !== "window" &&
  sel !== null &&
  sel.width >= SCROLL_MIN &&
  sel.height >= SCROLL_MIN;

/** The fixed-size rectangle around `p` (CAP-026), inside the monitor under it. */
function fixedRect(start: Start, p: Pt): Rect | null {
  const size = start.request.fixedSize;
  if (!size) return null;
  const m =
    start.request.monitors.find(
      (mon) =>
        p.x >= mon.rect.x &&
        p.x < mon.rect.x + mon.rect.width &&
        p.y >= mon.rect.y &&
        p.y < mon.rect.y + mon.rect.height,
    ) ?? start.request.monitors[0];
  if (!m) return null;
  const width = Math.min(size.width, m.rect.width);
  const height = Math.min(size.height, m.rect.height);
  const x = Math.round(p.x - width / 2);
  const y = Math.round(p.y - height / 2);
  return {
    x: Math.min(Math.max(x, m.rect.x), m.rect.x + m.rect.width - width),
    y: Math.min(Math.max(y, m.rect.y), m.rect.y + m.rect.height - height),
    width,
    height,
  };
}

/** Only windows can be picked: window capture, or Universal set to Window. */
const windowsOnly = (start: Start) =>
  start.request.mode === "window" || start.request.windowsOnly === true;

interface Sync {
  seq: number;
  from: number;
  pointer: Pt | null;
  sel: Rect | null;
  phase: Phase;
  /** Size guides and where the drag began (CAP-019). */
  guides: boolean;
  anchor: Pt | null;
  snapped: [number, number] | null;
  /** The keys panel (CAP-020). */
  help: boolean;
  /** Multiple areas (CAP-033): on, and the areas so far. */
  multi: boolean;
  areas: Rect[];
  /** Quick annotation's marks (CAP-057). */
  marks: Mark[];
  /** Freehand's shape so far (CAP-032). */
  shape: Pt[];
}

type Interaction =
  | { kind: "draw"; anchor: Pt }
  | { kind: "move"; start: Pt; orig: Rect }
  | { kind: "resize"; handle: Handle; orig: Rect }
  | null;

/** The selection over any screen: the logo's bright blue. */
const ACCENT = "#0c9afc";
/** A selection snapped to a standard size (CAP-019): the logo's star. */
const SNAPPED = "#ffd23f";
const GUIDE = "rgba(255, 255, 255, 0.55)";
const OUTLINE = "rgba(0, 0, 0, 0.65)";
const DIM = "rgba(0, 0, 0, 0.4)";
const MAG_CSS = 168; // magnifier box, CSS px
const HANDLES: Handle[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];
const DRAG_THRESHOLD = 3; // physical px before a press becomes a drag

const myIndex = Number(getCurrentWebviewWindow().label.replace(/^overlay-/, "")) || 0;
const frameUrl = (seq: number, index: number) =>
  navigator.userAgent.includes("Windows")
    ? `http://xsframe.localhost/${seq}/${index}`
    : `xsframe://localhost/${seq}/${index}`;

function handlePoint(r: Rect, h: Handle): Pt {
  const cx = r.x + r.width / 2;
  const cy = r.y + r.height / 2;
  const x = h.includes("w") ? r.x : h.includes("e") ? r.x + r.width : cx;
  const y = h.includes("n") ? r.y : h.includes("s") ? r.y + r.height : cy;
  return { x, y };
}

export function Overlay() {
  const { t } = useTranslation();
  const [start, setStart] = useState<Start | null>(null);
  const [phase, setPhase] = useState<Phase>("aiming");
  const [sel, setSel] = useState<Rect | null>(null);
  const [clamped, setClamped] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const bg = useRef<HTMLCanvasElement>(null);
  const fg = useRef<HTMLCanvasElement>(null);
  const mag = useRef<HTMLCanvasElement>(null);
  const patchCanvas = useRef<HTMLCanvasElement>(document.createElement("canvas"));
  const drawRef = useRef<() => void>(() => undefined);
  const s = useRef({
    pointer: null as Pt | null,
    hover: null as Win | null,
    sel: null as Rect | null,
    phase: "aiming" as Phase,
    interaction: null as Interaction,
    magnifier: true,
    zoom: 4,
    shift: false,
    alt: false,
    lastLock: null as number | null,
    raf: 0,
    patch: null as { x: number; y: number; w: number; h: number } | null,
    patchBusy: false,
    color: "",
    finished: false,
    guides: false,
    anchor: null as Pt | null,
    snapped: null as [number, number] | null,
    help: false,
    // Frame timing, to notice a slow crosshair (CAP-014, APP-053).
    asked: 0,
    lag: { sum: 0, count: 0, since: 0 },
    slow: false,
    // Multiple areas (CAP-033): on from the start (Selection = Multiple areas) or by
    // holding Ctrl while dragging, which also ends it when Ctrl is let go.
    multi: false,
    ctrlMulti: false,
    areas: [] as Rect[],
    // Quick annotation (CAP-057): the marks, the ones undone, the one being drawn.
    marks: [] as Mark[],
    undone: [] as Mark[],
    drawing: null as { from: Pt; to: Pt } | null,
    textAt: null as Pt | null,
    // The mark chosen, and where its drag is (CAP-057).
    picked: -1,
    pickedFrom: null as Pt | null,
    // Freehand (CAP-032): the shape so far, a free outline being drawn, an ellipse's
    // first corner, and the next polygon corner under the pointer.
    shape: [] as Pt[],
    freeDrawing: false,
    ellipseFrom: null as Pt | null,
    polyNext: null as Pt | null,
    typing: null as { at: Pt; text: string } | null,
    markTool: "arrow" as Tool,
    markColor: COLORS[0] as string,
    markWidth: 4,
    markEllipse: false,
    discardAsked: false,
  });
  const [help, setHelp] = useState(false);
  const [areas, setAreas] = useState<Rect[]>([]);
  // Quick annotation (CAP-057): what the bar shows, and a text being typed.
  const [quick, setQuick] = useState({
    tool: "arrow" as Tool,
    colors: COLORS,
    color: COLORS[0] as string,
    width: 4,
    ellipse: false,
    marks: 0,
    undone: 0,
  });
  const [typing, setTyping] = useState<{ at: Pt; text: string } | null>(null);
  /** The text being typed, also where the handlers can read it at once. */
  const showText = useCallback((v: { at: Pt; text: string } | null) => {
    s.current.typing = v;
    setTyping(v);
  }, []);

  const monitor = start?.request.monitors[myIndex] ?? null;
  const desktop = start ? bounding(start.request.monitors.map((m) => m.rect)) : null;

  const toLocal = useCallback(
    (p: Pt): Pt => (monitor ? { x: p.x - monitor.rect.x, y: p.y - monitor.rect.y } : p),
    [monitor],
  );
  const cssPerPx = monitor ? window.innerWidth / monitor.rect.width : 1;

  // ---------------------------------------------------------------- drawing
  /** Pixels around the pointer, from the shell (native background) or the frame canvas. */
  const requestPatch = useCallback(
    (l: Pt, cells: number) => {
      const st = s.current;
      if (!start || st.patchBusy) return;
      const half = Math.floor(cells / 2);
      const want = { x: l.x - half, y: l.y - half, w: cells, h: cells };
      const have = st.patch;
      if (have && have.x === want.x && have.y === want.y && have.w === want.w) return;
      st.patchBusy = true;
      void invoke<ArrayBuffer>("xs_frame_patch", { seq: start.seq, index: myIndex, ...want })
        .then((buf) => {
          const c = patchCanvas.current;
          c.width = want.w;
          c.height = want.h;
          c.getContext("2d")?.putImageData(
            new ImageData(new Uint8ClampedArray(buf), want.w, want.h),
            0,
            0,
          );
          st.patch = want;
        })
        .finally(() => {
          st.patchBusy = false;
          if (!st.raf) st.raf = requestAnimationFrame(() => drawRef.current());
        });
    },
    [start],
  );

  const drawMagnifier = useCallback(() => {
    const st = s.current;
    const canvas = mag.current;
    if (!canvas || !monitor || !start) return;
    const p = st.pointer;
    const visible = st.magnifier && p && st.phase !== "adjusting" && contains(monitor.rect, p);
    canvas.style.display = visible ? "block" : "none";
    if (!visible || !p) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const size = Math.round(MAG_CSS * dpr);
    if (canvas.width !== size) {
      canvas.width = size;
      canvas.height = size;
    }
    const cells = Math.max(3, Math.floor(size / st.zoom) | 1); // odd, centered
    const cell = size / cells;
    const l = toLocal(p);
    const half = Math.floor(cells / 2);
    ctx.imageSmoothingEnabled = false;
    ctx.fillStyle = "#000";
    ctx.fillRect(0, 0, size, size);
    let color = "";
    if (start.nativeBackground) {
      requestPatch(l, cells);
      const patch = st.patch;
      if (patch) {
        ctx.drawImage(
          patchCanvas.current,
          l.x - half - patch.x,
          l.y - half - patch.y,
          cells,
          cells,
          0,
          0,
          size,
          size,
        );
        color = pixelHex(patchCanvas.current, { x: l.x - patch.x, y: l.y - patch.y });
      }
    } else if (bg.current) {
      ctx.drawImage(bg.current, l.x - half, l.y - half, cells, cells, 0, 0, size, size);
      color = pixelHex(bg.current, l);
    }
    st.color = color;
    if (cell >= 6) {
      ctx.strokeStyle = "rgba(255,255,255,0.12)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (let i = 1; i < cells; i++) {
        ctx.moveTo(Math.round(i * cell) + 0.5, 0);
        ctx.lineTo(Math.round(i * cell) + 0.5, size);
        ctx.moveTo(0, Math.round(i * cell) + 0.5);
        ctx.lineTo(size, Math.round(i * cell) + 0.5);
      }
      ctx.stroke();
    }
    ctx.strokeStyle = ACCENT;
    ctx.lineWidth = Math.max(2, dpr);
    ctx.strokeRect(half * cell, half * cell, cell, cell);
    // Position: CSS px inside this window.
    const css = placeMagnifier(
      { x: l.x * cssPerPx, y: l.y * cssPerPx },
      { width: MAG_CSS, height: MAG_CSS + 40 },
      { x: 0, y: 0, width: window.innerWidth, height: window.innerHeight },
    );
    const box = canvas.parentElement;
    if (box) {
      box.style.left = `${css.x}px`;
      box.style.top = `${css.y}px`;
      const info = box.querySelector<HTMLElement>("[data-info]");
      if (info) info.textContent = `${p.x}, ${p.y}  ·  ${color}  ·  ×${st.zoom}`;
    }
  }, [monitor, start, toLocal, cssPerPx, requestPatch]);

  const draw = useCallback(() => {
    const st = s.current;
    st.raf = 0;
    const canvas = fg.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx || !monitor || !start) return;
    const M = monitor.rect;
    const px = Math.max(1, Math.round(monitor.scale));
    const simple = start.request.simpleCrosshair === true;
    trackLag(st);
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Dim everything except the selection or the detected window (CAP-013, CAP-016).
    // The simplified crosshair dims nothing and draws thin lines (CAP-014).
    const fixed = st.phase === "aiming" && st.pointer ? fixedRect(start, st.pointer) : null;
    const focus =
      st.sel ??
      fixed ??
      (st.phase === "aiming" && !start.request.freehand ? (st.hover?.rect ?? null) : null);
    if (!simple) {
      ctx.fillStyle = DIM;
      ctx.fillRect(0, 0, canvas.width, canvas.height);
    }
    // Menu capture (CAP-031): the menus found, clear and outlined, while aiming.
    if (st.phase === "aiming" && start.request.menus) {
      for (const menu of start.request.menus) {
        const a = intersect(menu, M);
        if (!a) continue;
        if (!simple) ctx.clearRect(a.x - M.x, a.y - M.y, a.width, a.height);
        ctx.lineWidth = 2 * px;
        ctx.setLineDash([]);
        ctx.strokeStyle = ACCENT;
        ctx.strokeRect(a.x - M.x, a.y - M.y, a.width, a.height);
      }
    }
    // Multiple areas (CAP-033): each one clear, outlined and numbered.
    st.areas.forEach((area, i) => {
      const a = intersect(area, M);
      if (!a) return;
      if (!simple) ctx.clearRect(a.x - M.x, a.y - M.y, a.width, a.height);
      ctx.lineWidth = 2 * px;
      ctx.setLineDash([]);
      ctx.strokeStyle = ACCENT;
      ctx.strokeRect(a.x - M.x, a.y - M.y, a.width, a.height);
      const r = 12 * px;
      const cx = area.x - M.x + r + 4 * px;
      const cy = area.y - M.y + r + 4 * px;
      ctx.fillStyle = ACCENT;
      ctx.beginPath();
      ctx.arc(cx, cy, r, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = "#ffffff";
      ctx.font = `600 ${13 * px}px "Segoe UI", system-ui, sans-serif`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(String(i + 1), cx, cy + px);
      ctx.textAlign = "start";
      ctx.textBaseline = "alphabetic";
    });
    const f = focus ? intersect(focus, M) : null;
    if (f) {
      if (!simple) ctx.clearRect(f.x - M.x, f.y - M.y, f.width, f.height);
      ctx.lineWidth = (simple ? 1 : 2) * px;
      ctx.setLineDash(st.sel ? [] : [6 * px, 4 * px]);
      if (!simple) {
        ctx.strokeStyle = OUTLINE;
        ctx.strokeRect(f.x - M.x - px, f.y - M.y - px, f.width + 2 * px, f.height + 2 * px);
      }
      ctx.strokeStyle = st.snapped && st.sel ? SNAPPED : ACCENT;
      ctx.strokeRect(f.x - M.x, f.y - M.y, f.width, f.height);
      ctx.setLineDash([]);
    }

    // Standard sizes from where the drag began, toward the pointer (CAP-019).
    if (st.guides && st.anchor && st.phase === "dragging" && st.pointer) {
      const a = st.anchor;
      const sx = st.pointer.x < a.x ? -1 : 1;
      const sy = st.pointer.y < a.y ? -1 : 1;
      ctx.lineWidth = px;
      ctx.setLineDash([4 * px, 4 * px]);
      ctx.font = `${11 * px}px "Segoe UI", system-ui, sans-serif`;
      ctx.textBaseline = "middle";
      for (const [w, h] of GUIDE_SIZES) {
        const r = fromPoints(a, { x: a.x + sx * w, y: a.y + sy * h });
        const g = intersect(r, M);
        if (!g) continue;
        const exact = st.snapped?.[0] === w && st.snapped?.[1] === h;
        ctx.strokeStyle = exact ? SNAPPED : GUIDE;
        ctx.strokeRect(r.x - M.x, r.y - M.y, r.width, r.height);
        const corner = { x: a.x + sx * w, y: a.y + sy * h };
        if (contains(M, corner)) {
          ctx.fillStyle = exact ? SNAPPED : GUIDE;
          ctx.fillText(
            `${w}×${h}`,
            corner.x - M.x + (sx > 0 ? -60 * px : 6 * px),
            corner.y - M.y + (sy > 0 ? -8 * px : 8 * px),
          );
        }
      }
      ctx.setLineDash([]);
    }

    // Handles while adjusting (CAP-011).
    if (st.sel && st.phase === "adjusting") {
      const size = 8 * px;
      for (const h of HANDLES) {
        const p = handlePoint(st.sel, h);
        if (
          !contains(M, {
            x: Math.min(p.x, M.x + M.width - 1),
            y: Math.min(p.y, M.y + M.height - 1),
          })
        )
          continue;
        ctx.fillStyle = "#ffffff";
        ctx.strokeStyle = ACCENT;
        ctx.lineWidth = 2 * px;
        ctx.fillRect(p.x - M.x - size / 2, p.y - M.y - size / 2, size, size);
        ctx.strokeRect(p.x - M.x - size / 2, p.y - M.y - size / 2, size, size);
      }
    }

    // Quick annotation's marks, inside the selection (CAP-057).
    if (start.request.quickAnnotate && st.sel && (st.marks.length || st.drawing)) {
      const scale = Math.max(1, monitor.scale);
      const live =
        st.drawing &&
        markFromDrag(
          st.markTool,
          st.drawing.from,
          st.drawing.to,
          st.markColor,
          st.markWidth * scale,
          st.markEllipse,
        );
      ctx.save();
      ctx.beginPath();
      ctx.rect(st.sel.x - M.x, st.sel.y - M.y, st.sel.width, st.sel.height);
      ctx.clip();
      drawMarks(ctx, live ? [...st.marks, live] : st.marks, M);
      // The chosen mark: a dashed box around it.
      const chosen = st.marks[st.picked];
      if (chosen) {
        const b = markBounds(chosen);
        const pad = 4 * scale;
        ctx.lineWidth = 1.5 * scale;
        ctx.setLineDash([6 * scale, 4 * scale]);
        ctx.strokeStyle = ACCENT;
        ctx.strokeRect(b.x - M.x - pad, b.y - M.y - pad, b.width + 2 * pad, b.height + 2 * pad);
        ctx.setLineDash([]);
      }
      ctx.restore();
    }

    // Freehand (CAP-032): the shape clear and outlined; a polygon shows its next side.
    if (start.request.freehand && st.shape.length >= 2) {
      const pts = st.shape.map((q) => ({ x: q.x - M.x, y: q.y - M.y }));
      const next = start.request.freehand === "polygon" && st.polyNext;
      ctx.save();
      ctx.beginPath();
      pts.forEach((q, i) => (i ? ctx.lineTo(q.x, q.y) : ctx.moveTo(q.x, q.y)));
      if (next) ctx.lineTo(next.x - M.x, next.y - M.y);
      ctx.closePath();
      if (!simple) {
        ctx.globalCompositeOperation = "destination-out";
        ctx.fillStyle = "#000";
        ctx.fill();
        ctx.globalCompositeOperation = "source-over";
      }
      ctx.lineWidth = 2 * px;
      ctx.setLineDash([]);
      ctx.strokeStyle = ACCENT;
      ctx.stroke();
      ctx.restore();
    } else if (start.request.freehand === "polygon" && st.shape.length === 1 && st.polyNext) {
      const a = st.shape[0] as Pt;
      ctx.lineWidth = 2 * px;
      ctx.strokeStyle = ACCENT;
      ctx.beginPath();
      ctx.moveTo(a.x - M.x, a.y - M.y);
      ctx.lineTo(st.polyNext.x - M.x, st.polyNext.y - M.y);
      ctx.stroke();
    }

    // Crosshair across the monitor, with an outline for any background (CAP-013).
    const p = st.pointer;
    if (p && st.phase !== "adjusting" && contains(M, p)) {
      const l = toLocal(p);
      if (!simple) {
        ctx.fillStyle = OUTLINE;
        ctx.fillRect(0, l.y - px, canvas.width, 3 * px);
        ctx.fillRect(l.x - px, 0, 3 * px, canvas.height);
      }
      ctx.fillStyle = ACCENT;
      ctx.fillRect(0, l.y, canvas.width, simple ? 1 : px);
      ctx.fillRect(l.x, 0, simple ? 1 : px, canvas.height);
      if (simple) {
        drawMagnifier();
        return;
      }

      // Position or size label (physical pixels).
      const label =
        st.sel && st.phase === "dragging"
          ? `${st.sel.width} × ${st.sel.height}${st.lastLock ? lockName(st.lastLock) : ""}`
          : `${p.x}, ${p.y}`;
      ctx.font = `${12 * px}px "Segoe UI", system-ui, sans-serif`;
      const w = ctx.measureText(label).width + 12 * px;
      const h = 20 * px;
      let lx = l.x + 14 * px;
      let ly = l.y - 14 * px - h;
      if (lx + w > canvas.width) lx = l.x - 14 * px - w;
      if (ly < 0) ly = l.y + 14 * px;
      ctx.fillStyle = "rgba(20, 24, 28, 0.88)";
      ctx.fillRect(lx, ly, w, h);
      ctx.fillStyle = "#ffffff";
      ctx.textBaseline = "middle";
      ctx.fillText(label, lx + 6 * px, ly + h / 2);
    }

    drawMagnifier();
  }, [monitor, start, toLocal, drawMagnifier]);

  const schedule = useCallback(() => {
    if (!s.current.raf) {
      s.current.asked = performance.now();
      s.current.raf = requestAnimationFrame(draw);
    }
  }, [draw]);

  // ---------------------------------------------------------------- session
  const broadcast = useCallback(() => {
    if (!start) return;
    const st = s.current;
    const msg: Sync = {
      seq: start.seq,
      from: myIndex,
      pointer: st.pointer,
      sel: st.sel,
      phase: st.phase,
      guides: st.guides,
      anchor: st.anchor,
      snapped: st.snapped,
      help: st.help,
      multi: st.multi,
      areas: st.areas,
      marks: st.marks,
      shape: st.shape,
    };
    void emit("xs://overlay-sync", msg);
  }, [start]);

  const finish = useCallback(
    (outcome: Record<string, unknown>) => {
      if (!start || s.current.finished) return;
      s.current.finished = true;
      void invoke("xs_overlay_finish", {
        seq: start.seq,
        outcome: { ...outcome, magnifierZoom: s.current.zoom, slow: s.current.slow },
      });
    },
    [start],
  );

  const cancel = useCallback(() => finish({ kind: "cancelled" }), [finish]);

  const setPhaseBoth = useCallback(
    (next: Phase) => {
      s.current.phase = next;
      setPhase(next);
    },
    [setPhase],
  );

  const setSelBoth = useCallback((next: Rect | null) => {
    s.current.sel = next;
    setSel(next);
  }, []);

  /** A rectangle was chosen: capture now, or adjust first in Universal (CAP-009, CAP-025). */
  const chooseRect = useCallback(
    (r: Rect) => {
      if (!start || !desktop) return;
      const rect = intersect(r, desktop) ?? r;
      // Quick annotation (CAP-057) always stops here to mark the area.
      if (start.request.mode === "universal" || start.request.quickAnnotate) {
        setSelBoth(rect);
        setPhaseBoth("adjusting");
        broadcast();
        schedule();
      } else {
        finish({ kind: "rect", rect });
      }
    },
    [start, desktop, setSelBoth, setPhaseBoth, broadcast, schedule, finish],
  );

  const chooseWindow = useCallback(
    (w: Win) => {
      if (start?.request.mode === "window" && !start.request.quickAnnotate)
        finish({ kind: "window", id: w.id });
      else chooseRect(w.rect);
    },
    [start, finish, chooseRect],
  );

  // Quick annotation (CAP-057).
  const syncQuick = () => {
    const st = s.current;
    setQuick((q) => ({ ...q, marks: st.marks.length, undone: st.undone.length }));
  };
  const addMark = (mark: Mark) => {
    const st = s.current;
    st.marks = [...st.marks, mark];
    st.undone = [];
    st.discardAsked = false;
    syncQuick();
    broadcast();
  };
  const undoMark = () => {
    const st = s.current;
    const last = st.marks[st.marks.length - 1];
    if (!last) return;
    st.marks = st.marks.slice(0, -1);
    st.undone = [...st.undone, last];
    syncQuick();
    broadcast();
    schedule();
  };
  const redoMark = () => {
    const st = s.current;
    const back = st.undone[st.undone.length - 1];
    if (!back) return;
    st.undone = st.undone.slice(0, -1);
    st.marks = [...st.marks, back];
    syncQuick();
    broadcast();
    schedule();
  };
  const setTool = (tool: Tool) => {
    s.current.markTool = tool;
    setQuick((q) => ({ ...q, tool }));
  };
  // Once: Enter, then the blur of the box going away, must not add it twice.
  const commitText = () => {
    const st = s.current;
    const current = st.typing;
    showText(null);
    if (current && current.text.trim()) {
      const size = 22 * Math.max(1, monitor?.scale ?? 1);
      addMark({ kind: "text", at: current.at, text: current.text, color: st.markColor, size });
      schedule();
    }
  };
  const stepSize = () => 32 * Math.max(1, monitor?.scale ?? 1);
  const markScale = () => Math.max(1, monitor?.scale ?? 1);
  const finishQuick = (action: "copy" | "editor") => {
    // A text still being typed is part of it.
    commitText();
    const st = s.current;
    if (!st.sel || !monitor) return;
    finish({
      kind: "quick",
      rect: st.sel,
      action,
      tool: st.markTool,
      objects: toObjects(st.marks, st.sel, monitor.scale),
    });
  };

  const quickRef = useRef({
    addMark,
    undoMark,
    redoMark,
    finishQuick,
    stepSize,
    markScale,
    syncQuick,
  });
  useEffect(() => {
    quickRef.current = {
      addMark,
      undoMark,
      redoMark,
      finishQuick,
      stepSize,
      markScale,
      syncQuick,
    };
  });

  const redo = useCallback(() => {
    setSelBoth(null);
    s.current.interaction = null;
    setPhaseBoth("aiming");
    broadcast();
    schedule();
  }, [setSelBoth, setPhaseBoth, broadcast, schedule]);

  useEffect(() => {
    const unStart = listen<Start>("xs://overlay-start", async (e) => {
      const payload = e.payload;
      if (payload.index !== myIndex) return;
      setLanguage(payload.lang);
      const st = s.current;
      st.finished = false;
      st.sel = null;
      st.hover = null;
      st.interaction = null;
      st.phase = "aiming";
      // Each selection starts without guides or the keys panel (CAP-019, CAP-020).
      st.guides = false;
      st.anchor = null;
      st.snapped = null;
      st.help = false;
      setHelp(false);
      st.multi = payload.request.mode === "multi";
      st.ctrlMulti = false;
      st.areas = [];
      setAreas([]);
      st.marks = [];
      st.undone = [];
      st.drawing = null;
      st.textAt = null;
      st.discardAsked = false;
      st.picked = -1;
      st.pickedFrom = null;
      st.shape = [];
      st.freeDrawing = false;
      st.ellipseFrom = null;
      st.polyNext = null;
      // Freehand's scissors (CAP-032).
      if (fg.current) fg.current.style.cursor = payload.request.freehand ? SCISSORS : "";
      showText(null);
      const firstTool =
        payload.request.quickTool && TOOLS.includes(payload.request.quickTool)
          ? payload.request.quickTool
          : "arrow";
      st.markTool = firstTool;
      const palette = payload.request.quickColors?.length ? payload.request.quickColors : COLORS;
      const startColor = firstColor(palette);
      st.markColor = startColor;
      setQuick((q) => ({
        ...q,
        tool: firstTool,
        colors: palette,
        color: startColor,
        marks: 0,
        undone: 0,
      }));
      st.asked = 0;
      st.lag = { sum: 0, count: 0, since: 0 };
      st.slow = false;
      st.magnifier = payload.request.showMagnifier;
      st.zoom = payload.request.magnifierZoom || 4;
      st.pointer = payload.request.pointer
        ? { x: payload.request.pointer[0], y: payload.request.pointer[1] }
        : null;
      st.hover = st.pointer ? windowAt(payload, st.pointer) : null;
      setSel(null);
      setPhase("aiming");
      if (payload.request.startFullscreen) {
        const ms = payload.request.monitors;
        const p = st.pointer;
        const target =
          (p && ms.find((x) => contains(x.rect, p))) ?? ms.find((x) => x.primary) ?? ms[0];
        if (target) {
          st.sel = target.rect;
          st.phase = "adjusting";
          setSel(target.rect);
          setPhase("adjusting");
        }
      }
      const m = payload.request.monitors[payload.index];
      const canvas = bg.current;
      if (!m || !canvas) return;
      const { width, height } = m.rect;
      const t0 = performance.now();
      let fetchMs = 0;
      st.patch = null;
      if (fg.current) {
        fg.current.width = width;
        fg.current.height = height;
      }
      // Windows paints the frozen screen natively underneath (APP-048); elsewhere the
      // frame comes through the xsframe protocol.
      document.documentElement.classList.toggle("overlay-native", payload.nativeBackground);
      if (!payload.nativeBackground) {
        try {
          const buf = await fetch(frameUrl(payload.seq, payload.index)).then((r) =>
            r.arrayBuffer(),
          );
          fetchMs = performance.now() - t0;
          canvas.width = width;
          canvas.height = height;
          const ctx = canvas.getContext("2d", { willReadFrequently: true });
          ctx?.putImageData(new ImageData(new Uint8ClampedArray(buf), width, height), 0, 0);
        } catch {
          // Without the frame the selection still works, over a black background.
        }
      }
      setStart(payload);
      const timings = { fetchMs: Math.round(fetchMs), totalMs: Math.round(performance.now() - t0) };
      await invoke("xs_overlay_ready", { seq: payload.seq, index: payload.index, timings });
    });
    const unEnd = listen("xs://overlay-end", () => {
      // Free the frozen frame right away.
      for (const c of [bg.current, fg.current]) {
        if (c) {
          c.width = 1;
          c.height = 1;
        }
      }
      setStart(null);
    });
    return () => {
      void unStart.then((f) => f());
      void unEnd.then((f) => f());
    };
  }, [showText]);

  // Other monitors' overlays tell us where the pointer and the selection are.
  useEffect(() => {
    if (!start) return;
    const un = listen<Sync>("xs://overlay-sync", (e) => {
      const m = e.payload;
      if (m.seq !== start.seq || m.from === myIndex) return;
      s.current.pointer = m.pointer;
      s.current.sel = m.sel;
      s.current.phase = m.phase;
      s.current.guides = m.guides;
      s.current.anchor = m.anchor;
      s.current.snapped = m.snapped;
      s.current.help = m.help;
      s.current.multi = m.multi;
      s.current.areas = m.areas;
      setAreas(m.areas);
      s.current.marks = m.marks;
      s.current.shape = m.shape;
      setSel(m.sel);
      setPhase(m.phase);
      setHelp(m.help);
      schedule();
    });
    return () => void un.then((f) => f());
  }, [start, schedule]);

  useEffect(() => {
    drawRef.current = draw;
  }, [draw]);

  useEffect(() => {
    if (start) schedule();
  }, [start, schedule]);

  // ---------------------------------------------------------------- pointer
  const toDesktop = useCallback(
    (e: { clientX: number; clientY: number }): Pt => {
      if (!monitor) return { x: 0, y: 0 };
      const k = monitor.rect.width / window.innerWidth;
      return {
        x: monitor.rect.x + Math.round(e.clientX * k),
        y: monitor.rect.y + Math.round(e.clientY * k),
      };
    },
    [monitor],
  );

  const hitHandle = useCallback(
    (p: Pt): Handle | "inside" | null => {
      const r = s.current.sel;
      if (!r || !monitor) return null;
      const tol = 8 * Math.max(1, monitor.scale);
      for (const h of HANDLES) {
        const hp = handlePoint(r, h);
        if (Math.abs(hp.x - p.x) <= tol && Math.abs(hp.y - p.y) <= tol) return h;
      }
      return contains(r, p) ? "inside" : null;
    },
    [monitor],
  );

  const onPointerMove = useCallback(
    (e: React.PointerEvent) => {
      if (!start) return;
      const st = s.current;
      const p = toDesktop(e);
      st.pointer = p;
      st.shift = e.shiftKey;
      st.alt = e.altKey;
      const it = st.interaction;
      // Freehand (CAP-032): the outline grows, the ellipse follows, the polygon's next
      // corner shows.
      if (start.request.freehand) {
        const shape = start.request.freehand;
        if (shape === "free" && st.freeDrawing) {
          st.shape = extend(st.shape, p, 2 * quickRef.current.markScale());
        } else if (shape === "ellipse" && st.ellipseFrom) {
          st.shape = ellipsePoints(st.ellipseFrom, p, e.shiftKey);
        } else if (shape === "polygon") {
          st.polyNext = p;
        }
        st.hover = null;
        broadcast();
        schedule();
        return;
      }
      if (it?.kind === "draw") {
        const moved =
          Math.abs(p.x - it.anchor.x) > DRAG_THRESHOLD ||
          Math.abs(p.y - it.anchor.y) > DRAG_THRESHOLD;
        if (st.phase === "dragging" || moved) {
          if (st.phase !== "dragging") setPhaseBoth("dragging");
          const compat = start.request.compatibility === true;
          const ratio = ratioFromKeys(e.shiftKey, e.altKey, e.ctrlKey, compat);
          st.lastLock = ratio;
          let end = ratio ? lockAspect(it.anchor, p, ratio) : p;
          // Near a standard size, with the guides shown, take it exactly (CAP-019).
          st.snapped = null;
          if (!ratio && st.guides) {
            const snap = snapToGuides(it.anchor, end);
            if (snap) {
              end = snap.end;
              st.snapped = snap.size;
            }
          }
          st.anchor = it.anchor;
          st.sel = fromPoints(it.anchor, end);
        }
      } else if (it?.kind === "move" && desktop) {
        const moved = {
          ...it.orig,
          x: it.orig.x + p.x - it.start.x,
          y: it.orig.y + p.y - it.start.y,
        };
        st.sel = clampInside(moved, desktop);
        setSel(st.sel);
      } else if (it?.kind === "resize") {
        st.sel = resizeByHandle(it.orig, it.handle, p);
        setSel(st.sel);
      } else if (st.pickedFrom && st.marks[st.picked]) {
        // Moving the chosen mark (CAP-057).
        const from = st.pickedFrom;
        st.marks = st.marks.map((m, i) =>
          i === st.picked ? moveMark(m, p.x - from.x, p.y - from.y) : m,
        );
        st.pickedFrom = p;
      } else if (st.drawing) {
        st.drawing = { ...st.drawing, to: p };
      } else if (st.phase === "aiming") {
        st.hover = windowAt(start, p);
      } else if (st.phase === "adjusting" && fg.current) {
        const h = hitHandle(p);
        fg.current.style.cursor = h === "inside" ? "move" : h ? `${h}-resize` : "crosshair";
      }
      broadcast();
      schedule();
    },
    [start, desktop, toDesktop, setPhaseBoth, hitHandle, broadcast, schedule],
  );

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      if (!start || e.button !== 0) return;
      (e.target as Element).setPointerCapture(e.pointerId);
      const st = s.current;
      const p = toDesktop(e);
      st.pointer = p;
      // Ctrl while dragging adds areas (CAP-033); the compatibility profile uses Ctrl
      // for the aspect ratio (CAP-066).
      if (
        !st.multi &&
        e.ctrlKey &&
        start.request.compatibility !== true &&
        (start.request.mode === "universal" || start.request.mode === "region") &&
        st.phase === "aiming"
      ) {
        st.multi = true;
        st.ctrlMulti = true;
      }
      // A fixed-size region: a click places it and captures (CAP-026).
      const fixed = st.phase === "aiming" ? fixedRect(start, p) : null;
      if (fixed) {
        if (start.request.quickAnnotate) chooseRect(fixed);
        else finish({ kind: "rect", rect: fixed });
        return;
      }
      // The pointer may not have moved since the selection appeared.
      if (st.phase === "aiming") st.hover = windowAt(start, p);
      if (st.phase === "adjusting" && st.sel) {
        const h = hitHandle(p);
        // Quick annotation (CAP-057): inside the area the tools draw; its handles work
        // only until the first mark.
        if (start.request.quickAnnotate && (h === "inside" || (h && st.marks.length))) {
          if (!contains(st.sel, p)) return;
          // A mark under the pointer is chosen and can be dragged; elsewhere the tool
          // draws (CAP-057).
          const hit = hitMark(st.marks, p, 8 * quickRef.current.markScale());
          if (hit >= 0) {
            st.picked = hit;
            st.pickedFrom = p;
            drawRef.current();
            return;
          }
          st.picked = -1;
          if (st.markTool === "step") {
            const n = st.marks.filter((m) => m.kind === "step").length + 1;
            const size = quickRef.current.stepSize();
            quickRef.current.addMark({ kind: "step", at: p, value: n, color: st.markColor, size });
          } else if (st.markTool === "text") {
            // The box opens when the button is let go: on press, the page would take
            // its focus back.
            st.textAt = p;
          } else {
            st.drawing = { from: p, to: p };
          }
          drawRef.current();
          return;
        }
        if (h === "inside") st.interaction = { kind: "move", start: p, orig: st.sel };
        else if (h) st.interaction = { kind: "resize", handle: h, orig: st.sel };
        return;
      }
      if (windowsOnly(start)) return;
      // Freehand (CAP-032): a free outline starts, a polygon gets a corner (a double
      // click closes it), an ellipse starts at a corner.
      if (start.request.freehand) {
        const shape = start.request.freehand;
        if (shape === "free") {
          st.shape = [p];
          st.freeDrawing = true;
        } else if (shape === "ellipse") {
          st.ellipseFrom = p;
          st.shape = [];
        } else if (e.detail >= 2 && closes(st.shape)) {
          finish({ kind: "shape", points: st.shape.map((q) => [q.x, q.y]) });
          return;
        } else {
          st.shape = [...st.shape, p];
        }
        broadcast();
        schedule();
        return;
      }
      st.interaction = { kind: "draw", anchor: p };
    },
    [start, toDesktop, hitHandle, finish, chooseRect, broadcast, schedule],
  );

  const onPointerUp = useCallback(
    (e: React.PointerEvent) => {
      if (!start || e.button !== 0) return;
      const st = s.current;
      const it = st.interaction;
      st.interaction = null;
      // Freehand (CAP-032): letting go closes a free outline or an ellipse.
      if (start.request.freehand) {
        // A polygon goes on with the next click (or Enter).
        if (start.request.freehand === "polygon") return;
        const shape = st.shape;
        st.freeDrawing = false;
        st.ellipseFrom = null;
        if (closes(shape)) {
          finish({ kind: "shape", points: shape.map((q) => [q.x, q.y]) });
        } else {
          st.shape = [];
          broadcast();
          schedule();
        }
        return;
      }
      if (st.pickedFrom) {
        st.pickedFrom = null;
        broadcast();
        return;
      }
      if (st.textAt) {
        showText({ at: st.textAt, text: "" });
        st.textAt = null;
        return;
      }
      if (st.drawing) {
        const scale = quickRef.current.markScale();
        const mark = markFromDrag(
          st.markTool,
          st.drawing.from,
          st.drawing.to,
          st.markColor,
          st.markWidth * scale,
          st.markEllipse,
        );
        st.drawing = null;
        if (mark) quickRef.current.addMark(mark);
        schedule();
        return;
      }
      if (st.multi) {
        // Each drag adds an area, and each click the window under it (CAP-033).
        const added =
          st.phase === "dragging" && st.sel
            ? st.sel
            : !it || it.kind === "draw"
              ? (st.hover?.rect ?? null)
              : null;
        if (added && desktop) {
          st.areas = [...st.areas, intersect(added, desktop) ?? added];
          setAreas(st.areas);
        }
        st.sel = null;
        setSel(null);
        if (st.phase !== "aiming") setPhaseBoth("aiming");
        broadcast();
        schedule();
        return;
      }
      if (st.phase === "dragging" && st.sel) {
        setSelBoth(st.sel);
        chooseRect(st.sel);
        return;
      }
      if (st.phase === "adjusting") {
        schedule();
        return;
      }
      // Menu capture: a click without dragging takes the menus found (CAP-031).
      if (
        start.request.mode === "menu" &&
        start.request.menus?.length &&
        (!it || it.kind === "draw")
      ) {
        finish({ kind: "menu" });
        return;
      }
      // A click without dragging: the detected window, or nothing (CAP-025 edge case).
      if (!it || it.kind === "draw") {
        st.sel = null;
        const w = st.hover;
        if (w) chooseWindow(w);
      }
      schedule();
    },
    [
      start,
      desktop,
      showText,
      chooseRect,
      chooseWindow,
      setSelBoth,
      setPhaseBoth,
      broadcast,
      schedule,
      finish,
    ],
  );

  // ---------------------------------------------------------------- keyboard
  useEffect(() => {
    if (!start || !monitor || !desktop) return;
    const onKey = (e: KeyboardEvent) => {
      const st = s.current;
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" && e.key !== "Escape") return;
      if (target.tagName === "TEXTAREA") return;
      if (e.key === "Escape") {
        e.preventDefault();
        if (start.request.quickAnnotate && st.drawing) {
          st.drawing = null;
          schedule();
          return;
        }
        if (start.request.quickAnnotate && st.picked >= 0) {
          st.picked = -1;
          schedule();
          return;
        }
        if (start.request.quickAnnotate && st.marks.length && !st.discardAsked) {
          st.discardAsked = true;
          setToast(t("ui.quick.discard"));
          window.setTimeout(() => setToast(null), 2500);
          return;
        }
        cancel();
        return;
      }
      // Freehand's keys (CAP-032): Enter closes the polygon, Backspace removes the
      // last corner.
      if (start.request.freehand === "polygon") {
        if (e.key === "Enter" && closes(st.shape)) {
          e.preventDefault();
          finish({ kind: "shape", points: st.shape.map((q) => [q.x, q.y]) });
          return;
        }
        if (e.key === "Backspace" && st.shape.length) {
          e.preventDefault();
          st.shape = st.shape.slice(0, -1);
          broadcast();
          schedule();
          return;
        }
      }
      // Quick annotation's keys (CAP-057).
      if (start.request.quickAnnotate && st.phase === "adjusting" && st.sel) {
        const ctrl = e.ctrlKey || e.metaKey;
        const k = e.key.toLowerCase();
        const n = Number(e.key);
        if (!ctrl && n >= 1 && n <= TOOLS.length) {
          setTool(TOOLS[n - 1] as Tool);
        } else if (ctrl && k === "z" && e.shiftKey) {
          quickRef.current.redoMark();
        } else if (ctrl && k === "z") {
          quickRef.current.undoMark();
        } else if (e.key === "Enter" || (ctrl && k === "c")) {
          quickRef.current.finishQuick("copy");
        } else if (ctrl && k === "e") {
          quickRef.current.finishQuick("editor");
        } else if ((e.key === "Delete" || e.key === "Backspace") && st.marks[st.picked]) {
          // The chosen mark goes (CAP-057).
          st.marks = st.marks.filter((_, i) => i !== st.picked);
          st.picked = -1;
          quickRef.current.syncQuick();
          broadcast();
        } else if (e.key === "Delete" || e.key === "Backspace") {
          quickRef.current.undoMark();
        } else {
          return;
        }
        e.preventDefault();
        schedule();
        return;
      }
      const key = e.key.toLowerCase();
      const compat = start.request.compatibility === true;
      if (e.key === "F1" || (key === "h" && !compat)) {
        // The keys that work now (CAP-020); with the compatibility profile, F1 only.
        st.help = !st.help;
        setHelp(st.help);
        broadcast();
      } else if (key === "g" && !windowsOnly(start)) {
        // Standard size guides (CAP-019).
        st.guides = !st.guides;
        if (!st.guides) st.snapped = null;
        setToast(t(st.guides ? "ui.overlay.guides_on" : "ui.overlay.guides_off"));
        window.setTimeout(() => setToast(null), 1500);
        broadcast();
      } else if (key === "m") {
        st.magnifier = !st.magnifier;
      } else if (e.key === "+" || e.key === "=") {
        st.zoom = nextZoom(st.zoom, true);
      } else if (e.key === "-") {
        st.zoom = nextZoom(st.zoom, false);
      } else if (key === "c" && st.phase === "aiming") {
        if (st.color) {
          const color = st.color;
          void navigator.clipboard.writeText(color).then(() => {
            setToast(t("ui.overlay.color_copied", { color }));
            window.setTimeout(() => setToast(null), 1500);
          });
        }
      } else if (key === "f" && st.phase === "aiming" && !windowsOnly(start)) {
        chooseRect(monitor.rect);
      } else if (key === "a" && st.phase === "aiming" && !windowsOnly(start)) {
        chooseRect(desktop);
      } else if (
        e.key === "Enter" &&
        st.phase === "aiming" &&
        start.request.mode === "menu" &&
        start.request.menus?.length
      ) {
        finish({ kind: "menu" });
      } else if (st.multi && e.key === "Enter") {
        if (st.areas.length > 0) finish({ kind: "multi", rects: st.areas });
      } else if (st.multi && e.key === "Backspace") {
        st.areas = st.areas.slice(0, -1);
        setAreas(st.areas);
        broadcast();
      } else if (
        start.request.fixedSize &&
        st.phase === "aiming" &&
        (e.key === "Enter" || e.key.startsWith("Arrow"))
      ) {
        // The fixed-size region (CAP-026): arrows move it 1 px (Shift: 10), Enter captures.
        const p = st.pointer ?? {
          x: monitor.rect.x + monitor.rect.width / 2,
          y: monitor.rect.y + monitor.rect.height / 2,
        };
        if (e.key === "Enter") {
          const r = fixedRect(start, p);
          if (r && start.request.quickAnnotate) chooseRect(r);
          else if (r) finish({ kind: "rect", rect: r });
        } else {
          const step = e.shiftKey ? 10 : 1;
          st.pointer = {
            x: p.x + (e.key === "ArrowLeft" ? -step : e.key === "ArrowRight" ? step : 0),
            y: p.y + (e.key === "ArrowUp" ? -step : e.key === "ArrowDown" ? step : 0),
          };
          broadcast();
        }
      } else if (e.key === "Enter" && st.phase === "aiming" && st.hover) {
        chooseWindow(st.hover);
      } else if ((e.key === "Enter" || key === "i") && st.phase === "adjusting" && st.sel) {
        finish({ kind: "rect", rect: st.sel });
      } else if (
        ["s", "d", "r", "x"].includes(key) &&
        st.phase === "adjusting" &&
        scrollable(start, st.sel) &&
        st.sel
      ) {
        // Scrolling (CAP-034): S by hand; D, R and X scroll by themselves (CAP-035).
        const direction = { d: "vertical", r: "horizontal", x: "both" }[key];
        finish(
          direction
            ? { kind: "scroll", type: "auto", direction, rect: st.sel }
            : { kind: "scroll", type: "manual", rect: st.sel },
        );
      } else if (e.key === "Backspace" && st.phase === "adjusting") {
        redo();
      } else if (e.key.startsWith("Arrow") && st.phase === "adjusting" && st.sel) {
        // Move 1 px (Shift: 10 px); Alt grows or shrinks the right/bottom edge (CAP-017).
        const step = e.shiftKey ? 10 : 1;
        const dx = e.key === "ArrowLeft" ? -step : e.key === "ArrowRight" ? step : 0;
        const dy = e.key === "ArrowUp" ? -step : e.key === "ArrowDown" ? step : 0;
        const r = st.sel;
        const next = e.altKey
          ? { ...r, width: Math.max(1, r.width + dx), height: Math.max(1, r.height + dy) }
          : { ...r, x: r.x + dx, y: r.y + dy };
        setSelBoth(clampInside(next, desktop));
        broadcast();
      } else {
        return;
      }
      e.preventDefault();
      schedule();
    };
    // Letting go of Ctrl ends the areas begun with it, with two or more (CAP-033).
    const onKeyUp = (e: KeyboardEvent) => {
      const st = s.current;
      if (e.key === "Control" && st.ctrlMulti && st.areas.length >= 2) {
        finish({ kind: "multi", rects: st.areas });
      }
    };
    window.addEventListener("keydown", onKey);
    window.addEventListener("keyup", onKeyUp);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("keyup", onKeyUp);
    };
  }, [
    start,
    monitor,
    desktop,
    cancel,
    chooseRect,
    chooseWindow,
    finish,
    redo,
    setSelBoth,
    toLocal,
    broadcast,
    schedule,
    t,
  ]);

  // ---------------------------------------------------------------- render
  const showButtons = start && phase === "aiming" && !windowsOnly(start);
  const barHere =
    sel &&
    monitor &&
    phase === "adjusting" &&
    contains(monitor.rect, {
      x: Math.min(sel.x + sel.width - 1, monitor.rect.x + monitor.rect.width - 1),
      y: Math.min(sel.y + sel.height - 1, monitor.rect.y + monitor.rect.height - 1),
    });

  return (
    <div className="overlay" aria-hidden={!start}>
      <canvas ref={bg} className="overlay-layer" />
      <canvas
        ref={fg}
        className="overlay-layer overlay-input"
        onPointerMove={onPointerMove}
        onPointerDown={onPointerDown}
        onPointerUp={onPointerUp}
      />
      <div className="overlay-mag">
        <canvas ref={mag} style={{ width: MAG_CSS, height: MAG_CSS }} />
        <div data-info className="overlay-mag-info" />
      </div>
      {toast && (
        <div className="overlay-toast" role="status">
          {toast}
        </div>
      )}
      {start && myIndex === 0 && areas.length > 0 && (
        <div className="overlay-top multi" role="toolbar" aria-label={t("ui.overlay.bar")}>
          <span className="overlay-count">{t("ui.overlay.areas", { count: areas.length })}</span>
          <button
            className="overlay-btn primary"
            onClick={() => finish({ kind: "multi", rects: s.current.areas })}
          >
            {t("ui.overlay.finish_areas")} <kbd>Enter</kbd>
          </button>
        </div>
      )}
      {monitor &&
        areas.map((area, i) => {
          const a = intersect(area, monitor.rect);
          if (!a || a.x + a.width !== area.x + area.width || a.y !== area.y) return null;
          return (
            <button
              key={`${i}-${area.x}-${area.y}`}
              className="overlay-area-x"
              aria-label={t("ui.overlay.remove_area", { n: i + 1 })}
              title={t("ui.overlay.remove_area", { n: i + 1 })}
              style={{
                left: (area.x + area.width - monitor.rect.x) * cssPerPx - 26,
                top: (area.y - monitor.rect.y) * cssPerPx + 4,
              }}
              onClick={() => {
                const st = s.current;
                st.areas = st.areas.filter((_, k) => k !== i);
                setAreas(st.areas);
                broadcast();
                schedule();
              }}
            >
              ×
            </button>
          );
        })}
      {showButtons && areas.length === 0 && (
        <div className="overlay-top" role="toolbar" aria-label={t("ui.overlay.bar")}>
          <button className="overlay-btn" onClick={() => monitor && chooseRect(monitor.rect)}>
            {t("ui.overlay.fullscreen")} <kbd>F</kbd>
          </button>
          {myIndex === 0 && (start?.request.monitors.length ?? 0) > 1 && desktop && (
            <button className="overlay-btn" onClick={() => chooseRect(desktop)}>
              {t("ui.overlay.all_monitors")} <kbd>A</kbd>
            </button>
          )}
        </div>
      )}
      {start && phase === "aiming" && !help && (
        <div className="overlay-hint">
          {start.request.freehand
            ? t(`ui.overlay.freehand_${start.request.freehand}`)
            : start.request.mode === "menu" && start.request.menus?.length
              ? t("ui.overlay.hint_menu")
              : windowsOnly(start)
                ? t("ui.overlay.hint_window")
                : t("ui.overlay.hint")}
          {start.request.helpHint &&
            ` · ${t("ui.overlay.help_hint", { key: start.request.compatibility ? "F1" : "H" })}`}
        </div>
      )}
      {start && help && (
        <KeysHelp
          phase={phase}
          windowsOnly={windowsOnly(start)}
          compatibility={start.request.compatibility === true}
          monitors={start.request.monitors.length}
        />
      )}
      {barHere && sel && monitor && start?.request.quickAnnotate && (
        <QuickBar
          style={quickBarStyle(sel, monitor.rect, cssPerPx)}
          tool={quick.tool}
          colors={quick.colors}
          color={quick.color}
          width={quick.width}
          ellipse={quick.ellipse}
          canUndo={quick.marks > 0}
          canRedo={quick.undone > 0}
          onTool={setTool}
          onColor={(c) => {
            s.current.markColor = c;
            setQuick((q) => ({ ...q, color: c }));
          }}
          onWidth={(w) => {
            s.current.markWidth = w;
            setQuick((q) => ({ ...q, width: w }));
          }}
          onEllipse={(on) => {
            s.current.markEllipse = on;
            setQuick((q) => ({ ...q, ellipse: on }));
          }}
          onUndo={undoMark}
          onRedo={redoMark}
          onCopy={() => finishQuick("copy")}
          onEditor={() => finishQuick("editor")}
          onCancel={cancel}
        />
      )}
      {typing && monitor && (
        <textarea
          className="overlay-text"
          autoFocus
          aria-label={t("ui.quick.tool_text")}
          value={typing.text}
          style={{
            left: (typing.at.x - monitor.rect.x) * cssPerPx,
            top: (typing.at.y - monitor.rect.y) * cssPerPx,
            color: quick.color,
            fontSize: 22 * Math.max(1, monitor.scale) * cssPerPx,
          }}
          placeholder={t("ui.quick.text_hint")}
          onChange={(e) => showText({ ...typing, text: e.target.value })}
          onKeyDown={(e) => {
            e.stopPropagation();
            if (e.key === "Escape") {
              e.preventDefault();
              showText(null);
            } else if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              commitText();
            }
          }}
          onBlur={commitText}
        />
      )}
      {barHere && sel && monitor && desktop && !start?.request.quickAnnotate && (
        <CaptureBar
          sel={sel}
          monitor={monitor.rect}
          cssPerPx={cssPerPx}
          clamped={clamped}
          onApply={(r) => {
            const fixed = clampInside(r, desktop);
            setClamped(
              fixed.width !== r.width ||
                fixed.height !== r.height ||
                fixed.x !== r.x ||
                fixed.y !== r.y,
            );
            setSelBoth(fixed);
            broadcast();
            schedule();
          }}
          onImage={() => finish({ kind: "rect", rect: sel })}
          onScroll={
            start && start.request.mode !== "window" && start.request.mode !== "scrolling"
              ? (direction) =>
                  finish(
                    direction
                      ? { kind: "scroll", type: "auto", direction, rect: sel }
                      : { kind: "scroll", type: "manual", rect: sel },
                  )
              : undefined
          }
          scrollOk={start ? scrollable(start, sel) : false}
          onRedo={redo}
          onCancel={cancel}
        />
      )}
    </div>
  );
}

/**
 * The keys that work at this moment of the selection (CAP-017, CAP-020), as a panel that
 * never appears in the capture: the capture is cut from the frozen screens.
 */
function KeysHelp({
  phase,
  windowsOnly: windowOnly,
  compatibility,
  monitors,
}: {
  phase: Phase;
  windowsOnly: boolean;
  compatibility: boolean;
  monitors: number;
}) {
  const { t } = useTranslation();
  const rows: [string, string][] = [];
  const k = (keys: string, what: string) => rows.push([keys, t(`ui.overlay.keys.${what}`)]);
  if (phase === "adjusting") {
    k("Enter · I", "capture");
    k("S", "scroll");
    k("D · R · X", "scroll_auto");
    k("← ↑ → ↓", "move");
    k("Shift", "move_more");
    k("Alt + ← ↑ → ↓", "resize");
    k("⌫", "redo");
  } else if (windowOnly) {
    k(t("ui.overlay.keys.click_key"), "click_window");
    k("Enter", "enter_window");
  } else {
    k(t("ui.overlay.keys.drag_key"), "drag");
    k(t("ui.overlay.keys.click_key"), "click_window");
    k("Enter", "enter_window");
    k(compatibility ? "Shift · Ctrl · Ctrl+Shift" : "Shift · Alt · Alt+Shift", "ratio");
    if (!compatibility) k("Ctrl + " + t("ui.overlay.keys.drag_key"), "add_area");
    k("⌫", "remove_last_area");
    k("G", "guides");
    k("F", "fullscreen");
    if (monitors > 1) k("A", "all_monitors");
    k("C", "color");
  }
  k("M", "magnifier");
  k("+ −", "zoom");
  k(compatibility ? "F1" : "H · F1", "help");
  k("Esc", "cancel");
  return (
    <div className="overlay-help" role="dialog" aria-label={t("ui.overlay.keys.title")}>
      <strong>{t("ui.overlay.keys.title")}</strong>
      <dl>
        {rows.map(([keys, what]) => (
          <div key={what}>
            <dt>
              <kbd>{keys}</kbd>
            </dt>
            <dd>{what}</dd>
          </div>
        ))}
      </dl>
    </div>
  );
}

/** Below 30 frames a second, a frame waits more than this (CAP-014, APP-053). */
const SLOW_FRAME_MS = 1000 / 30;

/**
 * Measures how long each frame waits from the event that asked for it until it is
 * drawn. Averaged over at least 2 s of moving, above SLOW_FRAME_MS the crosshair is
 * slow on this computer, and the capture tells XcreenShot so (CAP-014).
 */
function trackLag(st: {
  asked: number;
  lag: { sum: number; count: number; since: number };
  slow: boolean;
}): void {
  const now = performance.now();
  if (!st.asked || st.slow) return;
  const wait = now - st.asked;
  st.asked = 0;
  // A pause in moving starts a new window.
  if (!st.lag.since || wait > 1000) {
    st.lag = { sum: 0, count: 0, since: now };
    return;
  }
  st.lag.sum += wait;
  st.lag.count += 1;
  if (now - st.lag.since >= 2000) {
    if (st.lag.count >= 20 && st.lag.sum / st.lag.count > SLOW_FRAME_MS) st.slow = true;
    st.lag = { sum: 0, count: 0, since: now };
  }
}

/** Topmost detected window under the pointer, if detection applies (CAP-016). */
function windowAt(start: Start, p: Pt): Win | null {
  if (!start.request.autoDetect && !windowsOnly(start)) return null;
  return start.request.windows.find((w) => contains(w.rect, p)) ?? null;
}

function lockName(ratio: number): string {
  if (Math.abs(ratio - 1) < 0.01) return "  1:1";
  if (Math.abs(ratio - 16 / 9) < 0.01) return "  16:9";
  if (Math.abs(ratio - 4 / 3) < 0.01) return "  4:3";
  return "";
}

function pixelHex(canvas: HTMLCanvasElement, l: Pt): string {
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  const d = ctx?.getImageData(Math.max(0, l.x), Math.max(0, l.y), 1, 1).data;
  return d ? toHex(d[0] ?? 0, d[1] ?? 0, d[2] ?? 0) : "#000000";
}

interface BarProps {
  sel: Rect;
  monitor: Rect;
  cssPerPx: number;
  clamped: boolean;
  onApply: (r: Rect) => void;
  onImage: () => void;
  /** Scrolling (CAP-034): by hand, or by itself in a direction (CAP-035). */
  onScroll: ((direction?: "vertical" | "horizontal" | "both") => void) | undefined;
  /** The selection is big enough to scroll. */
  scrollOk: boolean;
  onRedo: () => void;
  onCancel: () => void;
}

/** Capture bar (CAP-009, CAP-011, CAP-012). Never part of the image. */
function CaptureBar({
  sel,
  monitor,
  cssPerPx,
  clamped,
  onApply,
  onImage,
  onScroll,
  scrollOk,
  onRedo,
  onCancel,
}: BarProps) {
  const { t } = useTranslation();
  const ref = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 520, height: 48 });
  const [moved, setMoved] = useState<Pt | null>(null);
  useEffect(() => {
    const el = ref.current;
    if (el) setSize({ width: el.offsetWidth, height: el.offsetHeight });
  }, []);
  // Placement in CSS px of this window.
  const selCss = {
    x: (sel.x - monitor.x) * cssPerPx,
    y: (sel.y - monitor.y) * cssPerPx,
    width: sel.width * cssPerPx,
    height: sel.height * cssPerPx,
  };
  const auto = placeBar(selCss, size, {
    x: 0,
    y: 0,
    width: window.innerWidth,
    height: window.innerHeight,
  });
  const pos = moved ?? auto;
  const field = (key: keyof Rect, label: string) => (
    <label className="overlay-field">
      <span>{label}</span>
      <input
        key={`${key}-${sel[key]}`}
        type="number"
        defaultValue={sel[key]}
        className={clamped ? "overlay-clamped" : undefined}
        aria-label={label}
        onKeyDown={(e) => {
          tenStep(e, (v) => (e.currentTarget.value = String(v)));
          if (e.key === "Enter") {
            e.preventDefault();
            e.stopPropagation();
            const value = Math.round(Number((e.target as HTMLInputElement).value));
            if (Number.isFinite(value))
              onApply({
                ...sel,
                [key]: key === "width" || key === "height" ? Math.max(1, value) : value,
              });
          }
        }}
      />
    </label>
  );
  return (
    <div
      ref={ref}
      className="overlay-bar"
      role="toolbar"
      aria-label={t("ui.overlay.bar")}
      style={{ left: pos.x, top: pos.y }}
    >
      <span
        className="overlay-grip"
        aria-hidden
        onPointerDown={(e) => {
          const startX = e.clientX - pos.x;
          const startY = e.clientY - pos.y;
          const move = (ev: PointerEvent) =>
            setMoved({ x: ev.clientX - startX, y: ev.clientY - startY });
          const up = () => {
            window.removeEventListener("pointermove", move);
            window.removeEventListener("pointerup", up);
          };
          window.addEventListener("pointermove", move);
          window.addEventListener("pointerup", up);
        }}
      >
        ⠿
      </span>
      {field("x", t("ui.overlay.x"))}
      {field("y", t("ui.overlay.y"))}
      {field("width", t("ui.overlay.width"))}
      {field("height", t("ui.overlay.height"))}
      <button className="overlay-btn primary" onClick={onImage} autoFocus>
        {t("ui.overlay.image")} <kbd>Enter</kbd>
      </button>
      {onScroll && (
        <span
          className="overlay-scroll"
          title={scrollOk ? undefined : t("ui.overlay.scroll_small")}
        >
          <button className="overlay-btn" disabled={!scrollOk} onClick={() => onScroll()}>
            {t("ui.overlay.scrolling")} <kbd>S</kbd>
          </button>
          {(
            [
              ["vertical", "D", "↓", "scroll_auto_v"],
              ["horizontal", "R", "→", "scroll_auto_h"],
              ["both", "X", "↘", "scroll_auto_b"],
            ] as const
          ).map(([direction, key, arrow, label]) => (
            <button
              key={direction}
              className="overlay-btn icon"
              disabled={!scrollOk}
              aria-label={t(`ui.overlay.${label}`)}
              title={t(`ui.overlay.${label}`)}
              aria-keyshortcuts={key}
              onClick={() => onScroll(direction)}
            >
              {arrow}
            </button>
          ))}
        </span>
      )}
      <button className="overlay-btn" onClick={onRedo} title="Backspace">
        {t("ui.overlay.redo")}
      </button>
      <button className="overlay-btn" onClick={onCancel} title="Esc">
        {t("ui.overlay.cancel")}
      </button>
      {clamped && (
        <span className="overlay-warn" role="status">
          {t("ui.overlay.clamped")}
        </span>
      )}
    </div>
  );
}

/** Where quick annotation's bar goes: under the area, or above it near the bottom. */
function quickBarStyle(sel: Rect, monitor: Rect, cssPerPx: number): React.CSSProperties {
  const left = Math.max(8, (sel.x - monitor.x) * cssPerPx);
  const bottom = (sel.y + sel.height - monitor.y) * cssPerPx;
  const roomBelow = window.innerHeight - bottom;
  return roomBelow > 64
    ? { left, top: bottom + 10 }
    : { left, top: Math.max(8, (sel.y - monitor.y) * cssPerPx - 54) };
}

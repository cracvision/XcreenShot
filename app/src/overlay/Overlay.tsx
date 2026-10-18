// On-screen selection over the frozen screen (CAP-009, CAP-011, CAP-013, CAP-015,
// CAP-016, CAP-017, CAP-018, CAP-021, CAP-025, CAP-027). One instance per monitor
// window. Everything is computed in physical desktop pixels; the canvases use the
// monitor's physical size so the frozen image and the pixel grid are exact.
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { setLanguage } from "../i18n";
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
  resizeByHandle,
  toHex,
  type Handle,
  type Pt,
  type Rect,
} from "./geometry";

type Mode = "region" | "window" | "universal";
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
  };
}

interface Sync {
  seq: number;
  from: number;
  pointer: Pt | null;
  sel: Rect | null;
  phase: Phase;
}

type Interaction =
  | { kind: "draw"; anchor: Pt }
  | { kind: "move"; start: Pt; orig: Rect }
  | { kind: "resize"; handle: Handle; orig: Rect }
  | null;

const ACCENT = "#2ec4b6";
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
  });

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
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Dim everything except the selection or the detected window (CAP-013, CAP-016).
    const focus = st.sel ?? (st.phase === "aiming" ? (st.hover?.rect ?? null) : null);
    ctx.fillStyle = DIM;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    const f = focus ? intersect(focus, M) : null;
    if (f) {
      ctx.clearRect(f.x - M.x, f.y - M.y, f.width, f.height);
      ctx.lineWidth = 2 * px;
      ctx.setLineDash(st.sel ? [] : [6 * px, 4 * px]);
      ctx.strokeStyle = OUTLINE;
      ctx.strokeRect(f.x - M.x - px, f.y - M.y - px, f.width + 2 * px, f.height + 2 * px);
      ctx.strokeStyle = ACCENT;
      ctx.strokeRect(f.x - M.x, f.y - M.y, f.width, f.height);
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

    // Crosshair across the monitor, with an outline for any background (CAP-013).
    const p = st.pointer;
    if (p && st.phase !== "adjusting" && contains(M, p)) {
      const l = toLocal(p);
      ctx.fillStyle = OUTLINE;
      ctx.fillRect(0, l.y - px, canvas.width, 3 * px);
      ctx.fillRect(l.x - px, 0, 3 * px, canvas.height);
      ctx.fillStyle = ACCENT;
      ctx.fillRect(0, l.y, canvas.width, px);
      ctx.fillRect(l.x, 0, px, canvas.height);

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
    if (!s.current.raf) s.current.raf = requestAnimationFrame(draw);
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
    };
    void emit("xs://overlay-sync", msg);
  }, [start]);

  const finish = useCallback(
    (outcome: Record<string, unknown>) => {
      if (!start || s.current.finished) return;
      s.current.finished = true;
      void invoke("xs_overlay_finish", {
        seq: start.seq,
        outcome: { ...outcome, magnifierZoom: s.current.zoom },
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
      if (start.request.mode === "universal") {
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
      if (start?.request.mode === "window") finish({ kind: "window", id: w.id });
      else chooseRect(w.rect);
    },
    [start, finish, chooseRect],
  );

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
      st.magnifier = payload.request.showMagnifier;
      st.zoom = payload.request.magnifierZoom || 4;
      st.pointer = payload.request.pointer
        ? { x: payload.request.pointer[0], y: payload.request.pointer[1] }
        : null;
      st.hover = st.pointer ? windowAt(payload, st.pointer) : null;
      setSel(null);
      setPhase("aiming");
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
  }, []);

  // Other monitors' overlays tell us where the pointer and the selection are.
  useEffect(() => {
    if (!start) return;
    const un = listen<Sync>("xs://overlay-sync", (e) => {
      const m = e.payload;
      if (m.seq !== start.seq || m.from === myIndex) return;
      s.current.pointer = m.pointer;
      s.current.sel = m.sel;
      s.current.phase = m.phase;
      setSel(m.sel);
      setPhase(m.phase);
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
      if (it?.kind === "draw") {
        const moved =
          Math.abs(p.x - it.anchor.x) > DRAG_THRESHOLD ||
          Math.abs(p.y - it.anchor.y) > DRAG_THRESHOLD;
        if (st.phase === "dragging" || moved) {
          if (st.phase !== "dragging") setPhaseBoth("dragging");
          const ratio = ratioFromKeys(e.shiftKey, e.altKey);
          st.lastLock = ratio;
          const end = ratio ? lockAspect(it.anchor, p, ratio) : p;
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
      // The pointer may not have moved since the selection appeared.
      if (st.phase === "aiming") st.hover = windowAt(start, p);
      if (st.phase === "adjusting" && st.sel) {
        const h = hitHandle(p);
        if (h === "inside") st.interaction = { kind: "move", start: p, orig: st.sel };
        else if (h) st.interaction = { kind: "resize", handle: h, orig: st.sel };
        return;
      }
      if (start.request.mode === "window") return;
      st.interaction = { kind: "draw", anchor: p };
    },
    [start, toDesktop, hitHandle],
  );

  const onPointerUp = useCallback(
    (e: React.PointerEvent) => {
      if (!start || e.button !== 0) return;
      const st = s.current;
      const it = st.interaction;
      st.interaction = null;
      if (st.phase === "dragging" && st.sel) {
        setSelBoth(st.sel);
        chooseRect(st.sel);
        return;
      }
      if (st.phase === "adjusting") {
        schedule();
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
    [start, chooseRect, chooseWindow, setSelBoth, schedule],
  );

  // ---------------------------------------------------------------- keyboard
  useEffect(() => {
    if (!start || !monitor || !desktop) return;
    const onKey = (e: KeyboardEvent) => {
      const st = s.current;
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" && e.key !== "Escape") return;
      if (e.key === "Escape") {
        e.preventDefault();
        cancel();
        return;
      }
      const key = e.key.toLowerCase();
      if (key === "m") {
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
      } else if (key === "f" && st.phase === "aiming" && start.request.mode !== "window") {
        chooseRect(monitor.rect);
      } else if (key === "a" && st.phase === "aiming" && start.request.mode !== "window") {
        chooseRect(desktop);
      } else if (e.key === "Enter" && st.phase === "aiming" && st.hover) {
        chooseWindow(st.hover);
      } else if ((e.key === "Enter" || key === "i") && st.phase === "adjusting" && st.sel) {
        finish({ kind: "rect", rect: st.sel });
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
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
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
  const showButtons = start && phase === "aiming" && start.request.mode !== "window";
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
      {showButtons && (
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
      {start && phase === "aiming" && (
        <div className="overlay-hint">
          {start.request.mode === "window" ? t("ui.overlay.hint_window") : t("ui.overlay.hint")}
        </div>
      )}
      {barHere && sel && monitor && desktop && (
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
          onRedo={redo}
          onCancel={cancel}
        />
      )}
    </div>
  );
}

/** Topmost detected window under the pointer, if detection applies (CAP-016). */
function windowAt(start: Start, p: Pt): Win | null {
  if (!start.request.autoDetect && start.request.mode !== "window") return null;
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

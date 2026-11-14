// Live drawing (VID-048 to VID-051): the overlay that shows the shapes over a whole
// monitor (it shows in the video; clicks go through it), and its toolbar (never in the
// video). The core keeps the shapes and the hooks draw them; this only shows them.
// Shapes fade by themselves: each comes with the seconds until it starts to fade.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";

type Tool = "arrow" | "rect" | "oval" | "pen" | "step";

interface Shape {
  id: number;
  tool: Tool;
  color: string;
  width: number;
  points: [number, number][];
  even: boolean;
  number: number | null;
  /** Seconds until it starts to fade, when the state was sent; null: not yet. */
  fadeIn: number | null;
}

interface DrawState {
  on: boolean;
  /** When this window got it (performance.now()). */
  at?: number;
  tool?: Tool;
  color?: string;
  width?: string;
  fade?: number | null;
  palette?: string[];
  scale?: number;
  fadeOut?: number;
  shapes?: Shape[];
}

/** A shape with its fade on this window's clock (ms). */
type Shown = Shape & { fadeAt: number | null };

function useDrawState(): DrawState {
  const [state, setState] = useState<DrawState>({ on: false });
  useEffect(() => {
    document.documentElement.classList.add("launcher-root");
    // What it shows now: the first state may have come before this window listened.
    void run<DrawState>("record.draw", { query: true })
      .then((s) => setState({ ...s, at: performance.now() }))
      .catch(() => undefined);
    const off = listen<DrawState>("xs://livedraw", (e) =>
      setState({ ...e.payload, at: performance.now() }),
    );
    return () => void off.then((f) => f());
  }, []);
  return state;
}

/** The rectangle between two corners; a square or circle with Shift. */
function box(a: [number, number], b: [number, number], even: boolean) {
  let w = b[0] - a[0];
  let h = b[1] - a[1];
  if (even) {
    const side = Math.max(Math.abs(w), Math.abs(h));
    w = Math.sign(w || 1) * side;
    h = Math.sign(h || 1) * side;
  }
  return {
    x: Math.min(a[0], a[0] + w),
    y: Math.min(a[1], a[1] + h),
    w: Math.abs(w),
    h: Math.abs(h),
  };
}

function ShapeView({ s, k }: { s: Shown; k: number }) {
  const pts = s.points.map(([x, y]) => [x / k, y / k] as [number, number]);
  const w = s.width / k;
  const first = pts[0] ?? [0, 0];
  const last = pts[pts.length - 1] ?? first;
  const common = {
    stroke: s.color,
    strokeWidth: w,
    fill: "none",
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
  };
  switch (s.tool) {
    case "arrow": {
      const [x0, y0] = first;
      const [x1, y1] = last;
      const len = Math.hypot(x1 - x0, y1 - y0) || 1;
      const [ux, uy] = [(x1 - x0) / len, (y1 - y0) / len];
      const head = Math.min(len * 0.6, w * 3.5 + 8);
      const half = head * 0.55;
      const [bx, by] = [x1 - ux * head, y1 - uy * head];
      return (
        <g>
          <line x1={x0} y1={y0} x2={bx + ux * 1} y2={by + uy * 1} {...common} />
          <polygon
            points={`${x1},${y1} ${bx - uy * half},${by + ux * half} ${bx + uy * half},${by - ux * half}`}
            fill={s.color}
            stroke={s.color}
            strokeWidth={1}
            strokeLinejoin="round"
          />
        </g>
      );
    }
    case "rect": {
      const b = box(first, last, s.even);
      return <rect x={b.x} y={b.y} width={b.w} height={b.h} rx={w} {...common} />;
    }
    case "oval": {
      const b = box(first, last, s.even);
      return (
        <ellipse cx={b.x + b.w / 2} cy={b.y + b.h / 2} rx={b.w / 2} ry={b.h / 2} {...common} />
      );
    }
    case "pen":
      return <polyline points={pts.map((p) => p.join(",")).join(" ")} {...common} />;
    case "step": {
      const r = 14 + w;
      return (
        <g>
          <circle cx={first[0]} cy={first[1]} r={r} fill={s.color} />
          <text
            x={first[0]}
            y={first[1]}
            fill="#fff"
            fontSize={r * 1.15}
            fontWeight={700}
            textAnchor="middle"
            dominantBaseline="central"
          >
            {s.number ?? ""}
          </text>
        </g>
      );
    }
  }
}

/** The overlay: every shape, fading by itself (VID-050). */
export function LiveDrawOverlay() {
  const state = useDrawState();
  const [now, setNow] = useState(() => performance.now());
  const frame = useRef(0);
  // Each shape's fade on this window's clock.
  const at = state.at ?? 0;
  const shapes: Shown[] = (state.shapes ?? []).map((s) => ({
    ...s,
    fadeAt: s.fadeIn === null ? null : at + s.fadeIn * 1000,
  }));
  const fading = shapes.some((s) => s.fadeAt !== null);

  // While something fades, a frame at a time.
  useEffect(() => {
    if (!fading) return;
    const tick = () => {
      setNow(performance.now());
      frame.current = requestAnimationFrame(tick);
    };
    frame.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame.current);
  }, [fading, state]);

  const k = window.devicePixelRatio || 1;
  const out = (state.fadeOut ?? 0.5) * 1000;
  return (
    <svg className="livedraw" aria-hidden="true">
      {shapes.map((s) => {
        const opacity =
          s.fadeAt === null || now < s.fadeAt ? 1 : Math.max(0, 1 - (now - s.fadeAt) / out);
        return opacity > 0 ? (
          <g key={s.id} opacity={opacity}>
            <ShapeView s={s} k={k} />
          </g>
        ) : null;
      })}
    </svg>
  );
}

const TOOLS: Tool[] = ["arrow", "rect", "oval", "pen", "step"];
const FADES = [0, 1, 2, 3, 5, 10, 20, 30];

/** The toolbar (VID-049, VID-050): tools, colors, thickness and fading. */
export function LiveDrawBar() {
  const { t } = useTranslation();
  const state = useDrawState();
  const set = (p: Record<string, unknown>) => void run("record.draw", p).catch(() => undefined);
  const palette = state.palette ?? [];
  return (
    <div
      className="drawbar"
      data-tauri-drag-region
      role="toolbar"
      aria-label={t("ui.livedraw.title")}
    >
      {TOOLS.map((tool) => (
        <button
          key={tool}
          type="button"
          className="btn small drawbar-tool"
          aria-pressed={state.tool === tool}
          title={t(`ui.livedraw.${tool}`)}
          aria-label={t(`ui.livedraw.${tool}`)}
          onClick={() => set({ tool })}
        >
          <ToolGlyph tool={tool} />
        </button>
      ))}
      <span className="drawbar-sep" />
      {palette.slice(0, 6).map((c, i) => (
        <button
          key={c + i}
          type="button"
          className="drawbar-color"
          aria-pressed={state.color?.toLowerCase() === c.toLowerCase()}
          title={t("ui.livedraw.color", { n: i + 1, key: i + 2 })}
          aria-label={t("ui.livedraw.color", { n: i + 1, key: i + 2 })}
          style={{ background: c }}
          onClick={() => set({ color: c })}
        />
      ))}
      <span className="drawbar-sep" />
      <select
        value={state.width ?? "medium"}
        aria-label={t("ui.livedraw.width")}
        title={t("ui.livedraw.width")}
        onChange={(e) => set({ width: e.target.value })}
      >
        {["thin", "medium", "thick"].map((w) => (
          <option key={w} value={w}>
            {t(`ui.livedraw.${w}`)}
          </option>
        ))}
      </select>
      <select
        value={state.fade ?? 0}
        aria-label={t("ui.livedraw.fade")}
        title={t("ui.livedraw.fade")}
        onChange={(e) => set({ fade: Number(e.target.value) })}
      >
        {FADES.map((f) => (
          <option key={f} value={f}>
            {f === 0 ? t("ui.livedraw.never") : t("ui.livedraw.seconds", { n: f })}
          </option>
        ))}
      </select>
      <button
        type="button"
        className="btn small"
        title={t("ui.livedraw.undo")}
        aria-label={t("ui.livedraw.undo")}
        onClick={() => set({ undo: true })}
      >
        ↶
      </button>
      <button
        type="button"
        className="btn small"
        title={t("ui.livedraw.clear")}
        aria-label={t("ui.livedraw.clear")}
        onClick={() => set({ clear: true })}
      >
        ⌫
      </button>
      <button
        type="button"
        className="btn small"
        title={t("ui.livedraw.exit")}
        aria-label={t("ui.livedraw.exit")}
        onClick={() => set({ on: false })}
      >
        ✕
      </button>
    </div>
  );
}

function ToolGlyph({ tool }: { tool: Tool }) {
  const p = {
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 1.8,
    strokeLinecap: "round" as const,
  };
  return (
    <svg width="18" height="18" viewBox="0 0 18 18" aria-hidden="true">
      {tool === "arrow" && <path d="M3 15L14 4M8 4h6v6" {...p} />}
      {tool === "rect" && <rect x="3" y="4" width="12" height="10" rx="1.5" {...p} />}
      {tool === "oval" && <ellipse cx="9" cy="9" rx="6.5" ry="5" {...p} />}
      {tool === "pen" && <path d="M3 13c3-6 5 2 8-3s3-4 4-5" {...p} />}
      {tool === "step" && (
        <>
          <circle cx="9" cy="9" r="6.5" {...p} />
          <path d="M8 6.5l1.5-1v7" {...p} />
        </>
      )}
    </svg>
  );
}

// A fill: a color or a gradient (EDT-020). Solid uses the color selector (EDT-019);
// Gradient edits the type, the angle (field and dial), the stops on a bar (click adds,
// drag moves, drag away removes), Invert, and My gradients.
import { useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import { useTranslation } from "react-i18next";
import { useIndicator } from "../components/useIndicator";
import { run } from "../lib/commands";
import { ColorPicker } from "./ColorPicker";
import { parseHex, toHex } from "./color";
import { useEditor, type Fill, type Gradient } from "./state";

/** CSS for a gradient preview (a bar or a swatch). */
function gradientCss(g: Gradient, bar = false): string {
  const stops = [...g.stops]
    .sort((a, b) => a.pos - b.pos)
    .map((s) => `${s.color} ${s.pos}%`)
    .join(", ");
  if (bar) return `linear-gradient(90deg, ${stops})`;
  return g.type === "radial"
    ? `radial-gradient(circle, ${stops})`
    : `linear-gradient(${(g.angle + 90) % 360}deg, ${stops})`;
}

/** The color of a gradient at `pos` (0–100), between its stops. */
function colorAt(g: Gradient, pos: number): string {
  const stops = [...g.stops].sort((a, b) => a.pos - b.pos);
  const after = stops.find((s) => s.pos >= pos);
  const before = [...stops].reverse().find((s) => s.pos <= pos);
  if (!before) return after?.color ?? "#FFFFFF";
  if (!after || after === before) return before.color;
  const a = parseHex(before.color);
  const b = parseHex(after.color);
  if (!a || !b) return before.color;
  const k = (pos - before.pos) / (after.pos - before.pos || 1);
  const mix = (x: number, y: number) => Math.round(x + (y - x) * k);
  return toHex({ r: mix(a.r, b.r), g: mix(a.g, b.g), b: mix(a.b, b.b), a: mix(a.a, b.a) });
}

/** A gradient from a color, to start from (EDT-020). */
function fromColor(c: string | null): Gradient {
  return {
    type: "linear",
    angle: 90,
    stops: [
      { color: c ?? "#2D7FF9", pos: 0 },
      { color: "#FFFFFF", pos: 100 },
    ],
  };
}

export function FillPicker(props: {
  label: string;
  value: Fill | null;
  mixed?: boolean;
  allowTransparent?: boolean;
  onCommit: (value: Fill | null) => void;
}) {
  const { t } = useTranslation();
  const { label, value, mixed, allowTransparent, onCommit } = props;
  const gradient = value !== null && typeof value === "object" ? value : null;
  const seg = useRef<HTMLDivElement>(null);
  useIndicator(seg, gradient ? "gradient" : "solid");
  return (
    <div className="fill-picker">
      <div className="fill-head">
        <span className="prop-label">{label}</span>
        <div className="seg small" role="radiogroup" aria-label={label} ref={seg}>
          <span className="seg-indicator" aria-hidden="true" />
          <button
            type="button"
            role="radio"
            aria-checked={!gradient}
            onClick={() => gradient && onCommit(gradient.stops[0]?.color ?? null)}
          >
            {t("ui.editor.gradient.solid")}
          </button>
          <button
            type="button"
            role="radio"
            aria-checked={Boolean(gradient)}
            onClick={() =>
              !gradient && onCommit(fromColor(typeof value === "string" ? value : null))
            }
          >
            {t("ui.editor.gradient.gradient")}
          </button>
        </div>
      </div>
      {gradient ? (
        <GradientEditor value={gradient} onCommit={onCommit} />
      ) : (
        <ColorPicker
          label={label}
          value={typeof value === "string" ? value : null}
          mixed={mixed ?? false}
          allowTransparent={allowTransparent ?? false}
          onCommit={onCommit}
        />
      )}
    </div>
  );
}

function GradientEditor({ value, onCommit }: { value: Gradient; onCommit: (g: Gradient) => void }) {
  const { t } = useTranslation();
  const mine = useEditor((s) => s.colors.gradients ?? []);
  const [selected, setSelected] = useState(0);
  // A stop being dragged: where it is now, before the change is sent.
  const [draft, setDraft] = useState<Gradient | null>(null);
  const g = draft ?? value;
  const bar = useRef<HTMLDivElement>(null);
  const typeSeg = useRef<HTMLDivElement>(null);
  useIndicator(typeSeg, g.type);
  const index = Math.min(selected, g.stops.length - 1);
  const stop = g.stops[index];
  const set = (next: Partial<Gradient>) => onCommit({ ...g, ...next });
  const setStop = (i: number, s: Partial<Gradient["stops"][number]>) =>
    set({ stops: g.stops.map((x, j) => (j === i ? { ...x, ...s } : x)) });
  const posOf = (clientX: number) => {
    const r = bar.current?.getBoundingClientRect();
    if (!r) return 0;
    return Math.round(Math.min(100, Math.max(0, ((clientX - r.left) / r.width) * 100)));
  };
  const onBar = (e: ReactPointerEvent<HTMLDivElement>) => {
    if ((e.target as HTMLElement).closest(".grad-stop")) return;
    if (g.stops.length >= 8) return;
    const pos = posOf(e.clientX);
    const stops = [...g.stops, { color: colorAt(g, pos), pos }];
    setSelected(stops.length - 1);
    set({ stops });
  };
  const dragStop = (i: number, e: ReactPointerEvent<HTMLButtonElement>) => {
    e.preventDefault();
    setSelected(i);
    const startY = e.clientY;
    const el = e.currentTarget;
    el.setPointerCapture(e.pointerId);
    let current = g;
    let away = false;
    const move = (ev: PointerEvent) => {
      // Dragged out of the bar: the stop goes (if two stay).
      away = Math.abs(ev.clientY - startY) > 30 && g.stops.length > 2;
      current = away
        ? { ...g, stops: g.stops.filter((_, j) => j !== i) }
        : { ...g, stops: g.stops.map((s, j) => (j === i ? { ...s, pos: posOf(ev.clientX) } : s)) };
      setDraft(current);
    };
    const up = () => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerup", up);
      setDraft(null);
      if (away) setSelected(0);
      if (JSON.stringify(current) !== JSON.stringify(g)) onCommit(current);
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerup", up);
  };
  const dial = (e: ReactPointerEvent<SVGSVGElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    const a =
      (Math.atan2(e.clientY - (r.top + r.height / 2), e.clientX - (r.left + r.width / 2)) * 180) /
      Math.PI;
    set({ angle: Math.round((a + 360) % 360) });
  };
  const rad = (g.angle * Math.PI) / 180;
  return (
    <div className="grad-editor">
      <div
        className="seg small"
        role="radiogroup"
        aria-label={t("ui.editor.gradient.type")}
        ref={typeSeg}
      >
        <span className="seg-indicator" aria-hidden="true" />
        {(["linear", "radial"] as const).map((k) => (
          <button
            key={k}
            type="button"
            role="radio"
            aria-checked={g.type === k}
            onClick={() => set({ type: k })}
          >
            {t(`ui.editor.gradient.${k}`)}
          </button>
        ))}
      </div>
      {g.type === "linear" && (
        <div className="grad-angle">
          <label className="prop-field">
            <span>{t("ui.editor.gradient.angle")}</span>
            <input
              type="number"
              min={0}
              max={359}
              value={g.angle}
              onChange={(e) => {
                const v = Number(e.target.value);
                if (Number.isFinite(v)) set({ angle: ((Math.round(v) % 360) + 360) % 360 });
              }}
            />
          </label>
          <svg
            className="grad-dial"
            viewBox="0 0 36 36"
            role="slider"
            aria-label={t("ui.editor.gradient.angle")}
            aria-valuemin={0}
            aria-valuemax={359}
            aria-valuenow={g.angle}
            tabIndex={0}
            onPointerDown={dial}
            onKeyDown={(e) => {
              if (e.key === "ArrowRight" || e.key === "ArrowUp")
                set({ angle: (g.angle + 15) % 360 });
              if (e.key === "ArrowLeft" || e.key === "ArrowDown")
                set({ angle: (g.angle + 345) % 360 });
            }}
          >
            <circle cx="18" cy="18" r="15" />
            <line x1="18" y1="18" x2={18 + Math.cos(rad) * 13} y2={18 + Math.sin(rad) * 13} />
          </svg>
        </div>
      )}
      <div
        className="grad-bar"
        ref={bar}
        style={{ background: gradientCss(g, true) }}
        onPointerDown={onBar}
        title={t("ui.editor.gradient.bar_help")}
      >
        {g.stops.map((s, i) => (
          <button
            key={i}
            type="button"
            className="grad-stop"
            aria-pressed={i === index}
            aria-label={t("ui.editor.gradient.stop", { n: i + 1, pos: s.pos })}
            style={{ left: `${s.pos}%`, background: s.color }}
            onPointerDown={(e) => dragStop(i, e)}
            onClick={() => setSelected(i)}
          />
        ))}
      </div>
      {stop && (
        <div className="grad-stop-props">
          <ColorPicker
            label={t("ui.editor.gradient.stop_color")}
            value={stop.color}
            onCommit={(c) => c && setStop(index, { color: c })}
          />
          <label className="prop-field">
            <span>{t("ui.editor.gradient.position")}</span>
            <input
              type="number"
              min={0}
              max={100}
              value={stop.pos}
              onChange={(e) => {
                const v = Number(e.target.value);
                if (Number.isFinite(v))
                  setStop(index, { pos: Math.min(100, Math.max(0, Math.round(v))) });
              }}
            />
          </label>
        </div>
      )}
      <div className="grad-actions">
        <button
          type="button"
          className="btn small"
          onClick={() => set({ stops: g.stops.map((s) => ({ ...s, pos: 100 - s.pos })) })}
        >
          {t("ui.editor.gradient.invert")}
        </button>
        <button
          type="button"
          className="btn small"
          disabled={g.stops.length <= 2}
          onClick={() => {
            setSelected(0);
            set({ stops: g.stops.filter((_, j) => j !== index) });
          }}
        >
          {t("ui.editor.gradient.remove_stop")}
        </button>
      </div>
      <div className="cp-label">{t("ui.editor.gradient.mine")}</div>
      <div className="swatches">
        {mine.map((m, i) => (
          <button
            key={i}
            type="button"
            className="swatch"
            style={{ background: gradientCss(m) }}
            aria-label={`${t("ui.editor.gradient.mine")} ${i + 1}`}
            title={t("ui.editor.gradient.mine_help")}
            onClick={() => onCommit(m)}
            onContextMenu={(e) => {
              e.preventDefault();
              void run("editor.color", { action: "remove_gradient", gradient: m }).catch(
                () => undefined,
              );
            }}
            onKeyDown={(e) => {
              if (e.key === "Delete" || e.key === "Backspace") {
                e.preventDefault();
                void run("editor.color", { action: "remove_gradient", gradient: m }).catch(
                  () => undefined,
                );
              }
            }}
          />
        ))}
        <button
          type="button"
          className="swatch add"
          aria-label={t("ui.editor.gradient.save")}
          title={t("ui.editor.gradient.save")}
          disabled={mine.length >= 16}
          onClick={() =>
            void run("editor.color", { action: "save_gradient", gradient: g }).catch(
              () => undefined,
            )
          }
        >
          +
        </button>
      </div>
    </div>
  );
}

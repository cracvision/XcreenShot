// The color selector (EDT-019): theme colors, a palette of 40, a hue and saturation
// area with a brightness bar, Hex, RGB and HSB fields, opacity, the eyedropper and,
// where it makes sense, Transparent. A swatch applies at once; Enter or a click
// outside accepts the rest, and Esc cancels.
import { useEffect, useRef, useState, type CSSProperties } from "react";
import { useTranslation } from "react-i18next";
import { hsbToRgb, PALETTE, parseHex, rgbToHsb, THEME_COLORS, toHex, type Hsb } from "./color";

interface EyeDropperResult {
  sRGBHex: string;
}
interface EyeDropperApi {
  open(): Promise<EyeDropperResult>;
}
declare global {
  interface Window {
    EyeDropper?: new () => EyeDropperApi;
  }
}

interface Props {
  label: string;
  /** `#RRGGBB(AA)`, or null for Transparent. */
  value: string | null;
  /** Offer Transparent (fills, canvas color; never an arrow, EDT-019). */
  allowTransparent?: boolean;
  /** Shown instead of the value when the selected objects differ (EDT-014). */
  mixed?: boolean;
  onCommit: (value: string | null) => void;
}

const checkerBg: CSSProperties = {
  backgroundImage:
    "linear-gradient(45deg, #c8c8c8 25%, transparent 25%, transparent 75%, #c8c8c8 75%), linear-gradient(45deg, #c8c8c8 25%, #e6e6e6 25%, #e6e6e6 75%, #c8c8c8 75%)",
  backgroundSize: "8px 8px",
  backgroundPosition: "0 0, 4px 4px",
};

function Swatch({ color, label, onPick }: { color: string; label: string; onPick: () => void }) {
  return (
    <button
      type="button"
      className="swatch"
      style={{ background: color }}
      title={label}
      aria-label={label}
      onClick={onPick}
    />
  );
}

export function ColorPicker({ label, value, allowTransparent, mixed, onCommit }: Props) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const anchor = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  // Draft while the selector is open: HSB keeps the hue of grays.
  const start = parseHex(value ?? "#000000") ?? { r: 0, g: 0, b: 0, a: 255 };
  const [hsb, setHsb] = useState<Hsb>(rgbToHsb(start));
  const [alpha, setAlpha] = useState(start.a);
  const [transparent, setTransparent] = useState(value === null);
  const [hexDraft, setHexDraft] = useState<string | null>(null);
  // Fixed to the window, so the side panel's scrolling never cuts it.
  const [place, setPlace] = useState<CSSProperties>({});

  const rgb = hsbToRgb(hsb, alpha);
  const hex = toHex(rgb);
  const draftValue = transparent ? null : hex;

  const reset = () => {
    const c = parseHex(value ?? "#000000") ?? { r: 0, g: 0, b: 0, a: 255 };
    setHsb(rgbToHsb(c));
    setAlpha(c.a);
    setTransparent(value === null);
    setHexDraft(null);
  };

  const close = (accept: boolean, refocus = true) => {
    setOpen(false);
    if (refocus) button.current?.focus();
    if (accept && draftValue !== value) onCommit(draftValue);
  };

  // A click anywhere outside accepts what is chosen. Pointer events, in the capture
  // phase: the canvas cancels the mouse events of its clicks.
  useEffect(() => {
    if (!open) return;
    const onDown = (e: PointerEvent) => {
      if (!anchor.current?.contains(e.target as Node)) close(true, false);
    };
    window.addEventListener("pointerdown", onDown, true);
    return () => window.removeEventListener("pointerdown", onDown, true);
  });

  /** A swatch: that color, at once. */
  const choose = (text: string) => {
    const c = parseHex(text);
    if (!c) return;
    setOpen(false);
    button.current?.focus();
    const chosen = toHex(c);
    if (chosen !== value) onCommit(chosen);
  };

  const pick = (text: string) => {
    const c = parseHex(text);
    if (!c) return;
    setHsb(rgbToHsb(c));
    setAlpha(c.a);
    setTransparent(false);
    setHexDraft(null);
  };

  const area = useRef<HTMLDivElement>(null);
  const fromArea = (e: React.PointerEvent) => {
    const r = area.current?.getBoundingClientRect();
    if (!r) return;
    const s = Math.min(100, Math.max(0, ((e.clientX - r.left) / r.width) * 100));
    const b = Math.min(100, Math.max(0, (1 - (e.clientY - r.top) / r.height) * 100));
    setHsb((h) => ({ ...h, s, b }));
    setTransparent(false);
  };

  const eyedropper = async () => {
    if (!window.EyeDropper) return;
    try {
      const { sRGBHex } = await new window.EyeDropper().open();
      pick(sRGBHex);
    } catch {
      // Cancelled with Esc: nothing changes.
    }
  };

  const num = (v: number, min: number, max: number, set: (n: number) => void, name: string) => (
    <label className="cp-num">
      <span>{name}</span>
      <input
        type="number"
        min={min}
        max={max}
        value={Math.round(v)}
        onChange={(e) => {
          const n = Number(e.target.value);
          if (Number.isFinite(n)) set(Math.min(max, Math.max(min, n)));
          setTransparent(false);
        }}
      />
    </label>
  );

  const shownStyle: CSSProperties =
    value === null || mixed ? checkerBg : { background: value, ...checkerBg };

  return (
    <div className="prop-field" ref={anchor}>
      <span id={`${label}-label`}>{label}</span>
      <button
        ref={button}
        type="button"
        className="color-button"
        aria-haspopup="dialog"
        aria-expanded={open}
        aria-labelledby={`${label}-label`}
        onClick={() => {
          if (open) close(true);
          else {
            reset();
            const r = button.current?.getBoundingClientRect();
            if (r) {
              setPlace({
                top: Math.max(8, Math.min(r.bottom + 4, window.innerHeight - 540)),
                right: Math.max(8, window.innerWidth - r.right),
              });
            }
            setOpen(true);
          }
        }}
      >
        <span className="color-chip" style={shownStyle}>
          {value !== null && !mixed && <span style={{ background: value }} />}
        </span>
        <span className="color-text">
          {mixed ? t("ui.editor.mixed") : value === null ? t("ui.editor.transparent") : value}
        </span>
      </button>
      {open && (
        <div
          className="color-popover"
          style={place}
          role="dialog"
          aria-label={label}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              e.preventDefault();
              e.stopPropagation();
              close(false);
            } else if (e.key === "Enter" && !(e.target instanceof HTMLButtonElement)) {
              e.preventDefault();
              close(true);
            }
          }}
        >
          <div className="cp-label">{t("ui.editor.color.theme")}</div>
          <div className="swatches">
            {THEME_COLORS.map((c) => (
              <Swatch key={c} color={c} label={c} onPick={() => choose(c)} />
            ))}
          </div>
          <div className="cp-label">{t("ui.editor.color.palette")}</div>
          <div className="cp-palette">
            {PALETTE.map((c) => (
              <Swatch key={c} color={c} label={c} onPick={() => choose(c)} />
            ))}
          </div>
          <div
            ref={area}
            className="cp-area"
            role="slider"
            tabIndex={0}
            aria-label={t("ui.editor.color.area")}
            aria-valuetext={`${Math.round(hsb.s)} %, ${Math.round(hsb.b)} %`}
            style={{ background: `hsl(${hsb.h}, 100%, 50%)` }}
            onPointerDown={(e) => {
              e.currentTarget.setPointerCapture(e.pointerId);
              fromArea(e);
            }}
            onPointerMove={(e) => e.buttons === 1 && fromArea(e)}
            onKeyDown={(e) => {
              const step = e.shiftKey ? 10 : 1;
              const d: Record<string, [number, number]> = {
                ArrowLeft: [-step, 0],
                ArrowRight: [step, 0],
                ArrowUp: [0, step],
                ArrowDown: [0, -step],
              };
              const m = d[e.key];
              if (!m) return;
              e.preventDefault();
              setHsb((h) => ({
                ...h,
                s: Math.min(100, Math.max(0, h.s + m[0])),
                b: Math.min(100, Math.max(0, h.b + m[1])),
              }));
            }}
          >
            <span
              className="cp-dot"
              style={{
                left: `${hsb.s}%`,
                top: `${100 - hsb.b}%`,
                background: toHex({ ...rgb, a: 255 }),
              }}
            />
          </div>
          <input
            className="cp-hue"
            type="range"
            min={0}
            max={359}
            value={Math.round(hsb.h)}
            aria-label={t("ui.editor.color.hue")}
            onChange={(e) => {
              setHsb((h) => ({ ...h, h: Number(e.target.value) }));
              setTransparent(false);
            }}
          />
          <label className="cp-row">
            <span>{t("ui.editor.prop.opacity")}</span>
            <input
              type="range"
              min={0}
              max={100}
              value={Math.round((alpha / 255) * 100)}
              onChange={(e) => setAlpha(Math.round((Number(e.target.value) / 100) * 255))}
            />
            <span>{Math.round((alpha / 255) * 100)} %</span>
          </label>
          <div className="cp-fields">
            <label className="cp-num cp-hex">
              <span>Hex</span>
              <input
                type="text"
                value={hexDraft ?? (transparent ? "" : hex)}
                onChange={(e) => {
                  setHexDraft(e.target.value);
                  const c = parseHex(e.target.value);
                  if (c) {
                    setHsb(rgbToHsb(c));
                    setAlpha(c.a);
                    setTransparent(false);
                  }
                }}
                onBlur={() => setHexDraft(null)}
              />
            </label>
            {num(rgb.r, 0, 255, (r) => setHsb(rgbToHsb({ ...rgb, r })), "R")}
            {num(rgb.g, 0, 255, (g) => setHsb(rgbToHsb({ ...rgb, g })), "G")}
            {num(rgb.b, 0, 255, (b) => setHsb(rgbToHsb({ ...rgb, b })), "B")}
            {num(hsb.h, 0, 359, (h) => setHsb((x) => ({ ...x, h })), "H")}
            {num(hsb.s, 0, 100, (s) => setHsb((x) => ({ ...x, s })), "S")}
            {num(
              hsb.b,
              0,
              100,
              (b) => setHsb((x) => ({ ...x, b })),
              t("ui.editor.color.brightness_short"),
            )}
          </div>
          <div className="cp-actions">
            {window.EyeDropper && (
              <button type="button" className="btn small" onClick={() => void eyedropper()}>
                {t("ui.editor.color.eyedropper")}
              </button>
            )}
            {allowTransparent && (
              <button
                type="button"
                className="btn small"
                aria-pressed={transparent}
                onClick={() => setTransparent((v) => !v)}
              >
                {t("ui.editor.transparent")}
              </button>
            )}
            <span className="grow" />
            <button type="button" className="btn small" onClick={() => close(false)}>
              {t("ui.editor.cancel")}
            </button>
            <button type="button" className="btn small primary" onClick={() => close(true)}>
              {t("ui.editor.accept")}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

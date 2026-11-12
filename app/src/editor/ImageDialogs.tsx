// The Image menu's dialogs (EDT-069 to EDT-077) and Revert to original (EDT-076). Each
// one runs a document.* command; Undo takes it back.
import { useRef, useState, type KeyboardEvent, type PointerEvent } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { useIndicator } from "../components/useIndicator";
import { run } from "../lib/commands";
import { FillPicker } from "./FillPicker";
import {
  ANCHORS,
  COMMON_DPI,
  UNITS,
  aboutSize,
  angleOf,
  closeImageDialog,
  fromPx,
  keepRatio,
  shown,
  toPx,
  QUICK_SIZES,
  useClipboard,
  useImageDialog,
  wheelAngle,
  type Anchor,
  type Unit,
} from "./imageMenu";
import { useEditor, xsdocUrl, type DocDetails, type Fill } from "./state";
import { useApp } from "../store";

const UNIT_KEY = "xs.editor.sizeUnit";

function lastUnit(): Unit {
  try {
    const v = localStorage.getItem(UNIT_KEY);
    if (v && (UNITS as string[]).includes(v)) return v as Unit;
  } catch {
    // Storage unavailable: pixels.
  }
  return "px";
}

function rememberUnit(u: Unit): void {
  try {
    localStorage.setItem(UNIT_KEY, u);
  } catch {
    // Not remembered: no harm.
  }
}

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

/** Runs `command` on the document and closes the dialog. */
function apply(command: string, doc: DocDetails, params: Record<string, unknown>): void {
  closeImageDialog();
  run(command, { documentId: doc.documentId, ...params }).catch(fail);
}

/** A number typed freely; the value counts from the first valid number. */
function NumberInput(props: {
  id: string;
  label: string;
  value: string;
  disabled?: boolean;
  autoFocus?: boolean;
  suffix?: string;
  onValue: (v: number) => void;
}) {
  const [draft, setDraft] = useState<string | null>(null);
  return (
    <div className="prop-field">
      <label htmlFor={props.id}>{props.label}</label>
      <span className="image-number">
        <input
          id={props.id}
          type="text"
          inputMode="decimal"
          value={draft ?? props.value}
          disabled={props.disabled}
          data-autofocus={props.autoFocus ? "" : undefined}
          onChange={(e) => {
            setDraft(e.target.value);
            const v = Number(e.target.value.replace(",", "."));
            if (e.target.value.trim() !== "" && Number.isFinite(v)) props.onValue(v);
          }}
          onBlur={() => setDraft(null)}
        />
        {props.suffix && <span className="muted">{props.suffix}</span>}
      </span>
    </div>
  );
}

function UnitSelect({
  value,
  onChange,
  units = UNITS,
}: {
  value: Unit;
  onChange: (u: Unit) => void;
  units?: Unit[];
}) {
  const { t } = useTranslation();
  return (
    <div className="prop-field">
      <label htmlFor="image-unit">{t("ui.editor.image.units")}</label>
      <select
        id="image-unit"
        value={value}
        onChange={(e) => {
          const u = e.target.value as Unit;
          rememberUnit(u);
          onChange(u);
        }}
      >
        {units.map((u) => (
          <option key={u} value={u}>
            {t(`ui.editor.image.unit_${u}`)}
          </option>
        ))}
      </select>
    </div>
  );
}

function suffixOf(unit: Unit): string {
  return unit === "percent" ? "%" : unit;
}

/** Image > Resize image (EDT-069). */
function Resize({ doc }: { doc: DocDetails }) {
  const { t, i18n } = useTranslation();
  const w0 = doc.canvas.width;
  const h0 = doc.canvas.height;
  const [unit, setUnit] = useState<Unit>(lastUnit);
  const [w, setW] = useState(w0);
  const [h, setH] = useState(h0);
  const [lock, setLock] = useState(true);
  const [dpi, setDpi] = useState(Math.round(doc.dpi * 100) / 100);
  const [resample, setResample] = useState("auto");
  const [resampleImage, setResampleImage] = useState(true);
  // Without resampling the pixels stay; inches and centimeters change the resolution.
  const physical = unit === "in" || unit === "cm";
  const fixed = !resampleImage;
  const sideIn = (px: number, whole: number) => shown(fromPx(px, unit, whole, dpi), unit);
  const setSide = (which: "w" | "h", v: number) => {
    if (fixed) {
      if (!physical || v <= 0) return;
      const inches = unit === "in" ? v : v / 2.54;
      setDpi(Math.round(((which === "w" ? w0 : h0) / inches) * 100) / 100);
      return;
    }
    const px = Math.max(1, toPx(v, unit, which === "w" ? w0 : h0, dpi));
    if (which === "w") {
      setW(px);
      if (lock) setH(Math.max(1, keepRatio(px, w0, h0)));
    } else {
      setH(px);
      if (lock) setW(Math.max(1, keepRatio(px, h0, w0)));
    }
  };
  const W = fixed ? w0 : Math.round(w);
  const H = fixed ? h0 : Math.round(h);
  const enlarging = !fixed && (W > w0 || H > h0);
  const valid = W >= 1 && H >= 1 && W <= 50_000 && H <= 50_000 && dpi >= 1 && dpi <= 10_000;
  const ok = () =>
    apply(
      "document.resize",
      doc,
      fixed ? { width: w0, resampleImage: false, dpi } : { width: W, height: H, resample, dpi },
    );
  return (
    <Dialog
      title={t("ui.editor.image.resize_title")}
      onCancel={closeImageDialog}
      actions={
        <>
          <button type="button" className="btn primary" disabled={!valid} onClick={ok}>
            {t("ui.editor.image.apply")}
          </button>
          <button type="button" className="btn" onClick={closeImageDialog}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <div className="image-form">
        <UnitSelect value={unit} onChange={setUnit} />
        <div className="image-sides">
          <NumberInput
            id="image-w"
            label={t("ui.editor.image.width")}
            value={sideIn(fixed ? w0 : w, w0)}
            disabled={fixed && !physical}
            autoFocus
            suffix={suffixOf(unit)}
            onValue={(v) => setSide("w", v)}
          />
          <NumberInput
            id="image-h"
            label={t("ui.editor.image.height")}
            value={sideIn(fixed ? h0 : h, h0)}
            disabled={fixed && !physical}
            suffix={suffixOf(unit)}
            onValue={(v) => setSide("h", v)}
          />
        </div>
        <label className="panel-check">
          <input
            type="checkbox"
            checked={lock || fixed}
            disabled={fixed}
            onChange={(e) => setLock(e.target.checked)}
          />
          {t("ui.editor.image.lock")}
        </label>
        <NumberInput
          id="image-dpi"
          label={t("ui.editor.image.resolution")}
          value={String(dpi)}
          suffix={t("ui.editor.image.dpi_unit")}
          onValue={(v) => v > 0 && setDpi(v)}
        />
        <div className="prop-field">
          <label htmlFor="image-resample">{t("ui.editor.image.resample")}</label>
          <select
            id="image-resample"
            value={resample}
            disabled={fixed}
            onChange={(e) => setResample(e.target.value)}
          >
            {["auto", "bicubic", "bilinear", "lanczos", "nearest"].map((r) => (
              <option key={r} value={r}>
                {t(`ui.editor.image.resample_${r}`)}
              </option>
            ))}
          </select>
        </div>
        <label className="panel-check">
          <input
            type="checkbox"
            checked={resampleImage}
            onChange={(e) => setResampleImage(e.target.checked)}
          />
          {t("ui.editor.image.resample_image")}
        </label>
      </div>
      <p className="muted" aria-live="polite">
        {t("ui.editor.image.result", { w: W, h: H, size: aboutSize(W, H, i18n.language) })}
      </p>
      {enlarging && <p className="warning-text">{t("ui.editor.image.enlarge_warning")}</p>}
    </Dialog>
  );
}

/** Image > Canvas size (EDT-070): a size and the anchor that stays put. */
function CanvasSize({ doc }: { doc: DocDetails }) {
  const { t } = useTranslation();
  const w0 = doc.canvas.width;
  const h0 = doc.canvas.height;
  const [unit, setUnit] = useState<Unit>(lastUnit);
  const [w, setW] = useState(w0);
  const [h, setH] = useState(h0);
  const [lock, setLock] = useState(false);
  const [anchor, setAnchor] = useState<Anchor>("center");
  const setSide = (which: "w" | "h", v: number) => {
    const px = Math.max(1, toPx(v, unit, which === "w" ? w0 : h0, doc.dpi));
    if (which === "w") {
      setW(px);
      if (lock) setH(Math.max(1, keepRatio(px, w0, h0)));
    } else {
      setH(px);
      if (lock) setW(Math.max(1, keepRatio(px, h0, w0)));
    }
  };
  const W = Math.round(w);
  const H = Math.round(h);
  const valid = W >= 1 && H >= 1 && W <= 50_000 && H <= 50_000;
  return (
    <Dialog
      title={t("ui.editor.image.canvas_title")}
      onCancel={closeImageDialog}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            disabled={!valid}
            onClick={() => apply("document.canvas_size", doc, { width: W, height: H, anchor })}
          >
            {t("ui.editor.image.apply")}
          </button>
          <button type="button" className="btn" onClick={closeImageDialog}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <div className="image-form">
        <UnitSelect value={unit} onChange={setUnit} />
        <div className="image-sides">
          <NumberInput
            id="canvas-w"
            label={t("ui.editor.image.width")}
            value={shown(fromPx(w, unit, w0, doc.dpi), unit)}
            autoFocus
            suffix={suffixOf(unit)}
            onValue={(v) => setSide("w", v)}
          />
          <NumberInput
            id="canvas-h"
            label={t("ui.editor.image.height")}
            value={shown(fromPx(h, unit, h0, doc.dpi), unit)}
            suffix={suffixOf(unit)}
            onValue={(v) => setSide("h", v)}
          />
        </div>
        <label className="panel-check">
          <input type="checkbox" checked={lock} onChange={(e) => setLock(e.target.checked)} />
          {t("ui.editor.image.lock")}
        </label>
        <fieldset className="shadow-section">
          <legend>{t("ui.editor.image.anchor")}</legend>
          <div className="shadow-grid" role="radiogroup" aria-label={t("ui.editor.image.anchor")}>
            {ANCHORS.map((a, i) => (
              <button
                key={a}
                type="button"
                role="radio"
                className="shadow-cell"
                aria-checked={anchor === a}
                aria-pressed={anchor === a}
                aria-label={t(`ui.editor.image.anchor_${a}`)}
                title={t(`ui.editor.image.anchor_${a}`)}
                onClick={() => setAnchor(a)}
              >
                <span
                  className="shadow-dot"
                  style={{ translate: `${((i % 3) - 1) * 4}px ${(Math.floor(i / 3) - 1) * 4}px` }}
                />
              </button>
            ))}
          </div>
        </fieldset>
      </div>
      <p className="muted">{t("ui.editor.image.canvas_hint")}</p>
    </Dialog>
  );
}

/** A dial for an angle: drag around it, or arrows (Shift: 15°). */
function AngleWheel({ value, onChange }: { value: number; onChange: (v: number) => void }) {
  const { t } = useTranslation();
  const ref = useRef<HTMLDivElement>(null);
  const read = (e: PointerEvent) => {
    const r = ref.current?.getBoundingClientRect();
    if (!r) return;
    onChange(
      angleOf(wheelAngle(e.clientX - (r.left + r.width / 2), e.clientY - (r.top + r.height / 2))),
    );
  };
  const onKey = (e: KeyboardEvent) => {
    const step = e.shiftKey ? 15 : 1;
    if (e.key === "ArrowRight" || e.key === "ArrowUp") onChange(angleOf(value + step));
    else if (e.key === "ArrowLeft" || e.key === "ArrowDown") onChange(angleOf(value - step));
    else return;
    e.preventDefault();
  };
  return (
    <div
      ref={ref}
      className="angle-wheel"
      role="slider"
      tabIndex={0}
      aria-label={t("ui.editor.image.wheel")}
      aria-valuemin={-359.9}
      aria-valuemax={359.9}
      aria-valuenow={value}
      aria-valuetext={`${value}°`}
      onKeyDown={onKey}
      onPointerDown={(e) => {
        e.currentTarget.setPointerCapture(e.pointerId);
        read(e);
      }}
      onPointerMove={(e) => {
        if (e.currentTarget.hasPointerCapture(e.pointerId)) read(e);
      }}
    >
      <span className="angle-hand" style={{ rotate: `${value}deg` }} aria-hidden="true" />
    </div>
  );
}

/** Image > Rotate > Custom angle (EDT-072). */
function Rotate({ doc }: { doc: DocDetails }) {
  const { t } = useTranslation();
  const [angle, setAngle] = useState(15);
  const [background, setBackground] = useState<"transparent" | "canvas">("transparent");
  const seg = useRef<HTMLDivElement>(null);
  useIndicator(seg, background);
  return (
    <Dialog
      title={t("ui.editor.image.rotate_title")}
      onCancel={closeImageDialog}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            onClick={() => apply("document.rotate", doc, { degrees: angle, background })}
          >
            {t("ui.editor.image.apply")}
          </button>
          <button type="button" className="btn" onClick={closeImageDialog}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <div className="rotate-row">
        <div className="image-form">
          <NumberInput
            id="rotate-angle"
            label={t("ui.editor.image.angle")}
            value={String(angle)}
            autoFocus
            suffix="°"
            onValue={(v) => setAngle(angleOf(v))}
          />
          <AngleWheel value={angle} onChange={setAngle} />
        </div>
        <figure className="rotate-preview" aria-label={t("ui.editor.image.preview")}>
          <img
            src={xsdocUrl(`${doc.documentId}/final?v=${doc.version}`)}
            alt=""
            style={{ rotate: `${angle}deg` }}
            draggable={false}
          />
        </figure>
      </div>
      <div className="prop-field">
        <span id="rotate-bg">{t("ui.editor.image.background")}</span>
        <div className="seg small" role="radiogroup" aria-labelledby="rotate-bg" ref={seg}>
          <span className="seg-indicator" aria-hidden="true" />
          {(["transparent", "canvas"] as const).map((b) => (
            <button
              key={b}
              type="button"
              role="radio"
              aria-checked={background === b}
              onClick={() => setBackground(b)}
            >
              {t(`ui.editor.image.bg_${b}`)}
            </button>
          ))}
        </div>
      </div>
    </Dialog>
  );
}

/** Image > Rounded corners (EDT-075): a radius, or a circle. */
function Corners({ doc }: { doc: DocDetails }) {
  const { t } = useTranslation();
  const max = Math.floor(Math.min(doc.canvas.width, doc.canvas.height) / 2);
  const [radius, setRadius] = useState(doc.canvas.cornerRadius > 0 ? doc.canvas.cornerRadius : 20);
  const r = Math.max(0, Math.min(max, Math.round(radius)));
  return (
    <Dialog
      title={t("ui.editor.image.corners_title")}
      onCancel={closeImageDialog}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            onClick={() => apply("document.canvas", doc, { cornerRadius: r })}
          >
            {t("ui.editor.image.apply")}
          </button>
          <button
            type="button"
            className="btn"
            onClick={() => apply("document.canvas", doc, { cornerRadius: "circle" })}
          >
            {t("ui.editor.image.circle")}
          </button>
          <button type="button" className="btn" onClick={closeImageDialog}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <div className="image-form">
        <NumberInput
          id="corners-radius"
          label={t("ui.editor.image.radius", { max })}
          value={String(r)}
          autoFocus
          suffix="px"
          onValue={setRadius}
        />
      </div>
      <p className="muted">{t("ui.editor.image.corners_hint")}</p>
    </Dialog>
  );
}

/** Image > Resolution (EDT-077): the pixels stay; only their size on paper changes. */
function Resolution({ doc }: { doc: DocDetails }) {
  const { t, i18n } = useTranslation();
  const [dpi, setDpi] = useState(Math.round(doc.dpi * 100) / 100);
  const valid = dpi >= 1 && dpi <= 10_000;
  return (
    <Dialog
      title={t("ui.editor.image.resolution_title")}
      onCancel={closeImageDialog}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            disabled={!valid}
            onClick={() => apply("document.canvas", doc, { dpi })}
          >
            {t("ui.editor.image.apply")}
          </button>
          <button type="button" className="btn" onClick={closeImageDialog}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <div className="image-form">
        <NumberInput
          id="resolution-dpi"
          label={t("ui.editor.image.resolution")}
          value={String(dpi)}
          autoFocus
          suffix={t("ui.editor.image.dpi_unit")}
          onValue={setDpi}
        />
        <div className="prop-field">
          <label htmlFor="resolution-common">{t("ui.editor.image.common")}</label>
          <select
            id="resolution-common"
            value={COMMON_DPI.includes(dpi) ? String(dpi) : ""}
            onChange={(e) => e.target.value && setDpi(Number(e.target.value))}
          >
            <option value="">—</option>
            {COMMON_DPI.map((d) => (
              <option key={d} value={d}>
                {t("ui.editor.dpi", { dpi: d })}
              </option>
            ))}
          </select>
        </div>
      </div>
      <p className="muted" aria-live="polite">
        {t("ui.editor.image.density", {
          d: (dpi / 96).toLocaleString(i18n.language, { maximumFractionDigits: 2 }),
        })}
      </p>
      <p className="muted">{t("ui.editor.image.resolution_hint")}</p>
    </Dialog>
  );
}

/** Image > Canvas color (EDT-074): applied as it is picked. */
function CanvasColor({ doc }: { doc: DocDetails }) {
  const { t } = useTranslation();
  return (
    <Dialog
      title={t("ui.editor.image.color_title")}
      onCancel={closeImageDialog}
      actions={
        <button type="button" className="btn primary" onClick={closeImageDialog}>
          {t("ui.editor.image.done")}
        </button>
      }
    >
      {/* A color, transparent or a gradient (EDT-020). */}
      <FillPicker
        label={t("ui.editor.image.color")}
        value={doc.canvas.gradient ?? doc.canvas.color ?? null}
        allowTransparent
        onCommit={(color) =>
          void run("document.canvas", { documentId: doc.documentId, color }).catch(fail)
        }
      />
      <p className="muted">{t("ui.editor.image.color_hint")}</p>
    </Dialog>
  );
}

/** Image > Trim edges (EDT-051): uniform borders go, within a tolerance. */
function Trim({ doc }: { doc: DocDetails }) {
  const { t } = useTranslation();
  const [tolerance, setTolerance] = useState(2);
  const valid = tolerance >= 0 && tolerance <= 20;
  const ok = () => {
    closeImageDialog();
    run<{ changed?: boolean; message?: string }>("document.trim", {
      documentId: doc.documentId,
      tolerance,
    })
      .then((r) => {
        if (r.changed === false && r.message) useEditor.getState().showToast(r.message);
      })
      .catch(fail);
  };
  return (
    <Dialog
      title={t("ui.editor.image.trim_title")}
      onCancel={closeImageDialog}
      actions={
        <>
          <button type="button" className="btn primary" disabled={!valid} onClick={ok}>
            {t("ui.editor.image.apply")}
          </button>
          <button type="button" className="btn" onClick={closeImageDialog}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <div className="image-form">
        <NumberInput
          id="trim-tolerance"
          label={t("ui.editor.image.tolerance")}
          value={String(tolerance)}
          autoFocus
          suffix="%"
          onValue={setTolerance}
        />
      </div>
      <p className="muted">{t("ui.editor.image.trim_hint")}</p>
    </Dialog>
  );
}

function settingOf<T>(key: string, fallback: T): T {
  const v = useApp.getState().settings.find((s) => s.key === key)?.value;
  return (v === undefined ? fallback : v) as T;
}

/** File > New image (EDT-103): size, quick sizes, canvas color and resolution. */
function NewImage() {
  const { t } = useTranslation();
  const clip = useClipboard((s) => s.info);
  // No percent: there is no image to take it from.
  const [picked, setUnit] = useState<Unit>(lastUnit);
  const unit: Unit = picked === "percent" ? "px" : picked;
  const [w, setW] = useState(() => settingOf("editor.newImage.width", 1280));
  const [h, setH] = useState(() => settingOf("editor.newImage.height", 720));
  const [dpi, setDpi] = useState(96);
  const [fill, setFill] = useState<Fill | null>(() =>
    settingOf<string | null>("editor.newImage.color", "#FFFFFF"),
  );
  const [remember, setRemember] = useState(false);
  const W = Math.round(w);
  const H = Math.round(h);
  const valid = W >= 1 && H >= 1 && W <= 50_000 && H <= 50_000 && dpi >= 1 && dpi <= 10_000;
  const quick = (key: string) => {
    if (key === "clipboard" && clip?.image) {
      setW(clip.image.width);
      setH(clip.image.height);
      return;
    }
    const q = QUICK_SIZES.find((s) => s.key === key);
    if (!q) return;
    setW(q.w);
    setH(q.h);
    setDpi(q.dpi ?? 96);
  };
  const ok = () => {
    closeImageDialog();
    run("document.new", { width: W, height: H, color: fill, dpi, remember }).catch(fail);
  };
  return (
    <Dialog
      title={t("ui.editor.image.new_title")}
      onCancel={closeImageDialog}
      actions={
        <>
          <button type="button" className="btn primary" disabled={!valid} onClick={ok}>
            {t("ui.editor.image.create")}
          </button>
          <button type="button" className="btn" onClick={closeImageDialog}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <div className="image-form">
        <div className="prop-field">
          <label htmlFor="new-quick">{t("ui.editor.image.quick")}</label>
          <select id="new-quick" value="" onChange={(e) => quick(e.target.value)}>
            <option value="">—</option>
            {QUICK_SIZES.map((q) => (
              <option key={q.key} value={q.key}>
                {t(`ui.editor.image.quick_${q.key}`, { w: q.w, h: q.h })}
              </option>
            ))}
            {clip?.image && (
              <option value="clipboard">{t("ui.editor.image.quick_clipboard", clip.image)}</option>
            )}
          </select>
        </div>
        <UnitSelect value={unit} onChange={setUnit} units={["px", "in", "cm"]} />
        <div className="image-sides">
          <NumberInput
            id="new-w"
            label={t("ui.editor.image.width")}
            value={shown(fromPx(w, unit, w, dpi), unit)}
            autoFocus
            suffix={suffixOf(unit)}
            onValue={(v) => v > 0 && setW(toPx(v, unit, w, dpi))}
          />
          <NumberInput
            id="new-h"
            label={t("ui.editor.image.height")}
            value={shown(fromPx(h, unit, h, dpi), unit)}
            suffix={suffixOf(unit)}
            onValue={(v) => v > 0 && setH(toPx(v, unit, h, dpi))}
          />
        </div>
        <FillPicker
          label={t("ui.editor.prop.canvas_color")}
          value={fill}
          allowTransparent
          onCommit={setFill}
        />
        <NumberInput
          id="new-dpi"
          label={t("ui.editor.image.resolution")}
          value={String(dpi)}
          suffix={t("ui.editor.image.dpi_unit")}
          onValue={(v) => v > 0 && setDpi(v)}
        />
        <label className="panel-check">
          <input
            type="checkbox"
            checked={remember}
            onChange={(e) => setRemember(e.target.checked)}
          />
          {t("ui.editor.image.remember")}
        </label>
      </div>
      <p className="muted">{t("ui.editor.image.result_new", { w: W, h: H })}</p>
    </Dialog>
  );
}

/** Edit > Revert to original (EDT-076): asks first; Undo brings the edits back. */
function Revert({ doc }: { doc: DocDetails }) {
  const { t } = useTranslation();
  const changed = doc.originalChanged === true;
  return (
    <Dialog
      title={t("ui.editor.image.revert_title")}
      onCancel={closeImageDialog}
      actions={
        <>
          <button
            type="button"
            className="btn danger"
            onClick={() => apply("document.revert", doc, {})}
          >
            {t("ui.editor.image.revert_ok")}
          </button>
          <button type="button" className="btn" onClick={closeImageDialog}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <p>{t("ui.editor.image.revert_body")}</p>
      {changed && <p className="warning-text">{t("ui.editor.image.revert_changed")}</p>}
    </Dialog>
  );
}

export function ImageDialogs() {
  const open = useImageDialog((s) => s.open);
  const doc = useEditor((s) => s.doc);
  // A new image needs no document (EDT-103).
  if (open === "new_image") return <NewImage />;
  if (!open || !doc) return null;
  // A key per document: a dialog never keeps another document's values.
  const key = `${open}:${doc.documentId}`;
  switch (open) {
    case "resize":
      return <Resize key={key} doc={doc} />;
    case "canvas_size":
      return <CanvasSize key={key} doc={doc} />;
    case "rotate":
      return <Rotate key={key} doc={doc} />;
    case "corners":
      return <Corners key={key} doc={doc} />;
    case "resolution":
      return <Resolution key={key} doc={doc} />;
    case "canvas_color":
      return <CanvasColor key={key} doc={doc} />;
    case "revert":
      return <Revert key={key} doc={doc} />;
    case "trim":
      return <Trim key={key} doc={doc} />;
  }
}

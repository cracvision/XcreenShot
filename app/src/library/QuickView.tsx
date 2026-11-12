// The quick view over the window (LIB-013): the large image, its name, size and date,
// and the arrows to the previous and next item. The small thumbnail shows at once and
// the sharp image replaces it as soon as it arrives; the neighbours are asked for ahead.
import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { when } from "./format";
import {
  closeQuickView,
  fitted,
  stepQuickView,
  useQuickView,
  type QuickView as View,
} from "./quickViewState";

export function QuickView() {
  const view = useQuickView((s) => s.view);
  return view ? <Overlay view={view} /> : null;
}

function Overlay({ view }: { view: View }) {
  const { t, i18n } = useTranslation();
  const root = useRef<HTMLDivElement>(null);
  const stage = useRef<HTMLDivElement>(null);
  const [box, setBox] = useState({ width: 0, height: 0 });
  const [ready, setReady] = useState<string | null>(null);
  const entry = view.entries[view.at];
  const count = view.entries.length;

  // The focus goes to the view while it is open and comes back on closing (done()).
  useEffect(() => {
    root.current?.focus();
  }, []);

  useLayoutEffect(() => {
    const el = stage.current;
    if (!el) return;
    const measure = () => setBox({ width: el.clientWidth, height: el.clientHeight });
    measure();
    const seen = new ResizeObserver(measure);
    seen.observe(el);
    return () => seen.disconnect();
  }, []);

  // ← and → are instant: the neighbours are already on their way.
  useEffect(() => {
    for (const d of [1, -1]) {
      const next = view.entries[view.at + d];
      if (next) new Image().src = next.src;
    }
  }, [view.at, view.entries]);

  if (!entry) return null;

  const onKey = (e: KeyboardEvent) => {
    const onButton = (e.target as HTMLElement).tagName === "BUTTON";
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      e.preventDefault();
      stepQuickView(e.key === "ArrowLeft" ? -1 : 1);
    } else if (e.key === "Escape" || (e.key === " " && !onButton)) {
      e.preventDefault();
      closeQuickView();
    } else if (e.key === "Enter" && !onButton) {
      e.preventDefault();
      closeQuickView(true);
    } else if (e.key === "Tab") {
      // The focus stays in the view.
      const stops = Array.from(
        root.current?.querySelectorAll<HTMLElement>("button:not(:disabled)") ?? [],
      );
      if (stops.length === 0) return;
      const at = stops.indexOf(document.activeElement as HTMLElement);
      const next = e.shiftKey
        ? stops[at <= 0 ? stops.length - 1 : at - 1]
        : stops[at < 0 || at === stops.length - 1 ? 0 : at + 1];
      e.preventDefault();
      next?.focus();
    }
    e.stopPropagation();
  };

  const size = fitted(entry.width, entry.height, box, window.devicePixelRatio || 1);
  const dims = entry.width && entry.height ? `${entry.width} × ${entry.height} px` : null;
  const date = entry.date ? when(entry.date, i18n.language) : null;
  const sharp = ready === entry.src;
  return (
    <div
      ref={root}
      className="quick-view"
      role="dialog"
      aria-modal="true"
      aria-label={t("ui.library.quick.title", { name: entry.name })}
      tabIndex={-1}
      onKeyDown={onKey}
    >
      <div
        ref={stage}
        className="qv-stage"
        onClick={(e) => {
          if (e.target === e.currentTarget) closeQuickView();
        }}
      >
        <div
          className="qv-frame"
          style={size ? { width: size.width, height: size.height } : undefined}
        >
          {!sharp && <img className="qv-thumb" src={entry.thumb} alt="" draggable={false} />}
          <img
            key={entry.src}
            className={`qv-image${sharp ? " ready" : ""}`}
            src={entry.src}
            alt={entry.name}
            draggable={false}
            onLoad={() => setReady(entry.src)}
          />
        </div>
      </div>
      <div className="qv-bar">
        <button
          type="button"
          className="btn qv-step"
          disabled={view.at === 0}
          aria-label={t("ui.library.quick.previous")}
          title={t("ui.library.quick.previous")}
          onClick={() => stepQuickView(-1)}
        >
          ‹
        </button>
        <div className="qv-caption" aria-live="polite">
          <strong>{entry.name}</strong>
          <span className="muted">{[dims, date].filter(Boolean).join(" · ")}</span>
        </div>
        {count > 1 && (
          <span className="qv-count muted">
            {t("ui.library.quick.position", { at: view.at + 1, count })}
          </span>
        )}
        <button
          type="button"
          className="btn qv-step"
          disabled={view.at === count - 1}
          aria-label={t("ui.library.quick.next")}
          title={t("ui.library.quick.next")}
          onClick={() => stepQuickView(1)}
        >
          ›
        </button>
        <button type="button" className="btn small" onClick={() => closeQuickView(true)}>
          {t("ui.library.quick.open")}
        </button>
        <button
          type="button"
          className="btn small"
          aria-label={t("ui.library.quick.close")}
          title={t("ui.library.quick.close")}
          onClick={() => closeQuickView()}
        >
          ✕
        </button>
      </div>
    </div>
  );
}

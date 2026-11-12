// The image with its effects, for the canvas (EDT-053): the core draws it (xsdoc
// `/final`) and says where the canvas is in it (header `X-Origin`). Loaded again when
// the document changes; the last one stays meanwhile.
import { useEffect, useRef, useState } from "react";
import { useEditor, xsdocUrl, type DocDetails } from "./state";

export interface FinalView {
  image: HTMLImageElement;
  /** Where the canvas's top-left corner is in the image. */
  origin: [number, number];
}

type Stack = { enabled?: boolean }[] | undefined;

/** Whether the document or one of its floating images has an effect on. */
export function hasEffects(doc: DocDetails): boolean {
  const on = (s: Stack) => (s ?? []).some((e) => e.enabled !== false);
  return (
    on(doc.effects) ||
    doc.objects.some((o) => o.type === "image" && on((o as unknown as { effects?: Stack }).effects))
  );
}

export function useFinalPreview(doc: DocDetails): FinalView | null {
  const active = hasEffects(doc);
  const [view, setView] = useState<{ doc: string; v: FinalView } | null>(null);
  const lastUrl = useRef<string | null>(null);
  const key = `${doc.documentId}:${doc.version}`;
  useEffect(() => {
    if (!active) return;
    let alive = true;
    // A short wait: a burst of changes loads once.
    const timer = setTimeout(() => {
      void (async () => {
        try {
          const r = await fetch(xsdocUrl(`${doc.documentId}/final?v=${doc.version}`));
          if (!r.ok) return;
          const [x, y] = (r.headers.get("X-Origin") ?? "0,0").split(",").map(Number);
          const url = URL.createObjectURL(await r.blob());
          const img = new Image();
          img.onload = () => {
            if (!alive) {
              URL.revokeObjectURL(url);
              return;
            }
            if (lastUrl.current) URL.revokeObjectURL(lastUrl.current);
            lastUrl.current = url;
            setView({ doc: doc.documentId, v: { image: img, origin: [x || 0, y || 0] } });
            // The size with effects, for the status bar.
            useEditor.setState({ fxSize: [img.naturalWidth, img.naturalHeight] });
          };
          img.src = url;
        } catch {
          // The canvas keeps showing the editable layers.
        }
      })();
    }, 120);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, active]);
  if (!active || !view || view.doc !== doc.documentId) return null;
  return view.v;
}

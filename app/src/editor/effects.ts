// The effects of the Effects tab (EDT-053 to EDT-068): their order and what each starts
// with, and Image > Effects, which turns one on from the menu.
import { run } from "../lib/commands";
import { usePanel, useEditor } from "./state";

/** The effects of the panel, in order (EDT-053). */
export const TYPES = [
  "border",
  "edges",
  "shadow",
  "perspective",
  "pageCurl",
  "reflection",
  "fade",
  "filters",
  "color",
  "replace",
  "watermark",
  "details",
  "trim",
  "corners",
  "scale",
  "rotate",
] as const;

/** What an effect starts with when it is turned on for the first time. */
export const DEFAULTS: Record<string, Record<string, unknown>> = {
  border: { color: "#1F2430", width: 2, position: "outside" },
  edges: { style: "torn", size: 16, sides: { bottom: true }, randomness: 50, shadow: false },
  shadow: {
    angle: 45,
    distance: 10,
    blur: 20,
    opacity: 50,
    color: "#000000",
    background: "transparent",
  },
  perspective: { style: "perspective", horizontal: 15, vertical: 0, depth: 50 },
  pageCurl: { corner: "bottomRight", size: 20, color: "#D9D9D9" },
  reflection: { size: 30, opacity: 40, gap: 0 },
  fade: { size: 40, intensity: 100, sides: { top: true, bottom: true, left: true, right: true } },
  filters: { grayscale: { on: true } },
  color: { brightness: 0, contrast: 0, hue: 0, saturation: 0, gamma: 1 },
  replace: { mode: "replace", a: "#FFFFFF", b: null, tolerance: 5 },
  watermark: {
    source: "text",
    text: "XcreenShot",
    size: 36,
    color: "#FFFFFF",
    position: "bottomRight",
    margin: 20,
    scale: 100,
    opacity: 30,
    rotation: 0,
    layer: "above",
  },
  details: {
    placement: "band",
    side: "bottom",
    fields: { app: true, url: true, title: true, date: true },
    size: 12,
    color: "#FFFFFF",
    background: "#1F2430CC",
    align: "left",
    padding: 8,
    dateFormat: "system",
  },
  trim: { tolerance: 5 },
  corners: { shape: "radius", radius: 20 },
  scale: { mode: "percent", value: 50 },
  rotate: { angle: 90 },
};

/** Image > Effects (EDT-002): turns `type` on for the document shown (or the selected
 * floating image) and opens the Effects tab on it. */
export function turnOnEffect(type: string): boolean {
  const { doc, selection } = useEditor.getState();
  if (!doc || !(TYPES as readonly string[]).includes(type)) return false;
  const image =
    selection.length === 1
      ? doc.objects.find((o) => o.id === selection[0] && o.type === "image")
      : undefined;
  const stack =
    ((image ? (image as unknown as { effects?: Effect[] }).effects : doc.effects) as
      Effect[] | undefined) ?? [];
  const target = image ? { objectId: image.id } : {};
  const i = stack.findIndex((e) => e.type === type);
  const params =
    i < 0
      ? { action: "add", effect: { type, ...DEFAULTS[type] } }
      : { action: "update", index: i, effect: { enabled: true } };
  usePanel.getState().set({ visible: true, tab: "effects" });
  run("document.effects", { documentId: doc.documentId, ...target, ...params }).catch((e) =>
    useEditor.getState().showError(e),
  );
  return true;
}

type Effect = Record<string, unknown> & { type: string; enabled?: boolean };

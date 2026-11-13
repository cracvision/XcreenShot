// What each effect can change (EDT-053 to EDT-068), shared by the Effects tab and the
// capture panel (CAP-051).

export type Effect = Record<string, unknown> & { type: string; enabled?: boolean };

export type Field =
  | { k: string; kind: "number"; min: number; max: number; step?: number; unit?: string }
  | { k: string; kind: "select"; options: string[] }
  | { k: string; kind: "color"; transparent?: boolean }
  /** `capture`: only among the effects on capture (CAP-051). */
  | { k: string; kind: "check"; capture?: boolean }
  | { k: string; kind: "text" }
  /** A filter's switch and its value (EDT-062). */
  | { k: string; kind: "filter"; min?: number; max?: number; unit?: string };

const SIDES: Field[] = ["top", "bottom", "left", "right"].map((s) => ({
  k: `sides.${s}`,
  kind: "check",
}));

/** The options of each effect. Dotted keys are inside an object (`sides.top`). */
export const FIELDS: Record<string, Field[]> = {
  border: [
    { k: "color", kind: "color" },
    { k: "width", kind: "number", min: 1, max: 100, unit: "px" },
    { k: "position", kind: "select", options: ["outside", "inside"] },
  ],
  edges: [
    {
      k: "style",
      kind: "select",
      options: ["torn", "wavy", "sawtooth", "bevel", "fade", "scallop"],
    },
    { k: "size", kind: "number", min: 2, max: 80, unit: "px" },
    ...SIDES,
    { k: "randomness", kind: "number", min: 0, max: 100, unit: "%" },
    { k: "shadow", kind: "check" },
    { k: "shadowColor", kind: "color" },
    { k: "shadowAngle", kind: "number", min: 0, max: 359, unit: "°" },
  ],
  shadow: [
    { k: "angle", kind: "number", min: 0, max: 359, unit: "°" },
    { k: "distance", kind: "number", min: 0, max: 100, unit: "px" },
    { k: "blur", kind: "number", min: 0, max: 100, unit: "px" },
    { k: "opacity", kind: "number", min: 0, max: 100, unit: "%" },
    { k: "color", kind: "color" },
    { k: "background", kind: "select", options: ["transparent", "canvas"] },
  ],
  perspective: [
    { k: "style", kind: "select", options: ["perspective", "skew"] },
    { k: "horizontal", kind: "number", min: -45, max: 45, unit: "°" },
    { k: "vertical", kind: "number", min: -45, max: 45, unit: "°" },
    { k: "depth", kind: "number", min: 0, max: 100, unit: "%" },
  ],
  pageCurl: [
    { k: "corner", kind: "select", options: ["bottomRight", "bottomLeft", "topRight", "topLeft"] },
    { k: "size", kind: "number", min: 5, max: 50, unit: "%" },
    { k: "color", kind: "color" },
  ],
  reflection: [
    { k: "size", kind: "number", min: 10, max: 100, unit: "%" },
    { k: "opacity", kind: "number", min: 0, max: 100, unit: "%" },
    { k: "gap", kind: "number", min: 0, max: 50, unit: "px" },
  ],
  fade: [
    { k: "size", kind: "number", min: 2, max: 200, unit: "px" },
    { k: "intensity", kind: "number", min: 0, max: 100, unit: "%" },
    ...SIDES,
  ],
  filters: [
    { k: "soften", kind: "filter", min: 1, max: 10 },
    { k: "sharpen", kind: "filter", min: 1, max: 10 },
    { k: "grayscale", kind: "filter" },
    { k: "sepia", kind: "filter", min: 0, max: 100, unit: "%" },
    { k: "mono", kind: "filter", min: 0, max: 255 },
    { k: "invert", kind: "filter" },
    { k: "opacity", kind: "filter", min: 0, max: 100, unit: "%" },
    { k: "baseOnly", kind: "check" },
  ],
  color: [
    { k: "brightness", kind: "number", min: -100, max: 100 },
    { k: "contrast", kind: "number", min: -100, max: 100 },
    { k: "hue", kind: "number", min: -180, max: 180, unit: "°" },
    { k: "saturation", kind: "number", min: -100, max: 100 },
    { k: "gamma", kind: "number", min: 0.1, max: 5, step: 0.1 },
  ],
  replace: [
    { k: "mode", kind: "select", options: ["replace", "swap"] },
    { k: "a", kind: "color", transparent: true },
    { k: "b", kind: "color", transparent: true },
    { k: "tolerance", kind: "number", min: 0, max: 100, unit: "%" },
  ],
  watermark: [
    { k: "source", kind: "select", options: ["text", "image"] },
    { k: "text", kind: "text" },
    { k: "size", kind: "number", min: 6, max: 400, unit: "pt" },
    { k: "color", kind: "color" },
    {
      k: "position",
      kind: "select",
      options: [
        "topLeft",
        "top",
        "topRight",
        "left",
        "center",
        "right",
        "bottomLeft",
        "bottom",
        "bottomRight",
        "tile",
      ],
    },
    { k: "margin", kind: "number", min: 0, max: 200, unit: "px" },
    { k: "scale", kind: "number", min: 5, max: 200, unit: "%" },
    { k: "opacity", kind: "number", min: 0, max: 100, unit: "%" },
    { k: "rotation", kind: "number", min: -90, max: 90, unit: "°" },
    { k: "layer", kind: "select", options: ["above", "below"] },
  ],
  details: [
    ...["app", "url", "title", "date", "os", "user", "computer"].map((f): Field => ({
      k: `fields.${f}`,
      kind: "check",
    })),
    { k: "customText", kind: "text" },
    { k: "askText", kind: "check", capture: true },
    { k: "dateFormat", kind: "select", options: ["system", "iso", "custom"] },
    { k: "dateCustom", kind: "text" },
    { k: "placement", kind: "select", options: ["band", "overlay"] },
    { k: "side", kind: "select", options: ["bottom", "top"] },
    { k: "corner", kind: "select", options: ["bottomRight", "bottomLeft", "topRight", "topLeft"] },
    { k: "size", kind: "number", min: 6, max: 72, unit: "pt" },
    { k: "color", kind: "color" },
    { k: "background", kind: "color", transparent: true },
    { k: "align", kind: "select", options: ["left", "center", "right"] },
    { k: "padding", kind: "number", min: 0, max: 100, unit: "px" },
  ],
  trim: [{ k: "tolerance", kind: "number", min: 0, max: 100, unit: "%" }],
  corners: [
    { k: "shape", kind: "select", options: ["radius", "circle"] },
    { k: "radius", kind: "number", min: 0, max: 2000, unit: "px" },
  ],
  scale: [
    { k: "mode", kind: "select", options: ["percent", "longest", "width", "height"] },
    { k: "value", kind: "number", min: 1, max: 20000 },
  ],
  rotate: [{ k: "angle", kind: "select", options: ["90", "180", "270"] }],
};

/** A value inside an effect, by a dotted key. */
export function read(e: Effect | undefined, key: string): unknown {
  return key
    .split(".")
    .reduce<unknown>((v, k) => (v as Record<string, unknown> | undefined)?.[k], e);
}

/** The top-level change that sets a dotted key to `value`. */
export function change(
  e: Effect | undefined,
  key: string,
  value: unknown,
): Record<string, unknown> {
  const [head, ...rest] = key.split(".");
  if (!head) return {};
  if (rest.length === 0) return { [head]: value };
  const inner = { ...((e?.[head] as Record<string, unknown> | undefined) ?? {}) };
  inner[rest.join(".")] = value;
  return { [head]: inner };
}

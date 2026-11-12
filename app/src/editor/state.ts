// Editor view state. Documents, tools and styles live in the core (ADR 0004): this
// store mirrors what the `xs://editor` events report and keeps only what belongs to
// the view (selection, views, open dialogs).
import { listen } from "@tauri-apps/api/event";
import i18next from "i18next";
import { create } from "zustand";
import { isCommandError, run } from "../lib/commands";
import type { Rect } from "./boxes";
import type { ArrowStyle, LineStyle, Point } from "./geometry";
import type { View } from "./view";
import { aiProgress } from "./aiState";
import { showView, useLibrary, useTray, type MainView } from "../library/state";
import { arrived } from "./tray";

export type Tool =
  | "favorites"
  | "move"
  | "arrow"
  | "line"
  | "text"
  | "callout"
  | "shape"
  | "stamp"
  | "step"
  | "highlight"
  | "pen"
  | "magnify"
  | "spotlight"
  | "blur"
  | "simplify"
  | "select"
  | "wand"
  | "fill"
  | "eraser"
  | "crop"
  | "cutout";

export interface DocSummary {
  documentId: string;
  name: string;
  path: string | null;
  kind: "library" | "external";
  format: "xshot" | "png" | "jpg";
  width: number;
  height: number;
  objects: number;
  dirty: boolean;
  shown: boolean;
  version: number;
  /** Its file was moved or deleted outside XcreenShot (TRY-002). */
  missing: boolean;
}

export interface ArrowObject {
  id: string;
  type: "arrow";
  from: Point;
  to: Point;
  style: ArrowStyle;
}

export type TextAlign = "left" | "center" | "right" | "justify";

/** Text properties of M1 (EDT-026). Sizes are points at the document's resolution. */
export interface TextStyle {
  font: string;
  size: number;
  bold: boolean;
  italic: boolean;
  color: string;
  align: TextAlign;
  opacity: number;
}

export interface TextObject {
  id: string;
  type: "text";
  at: Point;
  /** Fixed width (wraps), or null: the box grows with the text. */
  width: number | null;
  text: string;
  rotation: number;
  style: TextStyle;
  /** Laid-out size, computed by the core. */
  layout: { width: number; height: number };
}

export type ShapeKind =
  "rectangle" | "ellipse" | "roundedRectangle" | "triangle" | "diamond" | "polygon";
export const SHAPE_KINDS: ShapeKind[] = [
  "rectangle",
  "ellipse",
  "roundedRectangle",
  "triangle",
  "diamond",
  "polygon",
];

/** A gradient (EDT-020): `angle` turns clockwise from the right (90° = top to bottom). */
export interface Gradient {
  type: "linear" | "radial";
  angle: number;
  stops: { color: string; pos: number }[];
}

/** A fill: a color or a gradient (EDT-020). */
export type Fill = string | Gradient;

/** Shape properties of M1 (EDT-029). */
export interface ShapeStyle {
  shapeType: ShapeKind;
  /** null = transparent: only the outline. */
  fill: Fill | null;
  outline: string;
  outlineWidth: number;
  lineStyle: LineStyle;
  opacity: number;
  /** Rounded rectangle (EDT-030). */
  cornerRadius: number;
}

export interface ShapeObject {
  id: string;
  type: "shape";
  rect: Rect;
  rotation: number;
  style: ShapeStyle;
  /** A polygon's corners, as shares of its box (EDT-030). */
  points?: Point[];
}

/** Simplify (EDT-041). */
export type SimpleKind =
  "block" | "roundedBlock" | "textLine" | "paragraph" | "circle" | "button" | "icon";
export const SIMPLE_KINDS: SimpleKind[] = [
  "block",
  "roundedBlock",
  "textLine",
  "paragraph",
  "circle",
  "button",
  "icon",
];

export interface Palette {
  name: string;
  colors: string[];
}

export interface SimpleStyle {
  kind: SimpleKind;
  fill: string;
  background: string | null;
  textHeight: number;
  lineSpacing: number;
  opacity: number;
  /** The tool's: saved palettes and the one in use. */
  palettes?: Palette[];
  palette?: string;
}

export interface SimplifyObject {
  id: string;
  type: "simplify";
  rect: Rect;
  rotation: number;
  style: SimpleStyle;
}

/** Stamps (EDT-031). */
/** What fills the place selected pixels leave (EDT-045). */
export type GapFill = "smart" | "transparent" | "canvas";

/** The Selection tool (EDT-043). */
export interface SelectStyle {
  shape: "rectangle" | "ellipse" | "polygon" | "freehand";
  antialias: boolean;
  feather: number;
  snap: boolean;
  level: number;
  gapFill: GapFill;
  quality: "fast" | "photo";
}

/** Cut out (EDT-052). */
export interface CutOutStyle {
  direction: "horizontal" | "vertical";
  mode: "remove" | "add";
  join: "straight" | "torn" | "wavy" | "zigzag";
  gap: number;
}

/** The Magic wand (EDT-046). */
export interface WandStyle {
  tolerance: number;
  global: boolean;
  gapFill: GapFill;
  quality: "fast" | "photo";
}

/** The pixel selection as `document.info` reports it, in canvas pixels (EDT-043). */
export interface PixelSelectionInfo {
  /** The floating image it is on, or null for the image. */
  target: string | null;
  bounds: { x: number; y: number; w: number; h: number };
  /** Closed loops; a hole is a loop too. */
  outline: [number, number][][];
  /** A rectangle or ellipse drawn alone: its numeric position and size (EDT-086). */
  shape: {
    type: "rectangle" | "ellipse";
    rect: { x: number; y: number; w: number; h: number };
  } | null;
}

/** The Fill tool (EDT-047): `color` null is Transparent. */
export interface FillStyle {
  color: Fill | null;
  tolerance: number;
  opacity: number;
  global: boolean;
}

/** The Eraser (EDT-048). */
export interface EraserStyle {
  width: number;
  shape: "round" | "square";
  opacity: number;
  antialias: boolean;
}

export interface StampStyle {
  /** The tool's stamp; objects keep theirs in `stamp`. */
  stamp?: string;
  /** null: the stamp's own color. */
  color: string | null;
  opacity: number;
}

export interface StampObject {
  id: string;
  type: "stamp";
  rect: Rect;
  rotation: number;
  stamp: string;
  ink: string;
  style: StampStyle;
}

/** A stamp of the collection's grid (EDT-031). */
export function stampUrl(id: string): string {
  return xsdocUrl(`stamp/${encodeURIComponent(id)}`);
}

/** Spotlight (EDT-039). */
export type SpotShape = "rectangle" | "roundedRectangle" | "ellipse";
export const SPOT_SHAPES: SpotShape[] = ["rectangle", "roundedRectangle", "ellipse"];

export interface SpotlightStyle {
  shape: SpotShape;
  color: string;
  opacity: number;
  blur: number;
  feather: number;
}

export interface SpotlightObject {
  id: string;
  type: "spotlight";
  rect: Rect;
  rotation: number;
  style: SpotlightStyle;
}

/** Magnify (EDT-038). */
export type LensShape = "circle" | "square" | "rectangle" | "roundedRectangle";
export const LENS_SHAPES: LensShape[] = ["circle", "square", "rectangle", "roundedRectangle"];

export interface MagnifyStyle {
  shape: LensShape;
  zoom: number;
  outline: string;
  outlineWidth: number;
  connectors: boolean;
  opacity: number;
}

export interface MagnifyObject {
  id: string;
  type: "magnify";
  /** The zone shown. */
  source: Rect;
  /** The lens's center. */
  at: Point;
  style: MagnifyStyle;
}

/** The lens: the zone times the zoom, around `at`. */
export function lensOf(m: Pick<MagnifyObject, "source" | "at" | "style">): Rect {
  const k = m.style.zoom / 100;
  const w = m.source.w * k;
  const h = m.source.h * k;
  return { x: m.at[0] - w / 2, y: m.at[1] - h / 2, w, h };
}

/** Pen (EDT-037). */
export type PenTip = "circle" | "square" | "slash" | "bar";
export const PEN_TIPS: PenTip[] = ["circle", "square", "slash", "bar"];

export interface PenStyle {
  color: string;
  width: number;
  opacity: number;
  tip: PenTip;
  antialias: boolean;
  smooth: number;
  editable: boolean;
}

export interface PenObject {
  id: string;
  type: "pen";
  points: Point[];
  style: PenStyle;
}

/** Highlighter (EDT-036). */
export type HighlightShapeKind = "rectangle" | "roundedRectangle" | "ellipse";
export const HIGHLIGHT_SHAPES: HighlightShapeKind[] = ["rectangle", "roundedRectangle", "ellipse"];

export interface HighlightStyle {
  color: string;
  opacity: number;
  shape: HighlightShapeKind;
}

export interface HighlightObject {
  id: string;
  type: "highlight";
  rect: Rect;
  rotation: number;
  style: HighlightStyle;
}

/** Callouts (EDT-028). */
export type CalloutShapeKind =
  | "rectangle"
  | "roundedRectangle"
  | "ellipse"
  | "cloud"
  | "thought"
  | "explosion"
  | "tag"
  | "arrow"
  | "text";
export const CALLOUT_SHAPES: CalloutShapeKind[] = [
  "rectangle",
  "roundedRectangle",
  "ellipse",
  "cloud",
  "thought",
  "explosion",
  "tag",
  "arrow",
  "text",
];
export type TailStyle = "straight" | "curved" | "arrow";
export type VAlign = "top" | "middle" | "bottom";

export interface CalloutStyle {
  shape: CalloutShapeKind;
  fill: Fill | null;
  shading: boolean;
  outline: string;
  outlineWidth: number;
  lineStyle: LineStyle;
  cornerRadius: number;
  tailStyle: TailStyle;
  tailWidth: number;
  tailColor: string | null;
  font: string;
  size: number;
  bold: boolean;
  italic: boolean;
  textColor: string;
  align: TextAlign;
  valign: VAlign;
  padding: number;
  fitText: boolean;
  opacity: number;
}

export interface CalloutObject {
  id: string;
  type: "callout";
  rect: Rect;
  rotation: number;
  text: string;
  tails: Point[];
  style: CalloutStyle;
}

/** Step markers (EDT-035). */
export type StepSequence = "number" | "upper" | "lower" | "romanUpper" | "romanLower";
export type StepShapeKind = "circle" | "square" | "roundedSquare" | "diamond" | "marker" | "text";
export const STEP_SEQUENCES: StepSequence[] = [
  "number",
  "upper",
  "lower",
  "romanUpper",
  "romanLower",
];
export const STEP_SHAPES: StepShapeKind[] = [
  "circle",
  "square",
  "roundedSquare",
  "diamond",
  "marker",
  "text",
];

export interface StepStyle {
  sequence: StepSequence;
  shape: StepShapeKind;
  fill: Fill | null;
  textColor: string;
  outline: string;
  outlineWidth: number;
  size: number;
  opacity: number;
}

export interface StepObject {
  id: string;
  type: "step";
  rect: Rect;
  rotation: number;
  value: string;
  style: StepStyle;
}

export type BlurType = "blur" | "pixelate" | "solid";

/** Blur properties of M1 (EDT-040). */
export interface BlurStyle {
  blurType: BlurType;
  color: string;
  intensity: number;
  shapeType: ShapeKind;
}

export interface BlurObject {
  id: string;
  type: "blur";
  rect: Rect;
  rotation: number;
  style: BlurStyle;
}

/** Floating image properties of M1 (EDT-078). */
export interface ImageStyle {
  opacity: number;
  /** Rounded corners in canvas pixels (EDT-075). */
  cornerRadius?: number;
}

/** A picture of the document's `images/` folder in a rectangle (EDT-078). */
export interface ImageObject {
  id: string;
  type: "image";
  /** File name in `images/`. */
  image: string;
  rect: Rect;
  rotation: number;
  style: ImageStyle;
}

/** Objects of later builds are kept and drawn by the core, not edited here. */
/** A line (EDT-025): an arrow of its own tool, without ends by default. */
export type LineObject = Omit<ArrowObject, "type"> & { type: "line" };

/** An arrow or a line: the same geometry and handles. */
export function isSegment(o: { type: string }): o is ArrowObject | LineObject {
  return o.type === "arrow" || o.type === "line";
}

export type DocObject = (
  | ArrowObject
  | LineObject
  | TextObject
  | ShapeObject
  | BlurObject
  | ImageObject
  | StepObject
  | CalloutObject
  | HighlightObject
  | PenObject
  | MagnifyObject
  | SpotlightObject
  | StampObject
  | SimplifyObject
  | { id: string; type: "other" }
) & {
  /** Locked: the mouse does not select or move it (EDT-085). */
  locked?: boolean;
  /** The groups it belongs to, outermost first (EDT-082). */
  groups?: string[];
  /** Where its shadow goes, `[x, y, w, h]` in canvas pixels (EDT-021). */
  shadowAt?: [number, number, number, number];
};

/** An object's shadow (EDT-021): `angle` turns clockwise from the right. */
export interface Shadow {
  color: string;
  angle: number;
  distance: number;
  opacity: number;
  blur: number;
}

/** The kinds that can have a shadow (EDT-021). */
export const WITH_SHADOW = [
  "arrow",
  "line",
  "text",
  "callout",
  "shape",
  "stamp",
  "step",
  "pen",
  "image",
] as const;

/** An object's shadow as Rust draws it (EDT-021). */
export function shadowUrl(documentId: string, objectId: string, version: string): string {
  return xsdocUrl(`${documentId}/shadow/${objectId}?v=${version}`);
}

export interface Styles {
  arrow: ArrowStyle;
  line: ArrowStyle;
  text: TextStyle;
  shape: ShapeStyle;
  blur: BlurStyle;
  step: StepStyle;
  callout: CalloutStyle;
  highlight: HighlightStyle;
  pen: PenStyle;
  magnify: MagnifyStyle;
  spotlight: SpotlightStyle;
  stamp: StampStyle;
  simplify: SimpleStyle;
  fill: FillStyle;
  eraser: EraserStyle;
  select: SelectStyle;
  wand: WandStyle;
  cutout: CutOutStyle;
}

/** A text box being typed (EDT-026): a new one (`id` null) or an existing one. */
export interface TextEdit {
  id: string | null;
  at: Point;
  width: number | null;
  text: string;
  rotation: number;
  style: TextStyle;
  /** Typing in a callout (EDT-028): the field covers its body with this fill. */
  callout?: { fill: string | null };
}

/** What `document.info` returns (EDT-119). */
/** A flattened layer (EDT-091): `permanent` says why it cannot become objects again. */
export interface FlatLayer {
  id: string;
  permanent?: "blur" | "covered" | string;
  bounds: [number, number, number, number] | null;
}

export interface DocDetails extends Omit<DocSummary, "objects"> {
  canvas: {
    width: number;
    height: number;
    color?: string;
    /** A gradient instead of the color (EDT-074); `color` holds its first stop. */
    gradient?: Gradient;
    cornerRadius: number;
  };
  crop: { x: number; y: number; width: number; height: number };
  /** Objects a crop left outside the canvas: they do not make it grow (EDT-071). */
  outside?: string[];
  dpi: number;
  objects: DocObject[];
  base: { width: number; height: number; mime: string };
  /** New original pixels in `images/` after resizing, turning, flipping or cutting
   * out (ADR 0006); they replace base.png until saved. */
  baseImage?: string | null;
  /** base.png is no longer the image as captured or opened (EDT-050, EDT-076). */
  originalChanged?: boolean;
  /** With flattened objects, the canvas shows a view of the image that may start
   * before the original pixels, in base pixels (EDT-091). */
  baseView?: { x: number; y: number; width: number; height: number } | null;
  /** Simplify (EDT-041): how the original shows under the simplified shapes. */
  simplify?: { showOriginal: boolean; opacity: number };
  /** The effect stack (EDT-053): `{type, enabled, …}` in order. */
  effects?: (Record<string, unknown> & { type: string; enabled?: boolean })[];
  /** The pixel selection, if there is one (EDT-043). */
  pixelSelection?: PixelSelectionInfo | null;
  /** Edits of pixels not yet saved (ADR 0005): the core shows them in the image. */
  patches?: { rect: { x: number; y: number; width: number; height: number }; image: string }[];
  /** What the next step of each sequence shows (EDT-035). */
  stepNext?: Partial<Record<StepSequence, string>>;
  /** Objects merged into the image (EDT-091), oldest first; `bounds` on the canvas. */
  flattened?: FlatLayer[];
  metadata: { display?: { scale?: number } };
  canUndo: boolean;
  canRedo: boolean;
  undoLabel: string | null;
  redoLabel: string | null;
  /** Remembered view (EDT-007): zoom and the image point at the center. */
  view: { zoom?: number; x?: number; y?: number } | null;
}

export interface Pending {
  documentId: string;
  name: string;
  path: string | null;
}

export interface RecoveryItem {
  id: string;
  name: string;
  originalPath: string | null;
  savedAt: string;
}

/** A zoom asked for one document: applied once, by that document's canvas. */
export interface ZoomRequest {
  mode: string;
  value: number | null;
  seq: number;
  documentId: string | null;
}

interface EditorState {
  ready: boolean;
  tool: Tool;
  tools: Tool[];
  styles: Styles | null;
  documents: DocSummary[];
  active: string | null;
  doc: DocDetails | null;
  selection: string[];
  /** Edit value of a step (EDT-035). */
  stepEdit: { id: string; value: string } | null;
  /** Flattening a blur waits for this question (EDT-091). */
  flattenAsk: { all: boolean } | null;
  /** A Photo fill's progress (0–1) while it takes long (EDT-045). */
  fillProgress: number | null;
  /** The size of the image with its effects, when it has some (EDT-053). */
  fxSize: [number, number] | null;
  /** Bumped when themes, saved styles or favorites change (EDT-015 to EDT-018). */
  themesVersion: number;
  /** The active theme's colors, for the color selector (EDT-019). */
  themeColors: string[];
  /** My colors and recent colors (EDT-019). */
  colors: { mine: string[]; recent: string[]; gradients?: Gradient[] };
  /** An imported theme has the name of one of the user's (EDT-018). */
  themeConflict: { path: string; name: string } | null;
  /** The group a double click entered, outermost first (EDT-082). */
  entered: string[];
  /** Last view of each document in this session. */
  views: Record<string, View>;
  /** A zoom asked by a command (CLI, menu) for the canvas to apply. */
  zoomRequest: ZoomRequest | null;
  recovery: RecoveryItem[];
  /** Recover work is on screen: at start, or from Help (APP-047). */
  recoveryShown: boolean;
  /** Documents to decide on before closing or quitting, and what was being closed. */
  decision: { kind: "close" | "quit"; pending: Pending[]; targets: string[] | "all" } | null;
  /** The text box being typed, if any (EDT-026). */
  textEdit: TextEdit | null;
  /** The crop being adjusted, in canvas pixels (EDT-049). */
  cropRect: Rect | null;
  /** Installed font families, loaded when first needed. */
  fonts: string[];
  /** Document waiting for the "objects will be flattened" confirmation (EDT-107). */
  flatten: string | null;
  toast: string | null;
  error: string | null;
  boot: () => Promise<void>;
  setSelection: (ids: string[]) => void;
  setView: (id: string, view: View) => void;
  showToast: (text: string) => void;
  showError: (e: unknown) => void;
}

let zoomSeq = 0;
let toastTimer: ReturnType<typeof setTimeout> | undefined;

/** The active theme's colors (EDT-019). */
async function loadThemeColors(): Promise<string[]> {
  try {
    const r = await run<{ active: string; themes: { id: string; colors: string[] }[] }>(
      "editor.themes",
    );
    return r.themes.find((t) => t.id === r.active)?.colors ?? [];
  } catch {
    return [];
  }
}

/** Document events received so far: a load that started before one is older than it. */
let docEvents = 0;

async function loadDoc(id: string | null): Promise<DocDetails | null> {
  if (!id) return null;
  try {
    return await run<DocDetails>("document.info", { documentId: id });
  } catch {
    return null;
  }
}

export const useEditor = create<EditorState>((set, get) => ({
  ready: false,
  tool: "move",
  tools: ["move", "arrow"],
  styles: null,
  documents: [],
  active: null,
  doc: null,
  selection: [],
  flattenAsk: null,
  fillProgress: null,
  themesVersion: 0,
  fxSize: null,
  themeColors: [],
  colors: { mine: [], recent: [] },
  themeConflict: null,
  stepEdit: null,
  entered: [],
  views: {},
  zoomRequest: null,
  recovery: [],
  recoveryShown: true,
  decision: null,
  flatten: null,
  textEdit: null,
  cropRect: null,
  fonts: [],
  toast: null,
  error: null,

  boot: async () => {
    // A capture that went to the clipboard says so, also to screen readers (APP-044).
    await listen<{ result: { copied?: boolean } }>("xs://capture", (e) => {
      if (e.payload.result.copied) get().showToast(i18next.t("ui.editor.capture_copied"));
    });
    await listen<{ kind: string; payload: Record<string, unknown> }>("xs://editor", (e) => {
      const { kind, payload } = e.payload;
      switch (kind) {
        case "documents": {
          const active = (payload.active as string | null) ?? null;
          const changed = active !== get().active;
          const documents = payload.documents as DocSummary[];
          // A capture that arrives with the tray hidden lights its button (TRY-001).
          if (arrived(get().documents, documents) && !useTray.getState().visible)
            useTray.setState({ fresh: true });
          set({ documents, active });
          if (changed) {
            set({ selection: [], cropRect: null, textEdit: null });
            const seen = docEvents;
            void loadDoc(active).then((doc) => {
              // A change reported meanwhile is newer than what this load read.
              if (
                get().active === active &&
                (docEvents === seen || get().doc?.documentId !== active)
              )
                set({ doc });
            });
          }
          break;
        }
        case "saved_effects":
          set({ themesVersion: get().themesVersion + 1 });
          break;
        case "themes":
          set({ themesVersion: get().themesVersion + 1 });
          void loadThemeColors().then((themeColors) => set({ themeColors }));
          break;
        case "colors":
          set({ colors: payload as unknown as EditorState["colors"] });
          break;
        case "progress": {
          // A long fill's progress; null when it ends (EDT-045).
          if (payload.task === "fill")
            set({ fillProgress: (payload.done as number | null) ?? null });
          break;
        }
        case "document": {
          const doc = payload as unknown as DocDetails;
          if (doc.documentId !== get().active) break;
          docEvents += 1;
          // When the canvas moves over the image (a crop, or growing to fit objects,
          // EDT-049, EDT-071), the view keeps the image in place.
          const before = get().doc;
          const view = get().views[doc.documentId];
          if (before?.documentId === doc.documentId && view) {
            const dx = before.crop.x - doc.crop.x;
            const dy = before.crop.y - doc.crop.y;
            if (dx !== 0 || dy !== 0)
              get().setView(doc.documentId, { ...view, cx: view.cx + dx, cy: view.cy + dy });
          }
          // Objects that no longer exist leave the selection (undo, CLI, delete).
          const ids = new Set(doc.objects.map((o) => o.id));
          set((s) => ({ doc, selection: s.selection.filter((id) => ids.has(id)) }));
          break;
        }
        case "tool": {
          const tool = payload.tool as Tool;
          const doc = get().doc;
          // The crop box starts on the whole canvas; leaving the tool cancels it (EDT-049).
          set({
            tool,
            cropRect:
              tool === "crop" && doc
                ? { x: 0, y: 0, w: doc.canvas.width, h: doc.canvas.height }
                : null,
            // Cropping works on the image, not on objects.
            ...(tool === "crop" ? { selection: [] } : {}),
          });
          break;
        }
        case "styles":
          set({ styles: payload as unknown as Styles });
          break;
        case "ai":
          aiProgress(payload.requestId as string, payload.text as string);
          break;
        case "zoom":
          set({
            zoomRequest: {
              mode: payload.mode as string,
              value: (payload.value as number | null) ?? null,
              seq: ++zoomSeq,
              documentId: (payload.documentId as string | null) ?? null,
            },
          });
          break;
        case "quit":
          set({
            decision: { kind: "quit", pending: payload.pending as Pending[], targets: "all" },
          });
          break;
        case "view":
          // Abrir biblioteca from the tray icon, `editor.show --view` (LIB-001).
          showView(payload.view as MainView);
          break;
        case "library":
          // The folder changed: the Library view, and the tray's "Not found" (TRY-002).
          void run<{ active: string | null; documents: DocSummary[] }>("document.list")
            .then((r) => set({ documents: r.documents }))
            .catch(() => undefined);
          if (useLibrary.getState().loaded)
            void useLibrary
              .getState()
              .load()
              .catch(() => undefined);
          break;
      }
    });
    const state = await run<{
      tool: Tool;
      tools: Tool[];
      styles: Styles;
      active: string | null;
      documents: DocSummary[];
      view: MainView | null;
    }>("editor.state");
    // A view asked for before this window loaded (LIB-001).
    if (state.view) showView(state.view);
    const recovery = await run<{ items: RecoveryItem[] }>("editor.recovery_list");
    // The color selector's theme colors, My colors and recent ones (EDT-019).
    void loadThemeColors().then((themeColors) => set({ themeColors }));
    void run<EditorState["colors"]>("editor.colors")
      .then((colors) => set({ colors }))
      .catch(() => undefined);
    set({
      tool: state.tool,
      tools: state.tools,
      styles: state.styles,
      documents: state.documents,
      active: state.active,
      doc: await loadDoc(state.active),
      recovery: recovery.items,
      ready: true,
    });
  },

  setSelection: (selection) => set({ selection }),
  setView: (id, view) => set((s) => ({ views: { ...s.views, [id]: view } })),
  showToast: (text) => {
    clearTimeout(toastTimer);
    set({ toast: text });
    toastTimer = setTimeout(() => set({ toast: null }), 2500);
  },
  showError: (e) => set({ error: isCommandError(e) ? e.message : String(e) }),
}));

/** Asks the canvas for a zoom: `actual`, `fit`, `original`, `in`, `out` or `percent`
 * with a value (EDT-007). Commands from the CLI arrive the same way. */
export function requestZoom(mode: string, value: number | null = null): void {
  useEditor.setState((s) => ({
    zoomRequest: { mode, value, seq: ++zoomSeq, documentId: s.active },
  }));
}

/** The pending zoom for document `id`, removed so it is applied only once. A request
 * kept after use was applied again to every document shown next (Carlos, 2026-10-24). */
export function takeZoomRequest(id: string): ZoomRequest | null {
  const r = useEditor.getState().zoomRequest;
  if (!r || (r.documentId !== null && r.documentId !== id)) return null;
  useEditor.setState({ zoomRequest: null });
  return r;
}

/** Side panel layout (EDT-005): shown or hidden, width (260–480 px) and tab. A view
 * convenience kept in this window's storage. */
export interface PanelLayout {
  visible: boolean;
  width: number;
  tab: "props" | "styles" | "effects";
}

const PANEL_KEY = "xs.editor.panel";

function loadPanel(): PanelLayout {
  const fallback: PanelLayout = { visible: true, width: 280, tab: "props" };
  try {
    const v = JSON.parse(localStorage.getItem(PANEL_KEY) ?? "null") as Partial<PanelLayout> | null;
    return { ...fallback, ...v };
  } catch {
    return fallback;
  }
}

export const usePanel = create<PanelLayout & { set: (p: Partial<PanelLayout>) => void }>(
  (set, get) => ({
    ...loadPanel(),
    set: (p) => {
      set(p);
      const { visible, width, tab } = get();
      try {
        localStorage.setItem(PANEL_KEY, JSON.stringify({ visible, width, tab }));
      } catch {
        // Not remembered; the panel still changes.
      }
    },
  }),
);

/** Image pixel under the pointer, for the status bar (EDT-011). Kept apart so moving
 * the mouse does not redraw the whole editor. */
export const usePointer = create<{
  point: Point | null;
  /** The same place in the canvas area, in CSS pixels (the `I` color picker). */
  screen: { x: number; y: number } | null;
}>(() => ({ point: null, screen: null }));

/** URL of an object Rust draws for the canvas (text, blur); `version` busts caches. */
/** URL of `path` on the `xsdoc` protocol (ADR 0004). */
export function xsdocUrl(path: string): string {
  return navigator.userAgent.includes("Windows")
    ? `http://xsdoc.localhost/${path}`
    : `xsdoc://localhost/${path}`;
}

export function objectUrl(documentId: string, objectId: string, version: string): string {
  return xsdocUrl(`${documentId}/object/${objectId}?v=${version}`);
}

/** A picture of a document's `images/` folder: a floating image (EDT-078). */
export function imageUrl(documentId: string, name: string): string {
  return xsdocUrl(`${documentId}/image/${encodeURIComponent(name)}`);
}

/** URL of a document's pixels for the canvas (`xsdoc` protocol, ADR 0004). */
export function docUrl(id: string, part: "base" | "thumb", version = 0): string {
  const path = `${id}/${part}?v=${version}`;
  return navigator.userAgent.includes("Windows")
    ? `http://xsdoc.localhost/${path}`
    : `xsdoc://localhost/${path}`;
}

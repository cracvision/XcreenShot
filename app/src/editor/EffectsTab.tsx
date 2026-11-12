// Effects tab (EDT-053 to EDT-068): every effect with its switch and its options. The
// active ones form an ordered stack (drag to reorder). The target is the document, or
// the selected floating image; with a pixel selection, some effects can be fixed into
// it. Everything runs `document.effects` (and `document.selection_action`).
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState, type DragEvent } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { ColorPicker } from "./ColorPicker";
import { NumberField, SelectField } from "./Fields";
import { selectionAction } from "./pixelSelection";
import { useEditor } from "./state";
import { Dialog } from "../components/Dialog";
import { useContextMenu } from "./menus";
import { sampleUrl } from "./themesUi";

type Effect = Record<string, unknown> & { type: string; enabled?: boolean };

/** The effects of the panel, in order (EDT-053). */
const TYPES = [
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
const DEFAULTS: Record<string, Record<string, unknown>> = {
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

type Field =
  | { k: string; kind: "number"; min: number; max: number; step?: number; unit?: string }
  | { k: string; kind: "select"; options: string[] }
  | { k: string; kind: "color"; transparent?: boolean }
  | { k: string; kind: "check" }
  | { k: string; kind: "text" }
  /** A filter's switch and its value (EDT-062). */
  | { k: string; kind: "filter"; min?: number; max?: number; unit?: string };

const SIDES: Field[] = ["top", "bottom", "left", "right"].map((s) => ({
  k: `sides.${s}`,
  kind: "check",
}));

/** The options of each effect. Dotted keys are inside an object (`sides.top`). */
const FIELDS: Record<string, Field[]> = {
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
function read(e: Effect | undefined, key: string): unknown {
  return key
    .split(".")
    .reduce<unknown>((v, k) => (v as Record<string, unknown> | undefined)?.[k], e);
}

/** The top-level change that sets a dotted key to `value`. */
function change(e: Effect | undefined, key: string, value: unknown): Record<string, unknown> {
  const [head, ...rest] = key.split(".");
  if (!head) return {};
  if (rest.length === 0) return { [head]: value };
  const inner = { ...((e?.[head] as Record<string, unknown> | undefined) ?? {}) };
  inner[rest.join(".")] = value;
  return { [head]: inner };
}

interface SavedEffect {
  id: string;
  name: string;
  stack: Effect[];
}

/** Saved effects (EDT-054): a row of samples; click applies, Shift+click adds at the
 * end, + saves the current stack; right-click renames, updates, deletes or stars. */
function SavedEffects({ target }: { target: Record<string, unknown> }) {
  const { t } = useTranslation();
  const version = useEditor((s) => s.themesVersion);
  const doc = useEditor((s) => s.doc);
  const [list, setList] = useState<SavedEffect[]>([]);
  const [ask, setAsk] = useState<{ id: string | null; name: string } | null>(null);
  const [dragged, setDragged] = useState<string | null>(null);
  useEffect(() => {
    let alive = true;
    void run<{ effects: SavedEffect[] }>("editor.saved_effects")
      .then((r) => alive && setList(r.effects))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [version]);
  if (!doc) return null;
  const act = (params: Record<string, unknown>) =>
    run("editor.saved_effect", { documentId: doc.documentId, ...target, ...params }).catch((e) =>
      useEditor.getState().showError(e),
    );
  const menu = (ev: React.MouseEvent, s: SavedEffect) => {
    ev.preventDefault();
    useContextMenu.setState({
      open: true,
      x: ev.clientX,
      y: ev.clientY,
      label: t("ui.editor.fx.saved.saved"),
      items: [
        {
          id: "fx-rename",
          label: t("ui.editor.fx.saved.rename"),
          onSelect: () => setAsk({ id: s.id, name: s.name }),
        },
        {
          id: "fx-update",
          label: t("ui.editor.fx.saved.update"),
          onSelect: () => void act({ action: "update", id: s.id }),
        },
        {
          id: "fx-favorite",
          label: t("ui.editor.fx.saved.favorite"),
          onSelect: () => void act({ action: "favorite", id: s.id }),
        },
        {
          id: "fx-delete",
          label: t("ui.editor.fx.saved.delete"),
          separator: true,
          onSelect: () => void act({ action: "delete", id: s.id }),
        },
      ],
    });
  };
  return (
    <div className="fx-saved">
      <span className="muted">{t("ui.editor.fx.saved.saved")}</span>
      <div className="style-grid" role="list" aria-label={t("ui.editor.fx.saved.saved")}>
        {list.map((s, i) => (
          <div
            key={s.id}
            className="style-cell"
            role="listitem"
            draggable
            onDragStart={() => setDragged(s.id)}
            onDragOver={(ev) => dragged && dragged !== s.id && ev.preventDefault()}
            onDrop={(ev) => {
              ev.preventDefault();
              if (dragged && dragged !== s.id) void act({ action: "move", id: dragged, to: i });
              setDragged(null);
            }}
          >
            <button
              type="button"
              className="style-sample"
              title={`${s.name} · ${t("ui.editor.fx.saved.apply_help")}`}
              onClick={(ev) => void act({ action: "apply", id: s.id, append: ev.shiftKey })}
              onContextMenu={(ev) => menu(ev, s)}
            >
              <img src={sampleUrl("effects", s.id, s.stack)} alt={s.name} />
            </button>
          </div>
        ))}
        <div className="style-cell temporary" role="listitem">
          <button
            type="button"
            className="style-sample fx-save"
            title={t("ui.editor.fx.saved.save")}
            aria-label={t("ui.editor.fx.saved.save")}
            onClick={() => setAsk({ id: null, name: "" })}
          >
            +
          </button>
        </div>
      </div>
      {ask && (
        <SaveName
          title={ask.id ? t("ui.editor.fx.saved.rename") : t("ui.editor.fx.saved.save_title")}
          value={ask.name}
          onCancel={() => setAsk(null)}
          onOk={(name) => {
            setAsk(null);
            void act(ask.id ? { action: "rename", id: ask.id, name } : { action: "save", name });
          }}
        />
      )}
    </div>
  );
}

function SaveName(props: {
  title: string;
  value: string;
  onCancel: () => void;
  onOk: (v: string) => void;
}) {
  const { t } = useTranslation();
  const [value, setValue] = useState(props.value);
  return (
    <Dialog
      title={props.title}
      onCancel={props.onCancel}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            disabled={!value.trim()}
            onClick={() => props.onOk(value.trim())}
          >
            {t("ui.editor.ok")}
          </button>
          <button type="button" className="btn" onClick={props.onCancel}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <label className="prop-field">
        <span>{t("ui.editor.fx.saved.name")}</span>
        <input
          type="text"
          data-autofocus
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && value.trim()) props.onOk(value.trim());
          }}
        />
      </label>
    </Dialog>
  );
}

export function EffectsTab() {
  const { t } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const selection = useEditor((s) => s.selection);
  const [open_, setOpen] = useState<string | null>(null);
  const [dragged, setDragged] = useState<number | null>(null);
  if (!doc) return <p className="muted">{t("ui.editor.no_document")}</p>;
  // A single floating image selected: its own stack (EDT-053).
  const image =
    selection.length === 1
      ? doc.objects.find((o) => o.id === selection[0] && o.type === "image")
      : undefined;
  const stack = ((image ? (image as unknown as { effects?: Effect[] }).effects : doc.effects) ??
    []) as Effect[];
  const target = image ? { objectId: image.id } : {};
  const fx = (params: Record<string, unknown>) =>
    void run("document.effects", { documentId: doc.documentId, ...target, ...params }).catch((e) =>
      useEditor.getState().showError(e),
    );
  const name = (k: string) => t(`ui.editor.fx.${k}.name`);
  const label = (type: string, key: string) =>
    t(`ui.editor.fx.${type}.${key.replaceAll(".", "_")}`);
  const pixels = doc.pixelSelection;

  const toggle = (type: string) => {
    const i = stack.findIndex((e) => e.type === type);
    const e = stack[i];
    if (!e) fx({ action: "add", effect: { type, ...DEFAULTS[type] } });
    else fx({ action: "update", index: i, effect: { enabled: e.enabled === false } });
  };
  const update = (i: number, changes: Record<string, unknown>) =>
    fx({ action: "update", index: i, effect: changes });
  // Stack first (its order), then the effects not used yet.
  const order = [
    ...stack.map((e, i) => ({ type: e.type, index: i })),
    ...TYPES.filter((type) => !stack.some((e) => e.type === type)).map((type) => ({
      type,
      index: -1,
    })),
  ];

  const fieldNode = (type: string, i: number, f: Field) => {
    const e = stack[i];
    const v = read(e, f.k);
    const set = (value: unknown) => update(i, change(e, f.k, value));
    const lbl = label(type, f.k);
    switch (f.kind) {
      case "number":
        return (
          <NumberField
            key={f.k}
            label={lbl}
            value={typeof v === "number" ? v : Number(v ?? f.min)}
            min={f.min}
            max={f.max}
            step={f.step ?? 1}
            unit={f.unit ?? ""}
            onCommit={set}
          />
        );
      case "select":
        return (
          <SelectField
            key={f.k}
            label={lbl}
            value={String(v ?? f.options[0])}
            options={f.options.map((o) => ({ value: o, label: t(`ui.editor.fx.opt.${o}`) }))}
            onCommit={(o) => set(type === "rotate" ? Number(o) : o)}
          />
        );
      case "color":
        return (
          <ColorPicker
            key={f.k}
            label={lbl}
            value={(v as string | null | undefined) ?? (f.transparent ? null : "#000000")}
            allowTransparent={f.transparent ?? false}
            onCommit={set}
          />
        );
      case "check":
        return (
          <label key={f.k} className="panel-check">
            <input type="checkbox" checked={v === true} onChange={(ev) => set(ev.target.checked)} />
            {lbl}
          </label>
        );
      case "text":
        return (
          <label key={f.k} className="prop-field">
            <span>{lbl}</span>
            <input
              type="text"
              defaultValue={String(v ?? "")}
              onBlur={(ev) => ev.target.value !== String(v ?? "") && set(ev.target.value)}
              onKeyDown={(ev) => ev.key === "Enter" && (ev.target as HTMLInputElement).blur()}
            />
          </label>
        );
      case "filter": {
        const on =
          v === true ||
          (typeof v === "object" && v !== null && (v as { on?: boolean }).on === true);
        const value =
          typeof v === "object" && v !== null ? (v as { value?: number }).value : undefined;
        return (
          <div key={f.k} className="fx-filter">
            <label className="panel-check">
              <input
                type="checkbox"
                checked={on}
                onChange={(ev) => set({ on: ev.target.checked, value: value ?? f.max ?? 0 })}
              />
              {lbl}
            </label>
            {on && f.min !== undefined && f.max !== undefined && (
              <NumberField
                label={`${lbl} · ${t("ui.editor.fx.value")}`}
                value={value ?? f.max}
                min={f.min}
                max={f.max}
                step={1}
                unit={f.unit ?? ""}
                onCommit={(n) => set({ on: true, value: n })}
              />
            )}
          </div>
        );
      }
    }
  };

  const onDrop = (to: number) => (ev: DragEvent) => {
    ev.preventDefault();
    if (dragged !== null && to >= 0 && dragged !== to) fx({ action: "move", index: dragged, to });
    setDragged(null);
  };

  return (
    <div className="effects-tab">
      <div className="fx-head">
        <span className="muted">
          {image ? t("ui.editor.fx.target_image") : t("ui.editor.fx.target_document")}
        </span>
        <button
          type="button"
          className="btn small"
          disabled={!stack.some((e) => e.enabled !== false)}
          onClick={() => fx({ action: "clear" })}
        >
          {t("ui.editor.fx.remove_all")}
        </button>
      </div>
      <SavedEffects target={target} />
      {pixels && (
        // Fixed into the selected pixels (EDT-053, EDT-065).
        <div className="fx-selection">
          <span className="muted">{t("ui.editor.fx.selection")}</span>
          {(["filters", "color", "replace", "spotlight"] as const).map((type) => (
            <button
              key={type}
              type="button"
              className="btn small"
              onClick={() =>
                void selectionAction("effect", {
                  effect:
                    type === "spotlight"
                      ? {
                          type,
                          dim: 50,
                          blur: 20,
                          zoom: 150,
                          borderColor: "#FFFFFF",
                          borderWidth: 3,
                        }
                      : { type, ...DEFAULTS[type] },
                })
              }
            >
              {name(type)}
            </button>
          ))}
        </div>
      )}
      <ul className="fx-list" aria-label={t("ui.editor.fx.title")}>
        {order.map(({ type, index }) => {
          const e = index >= 0 ? stack[index] : undefined;
          const on = Boolean(e && e.enabled !== false);
          const expanded = open_ === type && e;
          return (
            <li
              key={type}
              className="fx-item"
              draggable={index >= 0}
              onDragStart={() => setDragged(index)}
              onDragOver={(ev) => index >= 0 && dragged !== null && ev.preventDefault()}
              onDrop={onDrop(index)}
            >
              <div className="fx-row">
                <input
                  type="checkbox"
                  role="switch"
                  className="switch"
                  checked={on}
                  aria-label={name(type)}
                  onChange={() => toggle(type)}
                />
                <button
                  type="button"
                  className="fx-name"
                  aria-expanded={Boolean(expanded)}
                  disabled={!e}
                  onClick={() => setOpen(expanded ? null : type)}
                >
                  {name(type)}
                </button>
              </div>
              {expanded && index >= 0 && (
                <div className="fx-options">
                  {type === "watermark" && read(e, "source") === "image" && (
                    <button
                      type="button"
                      className="btn small"
                      onClick={() =>
                        void open({
                          filters: [{ name: "PNG, JPG", extensions: ["png", "jpg", "jpeg"] }],
                        }).then((path) => path && update(index, { path }))
                      }
                    >
                      {t("ui.editor.fx.watermark.choose")}
                    </button>
                  )}
                  {type === "edges" && read(e, "style") === "torn" && (
                    <button
                      type="button"
                      className="btn small"
                      onClick={() => update(index, { seed: Math.floor(Math.random() * 1e9) })}
                    >
                      {t("ui.editor.fx.edges.variant")}
                    </button>
                  )}
                  {type === "color" && (
                    <button
                      type="button"
                      className="btn small"
                      onClick={() => update(index, DEFAULTS.color ?? {})}
                    >
                      {t("ui.editor.fx.color.reset")}
                    </button>
                  )}
                  {type === "details" && (
                    <p className="muted hint">{t("ui.editor.fx.details.notice")}</p>
                  )}
                  {(FIELDS[type] ?? []).map((f) => fieldNode(type, index, f))}
                  <button
                    type="button"
                    className="btn small"
                    onClick={() => fx({ action: "remove", index })}
                  >
                    {t("ui.editor.fx.remove")}
                  </button>
                </div>
              )}
            </li>
          );
        })}
      </ul>
    </div>
  );
}

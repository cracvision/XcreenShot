// Effects tab (EDT-053 to EDT-068): every effect with its switch and its options. The
// active ones form an ordered stack (drag to reorder). The target is the document, or
// the selected floating image; with a pixel selection, some effects can be fixed into
// it. Everything runs `document.effects` (and `document.selection_action`).
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState, type DragEvent } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { selectionAction } from "./pixelSelection";
import { useEditor } from "./state";
import { Dialog } from "../components/Dialog";
import { useContextMenu } from "./menus";
import { sampleUrl } from "./themesUi";
import { DEFAULTS, TYPES } from "./effects";
import { EffectFields } from "./EffectFields";
import { read, type Effect } from "./effectOptions";

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
                  <EffectFields
                    effect={stack[index] as Effect}
                    onChange={(c) => update(index, c)}
                  />
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

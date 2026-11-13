// The Presets section of the capture panel (CAP-058 to CAP-064): run, add, edit,
// rename, duplicate, delete with Undo, reorder (drag or Alt+arrows), shortcut, export
// and import. The list is the one the icon menu shows, in the same order.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { Menu, type MenuItem } from "../components/Menu";
import { useIndicator } from "../components/useIndicator";
import { display } from "../lib/accelerator";
import { isCommandError, run, type PanelSettings } from "../lib/commands";
import { ShortcutRecorder } from "../prefs/ShortcutRecorder";
import { useApp, useSetting } from "../store";
import { ImageOptions, UniversalOptions } from "./CaptureOptions";
import {
  askImport,
  createPreset,
  deletePresets,
  duplicatePreset,
  exportPresets,
  importPresets,
  loadPresets,
  movePreset,
  renamePreset,
  restoreBuiltins,
  runPreset,
  savePreset,
  undoDelete,
  updateFromCurrent,
  usePresetUi,
} from "./presetActions";
import {
  DEFAULTS,
  dropIndex,
  filterPresets,
  FILTER_FROM,
  keyTarget,
  type Preset,
} from "./presetsState";

export function PresetsSection({ tab }: { tab: PanelSettings["tab"] }) {
  const { t } = useTranslation();
  const presets = usePresetUi((s) => s.presets);
  const renaming = usePresetUi((s) => s.renaming);
  const undo = usePresetUi((s) => s.undo);
  const message = usePresetUi((s) => s.message);
  const confirm = useSetting("capture.confirmPresetDelete", true);
  const [filter, setFilter] = useState("");
  const [selected, setSelected] = useState<string | null>(null);
  const [picked, setPicked] = useState<string[]>([]);
  const [drag, setDrag] = useState<{ from: number; over: number; after: boolean } | null>(null);
  const rows = useRef<Record<string, HTMLLIElement | null>>({});

  useEffect(() => {
    void loadPresets();
    // A .xpreset opened with XcreenShot (double click, CAP-063): its import asks here.
    const takeOpened = () =>
      void run<{ paths: string[] }>("app.take_opening", { kind: "presets" })
        .then((r) => (r.paths.length > 0 ? askImport(r.paths) : undefined))
        .catch(() => undefined);
    takeOpened();
    // Changed elsewhere: the icon menu's shortcut, the CLI, an import.
    const off = listen<{ kind: string; payload?: { kind?: string } }>("xs://editor", (e) => {
      if (e.payload.kind === "presets") void loadPresets();
      if (e.payload.kind === "opening" && e.payload.payload?.kind === "presets") takeOpened();
    });
    const offKeys = listen("xs://shortcuts", () => void loadPresets());
    // A preset's interval or schedule starts or ends (CAP-058).
    const offTimed = listen("xs://interval", () => void loadPresets());
    return () => {
      void off.then((f) => f());
      void offKeys.then((f) => f());
      void offTimed.then((f) => f());
    };
  }, []);

  const shown = filterPresets(presets, presets.length >= FILTER_FROM ? filter : "");
  const filtering = shown.length !== presets.length || filter.trim() !== "";
  const current = shown.find((p) => p.id === selected) ?? shown[0] ?? null;
  // Several chosen with Ctrl or Shift + click, to export or delete together (CAP-058,
  // CAP-060). Only those still shown count.
  const chosen = shown.filter((p) => picked.includes(p.id));
  const several = chosen.length > 1;
  const pick = (e: React.MouseEvent, p: Preset, index: number) => {
    if (e.ctrlKey || e.metaKey) {
      setPicked((list) =>
        list.includes(p.id) ? list.filter((id) => id !== p.id) : [...list, p.id],
      );
    } else if (e.shiftKey && current) {
      const from = shown.findIndex((x) => x.id === current.id);
      const [a, b] = from < index ? [from, index] : [index, from];
      setPicked(shown.slice(a, b + 1).map((x) => x.id));
    } else {
      setPicked([p.id]);
    }
    setSelected(p.id);
  };
  // What a row's Export and Delete act on: the chosen ones, if it is one of them.
  const targets = (p: Preset) => (several && picked.includes(p.id) ? chosen : [p]);

  const focusRow = (id: string) => {
    setSelected(id);
    rows.current[id]?.focus();
  };

  const askDelete = (list: Preset[]) => {
    if (confirm) usePresetUi.setState({ dialog: { kind: "delete", presets: list } });
    else void deletePresets(list);
  };

  const onKey = (e: React.KeyboardEvent, p: Preset, index: number) => {
    if (renaming) return;
    if (e.altKey && (e.key === "ArrowUp" || e.key === "ArrowDown")) {
      // CAP-061: Alt + arrows move the preset.
      e.preventDefault();
      if (filtering) return;
      const to = e.key === "ArrowUp" ? index - 1 : index + 1;
      if (to < 0 || to >= presets.length) return;
      void movePreset(p, to).then(() => focusRow(p.id));
      return;
    }
    const target = keyTarget(index, shown.length, e.key);
    if (target !== null) {
      e.preventDefault();
      const next = shown[target];
      if (next) focusRow(next.id);
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      void runPreset(p);
    } else if (e.key === "F2") {
      e.preventDefault();
      usePresetUi.setState({ renaming: p.id });
    } else if (e.key === "Delete") {
      e.preventDefault();
      askDelete(targets(p));
    }
  };

  const itemsFor = (p: Preset): MenuItem[] => [
    { id: "run", label: t("ui.panel.presets.run"), onSelect: () => void runPreset(p) },
    {
      id: "edit",
      label: t("ui.panel.presets.edit"),
      separator: true,
      onSelect: () => usePresetUi.setState({ dialog: { kind: "edit", preset: p } }),
    },
    {
      id: "rename",
      label: t("ui.panel.presets.rename"),
      shortcut: "F2",
      onSelect: () => usePresetUi.setState({ renaming: p.id }),
    },
    {
      id: "duplicate",
      label: t("ui.panel.presets.duplicate"),
      onSelect: () => void duplicatePreset(p),
    },
    {
      id: "update",
      label: t("ui.panel.presets.update_current"),
      disabled: tab !== p.kind,
      title: t("ui.panel.presets.update_other_kind"),
      onSelect: () => usePresetUi.setState({ dialog: { kind: "update", preset: p } }),
    },
    {
      id: "shortcut",
      label: t("ui.panel.presets.shortcut"),
      onSelect: () => usePresetUi.setState({ dialog: { kind: "shortcut", preset: p } }),
    },
    {
      id: "export",
      label: t("ui.panel.presets.export"),
      onSelect: () => void exportPresets(targets(p)),
    },
    {
      id: "delete",
      label: t("ui.panel.presets.delete"),
      shortcut: t("ui.panel.presets.delete_key"),
      separator: true,
      onSelect: () => askDelete(targets(p)),
    },
  ];

  return (
    <section className="presets" aria-labelledby="presets-title">
      <div className="presets-head">
        <h2 id="presets-title">{t("ui.panel.presets.title")}</h2>
        <Menu
          triggerClass="btn icon small"
          align="end"
          chevron={false}
          ariaLabel={t("ui.panel.presets.add")}
          title={t("ui.panel.presets.add")}
          label={<span aria-hidden="true">＋</span>}
          items={[
            {
              id: "current",
              label: t("ui.panel.presets.from_current"),
              onSelect: () => void createPreset({ fromCurrent: true }),
            },
            {
              id: "new_image",
              label: t("ui.panel.presets.new_image"),
              separator: true,
              onSelect: () =>
                usePresetUi.setState({ dialog: { kind: "new", presetKind: "image" } }),
            },
            {
              id: "new_universal",
              label: t("ui.panel.presets.new_universal"),
              onSelect: () =>
                usePresetUi.setState({ dialog: { kind: "new", presetKind: "universal" } }),
            },
          ]}
        />
      </div>
      {presets.length >= FILTER_FROM && (
        <input
          type="search"
          className="presets-filter"
          placeholder={t("ui.panel.presets.filter")}
          aria-label={t("ui.panel.presets.filter")}
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        />
      )}
      {presets.length === 0 ? (
        <p className="muted presets-empty">{t("ui.panel.presets.empty")}</p>
      ) : (
        <>
          {several && (
            <div className="presets-picked" role="status">
              <span>{t("ui.panel.presets.picked", { count: chosen.length })}</span>
              <button
                type="button"
                className="btn small"
                onClick={() => void exportPresets(chosen)}
              >
                {t("ui.panel.presets.export")}
              </button>
              <button type="button" className="btn small" onClick={() => askDelete(chosen)}>
                {t("ui.panel.presets.delete")}
              </button>
              <button type="button" className="btn small" onClick={() => setPicked([])}>
                {t("ui.panel.presets.unpick")}
              </button>
            </div>
          )}
          <ul
            className="presets-list"
            role="listbox"
            aria-multiselectable="true"
            aria-labelledby="presets-title"
          >
            {shown.map((p, index) => {
              const isCurrent = current?.id === p.id;
              const dropping =
                drag && drag.over === index && drag.from !== index
                  ? drag.after
                    ? " drop-after"
                    : " drop-before"
                  : "";
              return (
                <li
                  key={p.id}
                  ref={(el) => {
                    rows.current[p.id] = el;
                  }}
                  role="option"
                  aria-selected={isCurrent || (several && picked.includes(p.id))}
                  tabIndex={isCurrent ? 0 : -1}
                  className={`preset-row${isCurrent ? " current" : ""}${
                    several && picked.includes(p.id) ? " picked" : ""
                  }${dropping}`}
                  draggable={!filtering && renaming !== p.id}
                  title={filtering ? t("ui.panel.presets.no_drag_filter") : undefined}
                  onFocus={() => setSelected(p.id)}
                  onClick={(e) => pick(e, p, index)}
                  onDoubleClick={() => void runPreset(p)}
                  onKeyDown={(e) => onKey(e, p, index)}
                  onDragStart={(e) => {
                    e.dataTransfer.effectAllowed = "move";
                    e.dataTransfer.setData("text/plain", p.id);
                    setDrag({ from: index, over: index, after: false });
                  }}
                  onDragOver={(e) => {
                    if (!drag) return;
                    e.preventDefault();
                    const box = e.currentTarget.getBoundingClientRect();
                    const after = e.clientY > box.top + box.height / 2;
                    if (drag.over !== index || drag.after !== after)
                      setDrag({ ...drag, over: index, after });
                  }}
                  onDrop={(e) => {
                    e.preventDefault();
                    if (!drag) return;
                    const moving = presets[drag.from];
                    const to = dropIndex(drag.from, drag.over, drag.after);
                    setDrag(null);
                    if (moving && to !== drag.from) void movePreset(moving, to);
                  }}
                  onDragEnd={() => setDrag(null)}
                >
                  <span className="preset-grip" aria-hidden="true">
                    ⋮⋮
                  </span>
                  <span className={`preset-kind kind-${p.kind}`} aria-hidden="true">
                    {p.kind === "universal" ? "U" : "I"}
                  </span>
                  <span className="preset-text">
                    {renaming === p.id ? (
                      <RenameField preset={p} />
                    ) : (
                      <span className="preset-name">{p.name}</span>
                    )}
                    <span className="preset-summary">
                      {t(`ui.panel.presets.kind_${p.kind}`)} · {p.summary}
                      {p.running && (
                        <span className="preset-running"> · {t("ui.panel.presets.running")}</span>
                      )}
                    </span>
                  </span>
                  {p.warnings.length > 0 && (
                    <span
                      className="preset-warning"
                      role="img"
                      aria-label={p.warnings[0]?.message}
                      title={p.warnings[0]?.message}
                    >
                      !
                    </span>
                  )}
                  {p.shortcutDisplay && <kbd className="preset-keys">{p.shortcutDisplay}</kbd>}
                  <button
                    type="button"
                    className="btn small preset-run"
                    tabIndex={-1}
                    onClick={(e) => {
                      e.stopPropagation();
                      void runPreset(p);
                    }}
                  >
                    {t("ui.panel.capture")}
                  </button>
                  <Menu
                    triggerClass="btn icon small preset-more"
                    align="end"
                    chevron={false}
                    ariaLabel={t("ui.panel.presets.more", { name: p.name })}
                    title={t("ui.panel.presets.more", { name: p.name })}
                    label={<span aria-hidden="true">⋯</span>}
                    items={itemsFor(p)}
                  />
                </li>
              );
            })}
          </ul>
        </>
      )}
      {undo && (
        <div className="presets-undo" role="status">
          <span>{undo.text}</span>
          <button type="button" className="btn small" onClick={() => void undoDelete()}>
            {t("ui.panel.presets.undo")}
          </button>
        </div>
      )}
      {message && (
        <p className={message.error ? "panel-status" : "presets-message"} role="status">
          {message.text}
        </p>
      )}
      <PresetDialogs tab={tab} />
    </section>
  );
}

/** The name, edited in its row (CAP-059, CAP-060): Enter keeps it, Esc does not. */
function RenameField({ preset }: { preset: Preset }) {
  const { t } = useTranslation();
  const [value, setValue] = useState(preset.name);
  const field = useRef<HTMLInputElement>(null);
  useEffect(() => {
    field.current?.focus();
    field.current?.select();
  }, []);
  return (
    <input
      ref={field}
      className="preset-rename"
      aria-label={t("ui.panel.presets.name")}
      value={value}
      maxLength={64}
      onChange={(e) => setValue(e.target.value)}
      onClick={(e) => e.stopPropagation()}
      onKeyDown={(e) => {
        e.stopPropagation();
        if (e.key === "Enter") void renamePreset(preset, value);
        if (e.key === "Escape") usePresetUi.setState({ renaming: null });
      }}
      onBlur={() => void renamePreset(preset, value)}
    />
  );
}

function PresetDialogs({ tab }: { tab: PanelSettings["tab"] }) {
  const dialog = usePresetUi((s) => s.dialog);
  if (!dialog) return null;
  switch (dialog.kind) {
    case "edit":
      return <EditDialog preset={dialog.preset} />;
    case "new":
      return <EditDialog kind={dialog.presetKind} />;
    case "shortcut":
      return <ShortcutDialog preset={dialog.preset} />;
    case "delete":
      return <DeleteDialog presets={dialog.presets} />;
    case "update":
      return <UpdateDialog preset={dialog.preset} tab={tab} />;
    case "import":
      return <ImportDialog paths={dialog.paths} rows={dialog.rows} />;
    case "builtins":
      return <BuiltinsDialog modified={dialog.modified} />;
  }
}

const close = () => usePresetUi.setState({ dialog: null });

/** Edit preset (CAP-060), or a New preset from the defaults (CAP-059). */
function EditDialog({ preset, kind }: { preset?: Preset; kind?: "universal" | "image" }) {
  const { t } = useTranslation();
  const presetKind = preset?.kind ?? kind ?? "image";
  const [name, setName] = useState(preset?.name ?? "");
  const [universal, setUniversal] = useState<PanelSettings["universal"]>({
    ...DEFAULTS.universal,
    ...(presetKind === "universal"
      ? (preset?.settings as Partial<PanelSettings["universal"]>)
      : {}),
  });
  const [image, setImage] = useState<PanelSettings["image"]>({
    ...DEFAULTS.image,
    ...(presetKind === "image" ? (preset?.settings as Partial<PanelSettings["image"]>) : {}),
  });
  const settings = presetKind === "universal" ? universal : image;
  const save = async () => {
    if (preset) {
      await savePreset(preset, name, settings as unknown as Record<string, unknown>);
    } else {
      await createPreset({
        kind: presetKind,
        settings,
        ...(name.trim() ? { name } : {}),
      });
    }
  };
  return (
    <Dialog
      title={preset ? t("ui.panel.presets.edit_title") : t(`ui.panel.presets.new_${presetKind}`)}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.panel.cancel")}
          </button>
          <button type="button" className="btn primary" onClick={() => void save()}>
            {t("ui.panel.presets.save")}
          </button>
        </>
      }
    >
      <div className="preset-editor">
        <label className="panel-field">
          <span>{t("ui.panel.presets.name")}</span>
          <input
            data-autofocus
            value={name}
            maxLength={64}
            placeholder={preset ? undefined : t("ui.panel.presets.name_auto")}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
        <p className="muted">{t(`ui.panel.presets.kind_${presetKind}`)}</p>
        {presetKind === "universal" ? (
          <UniversalOptions
            value={universal}
            onChange={(u) => setUniversal({ ...universal, ...u })}
          />
        ) : (
          <ImageOptions value={image} onChange={(i) => setImage({ ...image, ...i })} />
        )}
      </div>
    </Dialog>
  );
}

/** A preset's global shortcut (CAP-062), with Reassign for one in use (APP-018). */
function ShortcutDialog({ preset }: { preset: Preset }) {
  const { t } = useTranslation();
  const { shortcuts, layout, ping } = useApp();
  const platform = shortcuts?.platform ?? ping?.platform ?? "windows";
  const [error, setError] = useState<string | null>(null);
  const [reassign, setReassign] = useState<{ shortcut: string; other: string } | null>(null);
  const titleOf = (id: string) => shortcuts?.rows.find((r) => r.command === id)?.title ?? id;
  const assign = async (shortcut: string | null, again = false) => {
    setError(null);
    try {
      await run("shortcuts.set", {
        command: preset.slot,
        scope: "global",
        shortcut,
        reassign: again,
      });
      close();
      await loadPresets();
    } catch (e) {
      if (isCommandError(e) && e.details?.["conflict"] === "duplicate" && shortcut) {
        setReassign({ shortcut, other: titleOf(String(e.details["command"])) });
      } else {
        setError(isCommandError(e) ? e.message : String(e));
      }
    }
  };
  if (reassign)
    return (
      <Dialog
        title={t("ui.shortcuts.reassign_title")}
        onCancel={() => setReassign(null)}
        actions={
          <>
            <button type="button" className="btn" onClick={() => setReassign(null)}>
              {t("ui.shortcuts.cancel")}
            </button>
            <button
              type="button"
              className="btn primary"
              onClick={() => void assign(reassign.shortcut, true)}
            >
              {t("ui.shortcuts.reassign")}
            </button>
          </>
        }
      >
        <p>
          {t("ui.shortcuts.reassign_body", {
            shortcut: display(reassign.shortcut, platform, layout),
            command: reassign.other,
          })}
        </p>
      </Dialog>
    );
  return (
    <Dialog
      title={t("ui.panel.presets.shortcut_title", { name: preset.name })}
      onCancel={close}
      actions={
        <>
          {preset.shortcut && (
            <button type="button" className="btn" onClick={() => void assign(null)}>
              {t("ui.panel.presets.shortcut_remove")}
            </button>
          )}
          <button type="button" className="btn" onClick={close}>
            {t("ui.panel.cancel")}
          </button>
        </>
      }
    >
      <p className="muted">{t("ui.panel.presets.shortcut_help")}</p>
      <ShortcutRecorder
        value={preset.shortcut}
        platform={platform}
        layout={layout}
        conflict={error !== null}
        label={t("ui.panel.presets.shortcut")}
        onCommit={(s) => void assign(s)}
      />
      {error && (
        <p className="panel-status" role="alert">
          {error}
        </p>
      )}
    </Dialog>
  );
}

function DeleteDialog({ presets }: { presets: Preset[] }) {
  const { t } = useTranslation();
  const [dontAsk, setDontAsk] = useState(false);
  const first = presets[0]?.name ?? "";
  return (
    <Dialog
      title={t("ui.panel.presets.delete_title", { count: presets.length, name: first })}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.panel.cancel")}
          </button>
          <button
            type="button"
            className="btn danger"
            data-autofocus
            onClick={() => void deletePresets(presets, dontAsk)}
          >
            {t("ui.panel.presets.delete")}
          </button>
        </>
      }
    >
      <p>{t("ui.panel.presets.delete_body")}</p>
      <label className="panel-check">
        <input type="checkbox" checked={dontAsk} onChange={(e) => setDontAsk(e.target.checked)} />
        {t("ui.panel.presets.dont_ask")}
      </label>
    </Dialog>
  );
}

function UpdateDialog({ preset, tab }: { preset: Preset; tab: PanelSettings["tab"] }) {
  const { t } = useTranslation();
  return (
    <Dialog
      title={t("ui.panel.presets.update_title", { name: preset.name })}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.panel.cancel")}
          </button>
          <button
            type="button"
            className="btn primary"
            data-autofocus
            onClick={() => void updateFromCurrent(preset)}
          >
            {t("ui.panel.presets.update_go")}
          </button>
        </>
      }
    >
      <p>{t("ui.panel.presets.update_body", { tab: t(`ui.panel.tab_${tab}`) })}</p>
    </Dialog>
  );
}

/** Import presets (CAP-063): which ones, and what to do with a name that exists. */
function ImportDialog({
  paths,
  rows,
}: {
  paths: string[];
  rows: {
    name: string;
    exists: boolean;
    shortcut: string | null;
    shortcutTaken: boolean;
    problem: string | null;
  }[];
}) {
  const { t } = useTranslation();
  const usable = rows.filter((r) => !r.problem);
  const [chosen, setChosen] = useState<string[]>(usable.map((r) => r.name));
  const [onConflict, setOnConflict] = useState<"keepBoth" | "replace" | "skip">("keepBoth");
  const seg = useRef<HTMLDivElement>(null);
  useIndicator(seg, onConflict);
  const anyExists = rows.some((r) => r.exists && chosen.includes(r.name));
  return (
    <Dialog
      title={t("ui.panel.presets.import_title", { count: rows.length })}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.panel.cancel")}
          </button>
          <button
            type="button"
            className="btn primary"
            disabled={chosen.length === 0}
            onClick={() => void importPresets(paths, chosen, onConflict)}
          >
            {t("ui.panel.presets.import_go", { count: chosen.length })}
          </button>
        </>
      }
    >
      <ul className="preset-import-list">
        {rows.map((r) => (
          <li key={r.name}>
            <label className="panel-check">
              <input
                type="checkbox"
                disabled={r.problem !== null}
                checked={chosen.includes(r.name)}
                onChange={(e) =>
                  setChosen(
                    e.target.checked ? [...chosen, r.name] : chosen.filter((n) => n !== r.name),
                  )
                }
              />
              <span>
                {r.name}
                {r.shortcut && <kbd className="preset-keys">{r.shortcut}</kbd>}
                {r.exists && (
                  <span className="muted"> · {t("ui.panel.presets.import_exists")}</span>
                )}
                {r.shortcutTaken && (
                  <span className="muted"> · {t("ui.panel.presets.import_taken")}</span>
                )}
                {r.problem && <span className="warning-text"> · {r.problem}</span>}
              </span>
            </label>
          </li>
        ))}
      </ul>
      {anyExists && (
        <div className="preset-conflict">
          <span>{t("ui.panel.presets.import_conflict")}</span>
          <div
            className="seg"
            ref={seg}
            role="radiogroup"
            aria-label={t("ui.panel.presets.import_conflict")}
          >
            <span className="seg-indicator" aria-hidden="true" />
            {(["keepBoth", "replace", "skip"] as const).map((c) => (
              <button
                key={c}
                type="button"
                role="radio"
                aria-checked={onConflict === c}
                aria-pressed={onConflict === c}
                onClick={() => setOnConflict(c)}
              >
                {t(`ui.panel.presets.conflict_${c}`)}
              </button>
            ))}
          </div>
        </div>
      )}
    </Dialog>
  );
}

function BuiltinsDialog({ modified }: { modified: string[] }) {
  const { t } = useTranslation();
  return (
    <Dialog
      title={t("ui.panel.presets.builtins_title")}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.panel.presets.builtins_keep")}
          </button>
          <button type="button" className="btn primary" onClick={() => void restoreBuiltins(true)}>
            {t("ui.panel.presets.builtins_reset")}
          </button>
        </>
      }
    >
      <p>{t("ui.panel.presets.builtins_body", { names: modified.join(", ") })}</p>
    </Dialog>
  );
}

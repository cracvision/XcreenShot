// The options of the capture panel's tabs (CAP-002, CAP-010), shared by the panel and
// the preset editor (CAP-060).
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  isCommandError,
  run,
  type FileDest,
  type PanelSettings,
  type RegionProps,
  type ScrollProps,
} from "../lib/commands";
import { Menu, type MenuItem } from "../components/Menu";
import { tenStep } from "../lib/numberKeys";

const SELECTIONS = ["region", "window", "fullscreen"] as const;
/** The Image tab also scrolls (CAP-034, CAP-035), takes several areas (CAP-033) and
 * extracts text (SMT-005). */
const IMAGE_SELECTIONS = [...SELECTIONS, "picker", "scrolling", "menu", "multi", "text"] as const;

interface Common {
  selection: (typeof IMAGE_SELECTIONS)[number];
  copy: boolean;
  cursor: boolean;
  open: "editor" | "none";
}

/** Controls shared by both tabs, in the CAP-002 order (Selection … Capture cursor).
 * With Extract text, Open in and Capture cursor do not apply: off, with the reason. */
function CommonOptions<T extends Common>({
  value,
  onChange,
  selections = SELECTIONS,
}: {
  value: T;
  onChange: (patch: Partial<Common>) => void;
  selections?: readonly Common["selection"][];
}) {
  const { t } = useTranslation();
  const id = useId();
  const text = value.selection === "text";
  // A scrolling or menu capture has no pointer (CAP-002).
  const noCursor = text || value.selection === "scrolling" || value.selection === "menu";
  return (
    <>
      <label className="panel-field" htmlFor={`${id}-sel`}>
        <span>{t("ui.panel.selection")}</span>
        <select
          id={`${id}-sel`}
          value={value.selection}
          onChange={(e) => onChange({ selection: e.target.value as Common["selection"] })}
        >
          {selections.map((s) => (
            <option key={s} value={s}>
              {t(`ui.panel.sel_${s}`)}
            </option>
          ))}
        </select>
      </label>
      <label
        className="panel-field"
        htmlFor={`${id}-open`}
        title={text ? t("ui.panel.text_no_open") : undefined}
      >
        <span>{t("ui.panel.open_in")}</span>
        <select
          id={`${id}-open`}
          disabled={text}
          value={value.open}
          onChange={(e) => onChange({ open: e.target.value as Common["open"] })}
        >
          <option value="editor">{t("ui.panel.open_editor")}</option>
          <option value="none">{t("ui.panel.open_none")}</option>
        </select>
      </label>
      <label className="panel-check">
        <input
          type="checkbox"
          checked={value.copy}
          onChange={(e) => onChange({ copy: e.target.checked })}
        />
        {text ? t("ui.panel.copy_text") : t("ui.panel.copy")}
      </label>
      {/* The pointer arrives as its own object, to move or delete (CAP-047). */}
      <label
        className={`panel-check${noCursor ? " disabled" : ""}`}
        title={
          text
            ? t("ui.panel.text_no_cursor")
            : noCursor
              ? t("ui.panel.scroll_no_cursor")
              : t("ui.panel.cursor_help")
        }
      >
        <input
          type="checkbox"
          disabled={noCursor}
          checked={value.cursor && !noCursor}
          onChange={(e) => onChange({ cursor: e.target.checked })}
        />
        {t("ui.panel.cursor")}
      </label>
    </>
  );
}

export function UniversalOptions({
  value,
  onChange,
}: {
  value: PanelSettings["universal"];
  onChange: (patch: Partial<PanelSettings["universal"]>) => void;
}) {
  const { t } = useTranslation();
  return (
    <>
      {/* Its list has no Extract text, so no patch carries it. */}
      <CommonOptions
        value={value}
        onChange={(p) => onChange(p as Partial<PanelSettings["universal"]>)}
      />
      <label className="panel-check">
        <input
          type="checkbox"
          checked={value.delay5}
          onChange={(e) => onChange({ delay5: e.target.checked })}
        />
        {t("ui.panel.delay5")}
      </label>
      <ShareOptions share={value.share} file={value.file} onChange={onChange} />
    </>
  );
}

export function ImageOptions({
  value,
  onChange,
}: {
  value: PanelSettings["image"];
  onChange: (patch: Partial<PanelSettings["image"]>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  // What is being typed; the saved value shows otherwise.
  const [draft, setDraft] = useState<string | null>(null);
  const commitSeconds = () => {
    const n = Math.round(Number(draft));
    if (draft !== null && Number.isFinite(n) && n >= 1 && n <= 60) onChange({ delay: n });
    setDraft(null);
  };
  return (
    <>
      <CommonOptions value={value} onChange={onChange} selections={IMAGE_SELECTIONS} />
      {value.selection === "region" && (
        <RegionOptions
          value={value.region}
          onChange={(r) => onChange({ region: { ...value.region, ...r } })}
        />
      )}
      {value.selection === "menu" && (
        <fieldset className="panel-props">
          <label className="panel-check">
            <input
              type="checkbox"
              checked={value.menu.submenus}
              onChange={(e) => onChange({ menu: { ...value.menu, submenus: e.target.checked } })}
            />
            {t("ui.panel.menu_submenus")}
          </label>
          <label className="panel-check">
            <input
              type="checkbox"
              checked={value.menu.menubar}
              onChange={(e) => onChange({ menu: { ...value.menu, menubar: e.target.checked } })}
            />
            {t("ui.panel.menu_menubar")}
          </label>
        </fieldset>
      )}
      {value.selection === "multi" && (
        <AreasOptions
          value={value.areas}
          onChange={(a) => onChange({ areas: { ...value.areas, ...a } })}
        />
      )}
      {value.selection === "scrolling" && (
        <ScrollOptions
          value={value.scroll}
          onChange={(s) => onChange({ scroll: { ...value.scroll, ...s } })}
        />
      )}
      <div className="panel-row">
        <label className="panel-field" htmlFor={`${id}-timing`}>
          <span>{t("ui.panel.timing")}</span>
          <select
            id={`${id}-timing`}
            value={value.timing}
            onChange={(e) => onChange({ timing: e.target.value as "none" | "delay" })}
          >
            <option value="none">{t("ui.panel.timing_none")}</option>
            <option value="delay">{t("ui.panel.timing_delay")}</option>
          </select>
        </label>
        {value.timing === "delay" && (
          <label className="panel-field narrow" htmlFor={`${id}-sec`}>
            <span>{t("ui.panel.seconds")}</span>
            <input
              id={`${id}-sec`}
              type="number"
              min={1}
              max={60}
              step={1}
              value={draft ?? String(value.delay)}
              title={t("ui.panel.seconds_range")}
              onChange={(e) => setDraft(e.target.value)}
              onBlur={commitSeconds}
              onKeyDown={(e) => {
                tenStep(e, (n) => setDraft(String(n)));
                if (e.key === "Enter") commitSeconds();
              }}
            />
          </label>
        )}
      </div>
      {value.selection !== "text" && (
        <>
          <EffectsOptions value={value.effects} onChange={(effects) => onChange({ effects })} />
          <ShareOptions share={value.share} file={value.file} onChange={onChange} />
        </>
      )}
    </>
  );
}

/** The properties of Selection = Scrolling (CAP-034, CAP-035): manual or automatic, and
 * how an automatic one moves. */
function ScrollOptions({
  value,
  onChange,
}: {
  value: ScrollProps;
  onChange: (patch: Partial<ScrollProps>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  const auto = value.type === "auto";
  const [wait, setWait] = useState<string | null>(null);
  const commitWait = () => {
    const n = Math.round(Number(wait) * 10) / 10;
    if (wait !== null && Number.isFinite(n) && n >= 0 && n <= 5) onChange({ wait: n });
    setWait(null);
  };
  return (
    <fieldset className="panel-props">
      <label className="panel-field" htmlFor={`${id}-type`}>
        <span>{t("ui.panel.scroll_type")}</span>
        <select
          id={`${id}-type`}
          value={value.type}
          onChange={(e) => onChange({ type: e.target.value as ScrollProps["type"] })}
        >
          <option value="manual">{t("ui.panel.scroll_manual")}</option>
          <option value="auto">{t("ui.panel.scroll_auto")}</option>
        </select>
      </label>
      {auto && (
        <>
          <label className="panel-field" htmlFor={`${id}-dir`}>
            <span>{t("ui.panel.scroll_direction")}</span>
            <select
              id={`${id}-dir`}
              value={value.direction}
              onChange={(e) => onChange({ direction: e.target.value as ScrollProps["direction"] })}
            >
              <option value="vertical">{t("ui.panel.scroll_vertical")}</option>
              <option value="horizontal">{t("ui.panel.scroll_horizontal")}</option>
              <option value="both">{t("ui.panel.scroll_both")}</option>
            </select>
          </label>
          <label className="panel-field" htmlFor={`${id}-from`}>
            <span>{t("ui.panel.scroll_start_from")}</span>
            <select
              id={`${id}-from`}
              value={value.startFrom}
              onChange={(e) => onChange({ startFrom: e.target.value as ScrollProps["startFrom"] })}
            >
              <option value="start">{t("ui.panel.scroll_from_start")}</option>
              <option value="current">{t("ui.panel.scroll_from_current")}</option>
            </select>
          </label>
          <label className="panel-field narrow" htmlFor={`${id}-wait`}>
            <span>{t("ui.panel.scroll_wait")}</span>
            <input
              id={`${id}-wait`}
              type="number"
              min={0}
              max={5}
              step={0.1}
              value={wait ?? String(value.wait)}
              title={t("ui.panel.scroll_wait_range")}
              onChange={(e) => setWait(e.target.value)}
              onBlur={commitWait}
              onKeyDown={(e) => {
                if (e.key === "Enter") commitWait();
              }}
            />
          </label>
          <label className="panel-check">
            <input
              type="checkbox"
              checked={value.restore}
              onChange={(e) => onChange({ restore: e.target.checked })}
            />
            {t("ui.panel.scroll_restore")}
          </label>
        </>
      )}
    </fieldset>
  );
}

interface MonitorInfo {
  name: string;
  rect: { x: number; y: number; width: number; height: number };
  primary: boolean;
}

/** The properties of Selection = Region (CAP-026): a fixed size, and maybe a fixed place
 * (monitor, X and Y in physical pixels from its top left), or chosen on screen. */
function RegionOptions({
  value,
  onChange,
}: {
  value: RegionProps;
  onChange: (patch: Partial<RegionProps>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [drafts, setDrafts] = useState<Partial<Record<"width" | "height" | "x" | "y", string>>>({});
  const [note, setNote] = useState<string | null>(null);
  useEffect(() => {
    void run<MonitorInfo[]>("capture.monitors").then(setMonitors);
  }, []);
  // The largest the region can be: the chosen monitor, or the biggest one.
  const chosen = monitors.find((m) => m.name === value.monitor);
  const limit =
    value.positioned && chosen
      ? chosen.rect
      : monitors.reduce(
          (a, m) => ({
            ...a,
            width: Math.max(a.width, m.rect.width),
            height: Math.max(a.height, m.rect.height),
          }),
          { x: 0, y: 0, width: 100000, height: 100000 },
        );
  const maxW = monitors.length ? limit.width : 100000;
  const maxH = monitors.length ? limit.height : 100000;
  const commit = (field: "width" | "height" | "x" | "y") => {
    const text = drafts[field];
    setDrafts((d) => ({ ...d, [field]: undefined }));
    if (text === undefined) return;
    let n = Math.round(Number(text));
    if (!Number.isFinite(n)) return;
    if (field === "width" || field === "height") {
      const max = field === "width" ? maxW : maxH;
      n = Math.max(1, n);
      if (n > max) {
        n = max;
        setNote(t("ui.panel.region_limited", { width: maxW, height: maxH }));
      } else {
        setNote(null);
      }
    }
    onChange({ [field]: n });
  };
  const number = (field: "width" | "height" | "x" | "y", label: string) => (
    <label className="panel-field narrow" htmlFor={`${id}-${field}`}>
      <span>{label}</span>
      <input
        id={`${id}-${field}`}
        type="number"
        min={field === "width" || field === "height" ? 1 : 0}
        step={1}
        value={drafts[field] ?? String(value[field])}
        onChange={(e) => setDrafts((d) => ({ ...d, [field]: e.target.value }))}
        onBlur={() => commit(field)}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit(field);
        }}
      />
    </label>
  );
  const pick = () => {
    setNote(null);
    run<{ monitor: string; x: number; y: number; width: number; height: number }>(
      "capture.pick_area",
    )
      .then((a) => onChange({ fixed: true, positioned: true, ...a }))
      .catch((e: unknown) => {
        if (isCommandError(e) && e.code === "cancelled") return;
        setNote(isCommandError(e) ? e.message : String(e));
      });
  };
  return (
    <fieldset className="panel-props">
      <label className="panel-check">
        <input
          type="checkbox"
          checked={value.fixed}
          onChange={(e) => onChange({ fixed: e.target.checked })}
        />
        {t("ui.panel.region_fixed")}
      </label>
      {value.fixed && (
        <>
          <div className="panel-row">
            {number("width", t("ui.panel.region_width"))}
            {number("height", t("ui.panel.region_height"))}
          </div>
          <label className="panel-check">
            <input
              type="checkbox"
              checked={value.positioned}
              onChange={(e) =>
                onChange({
                  positioned: e.target.checked,
                  monitor:
                    value.monitor ||
                    monitors.find((m) => m.primary)?.name ||
                    monitors[0]?.name ||
                    "",
                })
              }
            />
            {t("ui.panel.region_positioned")}
          </label>
          {value.positioned && (
            <>
              <label className="panel-field" htmlFor={`${id}-mon`}>
                <span>{t("ui.panel.region_monitor")}</span>
                <select
                  id={`${id}-mon`}
                  value={value.monitor}
                  onChange={(e) => onChange({ monitor: e.target.value })}
                >
                  {!monitors.some((m) => m.name === value.monitor) && (
                    <option value={value.monitor}>{value.monitor || "—"}</option>
                  )}
                  {monitors.map((m) => (
                    <option key={m.name} value={m.name}>
                      {m.name} ({m.rect.width} × {m.rect.height})
                    </option>
                  ))}
                </select>
              </label>
              <div className="panel-row">
                {number("x", t("ui.panel.region_x"))}
                {number("y", t("ui.panel.region_y"))}
              </div>
            </>
          )}
        </>
      )}
      <button type="button" className="btn small" onClick={pick}>
        {t("ui.panel.region_pick")}
      </button>
      {note && (
        <p className="panel-note" role="status">
          {note}
        </p>
      )}
    </fieldset>
  );
}

/** The properties of Selection = Multiple areas (CAP-033): how the areas are put together. */
function AreasOptions({
  value,
  onChange,
}: {
  value: PanelSettings["image"]["areas"];
  onChange: (patch: Partial<PanelSettings["image"]["areas"]>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  const [gap, setGap] = useState<string | null>(null);
  const commitGap = () => {
    const n = Math.round(Number(gap));
    if (gap !== null && Number.isFinite(n) && n >= 0 && n <= 500) onChange({ gap: n });
    setGap(null);
  };
  return (
    <fieldset className="panel-props">
      <label className="panel-field" htmlFor={`${id}-comp`}>
        <span>{t("ui.panel.areas_composition")}</span>
        <select
          id={`${id}-comp`}
          value={value.composition}
          onChange={(e) => onChange({ composition: e.target.value as typeof value.composition })}
        >
          <option value="original">{t("ui.panel.areas_original")}</option>
          <option value="stacked">{t("ui.panel.areas_stacked")}</option>
          <option value="row">{t("ui.panel.areas_row")}</option>
        </select>
      </label>
      {value.composition !== "original" && (
        <label className="panel-field narrow" htmlFor={`${id}-gap`}>
          <span>{t("ui.panel.areas_gap")}</span>
          <input
            id={`${id}-gap`}
            type="number"
            min={0}
            max={500}
            step={1}
            value={gap ?? String(value.gap)}
            onChange={(e) => setGap(e.target.value)}
            onBlur={commitGap}
            onKeyDown={(e) => {
              if (e.key === "Enter") commitGap();
            }}
          />
        </label>
      )}
    </fieldset>
  );
}

/** Share (CAP-052): the File destination for now, with its options (CAP-053): folder,
 * prefix, name pattern with its markers and a preview, format and what to do when the
 * name is taken. */
function ShareOptions({
  share,
  file,
  onChange,
}: {
  share: "file"[];
  file: FileDest;
  onChange: (patch: { share?: "file"[]; file?: FileDest }) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  const on = share.includes("file");
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState<
    Partial<Record<"prefix" | "pattern" | "next" | "quality", string>>
  >({});
  const [info, setInfo] = useState<{
    folder: string;
    name: string;
    next: number;
    counter: boolean;
    markers: string[];
  } | null>(null);
  const patch = (p: Partial<FileDest>) => onChange({ file: { ...file, ...p } });
  const drop = (k: "prefix" | "pattern" | "next" | "quality") =>
    setDraft((d) => Object.fromEntries(Object.entries(d).filter(([key]) => key !== k)));
  useEffect(() => {
    if (!on || !open) return;
    run<{ folder: string; name: string; next: number; counter: boolean; markers: string[] }>(
      "capture.file_name",
      {
        file: {
          ...file,
          pattern: draft.pattern ?? file.pattern,
          prefix: draft.prefix ?? file.prefix,
        },
      },
    )
      .then(setInfo)
      .catch(() => setInfo(null));
  }, [on, open, file, draft.pattern, draft.prefix]);
  const commitText = (field: "prefix" | "pattern") => {
    const v = draft[field];
    drop(field);
    if (v !== undefined && v !== file[field]) patch({ [field]: v });
  };
  const insert = (marker: string) => {
    const current = draft.pattern ?? file.pattern;
    patch({ pattern: `${current}{${marker}}` });
    drop("pattern");
  };
  const chooseFolder = async () => {
    const dir = await openDialog({ directory: true, title: t("ui.panel.file_folder") });
    if (typeof dir === "string") patch({ folder: dir });
  };
  return (
    <fieldset className="panel-props">
      <legend className="sr-only">{t("ui.panel.share")}</legend>
      <div className="panel-row">
        <label className="panel-check">
          <input
            type="checkbox"
            checked={on}
            onChange={(e) => onChange({ share: e.target.checked ? ["file"] : [] })}
          />
          {t("ui.panel.share_file")}
        </label>
        {on && (
          <button
            type="button"
            className="btn small"
            aria-expanded={open}
            onClick={() => setOpen((o) => !o)}
          >
            {t("ui.panel.file_options")}
          </button>
        )}
      </div>
      {on && open && (
        <>
          <div className="panel-field">
            <span>{t("ui.panel.file_folder")}</span>
            <div className="panel-row">
              <code className="panel-path">
                {file.folder || info?.folder || t("ui.panel.file_default_folder")}
              </code>
              <button type="button" className="btn small" onClick={() => void chooseFolder()}>
                {t("ui.panel.file_choose")}
              </button>
            </div>
          </div>
          <label className="panel-field" htmlFor={`${id}-prefix`}>
            <span>{t("ui.panel.file_prefix")}</span>
            <input
              id={`${id}-prefix`}
              type="text"
              value={draft.prefix ?? file.prefix}
              onChange={(e) => setDraft((d) => ({ ...d, prefix: e.target.value }))}
              onBlur={() => commitText("prefix")}
            />
          </label>
          <div className="panel-field">
            <label htmlFor={`${id}-pattern`}>{t("ui.panel.file_pattern")}</label>
            <div className="panel-row">
              <input
                id={`${id}-pattern`}
                type="text"
                value={draft.pattern ?? file.pattern}
                aria-describedby={`${id}-preview`}
                onChange={(e) => setDraft((d) => ({ ...d, pattern: e.target.value }))}
                onBlur={() => commitText("pattern")}
              />
              <Menu
                label={t("ui.panel.file_insert")}
                triggerClass="btn small"
                align="end"
                items={(info?.markers ?? []).map((m) => ({
                  id: m,
                  label: `{${m}}`,
                  trailing: <span className="muted">{t(`ui.panel.marker_${m}`)}</span>,
                  onSelect: () => insert(m),
                }))}
              />
            </div>
            {info && (
              <span id={`${id}-preview`} className="panel-preview muted">
                {t("ui.panel.file_preview", { name: info.name })}
              </span>
            )}
          </div>
          <div className="panel-row">
            <label className="panel-field" htmlFor={`${id}-format`}>
              <span>{t("ui.panel.file_format")}</span>
              <select
                id={`${id}-format`}
                value={file.format}
                onChange={(e) => patch({ format: e.target.value as FileDest["format"] })}
              >
                <option value="png">PNG</option>
                <option value="jpg">JPG</option>
                <option value="xshot">.xshot</option>
              </select>
            </label>
            {file.format === "jpg" && (
              <label className="panel-field narrow" htmlFor={`${id}-quality`}>
                <span>{t("ui.panel.file_quality")}</span>
                <input
                  id={`${id}-quality`}
                  type="number"
                  min={1}
                  max={100}
                  value={draft.quality ?? String(file.quality)}
                  onChange={(e) => setDraft((d) => ({ ...d, quality: e.target.value }))}
                  onBlur={() => {
                    const n = Math.round(Number(draft.quality));
                    drop("quality");
                    if (Number.isFinite(n) && n >= 1 && n <= 100) patch({ quality: n });
                  }}
                />
              </label>
            )}
          </div>
          <label className="panel-field" htmlFor={`${id}-exists`}>
            <span>{t("ui.panel.file_on_exists")}</span>
            <select
              id={`${id}-exists`}
              value={file.onExists}
              onChange={(e) => patch({ onExists: e.target.value as FileDest["onExists"] })}
            >
              <option value="number">{t("ui.panel.file_number")}</option>
              <option value="overwrite">{t("ui.panel.file_overwrite")}</option>
            </select>
          </label>
          {info?.counter && (
            <label className="panel-field narrow" htmlFor={`${id}-next`}>
              <span>{t("ui.panel.file_next")}</span>
              <input
                id={`${id}-next`}
                type="number"
                min={1}
                value={draft.next ?? String(info.next)}
                onChange={(e) => setDraft((d) => ({ ...d, next: e.target.value }))}
                onBlur={() => {
                  const n = Math.round(Number(draft.next));
                  drop("next");
                  if (Number.isFinite(n) && n >= 1)
                    void run("capture.file_name", { file, next: n }).then((r) =>
                      setInfo(r as typeof info),
                    );
                }}
              />
            </label>
          )}
        </>
      )}
    </fieldset>
  );
}

/** Effects that need nothing from the user, for every capture (CAP-051). */
const CAPTURE_EFFECTS = [
  "border",
  "shadow",
  "edges",
  "corners",
  "perspective",
  "pageCurl",
  "reflection",
  "fade",
  "scale",
  "rotate",
  "filters",
  "color",
  "trim",
  "watermark",
  "details",
] as const;

type Effect = PanelSettings["image"]["effects"][number];

/** Effects on capture (CAP-051): applied to every capture in this order, editable in the
 * editor. Add a type (its defaults) or a saved effect; reorder; remove; None. */
function EffectsOptions({
  value,
  onChange,
}: {
  value: Effect[];
  onChange: (effects: Effect[]) => void;
}) {
  const { t } = useTranslation();
  const [saved, setSaved] = useState<{ id: string; name: string; stack: Effect[] }[]>([]);
  useEffect(() => {
    void run<{ effects: { id: string; name: string; stack: Effect[] }[] }>("editor.saved_effects")
      .then((r) => setSaved(r.effects))
      .catch(() => setSaved([]));
  }, []);
  const name = (e: Effect) => t(`ui.editor.fx.${e.type}.name`);
  const move = (i: number) => {
    if (i <= 0) return;
    const next = [...value];
    [next[i - 1], next[i]] = [next[i] as Effect, next[i - 1] as Effect];
    onChange(next);
  };
  const items: MenuItem[] = [
    ...CAPTURE_EFFECTS.map((type) => ({
      id: type,
      label: t(`ui.editor.fx.${type}.name`),
      onSelect: () => onChange([...value, { type }]),
    })),
    ...(saved.length
      ? [
          {
            id: "saved",
            label: t("ui.panel.effects_saved"),
            separator: true,
            onSelect: () => undefined,
            submenu: saved.map((s) => ({
              id: `saved-${s.id}`,
              label: s.name,
              onSelect: () => onChange([...value, ...s.stack]),
            })),
          },
        ]
      : []),
  ];
  return (
    <fieldset className="panel-props">
      <legend className="panel-legend">{t("ui.panel.effects")}</legend>
      {value.length === 0 ? (
        <p className="panel-preview muted">{t("ui.panel.effects_empty")}</p>
      ) : (
        <ol className="panel-effects">
          {value.map((e, i) => (
            <li key={`${i}-${e.type}`}>
              <span>{name(e)}</span>
              <button
                type="button"
                className="btn icon small"
                disabled={i === 0}
                aria-label={t("ui.panel.effects_up", { name: name(e) })}
                title={t("ui.panel.effects_up", { name: name(e) })}
                onClick={() => move(i)}
              >
                ↑
              </button>
              <button
                type="button"
                className="btn icon small"
                aria-label={t("ui.panel.effects_remove", { name: name(e) })}
                title={t("ui.panel.effects_remove", { name: name(e) })}
                onClick={() => onChange(value.filter((_, k) => k !== i))}
              >
                ×
              </button>
            </li>
          ))}
        </ol>
      )}
      <div className="panel-row">
        <Menu label={t("ui.panel.effects_add")} triggerClass="btn small" items={items} />
        {value.length > 0 && (
          <button type="button" className="btn small" onClick={() => onChange([])}>
            {t("ui.panel.effects_none")}
          </button>
        )}
      </div>
    </fieldset>
  );
}

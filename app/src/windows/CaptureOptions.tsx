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
  type IntervalProps,
  type ScheduleProps,
} from "../lib/commands";
import { Menu, type MenuItem } from "../components/Menu";
import { MicDeviceSelect } from "../components/MicDeviceSelect";
import { EffectFields } from "../editor/EffectFields";
import { dropIndex } from "./presetsState";
import { FIELDS } from "../editor/effectOptions";
import { DEFAULTS as FX_DEFAULTS } from "../editor/effects";
import { useSetting, useStringSetting } from "../store";
import { tenStep } from "../lib/numberKeys";
import { captureModes } from "../features";
import { useApp } from "../store";

const SELECTIONS = ["region", "window", "fullscreen"] as const;
/** The Image tab also scrolls (CAP-034, CAP-035), takes several areas (CAP-033) and
 * extracts text (SMT-005). */
const IMAGE_SELECTIONS = [
  ...SELECTIONS,
  "picker",
  "scrolling",
  "menu",
  "multi",
  "clipboard",
  "file",
  "pdf",
  "webpage",
  "freehand",
  "text",
  "steps",
] as const;

interface Common {
  selection: (typeof IMAGE_SELECTIONS)[number];
  copy: boolean;
  cursor: boolean;
  open: "editor" | "quick" | "none";
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
  const quick = useSetting("experimental.quickAnnotate", false);
  const id = useId();
  const text = value.selection === "text";
  // A scrolling, menu or clipboard capture has no pointer (CAP-002).
  const noCursor =
    text ||
    value.selection === "scrolling" ||
    value.selection === "menu" ||
    value.selection === "clipboard" ||
    value.selection === "file" ||
    value.selection === "pdf" ||
    value.selection === "webpage";
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
          {/* Quick annotation (CAP-057), while its trial is on (APP-034). */}
          {(quick || value.open === "quick") && (
            <option value="quick">{t("ui.panel.open_quick")}</option>
          )}
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

/** The Video tab (VID-001): Selection, the pointer and the system's sound (the same
 * settings as the recording bar and Preferences > Video) and the timer (VID-015). */
export function VideoOptions({
  value,
  onChange,
}: {
  value: PanelSettings["video"];
  onChange: (patch: Partial<PanelSettings["video"]>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  const cursor = useSetting("video.cursor", true);
  const sound = useSetting("video.systemAudio", false);
  // The camera (VID-031): when videos may use it, on with the first camera.
  const camAllowed = useSetting("video.cameraEnabled", true);
  const cam = useSetting("video.camera", true);
  const camDevice = useStringSetting("video.cameraDevice", "");
  // The microphone (VID-039): on with the system's default, its device and level.
  const mic = useSetting("video.microphone", true);
  const micDevice = useStringSetting("video.microphoneDevice", "");
  const micAuto = useSetting("video.microphoneAutoLevel", true);
  // Cursor and keyboard effects (VID-045 to VID-047): colours and sizes in Preferences.
  const highlight = useSetting("video.highlight", false);
  const clicks = useSetting("video.clicks", false);
  const keys = useSetting("video.keys", false);
  // Live drawing (VID-051): its button and shortcut while recording.
  const liveDraw = useSetting("video.liveDraw", true);
  const set = (key: string, v: boolean) =>
    void run("settings.set", { key, value: v }).catch(() => undefined);
  // A pointer effect switches the pointer on (VID-044): the command does it.
  const effect = (name: "highlight" | "clicks" | "keys", v: boolean) =>
    void run("record.effects", { [name]: v }).catch(() => undefined);
  const minutes = [1, 5, 10, 30, 60];
  return (
    <>
      <label className="panel-field" htmlFor={`${id}-vsel`}>
        <span>{t("ui.panel.selection")}</span>
        <select
          id={`${id}-vsel`}
          value={value.selection}
          onChange={(e) =>
            onChange({ selection: e.target.value as PanelSettings["video"]["selection"] })
          }
        >
          <option value="region">{t("ui.panel.video_selection_region")}</option>
          <option value="window">{t("ui.panel.video_selection_window")}</option>
          <option value="fullscreen">{t("ui.panel.video_selection_fullscreen")}</option>
        </select>
      </label>
      <label className="panel-check">
        <input
          type="checkbox"
          checked={cursor}
          onChange={(e) => set("video.cursor", e.target.checked)}
        />
        {t("ui.panel.video_cursor")}
      </label>
      <label className="panel-check">
        <input
          type="checkbox"
          checked={sound}
          onChange={(e) => set("video.systemAudio", e.target.checked)}
        />
        {t("ui.panel.video_sound")}
      </label>
      {camAllowed && (
        <>
          <label className="panel-check">
            <input
              type="checkbox"
              checked={cam}
              onChange={(e) => set("video.camera", e.target.checked)}
            />
            {t("ui.panel.video_camera")}
          </label>
          {cam && (
            <label className="panel-field" htmlFor={`${id}-vcam`}>
              <span>{t("ui.panel.video_camera_device")}</span>
              <MicDeviceSelect
                id={`${id}-vcam`}
                kind="camera"
                value={camDevice}
                onChange={(v) =>
                  void run("settings.set", { key: "video.cameraDevice", value: v }).catch(
                    () => undefined,
                  )
                }
              />
            </label>
          )}
        </>
      )}
      <label className="panel-check">
        <input
          type="checkbox"
          checked={mic}
          onChange={(e) => set("video.microphone", e.target.checked)}
        />
        {t("ui.panel.video_mic")}
      </label>
      {mic && (
        <>
          <label className="panel-field" htmlFor={`${id}-vmic`}>
            <span>{t("ui.panel.video_mic_device")}</span>
            <MicDeviceSelect
              id={`${id}-vmic`}
              value={micDevice}
              onChange={(v) =>
                void run("settings.set", { key: "video.microphoneDevice", value: v }).catch(
                  () => undefined,
                )
              }
            />
          </label>
          <label className="panel-check">
            <input
              type="checkbox"
              checked={micAuto}
              onChange={(e) => set("video.microphoneAutoLevel", e.target.checked)}
            />
            {t("ui.panel.video_mic_auto")}
          </label>
          <button
            type="button"
            className="btn link small"
            onClick={() => void run("record.sound_settings").catch(() => undefined)}
          >
            {t("ui.panel.video_sound_settings")}
          </button>
        </>
      )}
      <label className="panel-check">
        <input
          type="checkbox"
          checked={highlight && cursor}
          onChange={(e) => effect("highlight", e.target.checked)}
        />
        {t("ui.panel.video_highlight")}
      </label>
      <label className="panel-check">
        <input
          type="checkbox"
          checked={clicks && cursor}
          onChange={(e) => effect("clicks", e.target.checked)}
        />
        {t("ui.panel.video_clicks")}
      </label>
      <label className="panel-check">
        <input type="checkbox" checked={keys} onChange={(e) => effect("keys", e.target.checked)} />
        {t("ui.panel.video_keys")}
      </label>
      <label className="panel-check">
        <input
          type="checkbox"
          checked={liveDraw}
          onChange={(e) => set("video.liveDraw", e.target.checked)}
        />
        {t("ui.panel.video_livedraw")}
      </label>
      <label className="panel-field" htmlFor={`${id}-vlim`}>
        <span>{t("ui.panel.video_limit")}</span>
        <select
          id={`${id}-vlim`}
          value={String(value.limit)}
          onChange={(e) => onChange({ limit: Number(e.target.value) })}
        >
          <option value="0">{t("ui.panel.video_limit_none")}</option>
          {minutes.map((m) => (
            <option key={m} value={String(m * 60)}>
              {t("ui.panel.video_limit_min", { n: m })}
            </option>
          ))}
        </select>
      </label>
      {/* Open in (VID-052): the editor, or only the library and the tray. */}
      <label className="panel-field" htmlFor={`${id}-vopen`}>
        <span>{t("ui.panel.open_in")}</span>
        <select
          id={`${id}-vopen`}
          value={value.open}
          onChange={(e) => onChange({ open: e.target.value as PanelSettings["video"]["open"] })}
        >
          <option value="editor">{t("ui.panel.open_editor")}</option>
          <option value="none">{t("ui.panel.open_none")}</option>
        </select>
      </label>
      <p className="muted hint">{t("ui.panel.video_quality_hint")}</p>
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
  const settings = useApp((s) => s.settings);
  // What is being typed; the saved value shows otherwise.
  const [draft, setDraft] = useState<string | null>(null);
  const commitSeconds = () => {
    const n = Math.round(Number(draft));
    if (draft !== null && Number.isFinite(n) && n >= 1 && n <= 60) onChange({ delay: n });
    setDraft(null);
  };
  return (
    <>
      <CommonOptions
        value={value}
        onChange={onChange}
        selections={captureModes(IMAGE_SELECTIONS, settings, value.selection)}
      />
      {value.selection === "fullscreen" && (
        <label className="panel-check">
          <input
            type="checkbox"
            checked={value.windowsAsObjects}
            onChange={(e) => onChange({ windowsAsObjects: e.target.checked })}
          />
          {t("ui.panel.windows_as_objects")}
        </label>
      )}
      {value.selection === "freehand" && (
        <fieldset className="panel-props">
          <label className="panel-field">
            <span>{t("ui.panel.freehand_shape")}</span>
            <select
              value={value.freehand}
              onChange={(e) =>
                onChange({ freehand: e.target.value as PanelSettings["image"]["freehand"] })
              }
            >
              <option value="free">{t("ui.panel.freehand_free")}</option>
              <option value="polygon">{t("ui.panel.freehand_polygon")}</option>
              <option value="ellipse">{t("ui.panel.freehand_ellipse")}</option>
            </select>
          </label>
        </fieldset>
      )}
      {value.selection === "webpage" && (
        <WebOptions value={value.web} onChange={(w) => onChange({ web: { ...value.web, ...w } })} />
      )}
      {value.selection === "file" && (
        <SourceFileOptions
          path={value.sourceFile}
          onChange={(sourceFile) => onChange({ sourceFile })}
        />
      )}
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
      {/* The step recorder's area and title (SMT-022). */}
      {value.selection === "steps" && (
        <fieldset className="panel-props">
          <label className="panel-field" htmlFor={`${id}-steps-area`}>
            <span>{t("ui.panel.steps_area")}</span>
            <select
              id={`${id}-steps-area`}
              value={value.steps.area}
              onChange={(e) =>
                onChange({
                  steps: {
                    ...value.steps,
                    area: e.target.value as PanelSettings["image"]["steps"]["area"],
                  },
                })
              }
            >
              <option value="all">{t("ui.panel.steps_area_all")}</option>
              <option value="monitor">{t("ui.panel.steps_area_monitor")}</option>
              <option value="window">{t("ui.panel.steps_area_window")}</option>
            </select>
          </label>
          <label className="panel-field" htmlFor={`${id}-steps-title`}>
            <span>{t("ui.panel.steps_title")}</span>
            <input
              id={`${id}-steps-title`}
              type="text"
              value={value.steps.title}
              placeholder={t("ui.panel.steps_title_auto")}
              onChange={(e) => onChange({ steps: { ...value.steps, title: e.target.value } })}
            />
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
            onChange={(e) =>
              onChange({ timing: e.target.value as PanelSettings["image"]["timing"] })
            }
          >
            <option value="none">{t("ui.panel.timing_none")}</option>
            <option value="delay">{t("ui.panel.timing_delay")}</option>
            {/* Interval and Scheduled do not apply to the step recorder (SMT-022). */}
            <option value="interval" disabled={value.selection === "steps"}>
              {t("ui.panel.timing_interval")}
            </option>
            <option value="scheduled" disabled={value.selection === "steps"}>
              {t("ui.panel.timing_scheduled")}
            </option>
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
      {value.timing === "scheduled" && (
        <ScheduleOptions
          value={value.schedule}
          onChange={(s) => onChange({ schedule: { ...value.schedule, ...s } })}
        />
      )}
      {value.timing === "interval" && (
        <IntervalOptions
          value={value.interval}
          onChange={(i) => onChange({ interval: { ...value.interval, ...i } })}
        />
      )}
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
      <label className="panel-check">
        <input
          type="checkbox"
          checked={value.fixed}
          onChange={(e) => onChange({ fixed: e.target.checked })}
        />
        {t("ui.panel.scroll_fixed")}
      </label>
    </fieldset>
  );
}

/** Selection = Web page (CAP-037): the view's width, the scale, the wait after loading
 * and whether content that loads on scrolling is loaded first. */
function WebOptions({
  value,
  onChange,
}: {
  value: PanelSettings["image"]["web"];
  onChange: (patch: Partial<PanelSettings["image"]["web"]>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  const [width, setWidth] = useState<string | null>(null);
  const [wait, setWait] = useState<string | null>(null);
  const commitWidth = () => {
    const n = Math.round(Number(width));
    if (width !== null && Number.isFinite(n)) onChange({ width: Math.min(3840, Math.max(320, n)) });
    setWidth(null);
  };
  const commitWait = () => {
    const n = Math.round(Number(wait) * 10) / 10;
    if (wait !== null && Number.isFinite(n)) onChange({ wait: Math.min(10, Math.max(0, n)) });
    setWait(null);
  };
  return (
    <fieldset className="panel-props">
      <label className="panel-field" htmlFor={`${id}-w`}>
        <span>{t("ui.panel.web_width")}</span>
        <input
          id={`${id}-w`}
          type="number"
          min={320}
          max={3840}
          list={`${id}-widths`}
          value={width ?? String(value.width)}
          onChange={(e) => setWidth(e.target.value)}
          onBlur={commitWidth}
          onKeyDown={(e) => {
            if (e.key === "Enter") commitWidth();
          }}
        />
        <datalist id={`${id}-widths`}>
          {[375, 768, 1280, 1440, 1920].map((w) => (
            <option key={w} value={w} />
          ))}
        </datalist>
      </label>
      <label className="panel-field" htmlFor={`${id}-s`}>
        <span>{t("ui.panel.web_scale")}</span>
        <select
          id={`${id}-s`}
          value={value.scale}
          onChange={(e) => onChange({ scale: Number(e.target.value) as 0 | 1 | 2 })}
        >
          <option value={0}>{t("ui.panel.web_scale_auto")}</option>
          <option value={1}>1×</option>
          <option value={2}>2×</option>
        </select>
      </label>
      <label className="panel-field narrow" htmlFor={`${id}-wait`}>
        <span>{t("ui.panel.web_wait")}</span>
        <input
          id={`${id}-wait`}
          type="number"
          min={0}
          max={10}
          step={0.5}
          value={wait ?? String(value.wait)}
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
          checked={value.lazy}
          onChange={(e) => onChange({ lazy: e.target.checked })}
        />
        {t("ui.panel.web_lazy")}
      </label>
    </fieldset>
  );
}

/** Selection = File (CAP-040): a fixed file used without asking, or ask each time. */
function SourceFileOptions({ path, onChange }: { path: string; onChange: (path: string) => void }) {
  const { t } = useTranslation();
  const choose = async () => {
    const picked = await openDialog({
      multiple: false,
      filters: [
        {
          name: t("ui.panel.file_filter"),
          extensions: [
            "png",
            "jpg",
            "jpeg",
            "gif",
            "bmp",
            "tif",
            "tiff",
            "webp",
            "heic",
            "heif",
            "ico",
          ],
        },
      ],
    });
    if (typeof picked === "string") onChange(picked);
  };
  return (
    <fieldset className="panel-props">
      <legend className="panel-legend">{t("ui.panel.source_file")}</legend>
      <span className="panel-path">{path || t("ui.panel.source_file_none")}</span>
      <div className="panel-row">
        <button type="button" className="btn small" onClick={() => void choose()}>
          {t("ui.panel.source_file_choose")}
        </button>
        {path && (
          <button type="button" className="btn small" onClick={() => onChange("")}>
            {t("ui.panel.source_file_clear")}
          </button>
        )}
      </div>
    </fieldset>
  );
}

/** Timing = Scheduled (CAP-050): the day (empty is today), the minute and the repeat. */
function ScheduleOptions({
  value,
  onChange,
}: {
  value: ScheduleProps;
  onChange: (patch: Partial<ScheduleProps>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  const today = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  const todayText = `${today.getFullYear()}-${pad(today.getMonth() + 1)}-${pad(today.getDate())}`;
  return (
    <fieldset className="panel-props">
      <label className="panel-field" htmlFor={`${id}-date`}>
        <span>{t("ui.panel.schedule_date")}</span>
        <input
          id={`${id}-date`}
          type="date"
          min={todayText}
          value={value.date || todayText}
          onChange={(e) => onChange({ date: e.target.value === todayText ? "" : e.target.value })}
        />
      </label>
      <label className="panel-field" htmlFor={`${id}-time`}>
        <span>{t("ui.panel.schedule_time")}</span>
        <input
          id={`${id}-time`}
          type="time"
          value={value.time}
          onChange={(e) => {
            if (/^\d{2}:\d{2}$/.test(e.target.value)) onChange({ time: e.target.value });
          }}
        />
      </label>
      <label className="panel-field" htmlFor={`${id}-repeat`}>
        <span>{t("ui.panel.repeat")}</span>
        <select
          id={`${id}-repeat`}
          value={value.repeat}
          onChange={(e) => onChange({ repeat: e.target.value as ScheduleProps["repeat"] })}
        >
          <option value="once">{t("ui.panel.repeat_once")}</option>
          <option value="daily">{t("ui.panel.repeat_daily")}</option>
          <option value="weekdays">{t("ui.panel.repeat_weekdays")}</option>
          <option value="weekly">{t("ui.panel.repeat_weekly")}</option>
        </select>
      </label>
    </fieldset>
  );
}

/** Timing = Interval (CAP-049): every how many seconds (3 at least, with a notice),
 * when it stops, and whether a capture like the last one is dropped. */
function IntervalOptions({
  value,
  onChange,
}: {
  value: IntervalProps;
  onChange: (patch: Partial<IntervalProps>) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  const [every, setEvery] = useState<string | null>(null);
  const [raised, setRaised] = useState(false);
  const [count, setCount] = useState<string | null>(null);
  const [minutes, setMinutes] = useState<string | null>(null);
  const commitEvery = () => {
    const n = Math.round(Number(every));
    if (every !== null && Number.isFinite(n) && n > 0) {
      setRaised(n < 3);
      onChange({ every: Math.min(86400, Math.max(3, n)) });
    }
    setEvery(null);
  };
  const commitWhole = (text: string | null, max: number, apply: (n: number) => void) => {
    const n = Math.round(Number(text));
    if (text !== null && Number.isFinite(n) && n >= 1) apply(Math.min(max, n));
  };
  return (
    <fieldset className="panel-props">
      <label className="panel-field narrow" htmlFor={`${id}-every`}>
        <span>{t("ui.panel.every")}</span>
        <input
          id={`${id}-every`}
          type="number"
          min={3}
          max={86400}
          step={1}
          value={every ?? String(value.every)}
          aria-describedby={raised ? `${id}-raised` : undefined}
          onChange={(e) => setEvery(e.target.value)}
          onBlur={commitEvery}
          onKeyDown={(e) => {
            tenStep(e, (n) => setEvery(String(n)));
            if (e.key === "Enter") commitEvery();
          }}
        />
      </label>
      {raised && (
        <p className="panel-note" id={`${id}-raised`} role="status">
          {t("ui.panel.every_min")}
        </p>
      )}
      <label className="panel-field" htmlFor={`${id}-stop`}>
        <span>{t("ui.panel.stop_when")}</span>
        <select
          id={`${id}-stop`}
          value={value.stop}
          onChange={(e) => onChange({ stop: e.target.value as IntervalProps["stop"] })}
        >
          <option value="manual">{t("ui.panel.stop_manual")}</option>
          <option value="count">{t("ui.panel.stop_count")}</option>
          <option value="duration">{t("ui.panel.stop_duration")}</option>
          <option value="time">{t("ui.panel.stop_time")}</option>
        </select>
      </label>
      {value.stop === "count" && (
        <label className="panel-field narrow" htmlFor={`${id}-count`}>
          <span>{t("ui.panel.stop_captures")}</span>
          <input
            id={`${id}-count`}
            type="number"
            min={1}
            max={100000}
            value={count ?? String(value.count)}
            onChange={(e) => setCount(e.target.value)}
            onBlur={() => {
              commitWhole(count, 100000, (n) => onChange({ count: n }));
              setCount(null);
            }}
          />
        </label>
      )}
      {value.stop === "duration" && (
        <label className="panel-field narrow" htmlFor={`${id}-minutes`}>
          <span>{t("ui.panel.stop_minutes")}</span>
          <input
            id={`${id}-minutes`}
            type="number"
            min={1}
            max={10080}
            value={minutes ?? String(value.minutes)}
            onChange={(e) => setMinutes(e.target.value)}
            onBlur={() => {
              commitWhole(minutes, 10080, (n) => onChange({ minutes: n }));
              setMinutes(null);
            }}
          />
        </label>
      )}
      {value.stop === "time" && (
        <label className="panel-field narrow" htmlFor={`${id}-until`}>
          <span>{t("ui.panel.stop_at")}</span>
          <input
            id={`${id}-until`}
            type="time"
            value={value.until}
            onChange={(e) => {
              if (/^\d{2}:\d{2}$/.test(e.target.value)) onChange({ until: e.target.value });
            }}
          />
        </label>
      )}
      <label className="panel-check">
        <input
          type="checkbox"
          checked={value.discard}
          onChange={(e) => onChange({ discard: e.target.checked })}
        />
        {t("ui.panel.discard_same")}
      </label>
      {value.discard && (
        <label className="panel-field" htmlFor={`${id}-sens`}>
          <span>{t("ui.panel.sensitivity")}</span>
          <select
            id={`${id}-sens`}
            value={value.sensitivity}
            onChange={(e) =>
              onChange({ sensitivity: e.target.value as IntervalProps["sensitivity"] })
            }
          >
            <option value="exact">{t("ui.panel.sensitivity_exact")}</option>
            <option value="small">{t("ui.panel.sensitivity_small")}</option>
          </select>
        </label>
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
              <option value="ask">{t("ui.panel.file_ask_exists")}</option>
            </select>
          </label>
          <label className="panel-check">
            <input
              type="checkbox"
              checked={file.ask}
              onChange={(e) => patch({ ask: e.target.checked })}
            />
            {t("ui.panel.file_ask")}
          </label>
          <label className="panel-check">
            <input
              type="checkbox"
              checked={file.metadata}
              onChange={(e) => patch({ metadata: e.target.checked })}
            />
            {t("ui.panel.file_metadata")}
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
  // The effect whose options are open (CAP-051).
  const [open_, setOpen] = useState<number | null>(null);
  // Dragging an effect to another place (CAP-051).
  const [drag, setDrag] = useState<{ from: number; over: number; after: boolean } | null>(null);
  const drop = () => {
    if (!drag) return;
    const to = dropIndex(drag.from, drag.over, drag.after);
    setDrag(null);
    if (to === drag.from) return;
    const next = [...value];
    const [moving] = next.splice(drag.from, 1);
    if (moving) next.splice(to, 0, moving);
    setOpen(null);
    onChange(next);
  };
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
      onSelect: () => onChange([...value, { type, ...(FX_DEFAULTS[type] ?? {}) }]),
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
        <p className="panel-empty muted">{t("ui.panel.effects_empty")}</p>
      ) : (
        <ol className="panel-effects">
          {value.map((e, i) => (
            <li
              key={`${i}-${e.type}`}
              className={
                drag && drag.over === i && drag.from !== i
                  ? drag.after
                    ? "drop-after"
                    : "drop-before"
                  : undefined
              }
              draggable
              onDragStart={(ev) => {
                ev.dataTransfer.effectAllowed = "move";
                ev.dataTransfer.setData("text/plain", e.type);
                setDrag({ from: i, over: i, after: false });
              }}
              onDragOver={(ev) => {
                if (!drag) return;
                ev.preventDefault();
                const box = ev.currentTarget.getBoundingClientRect();
                const after = ev.clientY > box.top + Math.min(box.height, 36) / 2;
                if (drag.over !== i || drag.after !== after) setDrag({ ...drag, over: i, after });
              }}
              onDrop={(ev) => {
                ev.preventDefault();
                drop();
              }}
              onDragEnd={() => setDrag(null)}
            >
              <span className="preset-grip" aria-hidden="true">
                ⋮⋮
              </span>
              <span>{name(e)}</span>
              {(FIELDS[e.type] ?? []).length > 0 && (
                <button
                  type="button"
                  className="btn icon small"
                  aria-expanded={open_ === i}
                  aria-label={t("ui.panel.effects_options", { name: name(e) })}
                  title={t("ui.panel.effects_options", { name: name(e) })}
                  onClick={() => setOpen(open_ === i ? null : i)}
                >
                  ⚙
                </button>
              )}
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
              {open_ === i && (
                <div className="panel-fx-options">
                  <EffectFields
                    capture
                    effect={e}
                    onChange={(c) => onChange(value.map((x, k) => (k === i ? { ...x, ...c } : x)))}
                  />
                </div>
              )}
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

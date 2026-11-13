// Capture presets in the panel (CAP-058 to CAP-064): their shape and the small rules of
// the list (filter, keyboard moves, where a drag drops).
import type { PanelSettings } from "../lib/commands";

export interface Preset {
  id: string;
  name: string;
  kind: "universal" | "image";
  settings: Record<string, unknown>;
  summary: string;
  builtin: string | null;
  shortcut: string | null;
  shortcutDisplay: string | null;
  /** Its row in Preferences > Shortcuts (`preset.<id>`). */
  slot: string;
  running: boolean;
  warnings: { code: string; message: string }[];
}

/** The Filter field shows from this many presets on (CAP-058). */
export const FILTER_FROM = 9;

/** The File destination's defaults (CAP-053). */
const FILE_DEFAULTS: PanelSettings["image"]["file"] = {
  folder: "",
  prefix: "XcreenShot",
  pattern: "{prefix}_{date}_{time}",
  format: "png",
  quality: 90,
  onExists: "number",
};

/** CAP-003 defaults of each kind, for "New preset". */
export const DEFAULTS: { universal: PanelSettings["universal"]; image: PanelSettings["image"] } = {
  universal: {
    selection: "region",
    copy: true,
    cursor: false,
    open: "editor",
    delay5: false,
    share: [],
    file: FILE_DEFAULTS,
  },
  image: {
    selection: "region",
    copy: true,
    cursor: false,
    open: "editor",
    timing: "none",
    delay: 5,
    interval: {
      every: 60,
      stop: "manual",
      count: 10,
      minutes: 60,
      until: "18:00",
      discard: true,
      sensitivity: "exact",
    },
    schedule: { date: "", time: "09:00", repeat: "once" },
    sourceFile: "",
    web: { width: 1280, scale: 0, wait: 1, lazy: true },
    freehand: "free",
    windowsAsObjects: false,
    scroll: { type: "manual", direction: "vertical", startFrom: "start", wait: 0.3, restore: true },
    region: { fixed: false, width: 800, height: 600, positioned: false, monitor: "", x: 0, y: 0 },
    areas: { composition: "original", gap: 10 },
    menu: { submenus: true, menubar: false },
    effects: [],
    share: [],
    file: FILE_DEFAULTS,
  },
};

const fold = (s: string) => s.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

/** Presets whose name or summary has the text, without regard to case or accents. */
export function filterPresets(list: Preset[], text: string): Preset[] {
  const q = fold(text.trim());
  if (!q) return list;
  return list.filter((p) => fold(p.name).includes(q) || fold(p.summary).includes(q));
}

/** The row an arrow, Home or End moves to; `null` for other keys. */
export function keyTarget(index: number, count: number, key: string): number | null {
  if (count === 0) return null;
  switch (key) {
    case "ArrowDown":
      return Math.min(count - 1, index + 1);
    case "ArrowUp":
      return Math.max(0, index - 1);
    case "Home":
      return 0;
    case "End":
      return count - 1;
    default:
      return null;
  }
}

/** The index `preset.move` takes when a row from `from` drops before or after the row
 * at `over` (CAP-061): the others close the gap it leaves. */
export function dropIndex(from: number, over: number, after: boolean): number {
  const target = after ? over + 1 : over;
  return target > from ? target - 1 : target;
}

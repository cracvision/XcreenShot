// Thin client for the command registry (APP-027). Components never hold business
// logic: they call commands and render results.
import { invoke } from "@tauri-apps/api/core";

export interface CommandError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

export type Source = "ui" | "shortcut";

export function isCommandError(e: unknown): e is CommandError {
  return typeof e === "object" && e !== null && "code" in e && "message" in e;
}

export async function run<T = unknown>(
  id: string,
  params: Record<string, unknown> = {},
  source: Source = "ui",
): Promise<T> {
  return invoke<T>("xs_run", { id, params, source });
}

export interface Ping {
  version: string;
  build: string;
  platform: "windows" | "macos";
  arch: string;
  state: string;
  automationEnabled: boolean;
  language: "en" | "es";
  systemLanguage: "en" | "es";
  source: string;
}

export interface SettingEntry {
  key: string;
  section: string;
  label: string;
  description: string;
  schema: { type?: string | string[]; enum?: string[]; minimum?: number; maximum?: number };
  default: unknown;
  value: unknown;
  locked: boolean;
  automationWritable: boolean;
  specId: string;
}

export interface ShortcutStatus {
  state: "active" | "unassigned" | "paused" | "duplicate" | "system" | "other_app" | "taken_over";
  command?: string;
  kind?: "reserved" | "system_assigned";
  settings_uri?: string;
}

export interface ShortcutRow {
  /** Row id (a command id, or its own id when it runs a command with fixed parameters). */
  command: string;
  /** The command it runs, and with which parameters (CAP-065). */
  runs: string;
  params: Record<string, unknown>;
  scope: "global" | "app" | "editor";
  group: string;
  title: string;
  shortcut: string | null;
  display: string | null;
  default: string | null;
  custom: boolean;
  status: ShortcutStatus;
  conflict: boolean;
  statusText: string;
  canTakeOver: boolean;
}

export interface ShortcutList {
  profile: "coexistence" | "compatibility";
  paused: boolean;
  platform: "windows" | "macos";
  takeOver: boolean;
  rows: ShortcutRow[];
}

export interface PermissionStatus {
  permission: "screen_recording" | "accessibility" | "camera" | "microphone";
  state: "granted" | "denied" | "not_determined";
  can_request: boolean;
}

export interface CommandEvent {
  id: string;
  source: string;
  ok: boolean;
  result: unknown;
  error: CommandError | null;
  at: number;
}

/** Capture panel settings (CAP-001, CAP-002, CAP-010), as `capture.settings` returns them. */
/** The properties of Selection = Region (CAP-026). */
export interface RegionProps {
  fixed: boolean;
  width: number;
  height: number;
  /** Also a fixed place: captured at once, without selecting. */
  positioned: boolean;
  /** The monitor's name; x and y from its top left, physical pixels. */
  monitor: string;
  x: number;
  y: number;
}

/** The properties of Selection = Scrolling (CAP-034, CAP-035). */
export interface ScrollProps {
  type: "manual" | "auto";
  direction: "vertical" | "horizontal" | "both";
  startFrom: "start" | "current";
  /** Seconds between steps, 0 to 5. */
  wait: number;
  restore: boolean;
}

export interface PanelSettings {
  tab: "universal" | "image";
  expanded: boolean;
  universal: {
    selection: "region" | "window" | "fullscreen";
    copy: boolean;
    cursor: boolean;
    open: "editor" | "none";
    delay5: boolean;
  };
  image: {
    /** `scrolling`: CAP-034, CAP-035; `menu`: CAP-031; `multi`: CAP-033; `text`: SMT-005. */
    selection:
      "region" | "window" | "fullscreen" | "picker" | "scrolling" | "menu" | "multi" | "text";
    copy: boolean;
    cursor: boolean;
    open: "editor" | "none";
    timing: "none" | "delay";
    delay: number;
    /** Selection = Scrolling (CAP-034, CAP-035). */
    scroll: ScrollProps;
    /** Selection = Region (CAP-026). */
    region: RegionProps;
    /** Selection = Multiple areas (CAP-033). */
    areas: { composition: "original" | "stacked" | "row"; gap: number };
    /** Selection = Menu (CAP-031). */
    menu: { submenus: boolean; menubar: boolean };
  };
}

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
  schema: { type?: string; enum?: string[] };
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
  command: string;
  scope: "global" | "app";
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

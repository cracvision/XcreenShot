// UI state mirrors what the core reports. Every change goes through a command; the
// store only caches results and reacts to app events.
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import {
  run,
  type CommandEvent,
  type PermissionStatus,
  type Ping,
  type SettingEntry,
  type ShortcutList,
} from "./lib/commands";
import { loadLayoutMap, type LayoutMap } from "./lib/accelerator";
import { resolveLanguage, setLanguage } from "./i18n";
import { applyTheme } from "./theme";

interface State {
  ready: boolean;
  ping: Ping | null;
  systemLang: "en" | "es";
  settings: SettingEntry[];
  shortcuts: ShortcutList | null;
  permissions: PermissionStatus[];
  activity: (CommandEvent & { seq: number })[];
  layout: LayoutMap | undefined;
  /** Disables in-app shortcut handling while a shortcut is being recorded. */
  recording: boolean;
  setRecording: (recording: boolean) => void;
  boot: () => Promise<void>;
  refreshSettings: () => Promise<void>;
  refreshShortcuts: () => Promise<void>;
  refreshPermissions: () => Promise<void>;
}

function settingValue(settings: SettingEntry[], key: string): unknown {
  return settings.find((s) => s.key === key)?.value;
}

let seq = 0;

export const useApp = create<State>((set, get) => ({
  ready: false,
  ping: null,
  systemLang: "en",
  settings: [],
  shortcuts: null,
  permissions: [],
  activity: [],
  layout: undefined,
  recording: false,
  setRecording: (recording) => set({ recording }),

  boot: async () => {
    const ping = await run<Ping>("app.ping");
    const settings = await run<SettingEntry[]>("settings.list");
    const languageSetting = settingValue(settings, "general.language");
    const systemLang = ping.systemLanguage;
    set({ ping, settings, systemLang });
    setLanguage(resolveLanguage(languageSetting, systemLang));
    applyTheme(settingValue(settings, "general.theme"));
    const [layout] = await Promise.all([
      loadLayoutMap(),
      get().refreshShortcuts(),
      get().refreshPermissions(),
    ]);
    set({ layout, ready: true });

    await listen<{ key: string; value: unknown }>("xs://settings", (e) => {
      const { key, value } = e.payload;
      set((s) => ({
        settings: s.settings.map((x) => (x.key === key ? { ...x, value } : x)),
      }));
      if (key === "general.theme") applyTheme(value);
      if (key === "general.language") {
        setLanguage(resolveLanguage(value, get().systemLang));
        // Labels and descriptions come translated from the core.
        void get().refreshSettings();
        void get().refreshShortcuts();
      }
    });
    await listen("xs://shortcuts", () => void get().refreshShortcuts());
    await listen("xs://permissions", () => void get().refreshPermissions());
    await listen<CommandEvent>("xs://command", (e) => {
      // Show what proves the wiring (test command, CLI, shortcuts), not the UI's own
      // background refreshes.
      if (e.payload.id !== "app.ping" && e.payload.source === "ui") return;
      set((s) => ({ activity: [{ ...e.payload, seq: ++seq }, ...s.activity].slice(0, 20) }));
    });
  },

  refreshSettings: async () => set({ settings: await run<SettingEntry[]>("settings.list") }),
  refreshShortcuts: async () => set({ shortcuts: await run<ShortcutList>("shortcuts.list") }),
  refreshPermissions: async () =>
    set({ permissions: await run<PermissionStatus[]>("app.permissions") }),
}));

// UI state mirrors what the core reports. Every change goes through a command; the
// store only caches results and reacts to app events.
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import {
  run,
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

/** A yes/no preference as it is now; `fallback` until the settings arrive. */
export function setting(key: string, fallback: boolean): boolean {
  const value = settingValue(useApp.getState().settings, key);
  return typeof value === "boolean" ? value : fallback;
}

/** The same, for components: they update when it changes. */
export function useSetting(key: string, fallback: boolean): boolean {
  return useApp((s) => {
    const value = settingValue(s.settings, key);
    return typeof value === "boolean" ? value : fallback;
  });
}

export const useApp = create<State>((set, get) => ({
  ready: false,
  ping: null,
  systemLang: "en",
  settings: [],
  shortcuts: null,
  permissions: [],
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
  },

  refreshSettings: async () => set({ settings: await run<SettingEntry[]>("settings.list") }),
  refreshShortcuts: async () => set({ shortcuts: await run<ShortcutList>("shortcuts.list") }),
  refreshPermissions: async () =>
    set({ permissions: await run<PermissionStatus[]>("app.permissions") }),
}));

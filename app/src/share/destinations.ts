// The standard sharing destinations (SHR-020 to SHR-026) in the Share menu and the
// library's menu: email, another app, the printer, Word, PowerPoint and Excel.
import i18next from "i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { create } from "zustand";
import type { MenuItem } from "../components/Menu";
import { run } from "../lib/commands";
import { destinationOff, managedNow } from "../lib/managed";
import { useEditor } from "../editor/state";

export type DestId = "email" | "app" | "printer" | "word" | "powerpoint" | "excel";

export interface Dest {
  id: DestId;
  name: string;
  available: boolean;
}

/** What is shared: the documents open, or files. */
export type Target = { documentId: string } | { paths: string[] };

interface ShareUi {
  dests: Dest[];
  /** Email over 20 MB waiting for "open anyway" (SHR-020). */
  large: { target: Target; size: number } | null;
}

export const useShare = create<ShareUi>(() => ({ dests: [], large: null }));

let loaded = false;

export function loadDestinations(): void {
  if (loaded) return;
  loaded = true;
  void run<{ destinations: Dest[] }>("share.destinations")
    .then((r) => useShare.setState({ dests: r.destinations }))
    .catch(() => {
      loaded = false;
    });
}

const APPS_KEY = "xs.share.apps";

interface App {
  program: string;
  name: string;
}

function rememberedApps(): App[] {
  try {
    const v: unknown = JSON.parse(localStorage.getItem(APPS_KEY) ?? "[]");
    return Array.isArray(v) ? (v as App[]).filter((a) => typeof a.program === "string") : [];
  } catch {
    return [];
  }
}

function remember(app: App): void {
  const list = [app, ...rememberedApps().filter((a) => a.program !== app.program)].slice(0, 8);
  try {
    localStorage.setItem(APPS_KEY, JSON.stringify(list));
  } catch {
    // Without storage the app is simply not remembered.
  }
}

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

export async function shareTo(
  dest: DestId,
  target: Target,
  options: Record<string, unknown> = {},
): Promise<void> {
  try {
    const r = await run<{ status: string; size?: number }>("share.send", {
      destination: dest,
      ...target,
      options,
    });
    if (r.status === "too_large") {
      useShare.setState({ large: { target, size: r.size ?? 0 } });
      return;
    }
    const name = i18next.t(`share.dest_${dest}`);
    if (dest !== "printer")
      useEditor
        .getState()
        .showToast(
          i18next.t(r.status === "inserted" ? "ui.share.inserted" : "ui.share.opened", { name }),
        );
  } catch (e) {
    fail(e);
  }
}

async function pickApp(target: Target): Promise<void> {
  const chosen = (await open({
    filters: [{ name: i18next.t("ui.share.apps"), extensions: ["exe"] }],
  })) as string | null;
  if (!chosen) return;
  const name =
    chosen
      .split(/[\\/]/)
      .pop()
      ?.replace(/\.exe$/i, "") ?? chosen;
  remember({ program: chosen, name });
  await shareTo("app", target, { program: chosen });
}

/** The destinations as menu entries for `target` (only those available and not turned
 * off by the administrator). */
export function destinationItems(
  target: () => Target | null,
  t: (k: string) => string,
): MenuItem[] {
  loadDestinations();
  const managed = managedNow();
  const dests = useShare
    .getState()
    .dests.filter((d) => d.available && !destinationOff(managed, d.id));
  const go = (id: DestId) => () => {
    const tg = target();
    if (tg) void shareTo(id, tg);
  };
  const items: MenuItem[] = [];
  for (const d of dests) {
    if (d.id === "app") {
      const apps = rememberedApps();
      items.push({
        id: "dest_app",
        label: t("ui.share.app"),
        onSelect: () => undefined,
        submenu: [
          ...apps.map((a) => ({
            id: `app_${a.program}`,
            label: a.name,
            onSelect: () => {
              const tg = target();
              if (tg) void shareTo("app", tg, { program: a.program });
            },
          })),
          {
            id: "app_pick",
            label: t("ui.share.pick_app"),
            separator: apps.length > 0,
            onSelect: () => {
              const tg = target();
              if (tg) void pickApp(tg);
            },
          },
        ],
      });
    } else {
      items.push({ id: `dest_${d.id}`, label: t(`ui.share.${d.id}`), onSelect: go(d.id) });
    }
  }
  return items;
}

// The local network (SHR-042 to SHR-044): what the core says about it, reloaded when it
// tells something changed, and the Send dialog's state.
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { run } from "../lib/commands";

export interface LanStatus {
  enabled: boolean;
  running: boolean;
  name: string;
  device: string;
  fingerprint: string | null;
  port: number | null;
  error: string | null;
}

export interface LanPerson {
  fingerprint: string;
  name: string;
  theirName?: string;
  device: string;
  online: boolean;
  contact: boolean;
  alwaysAccept?: boolean;
  blocked?: boolean;
  lastSeen?: string | null;
}

export interface LanPairing {
  id: string;
  fingerprint: string;
  name: string;
  device: string;
  code: string;
  direction: "in" | "out";
  answered: boolean;
}

export interface LanIncoming {
  id: string;
  fingerprint: string;
  name: string;
  items: { name: string; size: number }[];
  total: number;
  receiving: boolean;
}

export type SendStatus =
  "pending" | "sending" | "delivered" | "rejected" | "expired" | "cancelled" | "failed";

export interface LanSend {
  id: string;
  to: string;
  toName: string;
  items: { path: string; name: string; size: number }[];
  created: string;
  updated: string;
  status: SendStatus;
  reason: string;
}

interface LanState {
  status: LanStatus | null;
  people: LanPerson[];
  pairings: LanPairing[];
  incoming: LanIncoming[];
  sends: LanSend[];
  /** Progress of sends and receptions in flight, by id. */
  progress: Record<string, { done: number; total: number }>;
  /** The Send dialog, with the files to send. */
  sending: string[] | null;
  /** The last thing that failed, shown where it was asked (Preferences). */
  error: string | null;
  load: () => Promise<void>;
}

export const useLan = create<LanState>((set) => ({
  status: null,
  people: [],
  pairings: [],
  incoming: [],
  sends: [],
  progress: {},
  sending: null,
  error: null,
  load: async () => {
    const [status, people, pairings, incoming, sends] = await Promise.all([
      run<LanStatus>("share.lan_status"),
      run<{ people: LanPerson[] }>("share.lan_people"),
      run<{ pairings: LanPairing[] }>("share.lan_pairings"),
      run<{ incoming: LanIncoming[] }>("share.lan_incoming"),
      run<{ sends: LanSend[] }>("share.lan_sends"),
    ]);
    set({
      status,
      people: people.people,
      pairings: pairings.pairings,
      incoming: incoming.incoming,
      sends: sends.sends,
    });
  },
}));

/** Keeps what failed to show it (Preferences has no error dialog of its own). */
export function lanFail(e: unknown): void {
  const message =
    typeof e === "object" && e && "message" in e
      ? String((e as { message: unknown }).message)
      : String(e);
  useLan.setState({ error: message });
}

let listening = false;

/** Follows the core's local network events (once per window). */
export function followLan(): void {
  if (listening) return;
  listening = true;
  void useLan
    .getState()
    .load()
    .catch(() => undefined);
  let timer: ReturnType<typeof setTimeout> | undefined;
  void listen<{
    kind: string;
    payload: { kind?: string; id?: string; done?: number; total?: number };
  }>("xs://editor", (e) => {
    if (e.payload.kind !== "lan") return;
    const p = e.payload.payload;
    if (p.kind === "progress" && p.id) {
      const { id, done = 0, total = 0 } = p;
      useLan.setState((s) => ({ progress: { ...s.progress, [id]: { done, total } } }));
      return;
    }
    // Several changes at once load once.
    clearTimeout(timer);
    timer = setTimeout(
      () =>
        void useLan
          .getState()
          .load()
          .catch(() => undefined),
      80,
    );
  });
}

export async function pair(fingerprint: string): Promise<void> {
  await run("share.lan_pair", { fingerprint });
  await useLan.getState().load();
}

export async function answerPairing(id: string, accept: boolean): Promise<void> {
  await run("share.lan_pair_answer", { id, accept });
  await useLan.getState().load();
}

export async function answerIncoming(id: string, accept: boolean): Promise<void> {
  await run("share.lan_answer", { id, accept });
  await useLan.getState().load();
}

export async function contactAction(
  fingerprint: string,
  action: "rename" | "alwaysAccept" | "remove" | "block" | "unblock",
  value?: unknown,
): Promise<void> {
  await run("share.lan_contact", {
    fingerprint,
    action,
    ...(value !== undefined ? { value } : {}),
  });
  await useLan.getState().load();
}

export async function cancelLan(id: string): Promise<void> {
  await run("share.lan_cancel", { id });
  await useLan.getState().load();
}

export async function sendTo(to: string[], paths: string[]): Promise<void> {
  await run("share.lan_send", { to, paths });
  await useLan.getState().load();
}

/** Opens the Send dialog for these files, or Preferences > Share while it is off. */
export async function askSend(paths: string[]): Promise<void> {
  if (paths.length === 0) return;
  const status = await run<LanStatus>("share.lan_status");
  if (!status.enabled) {
    await run("settings.open", { section: "share" });
    return;
  }
  await useLan.getState().load();
  useLan.setState({ sending: paths });
}

export function fileSize(bytes: number, lang: string): string {
  const mb = bytes / (1024 * 1024);
  if (mb >= 1) return `${mb.toLocaleString(lang, { maximumFractionDigits: 1 })} MB`;
  return `${Math.ceil(bytes / 1024).toLocaleString(lang)} KB`;
}

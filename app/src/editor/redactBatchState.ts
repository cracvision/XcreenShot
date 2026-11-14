// Auto redact over several captures (SMT-016): the review window's state. The core
// analyzes the captures in the background (`smart.redact_batch`); the window asks how it
// goes while it runs and sends each decision. Nothing changes in a capture until it is
// accepted; Undo all works until the window closes.
import { create } from "zustand";
import { run } from "../lib/commands";
import { useMainView } from "../library/state";
import { docUrl, useEditor } from "./state";
import { openRedact } from "./redactState";

export type BatchState =
  | "waiting"
  | "working"
  | "ready"
  | "nothing"
  | "accepted"
  | "skipped"
  | "review"
  | "video"
  | "failed"
  | "cancelled"
  | "undone";

export interface BatchItem {
  index: number;
  name: string;
  path: string | null;
  documentId: string | null;
  state: BatchState;
  redactions: number;
  counts: Record<string, number>;
  error?: string;
  output?: string;
}

export interface BatchStatus {
  jobId: number;
  running: boolean;
  done: number;
  total: number;
  /** How many texts redacted by hand are looked for. */
  texts: number;
  items: BatchItem[];
  shown?: string;
  closed?: boolean;
}

interface RedactBatch {
  status: BatchStatus | null;
  /** Put aside while a capture is reviewed in the editor: a bar brings it back. */
  minimized: boolean;
  busy: boolean;
  /** Each capture's picture with its redactions, kept once it was seen: an accepted
   * capture is closed, its picture stays in the list. */
  thumbs: Record<number, string>;
  /** Choosing the captures for "Search other captures…" (SMT-016). */
  picking: string | null;
}

export const useRedactBatch = create<RedactBatch>(() => ({
  status: null,
  minimized: false,
  busy: false,
  thumbs: {},
  picking: null,
}));

/** Keeps a new status, and the pictures of the captures just analyzed. */
function setStatus(s: BatchStatus): void {
  const thumbs = { ...useRedactBatch.getState().thumbs };
  for (const i of s.items) {
    if (i.documentId && i.state === "ready" && !thumbs[i.index]) {
      // A version of its own: the tray's picture of the same document is another one.
      thumbs[i.index] = docUrl(i.documentId, "thumb", s.jobId * 100_000 + i.index + 1);
    }
  }
  useRedactBatch.setState({ status: s, thumbs });
}

/** Where the captures come from: files (library), open documents (tray) and the
 * document whose texts by hand are looked for. */
export interface BatchSources {
  paths?: string[];
  documentIds?: string[];
  documentId?: string;
}

const POLL_MS = 300;
let timer: number | undefined;

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

function poll(): void {
  window.clearTimeout(timer);
  const status = useRedactBatch.getState().status;
  if (!status?.running) return;
  timer = window.setTimeout(() => {
    run<BatchStatus>("smart.redact_batch_status", { jobId: status.jobId })
      .then((s) => {
        if (useRedactBatch.getState().status?.jobId !== s.jobId) return;
        setStatus(s);
        poll();
      })
      .catch(() => useRedactBatch.setState({ status: { ...status, running: false } }));
  }, POLL_MS);
}

/** Starts the analysis and opens the review window. */
export async function startBatch(sources: BatchSources): Promise<void> {
  const open = useRedactBatch.getState().status;
  if (open) await closeBatch();
  try {
    const status = await run<BatchStatus>("smart.redact_batch", { ...sources });
    useRedactBatch.setState({ thumbs: {}, minimized: false, busy: false, picking: null });
    setStatus(status);
    poll();
  } catch (e) {
    fail(e);
  }
}

async function act(
  command: string,
  params: Record<string, unknown> = {},
): Promise<BatchStatus | null> {
  const status = useRedactBatch.getState().status;
  if (!status) return null;
  useRedactBatch.setState({ busy: true });
  try {
    const s = await run<BatchStatus>(command, { jobId: status.jobId, ...params });
    setStatus(s);
    return s;
  } catch (e) {
    fail(e);
    // What happened anyway (an accept that failed half-way).
    void run<BatchStatus>("smart.redact_batch_status", { jobId: status.jobId })
      .then(setStatus)
      .catch(() => undefined);
    return null;
  } finally {
    useRedactBatch.setState({ busy: false });
  }
}

/** Accepts captures (by index), or every one ready. */
export function acceptItems(items?: number[]): Promise<BatchStatus | null> {
  return act("smart.redact_batch_decide", { decision: "accept", ...(items ? { items } : {}) });
}

export function skipItems(items: number[]): Promise<BatchStatus | null> {
  return act("smart.redact_batch_decide", { decision: "skip", items });
}

/** Opens a capture in the editor with auto redact's panel; the review waits in a bar. */
export async function reviewItem(index: number): Promise<void> {
  const s = await act("smart.redact_batch_decide", { decision: "review", items: [index] });
  const shown = s?.shown;
  if (!shown) return;
  useRedactBatch.setState({ minimized: true });
  useMainView.getState().setView("editor");
  // The panel opens on it once the editor shows it.
  for (let i = 0; i < 40; i++) {
    if (useEditor.getState().doc?.documentId === shown) {
      await openRedact();
      return;
    }
    await new Promise((r) => setTimeout(r, 75));
  }
}

export function undoAll(): Promise<BatchStatus | null> {
  return act("smart.redact_batch_undo");
}

export function cancelBatch(): Promise<BatchStatus | null> {
  return act("smart.redact_batch_cancel");
}

export function restoreBatch(): void {
  useRedactBatch.setState({ minimized: false });
}

/** Closes the review: the captures still waiting are left as they were. */
export async function closeBatch(): Promise<void> {
  const status = useRedactBatch.getState().status;
  window.clearTimeout(timer);
  useRedactBatch.setState({ status: null, minimized: false, busy: false, thumbs: {} });
  if (!status) return;
  await run("smart.redact_batch_close", { jobId: status.jobId }).catch(() => undefined);
}

/** "Search other captures…" from auto redact's panel: the choice of captures opens. */
export function pickOthers(documentId: string): void {
  useRedactBatch.setState({ picking: documentId });
}

export function stopPicking(): void {
  useRedactBatch.setState({ picking: null });
}

/** The counts of a capture, largest first, for its line in the review. */
export function countsOf(item: BatchItem): [string, number][] {
  return Object.entries(item.counts)
    .filter(([, n]) => n > 0)
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
}

/** Can still be decided. */
export function pending(status: BatchStatus | null): BatchItem[] {
  return status?.items.filter((i) => i.state === "ready") ?? [];
}

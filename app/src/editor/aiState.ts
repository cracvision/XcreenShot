// Cloud AI in the editor (SMT-040, SMT-041): what the Ask the AI and Suggest name
// dialogs show. Each action runs a command; the answer streams in through the
// `ai` editor events.
import i18next from "i18next";
import { create } from "zustand";
import { isCommandError, run, type CommandError } from "../lib/commands";
import { useEditor } from "./state";

export interface Turn {
  prompt: string;
  answer: string;
  stopped?: boolean;
}

interface Usage {
  inputTokens: number;
  outputTokens: number;
}

interface AiState {
  /** Which dialog is open. */
  open: "ask" | "name" | "setup" | null;
  /** Why cloud AI cannot be used yet (setup dialog). */
  setupMessage: string;
  documentId: string | null;
  turns: Turn[];
  /** The question being answered, and what arrived so far. */
  pending: (Turn & { requestId: string }) | null;
  usage: Usage | null;
  error: string | null;
  /** Suggest name: the editable name, whether a suggestion is on its way, the ones given. */
  name: string;
  naming: boolean;
  suggestions: string[];
}

export const useAi = create<AiState>(() => ({
  open: null,
  setupMessage: "",
  documentId: null,
  turns: [],
  pending: null,
  usage: null,
  error: null,
  name: "",
  naming: false,
  suggestions: [],
}));

function text(e: unknown): string {
  return isCommandError(e) ? e.message : String(e);
}

/** A command failed because cloud AI is off or has no key: explain and offer the way. */
function needsSetup(e: unknown): boolean {
  if (isCommandError(e) && (e as CommandError).code === "feature_disabled") {
    useAi.setState({ open: "setup", setupMessage: e.message });
    return true;
  }
  return false;
}

/** Opens a dialog on the document shown, or says first what is missing (SMT-039). */
export async function openAi(kind: "ask" | "name"): Promise<void> {
  const documentId = useEditor.getState().active;
  if (!documentId) return;
  const enabled = await run<{ value: unknown }>("settings.get", { key: "ai.enabled" })
    .then((r) => r.value === true)
    .catch(() => false);
  if (!enabled) {
    useAi.setState({ open: "setup", setupMessage: i18next.t("errors.ai_disabled") });
    return;
  }
  const key = await run<{ saved: boolean }>("ai.key_status").catch(() => ({ saved: false }));
  if (!key.saved) {
    useAi.setState({ open: "setup", setupMessage: i18next.t("errors.ai_no_key") });
    return;
  }
  const same = useAi.getState().documentId === documentId;
  if (kind === "ask") {
    // The conversation stays while the dialog is reopened on the same document.
    useAi.setState({
      open: "ask",
      documentId,
      error: null,
      ...(same ? {} : { turns: [], usage: null }),
    });
  } else {
    useAi.setState({ open: "name", documentId, name: "", suggestions: [], error: null });
    void suggestName();
  }
}

export function closeAi(): void {
  const { pending, naming } = useAi.getState();
  if (pending || naming) void run("ai.cancel").catch(() => undefined);
  useAi.setState({ open: null });
}

/** The answer in progress so far. */
export function aiProgress(requestId: string, text: string): void {
  useAi.setState((s) =>
    s.pending && s.pending.requestId === requestId
      ? { pending: { ...s.pending, answer: text } }
      : {},
  );
}

let requests = 0;

export async function ask(prompt: string): Promise<void> {
  const { documentId, turns, pending } = useAi.getState();
  const question = prompt.trim();
  if (!documentId || !question || pending) return;
  const requestId = `ask-${++requests}`;
  useAi.setState({ pending: { prompt: question, answer: "", requestId }, error: null });
  try {
    const r = await run<{ text: string; usage: Usage }>("ai.ask", {
      documentId,
      prompt: question,
      history: turns.filter((t) => !t.stopped).map(({ prompt, answer }) => ({ prompt, answer })),
      requestId,
    });
    useAi.setState((s) => ({
      turns: [...s.turns, { prompt: question, answer: r.text }],
      usage: r.usage,
      pending: null,
    }));
  } catch (e) {
    const partial = useAi.getState().pending;
    useAi.setState({ pending: null });
    if (isCommandError(e) && e.code === "cancelled") {
      // Stop keeps what had arrived (SMT-040).
      if (partial?.answer)
        useAi.setState((s) => ({
          turns: [...s.turns, { prompt: question, answer: partial.answer, stopped: true }],
        }));
      return;
    }
    if (!needsSetup(e)) useAi.setState({ error: text(e) });
  }
}

export function stop(): void {
  void run("ai.cancel").catch(() => undefined);
}

export function newConversation(): void {
  useAi.setState({ turns: [], usage: null, error: null });
}

/** Adds an answer to the document as a text box, centered and editable (SMT-040). */
export async function insertAnswer(answer: string): Promise<void> {
  const { doc } = useEditor.getState();
  const { documentId } = useAi.getState();
  if (!doc || doc.documentId !== documentId) return;
  const { width: cw, height: ch } = doc.canvas;
  const width = Math.round(Math.max(120, Math.min(cw * 0.6, 640)));
  const at = [Math.round((cw - width) / 2), Math.round(ch / 4)];
  try {
    const r = await run<{ ids: string[] }>("document.annotate", {
      documentId,
      objects: [{ type: "text", at, width, text: answer.trim() }],
    });
    useEditor.getState().setSelection(r.ids);
    useEditor.getState().showToast(i18next.t("ui.ai.inserted"));
  } catch (e) {
    useAi.setState({ error: text(e) });
  }
}

export async function copyAnswer(answer: string): Promise<void> {
  await navigator.clipboard.writeText(answer.trim());
  useEditor.getState().showToast(i18next.t("ui.ai.copied"));
}

export async function suggestName(): Promise<void> {
  const { documentId, suggestions, naming } = useAi.getState();
  if (!documentId || naming) return;
  useAi.setState({ naming: true, error: null });
  try {
    const r = await run<{ name: string }>("ai.suggest_name", { documentId, avoid: suggestions });
    useAi.setState((s) => ({ name: r.name, suggestions: [...s.suggestions, r.name] }));
  } catch (e) {
    if (!(isCommandError(e) && e.code === "cancelled") && !needsSetup(e))
      useAi.setState({ error: text(e) });
  } finally {
    useAi.setState({ naming: false });
  }
}

export async function rename(): Promise<void> {
  const { documentId, name } = useAi.getState();
  if (!documentId || !name.trim()) return;
  try {
    const r = await run<{ name: string }>("document.rename", { documentId, name });
    useAi.setState({ open: null });
    useEditor.getState().showToast(i18next.t("ui.ai.renamed", { name: r.name }));
  } catch (e) {
    useAi.setState({ error: text(e) });
  }
}

export function openAiPreferences(): void {
  useAi.setState({ open: null });
  void run("settings.open", { section: "ai" }).catch(() => undefined);
}

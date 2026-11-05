import { useState } from "react";

/**
 * Text typed for a value that can also change from elsewhere (undo, the CLI): the
 * field shows the value until the user types, and drops the typing as soon as the
 * value changes. `set(null)` goes back to the value.
 */
export function useDraft(value: string): [string, (text: string | null) => void] {
  const [draft, setDraft] = useState<{ base: string; text: string } | null>(null);
  const text = draft && draft.base === value ? draft.text : value;
  return [text, (t) => setDraft(t === null ? null : { base: value, text: t })];
}

// Release notes in the app's language (APP-036).
/** Notes as plain text or `{"en": …, "es": …}`. */
export function notesFor(notes: string | null, lang: string): string {
  if (!notes) return "";
  try {
    const parsed: unknown = JSON.parse(notes);
    if (parsed && typeof parsed === "object") {
      const byLang = parsed as Record<string, unknown>;
      const text = byLang[lang.slice(0, 2)] ?? byLang.en;
      if (typeof text === "string") return text;
    }
  } catch {
    // Plain text.
  }
  return notes;
}

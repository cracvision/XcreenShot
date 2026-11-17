// The recognition languages chosen in Preferences (SMT-004).

/** The list after ticking or unticking a language: Automatic alone, or up to 3. */
export function nextLanguages(current: string[], code: string, on: boolean): string[] {
  if (code === "auto") return ["auto"];
  const chosen = current.filter((c) => c !== "auto" && c !== code);
  const next = on ? [...chosen, code].slice(-3) : chosen;
  return next.length > 0 ? next : ["auto"];
}

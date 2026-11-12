// The EN | ES switch as menu entries, for a bar folded into ⋯ (APP-013, EDT-001).
import { run } from "../lib/commands";
import type { MenuItem } from "./Menu";

/** English and Español as a choice of a folded bar (APP-013). */
export function languageItems(
  t: (key: string) => string,
  current: string,
  separator: boolean,
): MenuItem[] {
  const now = current === "es" ? "es" : "en";
  return (["en", "es"] as const).map((lang, i) => ({
    id: `lang-${lang}`,
    label: lang === "en" ? "English" : "Español",
    checked: now === lang,
    separator: separator && i === 0,
    title: t("ui.common.language"),
    onSelect: () => {
      if (lang !== now) void run("settings.set", { key: "general.language", value: lang });
    },
  }));
}

import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";

/**
 * Quick EN | ES switch (APP-013). Sets `general.language`; Preferences > General still
 * offers "System" as well. Every window and the tray menu follow at once.
 */
export function LanguageToggle() {
  const { t, i18n } = useTranslation();
  const current = i18n.language === "es" ? "es" : "en";
  const choose = (lang: "en" | "es") => {
    if (lang !== current) void run("settings.set", { key: "general.language", value: lang });
  };
  return (
    <div className="lang-toggle" role="group" aria-label={t("ui.common.language")}>
      {(["en", "es"] as const).map((lang) => (
        <button
          key={lang}
          type="button"
          lang={lang}
          aria-pressed={current === lang}
          title={lang === "en" ? "English" : "Español"}
          onClick={() => choose(lang)}
        >
          {lang.toUpperCase()}
        </button>
      ))}
    </div>
  );
}

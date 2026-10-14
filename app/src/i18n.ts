// UI strings come from the same locale files the Rust side uses (APP-013).
import i18next from "i18next";
import { initReactI18next } from "react-i18next";
import en from "../../crates/xs-core/locales/en.json";
import es from "../../crates/xs-core/locales/es.json";

void i18next.use(initReactI18next).init({
  resources: { en: { translation: en }, es: { translation: es } },
  lng: "en",
  fallbackLng: "en",
  interpolation: { escapeValue: false },
  returnNull: false,
});

export function setLanguage(lang: "en" | "es"): void {
  void i18next.changeLanguage(lang);
  document.documentElement.lang = lang;
}

/** Resolves the `general.language` setting value; "system" follows the OS locale. */
export function resolveLanguage(setting: unknown, systemLang: "en" | "es"): "en" | "es" {
  return setting === "en" || setting === "es" ? setting : systemLang;
}

export default i18next;

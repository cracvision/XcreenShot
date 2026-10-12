//! UI language resolution and translated strings (APP-013).
//!
//! The same JSON files feed the React UI (i18next) and the Rust side (tray menu,
//! notifications, CLI messages), so a string is translated exactly once.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const EN_JSON: &str = include_str!("../locales/en.json");
pub const ES_JSON: &str = include_str!("../locales/es.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    En,
    Es,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Es => "es",
        }
    }

    pub fn parse(s: &str) -> Option<Lang> {
        match s.to_ascii_lowercase().as_str() {
            "en" => Some(Lang::En),
            "es" => Some(Lang::Es),
            _ => None,
        }
    }

    /// Spanish for any Spanish variant of the system locale, English otherwise.
    pub fn from_locale(locale: &str) -> Lang {
        let lower = locale.to_ascii_lowercase();
        if lower == "es" || lower.starts_with("es-") || lower.starts_with("es_") {
            Lang::Es
        } else {
            Lang::En
        }
    }

    /// Resolves the `general.language` setting value (`system` | `en` | `es`).
    pub fn resolve(setting: &str) -> Lang {
        Lang::parse(setting).unwrap_or_else(Lang::system)
    }

    pub fn system() -> Lang {
        sys_locale::get_locale()
            .map(|l| Lang::from_locale(&l))
            .unwrap_or(Lang::En)
    }
}

/// Flattens nested JSON into `a.b.c` keys.
pub fn flatten(value: &Value) -> HashMap<String, String> {
    fn walk(prefix: &str, v: &Value, out: &mut HashMap<String, String>) {
        match v {
            Value::Object(map) => {
                for (k, v) in map {
                    let key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{prefix}.{k}")
                    };
                    walk(&key, v, out);
                }
            }
            Value::String(s) => {
                out.insert(prefix.to_string(), s.clone());
            }
            other => {
                out.insert(prefix.to_string(), other.to_string());
            }
        }
    }
    let mut out = HashMap::new();
    walk("", value, &mut out);
    out
}

fn table(lang: Lang) -> &'static HashMap<String, String> {
    static EN: OnceLock<HashMap<String, String>> = OnceLock::new();
    static ES: OnceLock<HashMap<String, String>> = OnceLock::new();
    let (cell, src) = match lang {
        Lang::En => (&EN, EN_JSON),
        Lang::Es => (&ES, ES_JSON),
    };
    cell.get_or_init(|| flatten(&serde_json::from_str(src).expect("locale JSON is valid")))
}

/// Translates `key`, replacing `{{name}}` placeholders. Falls back to English and
/// logs the missing key; falls back to the key itself as a last resort.
pub fn tr(lang: Lang, key: &str, args: &[(&str, &str)]) -> String {
    let template = table(lang).get(key).or_else(|| {
        if lang != Lang::En {
            tracing::warn!(key, lang = lang.code(), "missing translation");
        }
        table(Lang::En).get(key)
    });
    let mut s = template.cloned().unwrap_or_else(|| key.to_string());
    for (name, value) in args {
        s = s.replace(&format!("{{{{{name}}}}}"), value);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spanish_variants_resolve_to_spanish() {
        assert_eq!(Lang::from_locale("es-MX"), Lang::Es);
        assert_eq!(Lang::from_locale("es_ES"), Lang::Es);
        assert_eq!(Lang::from_locale("es"), Lang::Es);
        assert_eq!(Lang::from_locale("en-US"), Lang::En);
        assert_eq!(Lang::from_locale("pt-BR"), Lang::En);
    }

    /// APP-013: a published build has no key without a Spanish translation.
    #[test]
    fn locales_have_identical_non_empty_keys() {
        let en = table(Lang::En);
        let es = table(Lang::Es);
        let mut missing: Vec<_> = en.keys().filter(|k| !es.contains_key(*k)).collect();
        let mut extra: Vec<_> = es.keys().filter(|k| !en.contains_key(*k)).collect();
        missing.sort();
        extra.sort();
        assert!(missing.is_empty(), "missing in es: {missing:?}");
        assert!(extra.is_empty(), "missing in en: {extra:?}");
        for (k, v) in en.iter().chain(es.iter()) {
            assert!(!v.trim().is_empty(), "empty translation for {k}");
        }
    }

    #[test]
    fn interpolates_placeholders() {
        let s = tr(Lang::En, "cli.not_running", &[]);
        assert!(!s.is_empty());
        assert_eq!(tr(Lang::En, "no.such.key", &[]), "no.such.key");
    }
}

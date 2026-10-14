// Single Preferences window, same structure on Windows and macOS (APP-006).
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppShortcuts } from "../lib/appShortcuts";
import { run, type SettingEntry } from "../lib/commands";
import { PermissionsBlock } from "../prefs/PermissionsBlock";
import { SettingRow } from "../prefs/SettingRow";
import { ShortcutsSection } from "../prefs/ShortcutsSection";
import { useApp } from "../store";

const SECTIONS = [
  "general",
  "capture",
  "video",
  "editor",
  "library",
  "share",
  "shortcuts",
  "advanced",
  "experimental",
] as const;
type Section = (typeof SECTIONS)[number];

const LAST_SECTION_KEY = "xs.prefs.lastSection";

function initialSection(): Section {
  const saved = localStorage.getItem(LAST_SECTION_KEY);
  return (SECTIONS as readonly string[]).includes(saved ?? "") ? (saved as Section) : "general";
}

interface Navigation {
  section?: string | null;
  key?: string | null;
}

export function Preferences() {
  const { t } = useTranslation();
  useAppShortcuts();
  const settings = useApp((s) => s.settings);
  const [section, setSection] = useState<Section>(initialSection);
  const [query, setQuery] = useState("");
  const [highlight, setHighlight] = useState<string | null>(null);

  const go = (nav: Navigation) => {
    const target =
      nav.section ?? settings.find((s) => s.key === nav.key)?.section ?? nav.key?.split(".")[0];
    if (target && (SECTIONS as readonly string[]).includes(target)) {
      setSection(target as Section);
      localStorage.setItem(LAST_SECTION_KEY, target);
    }
    setQuery("");
    if (nav.key) {
      setHighlight(nav.key);
      requestAnimationFrame(() =>
        document.querySelector(`[data-key="${nav.key}"]`)?.scrollIntoView({ block: "center" }),
      );
    }
  };

  // Deep links: `xcreenshot settings open --section shortcuts` (APP-006).
  useEffect(() => {
    void invoke<Navigation | null>("xs_take_navigation").then((nav) => nav && go(nav));
    const un = listen<Navigation>("xs://navigate", (e) => go(e.payload));
    // Permissions and the login item can change in system settings (APP-003, APP-024).
    const refreshOnFocus = () => {
      void useApp.getState().refreshPermissions();
      void useApp.getState().refreshSettings();
    };
    window.addEventListener("focus", refreshOnFocus);
    return () => {
      void un.then((f) => f());
      window.removeEventListener("focus", refreshOnFocus);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [settings.length > 0]);

  const results = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return null;
    return settings.filter((s) =>
      [s.label, s.description, s.key].some((x) => x.toLowerCase().includes(q)),
    );
  }, [query, settings]);

  const inSection = (name: Section): SettingEntry[] =>
    settings.filter((s) => s.section === name && s.key !== "shortcuts.profile");

  const content = () => {
    if (results) {
      if (results.length === 0) return <p className="empty">{t("ui.prefs.no_results")}</p>;
      return SECTIONS.filter((sec) => results.some((r) => r.section === sec)).map((sec) => (
        <div key={sec}>
          <h2>{t(`ui.prefs.section.${sec}`)}</h2>
          {results
            .filter((r) => r.section === sec)
            .map((r) => (
              <button
                key={r.key}
                className="btn"
                style={{ display: "block", margin: "6px 0", textAlign: "left" }}
                onClick={() => go({ key: r.key })}
              >
                {r.label}
              </button>
            ))}
        </div>
      ));
    }
    if (section === "shortcuts") return <ShortcutsSection highlightKey={highlight} />;
    const rows = inSection(section);
    return (
      <>
        {rows.length === 0 && section !== "general" && (
          <p className="empty">{t("ui.prefs.empty_section")}</p>
        )}
        {rows.map((s) => (
          <SettingRow key={s.key} setting={s} highlighted={highlight === s.key} />
        ))}
        {section === "general" && (
          <>
            <h2>{t("ui.permissions.title")}</h2>
            <PermissionsBlock />
          </>
        )}
        {section === "advanced" && (
          <div className="row" style={{ marginTop: 16 }}>
            <button className="btn" onClick={() => void run("app.open_logs")}>
              {t("menu.open_logs")}
            </button>
            <button className="btn" onClick={() => void run("app.diagnostics", { copy: true })}>
              {t("menu.copy_diagnostics")}
            </button>
          </div>
        )}
      </>
    );
  };

  return (
    <div className="prefs">
      <nav aria-label={t("ui.prefs.title")}>
        <input
          type="search"
          placeholder={t("ui.prefs.search")}
          aria-label={t("ui.prefs.search")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        {SECTIONS.map((s) => (
          <button
            key={s}
            aria-current={!results && section === s ? "page" : undefined}
            onClick={() => go({ section: s })}
          >
            {t(`ui.prefs.section.${s}`)}
          </button>
        ))}
      </nav>
      <main>
        <h1>{results ? t("ui.prefs.search") : t(`ui.prefs.section.${section}`)}</h1>
        {content()}
      </main>
    </div>
  );
}

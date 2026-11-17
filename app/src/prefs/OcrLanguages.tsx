// Preferences > Capture > Recognition language (SMT-004): Automatic, or up to 3
// languages. A language this computer does not have says so, with the system's page
// where it is added; XcreenShot downloads nothing itself.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { useApp } from "../store";
import { nextLanguages } from "./languageList";

interface Row {
  code: string;
  name: string;
  available: boolean;
  required: boolean;
}

export function OcrLanguages({ value, highlighted }: { value: string[]; highlighted: boolean }) {
  const { t } = useTranslation();
  const [rows, setRows] = useState<Row[]>([]);
  useEffect(() => {
    run<{ languages: Row[] }>("smart.ocr_languages")
      .then((r) => setRows(r.languages))
      .catch(() => undefined);
  }, []);
  const set = (next: string[]) =>
    void run("settings.set", { key: "smart.ocr.languages", value: next }).then(() =>
      useApp.getState().refreshSettings(),
    );
  const auto = value.includes("auto");
  return (
    <div className={`setting${highlighted ? " highlight" : ""}`} data-key="smart.ocr.languages">
      <span className="label">{t("settings.smart_ocr_languages.label")}</span>
      <div className="control ocr-languages">
        <label className="panel-check">
          <input type="checkbox" checked={auto} onChange={() => set(["auto"])} />
          {t("ui.extract.auto")}
        </label>
        {rows.map((r) => (
          <label
            key={r.code}
            className={`panel-check${r.available ? "" : " disabled"}`}
            title={r.available ? undefined : t("ui.extract.not_here_help")}
          >
            <input
              type="checkbox"
              disabled={!r.available}
              checked={!auto && value.includes(r.code)}
              onChange={(e) => set(nextLanguages(value, r.code, e.target.checked))}
            />
            {r.name}
            {!r.available && <span className="muted"> · {t("ui.extract.not_here")}</span>}
          </label>
        ))}
        {rows.some((r) => !r.available) && (
          <button
            type="button"
            className="btn small"
            onClick={() => void run("smart.language_settings").catch(() => undefined)}
          >
            {t("ui.extract.add_language")}
          </button>
        )}
      </div>
      <div className="description">
        {t("settings.smart_ocr_languages.description")} {t("ui.extract.up_to_three")}
      </div>
    </div>
  );
}

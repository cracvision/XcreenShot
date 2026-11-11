// Preferences > Library: where the library is (LIB-027), with Open folder and a note
// when the folder syncs to the cloud (LIB-029). Moving it arrives with LIB-028.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";

export function LibraryLocation() {
  const { t } = useTranslation();
  const [where, setWhere] = useState<{ root: string; synced: boolean } | null>(null);
  useEffect(() => {
    run<{ root: string; synced: boolean }>("library.list", { limit: 1 })
      .then((r) => setWhere({ root: r.root, synced: r.synced }))
      .catch(() => undefined);
  }, []);
  if (!where) return null;
  return (
    <div className="setting library-location">
      <span className="label">{t("ui.library.location")}</span>
      <div className="control">
        <button
          type="button"
          className="btn"
          onClick={() => void run("library.show_in_folder").catch(() => undefined)}
        >
          {t("ui.library.open_folder")}
        </button>
      </div>
      <div className="description">
        <code className="path">{where.root}</code>
        <div>{t("ui.library.location_help")}</div>
        {where.synced && <div>{t("ui.library.synced_help", { path: where.root })}</div>}
      </div>
    </div>
  );
}

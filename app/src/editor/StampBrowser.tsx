// The stamp collection (EDT-031): categories, a search box that finds by name and
// keywords in English and Spanish, and a grid of stamps. A click picks the stamp the
// tool places, or replaces the selected stamp (same place and height).
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";
import { fail } from "./actions";
import { organizeStamps, useStampsUi, type StampsFound as Found } from "./stampActions";
import { stampUrl, useEditor } from "./state";

export function StampBrowser() {
  const { t, i18n } = useTranslation();
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("");
  const [found, setFound] = useState<Found | null>(null);
  // Own stamps added, changed or imported (EDT-032, EDT-034).
  const version = useStampsUi((s) => s.version);
  const current = useEditor((s) => s.styles?.stamp.stamp);
  const selected = useEditor((s) => {
    const one =
      s.selection.length === 1 ? s.doc?.objects.find((o) => o.id === s.selection[0]) : undefined;
    return one?.type === "stamp" ? one : undefined;
  });
  useEffect(() => {
    let alive = true;
    run<Found>("editor.stamps", { query, ...(category ? { category } : {}) })
      .then((r) => alive && setFound(r))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [query, category, i18n.language, version]);
  const pick = (id: string) => {
    const { active } = useEditor.getState();
    if (selected && active)
      run("document.update_object", {
        documentId: active,
        objectId: selected.id,
        set: { stamp: id },
      }).catch(fail);
    else run("editor.set_style", { tool: "stamp", style: { stamp: id } }).catch(fail);
  };
  const chosen = selected?.stamp ?? current;
  return (
    <section className="stamp-browser" aria-label={t("ui.editor.stamps.title")}>
      <h2 className="side-title">{t("ui.editor.stamps.title")}</h2>
      <input
        type="search"
        className="stamp-search"
        placeholder={t("ui.editor.stamps.search")}
        aria-label={t("ui.editor.stamps.search")}
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <select
        className="stamp-category"
        aria-label={t("ui.editor.stamps.category")}
        value={category}
        onChange={(e) => setCategory(e.target.value)}
      >
        <option value="">{t("ui.editor.stamps.all")}</option>
        {found?.categories.map((c) => (
          <option key={c.id} value={c.id}>
            {c.name}
          </option>
        ))}
      </select>
      <div className="stamp-own">
        <button
          type="button"
          className="btn small"
          onClick={() => organizeStamps(category.startsWith("own:") ? category : "own:mine")}
        >
          {t("ui.editor.stamps.organize_menu")}
        </button>
      </div>
      {/* Emoji are many: they show when searching or in their category (EDT-033). */}
      <div
        className="stamp-grid"
        role="listbox"
        aria-label={t("ui.editor.stamps.title")}
        title={t("ui.editor.stamps.drop_hint")}
      >
        {found?.stamps.map((s) => (
          <button
            key={s.id}
            type="button"
            role="option"
            aria-selected={s.id === chosen}
            className="stamp-cell"
            title={s.name}
            onClick={() => pick(s.id)}
          >
            <img src={stampUrl(s.id)} alt={s.name} loading="lazy" draggable={false} />
          </button>
        ))}
        {found && found.stamps.length === 0 && (
          <p className="muted">{t("ui.editor.stamps.none")}</p>
        )}
      </div>
    </section>
  );
}

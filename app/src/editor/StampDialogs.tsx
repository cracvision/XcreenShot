// Own stamps' dialogs (EDT-032, EDT-034): Organize stamps, File > Save as stamp, and
// what to do with a pack that is already here.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import {
  addStampFiles,
  exportStampCategory,
  importStampPack,
  saveAsStamp,
  stampsChanged,
  useStampsUi,
  type StampsFound,
} from "./stampActions";
import { stampUrl, useEditor } from "./state";

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

/** The own categories and their stamps, loaded again when they change. */
function useOwnStamps(category: string | null): StampsFound | null {
  const version = useStampsUi((s) => s.version);
  const [found, setFound] = useState<StampsFound | null>(null);
  useEffect(() => {
    let alive = true;
    run<StampsFound>("editor.stamps", category ? { category } : {})
      .then((r) => alive && setFound(r))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [category, version]);
  return found;
}

/** Organize stamps (EDT-032): own categories, and their stamps' names, keywords,
 * order and category; add, export, import. */
function OrganizeStamps({ initial }: { initial: string }) {
  const { t } = useTranslation();
  const [category, setCategory] = useState(initial);
  const all = useOwnStamps(null);
  const found = useOwnStamps(category);
  const own = (all?.categories ?? []).filter((c) => c.own);
  const current = own.find((c) => c.id === category);
  const stamps = (found?.stamps ?? []).filter((s) => s.own && s.category === category);
  const [exporting, setExporting] = useState(false);
  const [naming, setNaming] = useState<{ mode: "create" | "rename"; value: string } | null>(null);
  const close = () => useStampsUi.setState({ organize: null });
  const change = (id: string, set: Record<string, unknown>) =>
    run("stamps.update", { id, ...set })
      .then(stampsChanged)
      .catch(fail);
  const isMine = category === "own:mine";
  return (
    <Dialog
      title={t("ui.editor.stamps.organize")}
      onCancel={close}
      actions={
        <button type="button" className="btn primary" onClick={close}>
          {t("ui.editor.close")}
        </button>
      }
    >
      <div className="stamps-organize">
        <div className="stamps-organize-bar">
          <label htmlFor="stamps-cat">{t("ui.editor.stamps.category")}</label>
          <select id="stamps-cat" value={category} onChange={(e) => setCategory(e.target.value)}>
            {own.map((c) => (
              <option key={c.id} value={c.id}>
                {c.pack
                  ? t("ui.editor.stamps.pack_label", { name: c.name, version: c.pack.version })
                  : c.name}
              </option>
            ))}
          </select>
          <button
            type="button"
            className="btn small"
            onClick={() => setNaming({ mode: "create", value: "" })}
          >
            {t("ui.editor.stamps.new_category")}
          </button>
          {!isMine && (
            <>
              <button
                type="button"
                className="btn small"
                onClick={() => setNaming({ mode: "rename", value: current?.name ?? "" })}
              >
                {t("ui.editor.stamps.rename_category")}
              </button>
              <button
                type="button"
                className="btn small"
                onClick={() =>
                  run("stamps.category", { action: "delete", id: category })
                    .then(() => {
                      setCategory("own:mine");
                      stampsChanged();
                    })
                    .catch(fail)
                }
              >
                {current?.pack
                  ? t("ui.editor.stamps.remove_pack")
                  : t("ui.editor.stamps.delete_category")}
              </button>
            </>
          )}
        </div>
        {naming && (
          <form
            className="stamps-organize-bar"
            onSubmit={(e) => {
              e.preventDefault();
              const name = naming.value.trim();
              if (!name) return;
              void run<{ id: string }>(
                "stamps.category",
                naming.mode === "create"
                  ? { action: "create", name }
                  : { action: "rename", id: category, name },
              )
                .then((r) => {
                  setNaming(null);
                  if (r.id) setCategory(r.id);
                  stampsChanged();
                })
                .catch(fail);
            }}
          >
            <input
              data-autofocus
              aria-label={t("ui.editor.stamps.category_name")}
              placeholder={t("ui.editor.stamps.category_name")}
              value={naming.value}
              onChange={(e) => setNaming({ ...naming, value: e.target.value })}
            />
            <button type="submit" className="btn small primary">
              {t("ui.editor.ok")}
            </button>
            <button type="button" className="btn small" onClick={() => setNaming(null)}>
              {t("ui.editor.cancel")}
            </button>
          </form>
        )}
        <ul className="stamps-organize-list" aria-label={current?.name ?? ""}>
          {stamps.map((s, i) => (
            <li key={s.id} className="stamps-organize-row">
              <img src={stampUrl(s.id)} alt="" width={40} height={40} draggable={false} />
              <input
                aria-label={t("ui.editor.stamps.name")}
                defaultValue={s.name}
                onBlur={(e) => {
                  const v = e.target.value.trim();
                  if (v && v !== s.name) void change(s.id, { name: v });
                }}
              />
              <input
                aria-label={t("ui.editor.stamps.keywords")}
                placeholder={t("ui.editor.stamps.keywords")}
                defaultValue={s.keywords ?? ""}
                onBlur={(e) => {
                  if (e.target.value !== (s.keywords ?? ""))
                    void change(s.id, { keywords: e.target.value });
                }}
              />
              <select
                aria-label={t("ui.editor.stamps.move_to")}
                value={category}
                onChange={(e) => void change(s.id, { category: e.target.value })}
              >
                {own.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name}
                  </option>
                ))}
              </select>
              <button
                type="button"
                className="btn icon small"
                disabled={i === 0}
                aria-label={t("ui.editor.stamps.move_up")}
                title={t("ui.editor.stamps.move_up")}
                onClick={() => void change(s.id, { before: stamps[i - 1]?.id })}
              >
                ↑
              </button>
              <button
                type="button"
                className="btn icon small"
                disabled={i === stamps.length - 1}
                aria-label={t("ui.editor.stamps.move_down")}
                title={t("ui.editor.stamps.move_down")}
                onClick={() => void change(s.id, { before: stamps[i + 2]?.id ?? null })}
              >
                ↓
              </button>
              <button
                type="button"
                className="btn small"
                onClick={() =>
                  void run("stamps.delete", { ids: [s.id] })
                    .then(stampsChanged)
                    .catch(fail)
                }
              >
                {t("ui.editor.stamps.delete")}
              </button>
            </li>
          ))}
          {found && stamps.length === 0 && <p className="muted">{t("ui.editor.stamps.empty")}</p>}
        </ul>
        <div className="stamps-organize-bar">
          <button
            type="button"
            className="btn"
            onClick={() => void addStampFiles(undefined, category)}
          >
            {t("ui.editor.stamps.add")}
          </button>
          <button type="button" className="btn" onClick={() => void importStampPack()}>
            {t("ui.editor.stamps.import")}
          </button>
          <button
            type="button"
            className="btn"
            disabled={stamps.length === 0}
            onClick={() => setExporting(true)}
          >
            {t("ui.editor.stamps.export")}
          </button>
        </div>
      </div>
      {exporting && (
        <ExportPack
          category={category}
          name={current?.name ?? ""}
          onDone={() => setExporting(false)}
        />
      )}
    </Dialog>
  );
}

/** Export category… (EDT-034): the pack's name, author and license. */
function ExportPack(props: { category: string; name: string; onDone: () => void }) {
  const { t } = useTranslation();
  const [name, setName] = useState(props.name);
  const [author, setAuthor] = useState("");
  const [license, setLicense] = useState("");
  return (
    <form
      className="stamps-export"
      onSubmit={(e) => {
        e.preventDefault();
        void exportStampCategory(props.category, name.trim() || props.name, author, license).then(
          props.onDone,
        );
      }}
    >
      <div className="prop-field">
        <label htmlFor="pack-name">{t("ui.editor.stamps.pack_name")}</label>
        <input
          id="pack-name"
          data-autofocus
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </div>
      <div className="prop-field">
        <label htmlFor="pack-author">{t("ui.editor.stamps.pack_author")}</label>
        <input id="pack-author" value={author} onChange={(e) => setAuthor(e.target.value)} />
      </div>
      <div className="prop-field">
        <label htmlFor="pack-license">{t("ui.editor.stamps.pack_license")}</label>
        <input id="pack-license" value={license} onChange={(e) => setLicense(e.target.value)} />
      </div>
      <div className="actions">
        <button type="button" className="btn" onClick={props.onDone}>
          {t("ui.editor.cancel")}
        </button>
        <button type="submit" className="btn primary" disabled={!name.trim()}>
          {t("ui.editor.stamps.export")}
        </button>
      </div>
    </form>
  );
}

/** File > Save as stamp (EDT-032): name, keywords and category. */
function SaveAsStamp() {
  const { t } = useTranslation();
  const docName = useEditor((s) => s.doc?.name ?? "");
  const hasSelection = useEditor((s) => !!s.doc?.pixelSelection);
  const all = useOwnStamps(null);
  const own = (all?.categories ?? []).filter((c) => c.own);
  const [name, setName] = useState(docName);
  const [keywords, setKeywords] = useState("");
  const [category, setCategory] = useState("own:mine");
  const close = () => useStampsUi.setState({ saveAs: false });
  return (
    <Dialog
      title={t("ui.editor.stamps.save_title")}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
          <button
            type="button"
            className="btn primary"
            disabled={!name.trim()}
            onClick={() =>
              void saveAsStamp(name.trim(), keywords, category).then((ok) => ok && close())
            }
          >
            {t("ui.editor.save")}
          </button>
        </>
      }
    >
      <p className="muted">
        {t(hasSelection ? "ui.editor.stamps.save_selection" : "ui.editor.stamps.save_image")}
      </p>
      <div className="prop-field">
        <label htmlFor="stamp-name">{t("ui.editor.stamps.name")}</label>
        <input
          id="stamp-name"
          data-autofocus
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </div>
      <div className="prop-field">
        <label htmlFor="stamp-keywords">{t("ui.editor.stamps.keywords")}</label>
        <input id="stamp-keywords" value={keywords} onChange={(e) => setKeywords(e.target.value)} />
      </div>
      <div className="prop-field">
        <label htmlFor="stamp-category">{t("ui.editor.stamps.category")}</label>
        <select id="stamp-category" value={category} onChange={(e) => setCategory(e.target.value)}>
          {own.map((c) => (
            <option key={c.id} value={c.id}>
              {c.name}
            </option>
          ))}
        </select>
      </div>
    </Dialog>
  );
}

/** A pack with the name of one already here (EDT-034). */
function PackConflictDialog() {
  const { t } = useTranslation();
  const conflict = useStampsUi((s) => s.conflict);
  if (!conflict) return null;
  const close = () => useStampsUi.setState({ conflict: null });
  const choose = (existing: "update" | "both") => {
    close();
    void importStampPack(conflict.path, existing);
  };
  return (
    <Dialog
      title={t("ui.editor.stamps.conflict_title")}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
          <button type="button" className="btn" onClick={() => choose("both")}>
            {t("ui.editor.stamps.keep_both")}
          </button>
          {conflict.canUpdate && (
            <button type="button" className="btn primary" onClick={() => choose("update")}>
              {t("ui.editor.stamps.update")}
            </button>
          )}
        </>
      }
    >
      <p>
        {t(
          conflict.canUpdate ? "ui.editor.stamps.conflict_newer" : "ui.editor.stamps.conflict_same",
          {
            name: conflict.name,
            version: conflict.version,
            installed: conflict.installed,
          },
        )}
      </p>
    </Dialog>
  );
}

export function StampDialogs() {
  const organize = useStampsUi((s) => s.organize);
  const saveAs = useStampsUi((s) => s.saveAs);
  return (
    <>
      {organize && <OrganizeStamps initial={organize} />}
      {saveAs && <SaveAsStamp />}
      <PackConflictDialog />
    </>
  );
}

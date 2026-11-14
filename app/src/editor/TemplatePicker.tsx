// The template picker (EDT-095, EDT-096, EDT-101): thumbnails by family, own templates,
// a search field and "For N images"; Create opens a new document with the images in the
// zones. Also the Save as template dialog (EDT-101).
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ContextMenu, type MenuItem } from "../components/Menu";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import { useEditor, xsdocUrl } from "./state";
import { exportTemplate, importTemplate, newFromTemplate, saveAsTemplate } from "./templateActions";
import {
  FAMILIES,
  closePicker,
  grouped,
  thumbCount,
  usePicker,
  useSaveTemplate,
  type PickerTemplate,
} from "./templateUi";

interface Listing {
  templates: PickerTemplate[];
  max: number;
  theme: string;
}

export function TemplateDialogs() {
  const picker = usePicker((s) => s.open);
  const saving = useSaveTemplate((s) => s.open);
  return (
    <>
      {picker && <Picker />}
      {saving && <SaveTemplate />}
    </>
  );
}

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

function Picker() {
  const { t, i18n } = useTranslation();
  const images = usePicker((s) => s.images);
  const [data, setData] = useState<Listing | null>(null);
  const [reload, setReload] = useState(0);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState(true);
  const [chosen, setChosen] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [menu, setMenu] = useState<{ x: number; y: number; item: PickerTemplate } | null>(null);
  const [renaming, setRenaming] = useState<PickerTemplate | null>(null);
  const [deleting, setDeleting] = useState<PickerTemplate | null>(null);

  useEffect(() => {
    let alive = true;
    run<Listing>("editor.templates")
      .then((r) => alive && setData(r))
      .catch(fail);
    return () => {
      alive = false;
    };
  }, [reload]);

  const max = data?.max ?? 20;
  const n = images.length;
  const groups = data ? grouped(data.templates, query, n, filter && n > 0) : [];
  const visible = groups.flatMap((g) => g.items);
  const selected = visible.find((x) => x.id === chosen) ?? visible[0] ?? null;

  const create = async (item: PickerTemplate | null) => {
    if (!item || busy) return;
    setBusy(true);
    const ok = await newFromTemplate(item.id, images.slice(0, max));
    setBusy(false);
    if (ok) closePicker();
  };
  const own = async (action: string, item: PickerTemplate, extra: Record<string, unknown> = {}) => {
    try {
      await run("editor.templates", { action, id: item.id, ...extra });
      setReload((v) => v + 1);
    } catch (e) {
      fail(e);
    }
  };
  const ownMenu = (item: PickerTemplate): MenuItem[] => [
    { id: "rename", label: t("ui.editor.template.rename"), onSelect: () => setRenaming(item) },
    {
      id: "duplicate",
      label: t("ui.editor.template.duplicate"),
      onSelect: () => void own("duplicate", item),
    },
    {
      id: "export",
      label: t("ui.editor.template.export"),
      onSelect: () => void exportTemplate(item.id, item.name),
    },
    {
      id: "delete",
      label: t("ui.editor.template.delete"),
      separator: true,
      onSelect: () => setDeleting(item),
    },
  ];
  const thumb = (item: PickerTemplate) =>
    xsdocUrl(
      `template/${encodeURIComponent(item.id)}/${thumbCount(item, n)}?v=${data?.theme ?? ""}-${
        i18n.language
      }-${item.version ?? 0}`,
    );

  // One dialog at a time: Esc closes only the one shown.
  if (renaming)
    return (
      <RenameTemplate
        item={renaming}
        onDone={(name) => {
          const item = renaming;
          setRenaming(null);
          if (name) void own("rename", item, { name });
        }}
      />
    );
  if (deleting)
    return (
      <Dialog
        title={t("ui.editor.template.delete_title", { name: deleting.name })}
        onCancel={() => setDeleting(null)}
        actions={
          <>
            <button type="button" className="btn" onClick={() => setDeleting(null)}>
              {t("ui.editor.cancel")}
            </button>
            <button
              type="button"
              className="btn danger"
              onClick={() => {
                const item = deleting;
                setDeleting(null);
                void own("delete", item);
              }}
            >
              {t("ui.editor.template.delete")}
            </button>
          </>
        }
      >
        <p>{t("ui.editor.template.delete_body")}</p>
      </Dialog>
    );
  return (
    <Dialog
      title={t(n > 0 ? "ui.editor.template.combine_title" : "ui.editor.template.picker_title")}
      onCancel={closePicker}
      actions={
        <>
          <button type="button" className="btn" onClick={() => void importTemplate()}>
            {t("ui.editor.template.import")}
          </button>
          <span className="spacer" />
          <button type="button" className="btn" onClick={closePicker}>
            {t("ui.editor.cancel")}
          </button>
          <button
            type="button"
            className="btn primary"
            disabled={!selected || busy}
            onClick={() => void create(selected)}
          >
            {t("ui.editor.template.create")}
          </button>
        </>
      }
    >
      <div className="tpl-picker">
        <div className="tpl-search row">
          <input
            type="search"
            data-autofocus
            value={query}
            aria-label={t("ui.editor.template.search")}
            placeholder={t("ui.editor.template.search")}
            onChange={(e) => setQuery(e.target.value)}
          />
          {n > 0 && (
            <label className="panel-check">
              <input
                type="checkbox"
                checked={filter}
                onChange={(e) => setFilter(e.target.checked)}
              />
              {t("ui.editor.template.for_n", { count: Math.min(n, max) })}
            </label>
          )}
        </div>
        {n > max && (
          <p className="tpl-warning" role="alert">
            {t("ui.editor.template.too_many", { count: n, max })}
          </p>
        )}
        <div className="tpl-groups">
          {!data && <p className="muted">{t("ui.editor.template.loading")}</p>}
          {data && groups.length === 0 && <p className="muted">{t("ui.editor.template.none")}</p>}
          {groups.map((g) => (
            <section
              key={g.family}
              className="tpl-group"
              aria-label={t(`editor.template.family.${g.family}`)}
            >
              <h3>{t(`editor.template.family.${g.family}`)}</h3>
              <div
                className="tpl-grid"
                role="listbox"
                aria-label={t(`editor.template.family.${g.family}`)}
              >
                {g.items.map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    role="option"
                    aria-selected={selected?.id === item.id}
                    className="tpl-tile"
                    title={item.name}
                    onClick={() => setChosen(item.id)}
                    onDoubleClick={() => void create(item)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        void create(item);
                      }
                    }}
                    onContextMenu={(e) => {
                      if (!item.own) return;
                      e.preventDefault();
                      setChosen(item.id);
                      setMenu({ x: e.clientX, y: e.clientY, item });
                    }}
                  >
                    <span className="tpl-thumb">
                      <img src={thumb(item)} alt="" draggable={false} />
                    </span>
                    <span className="tpl-name">{item.name}</span>
                    <span className="tpl-count muted">
                      {item.min === item.max
                        ? t("ui.editor.template.images", { count: item.max })
                        : t("ui.editor.template.images_range", { min: item.min, max: item.max })}
                    </span>
                  </button>
                ))}
              </div>
            </section>
          ))}
        </div>
      </div>
      {menu && (
        <ContextMenu
          open
          x={menu.x}
          y={menu.y}
          label={menu.item.name}
          items={ownMenu(menu.item)}
          onClose={() => setMenu(null)}
        />
      )}
    </Dialog>
  );
}

function RenameTemplate(props: { item: PickerTemplate; onDone: (name: string | null) => void }) {
  const { t } = useTranslation();
  const [name, setName] = useState(props.item.name);
  return (
    <Dialog
      title={t("ui.editor.template.rename")}
      onCancel={() => props.onDone(null)}
      actions={
        <>
          <button type="button" className="btn" onClick={() => props.onDone(null)}>
            {t("ui.editor.cancel")}
          </button>
          <button
            type="button"
            className="btn primary"
            disabled={!name.trim()}
            onClick={() => props.onDone(name.trim())}
          >
            {t("ui.editor.template.rename")}
          </button>
        </>
      }
    >
      <div className="prop-field">
        <label htmlFor="tpl-rename">{t("ui.editor.template.name")}</label>
        <input
          id="tpl-rename"
          data-autofocus
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && name.trim() && props.onDone(name.trim())}
        />
      </div>
    </Dialog>
  );
}

/** File > Save as template (EDT-101): name, family and whether the zones are emptied. */
function SaveTemplate() {
  const { t } = useTranslation();
  const docName = useEditor((s) => s.doc?.name ?? "");
  const hasZones = useEditor((s) => (s.doc?.objects ?? []).some((o) => o.type === "zone"));
  const [name, setName] = useState(docName);
  const [family, setFamily] = useState<string>("mine");
  const [empty, setEmpty] = useState(true);
  const [taken, setTaken] = useState(false);
  const close = () => useSaveTemplate.setState({ open: false });
  const save = async (replace: boolean) => {
    const ok = await saveAsTemplate(name.trim(), family, empty, replace);
    if (ok) close();
    else if (!replace) setTaken(true);
  };
  return (
    <Dialog
      title={t("ui.editor.template.save_title")}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
          {taken && (
            <button type="button" className="btn" onClick={() => void save(true)}>
              {t("ui.editor.template.replace_template")}
            </button>
          )}
          <button
            type="button"
            className="btn primary"
            disabled={!name.trim() || !hasZones}
            onClick={() => void save(false)}
          >
            {t("ui.editor.save")}
          </button>
        </>
      }
    >
      {!hasZones && <p className="tpl-warning">{t("errors.template_without_zones")}</p>}
      <div className="prop-field">
        <label htmlFor="tpl-name">{t("ui.editor.template.name")}</label>
        <input
          id="tpl-name"
          data-autofocus
          value={name}
          onChange={(e) => {
            setName(e.target.value);
            setTaken(false);
          }}
        />
      </div>
      <div className="prop-field">
        <label htmlFor="tpl-family">{t("ui.editor.template.family")}</label>
        <select id="tpl-family" value={family} onChange={(e) => setFamily(e.target.value)}>
          {FAMILIES.map((f) => (
            <option key={f} value={f}>
              {t(`editor.template.family.${f}`)}
            </option>
          ))}
        </select>
      </div>
      <label className="panel-check">
        <input type="checkbox" checked={empty} onChange={(e) => setEmpty(e.target.checked)} />
        {t("ui.editor.template.empty_zones")}
      </label>
    </Dialog>
  );
}

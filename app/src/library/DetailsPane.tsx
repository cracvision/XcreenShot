// The Library's Details pane (LIB-012): it follows the selection. One item: its
// name (editable), type, size, place, dates, app, website, system, user, tags and
// favorite; each value can be copied from its menu. Several: how many, their size,
// their dates and the tags they share.
import { useEffect, useState, type MouseEvent, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { ContextMenu } from "../components/Menu";
import { run } from "../lib/commands";
import { useEditor } from "../editor/state";
import { useDraft } from "../editor/useDraft";
import { fileSize, when } from "./format";
import { editTags, toggleFavorite } from "./marks";
import { useLibrary, type LibraryItem } from "./state";

interface Details {
  created: string | null;
  modified: string | null;
  capturedAt: string | null;
  app: string | null;
  appVersion: string | null;
  url: string | null;
  os: string | null;
  user: string | null;
  scale: number | null;
  monitor: string | null;
}

function fail(e: unknown): void {
  useEditor.getState().showError(e);
}

export function DetailsPane() {
  const { t } = useTranslation();
  const items = useLibrary((s) => s.items);
  const selection = useLibrary((s) => s.selection);
  const chosen = items.filter((i) => selection.includes(i.id));
  return (
    <aside className="library-details" aria-label={t("ui.library.details.title")}>
      <h2 className="side-title">{t("ui.library.details.title")}</h2>
      {chosen.length === 0 ? (
        <p className="muted">{t("ui.library.details.none")}</p>
      ) : chosen.length === 1 && chosen[0] ? (
        <One key={chosen[0].id} item={chosen[0]} />
      ) : (
        <Many items={chosen} />
      )}
    </aside>
  );
}

/** A value with Copy in its menu. */
function Field(props: { label: string; value: string; children?: ReactNode }) {
  const { t } = useTranslation();
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
  const copy = (e: MouseEvent) => {
    e.preventDefault();
    if (props.value && props.value !== "—") setMenu({ x: e.clientX, y: e.clientY });
  };
  return (
    <>
      <dt>{props.label}</dt>
      <dd onContextMenu={copy}>
        {props.children ?? props.value}
        {menu && (
          <ContextMenu
            open
            x={menu.x}
            y={menu.y}
            label={props.label}
            items={[
              {
                id: "copy",
                label: t("ui.library.details.copy"),
                onSelect: () => void navigator.clipboard.writeText(props.value).catch(fail),
              },
            ]}
            onClose={() => setMenu(null)}
          />
        )}
      </dd>
    </>
  );
}

function Name({ item }: { item: LibraryItem }) {
  const { t } = useTranslation();
  const [draft, setDraft] = useDraft(item.name);
  const commit = () => {
    const name = draft.trim();
    if (name && name !== item.name)
      run("library.rename", { id: item.id, name }).catch((e: unknown) => {
        fail(e);
        setDraft(null);
      });
    else setDraft(null);
  };
  return (
    <input
      className="details-name"
      value={draft}
      aria-label={t("ui.library.details.name")}
      onChange={(e) => setDraft(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") commit();
        if (e.key === "Escape") setDraft(null);
      }}
    />
  );
}

function Tags({ ids, tags }: { ids: string[]; tags: string[] }) {
  const { t } = useTranslation();
  return (
    <div className="details-tags">
      {tags.length === 0 ? (
        <span className="muted">—</span>
      ) : (
        tags.map((tag) => (
          <span key={tag} className="tag-chip details-chip">
            {tag}
          </span>
        ))
      )}
      <button type="button" className="btn small" onClick={() => editTags(ids)}>
        {t("ui.library.details.edit_tags")}
      </button>
    </div>
  );
}

function One({ item }: { item: LibraryItem }) {
  const { t, i18n } = useTranslation();
  const lang = i18n.language;
  const [d, setD] = useState<Details | null>(null);
  useEffect(() => {
    let alive = true;
    run<{ details: Details }>("library.get", { id: item.id, details: true })
      .then((r) => alive && setD(r.details))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [item.id, item.modified]);
  const dash = (v: string | null | undefined) => (v ? v : "—");
  const dims =
    item.width && item.height
      ? `${item.width} × ${item.height} px${d?.scale ? ` · ${Math.round(d.scale * 100)} %` : ""}`
      : "—";
  const app = d?.app ? `${d.app}${d.appVersion ? ` ${d.appVersion}` : ""}` : item.app;
  const url = d?.url ?? item.url;
  return (
    <>
      <Name item={item} />
      <dl className="doc-props details-list">
        <Field
          label={t("ui.library.details.type")}
          value={t(`ui.library.type_${item.format === "jpeg" ? "jpg" : item.format}`)}
        />
        <Field label={t("ui.library.details.dimensions")} value={dims} />
        <Field label={t("ui.library.details.size")} value={fileSize(item.size, lang)} />
        <Field label={t("ui.library.details.location")} value={item.path}>
          <span className="details-path" title={item.path}>
            {item.path}
          </span>
          <button
            type="button"
            className="btn small"
            onClick={() => void run("library.show_in_folder", { id: item.id }).catch(fail)}
          >
            {t("ui.library.show")}
          </button>
        </Field>
        <Field
          label={t("ui.library.details.captured")}
          value={when(d?.capturedAt ?? item.captured, lang)}
        />
        <Field label={t("ui.library.details.created")} value={when(d?.created, lang)} />
        <Field label={t("ui.library.details.modified")} value={when(item.modified, lang)} />
        <Field label={t("ui.library.details.app")} value={dash(app)} />
        <Field label={t("ui.library.details.site")} value={dash(url)}>
          {url ? (
            <button
              type="button"
              className="link-button"
              title={url}
              onClick={() => void run("library.open_url", { id: item.id }).catch(fail)}
            >
              {url}
            </button>
          ) : (
            "—"
          )}
        </Field>
        <Field label={t("ui.library.details.os")} value={dash(d?.os)} />
        <Field label={t("ui.library.details.user")} value={dash(d?.user)} />
        <Field label={t("ui.library.details.tags")} value={item.tags.join(", ")}>
          <Tags ids={[item.id]} tags={item.tags} />
        </Field>
        <Field label={t("ui.library.details.favorite")} value="">
          <label className="panel-check">
            <input
              type="checkbox"
              checked={item.favorite}
              onChange={() => void toggleFavorite([item.id])}
            />
            {t("ui.library.favorite")}
          </label>
        </Field>
      </dl>
    </>
  );
}

function Many({ items }: { items: LibraryItem[] }) {
  const { t, i18n } = useTranslation();
  const lang = i18n.language;
  const total = items.reduce((n, i) => n + i.size, 0);
  const times = items.map((i) => new Date(i.captured).getTime()).filter(Number.isFinite);
  const range =
    times.length > 0
      ? `${when(new Date(Math.min(...times)).toISOString(), lang)} – ${when(
          new Date(Math.max(...times)).toISOString(),
          lang,
        )}`
      : "—";
  const common = (items[0]?.tags ?? []).filter((tag) =>
    items.every((i) => i.tags.some((x) => x.toLowerCase() === tag.toLowerCase())),
  );
  return (
    <>
      <p className="details-count">{t("ui.library.details.count", { count: items.length })}</p>
      <dl className="doc-props details-list">
        <Field label={t("ui.library.details.size")} value={fileSize(total, lang)} />
        <Field label={t("ui.library.details.dates")} value={range} />
        <Field label={t("ui.library.details.common_tags")} value={common.join(", ")}>
          <Tags ids={items.map((i) => i.id)} tags={common} />
        </Field>
      </dl>
    </>
  );
}

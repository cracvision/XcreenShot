// The Library's filter pane (LIB-005, LIB-006, LIB-009 to LIB-011): Favorites, Type,
// Date (quick ranges and Year ▸ Month ▸ Day), Applications, Websites and Tags, each
// value with its number of items. Values of a section add up; sections cross.
import { useState, type MouseEvent, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { menuPoint, useContextMenu } from "../editor/menus";
import { askTagAction } from "./marks";
import { dateLabel, isOn, toggle, visibleValues, type Count, type FilterSection } from "./query";
import { useLibrary } from "./state";

const COLLAPSED_KEY = "xs.library.collapsed";

function loadCollapsed(): string[] {
  try {
    const v: unknown = JSON.parse(localStorage.getItem(COLLAPSED_KEY) ?? "[]");
    return Array.isArray(v) ? v.filter((x): x is string => typeof x === "string") : [];
  } catch {
    return [];
  }
}

function Section(props: {
  id: string;
  title: string;
  collapsed: string[];
  setCollapsed: (v: string[]) => void;
  children: ReactNode;
}) {
  const open = !props.collapsed.includes(props.id);
  const flip = () => {
    const next = open
      ? [...props.collapsed, props.id]
      : props.collapsed.filter((x) => x !== props.id);
    props.setCollapsed(next);
    try {
      localStorage.setItem(COLLAPSED_KEY, JSON.stringify(next));
    } catch {
      // Not remembered.
    }
  };
  return (
    <section className="lf-section">
      <button type="button" className="lf-head" aria-expanded={open} onClick={flip}>
        <span className={`lf-caret${open ? " open" : ""}`} aria-hidden="true">
          ▸
        </span>
        {props.title}
      </button>
      {open && <div className="lf-body">{props.children}</div>}
    </section>
  );
}

function Value(props: {
  label: string;
  count: number;
  on: boolean;
  onFlip: () => void;
  onMenu?: ((e: MouseEvent) => void) | undefined;
  depth?: number;
  expandable?: { open: boolean; flip: () => void };
}) {
  return (
    <div className="lf-row" style={{ paddingLeft: (props.depth ?? 0) * 14 }}>
      {props.expandable ? (
        <button
          type="button"
          className="lf-expand"
          aria-label={props.label}
          aria-expanded={props.expandable.open}
          onClick={props.expandable.flip}
        >
          <span className={`lf-caret${props.expandable.open ? " open" : ""}`} aria-hidden="true">
            ▸
          </span>
        </button>
      ) : (
        props.depth !== undefined && <span className="lf-expand" aria-hidden="true" />
      )}
      <button
        type="button"
        role="checkbox"
        aria-checked={props.on}
        className={`lf-value${props.on ? " on" : ""}`}
        onClick={props.onFlip}
        onContextMenu={props.onMenu}
      >
        <span className="lf-label">{props.label}</span>
        <span className="lf-count">{props.count}</span>
      </button>
    </div>
  );
}

/** A section of plain values; with more than 10, its own field to find one (LIB-006). */
function Values(props: {
  section: FilterSection;
  counts: Count[];
  label: (v: string) => string;
  onMenu?: (value: string, e: MouseEvent) => void;
}) {
  const { t } = useTranslation();
  const filters = useLibrary((s) => s.filters);
  const setFilters = useLibrary((s) => s.setFilters);
  const [find, setFind] = useState("");
  const values = visibleValues(props.counts, filters[props.section]);
  const shown = find
    ? values.filter((c) => props.label(c.value).toLowerCase().includes(find.toLowerCase()))
    : values;
  if (values.length === 0) return <p className="lf-none muted">{t("ui.library.filter.none")}</p>;
  return (
    <>
      {values.length > 10 && (
        <input
          type="search"
          className="lf-find"
          value={find}
          placeholder={t("ui.library.filter.find")}
          aria-label={t("ui.library.filter.find")}
          onChange={(e) => setFind(e.target.value)}
        />
      )}
      {shown.map((c) => (
        <Value
          key={c.value}
          label={props.label(c.value)}
          count={c.count}
          on={isOn(filters, props.section, c.value)}
          onFlip={() => setFilters(toggle(filters, props.section, c.value))}
          onMenu={props.onMenu ? (e) => props.onMenu?.(c.value, e) : undefined}
        />
      ))}
    </>
  );
}

function Dates() {
  const { t, i18n } = useTranslation();
  const facets = useLibrary((s) => s.facets);
  const filters = useLibrary((s) => s.filters);
  const setFilters = useLibrary((s) => s.setFilters);
  const [open, setOpen] = useState<string[]>([]);
  const flip = (v: string) =>
    setOpen(open.includes(v) ? open.filter((x) => x !== v) : [...open, v]);
  const value = (v: string, count: number, depth: number, expandable?: boolean) => (
    <Value
      key={v}
      label={dateLabel(v, i18n.language)}
      count={count}
      depth={depth}
      on={isOn(filters, "dates", v)}
      onFlip={() => setFilters(toggle(filters, "dates", v))}
      {...(expandable ? { expandable: { open: open.includes(v), flip: () => flip(v) } } : {})}
    />
  );
  const quick = visibleValues(
    facets.quickDates,
    filters.dates.filter((d) => !/^\d/.test(d)),
  );
  return (
    <>
      {["today", "yesterday", "last7", "month"].map((q) => {
        const c = quick.find((x) => x.value === q);
        if (!c) return null;
        return (
          <Value
            key={q}
            label={t(`ui.library.filter.${q}`)}
            count={c.count}
            on={isOn(filters, "dates", q)}
            onFlip={() => setFilters(toggle(filters, "dates", q))}
          />
        );
      })}
      {facets.dates.map((y) => (
        <div key={y.value}>
          {value(y.value, y.count, 0, true)}
          {open.includes(y.value) &&
            y.months.map((m) => (
              <div key={m.value}>
                {value(m.value, m.count, 1, true)}
                {open.includes(m.value) && m.days.map((d) => value(d.value, d.count, 2))}
              </div>
            ))}
        </div>
      ))}
    </>
  );
}

export function FilterPane() {
  const { t } = useTranslation();
  const facets = useLibrary((s) => s.facets);
  const filters = useLibrary((s) => s.filters);
  const setFilters = useLibrary((s) => s.setFilters);
  const [collapsed, setCollapsed] = useState(loadCollapsed);
  const section = { collapsed, setCollapsed };
  const tagMenu = (tag: string, e: MouseEvent) => {
    e.preventDefault();
    const count = facets.tags.find((c) => c.value === tag)?.count ?? 0;
    useContextMenu.setState({
      open: true,
      ...menuPoint(e),
      label: tag,
      items: [
        {
          id: "rename",
          label: t("ui.library.tags.rename"),
          onSelect: () => askTagAction({ action: "rename", tag, count }),
        },
        {
          id: "merge",
          label: t("ui.library.tags.merge"),
          onSelect: () => askTagAction({ action: "merge", tag, count }),
        },
        {
          id: "delete",
          label: t("ui.library.tags.delete"),
          onSelect: () => askTagAction({ action: "delete", tag, count }),
        },
        {
          id: "select",
          label: t("ui.library.tags.select"),
          separator: true,
          onSelect: () => askTagAction({ action: "select", tag, count }),
        },
      ],
    });
  };
  return (
    <nav className="library-filters" aria-label={t("ui.library.filter.title")}>
      <Value
        label={t("ui.library.filter.favorites")}
        count={facets.favorites}
        on={filters.favorites}
        onFlip={() => setFilters({ ...filters, favorites: !filters.favorites })}
      />
      <Section id="types" title={t("ui.library.filter.types")} {...section}>
        <Values
          section="types"
          counts={facets.types}
          label={(v) => t(`ui.library.filter.type_${v}`, v)}
        />
      </Section>
      <Section id="dates" title={t("ui.library.filter.dates")} {...section}>
        <Dates />
      </Section>
      <Section id="apps" title={t("ui.library.filter.apps")} {...section}>
        <Values
          section="apps"
          counts={facets.apps}
          label={(v) => v || t("ui.library.filter.unknown_app")}
        />
      </Section>
      <Section id="sites" title={t("ui.library.filter.sites")} {...section}>
        <Values
          section="sites"
          counts={facets.sites}
          label={(v) => v || t("ui.library.filter.no_site")}
        />
      </Section>
      <Section id="tags" title={t("ui.library.filter.tags")} {...section}>
        <Values section="tags" counts={facets.tags} label={(v) => v} onMenu={tagMenu} />
      </Section>
    </nav>
  );
}

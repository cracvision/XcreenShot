// Styles tab (EDT-015 to EDT-018): the active theme and its menu, the saved styles of
// the active tool (with a temporary sample of the current style and its +), the
// Favorites, and the theme editor. Everything runs `editor.style` / `editor.theme`.
import { save } from "@tauri-apps/plugin-dialog";
import i18next from "i18next";
import { useEffect, useState, type DragEvent, type MouseEvent } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { Menu, type MenuItem } from "../components/Menu";
import { isCommandError, run } from "../lib/commands";
import { loadFonts } from "./actions";
import { ColorPicker } from "./ColorPicker";
import { Icon, type IconName } from "./icons";
import { useContextMenu } from "./menus";
import { announceImport, importTheme, STYLED_TOOLS, themeRun } from "./themesUi";
import { useEditor, xsdocUrl, type Tool } from "./state";

interface Saved {
  id: string;
  name?: string;
  style: Record<string, unknown>;
}
interface Favorite {
  id: string;
  tool: string;
  style: Record<string, unknown>;
}
interface StylesData {
  theme: { id: string; name: string };
  tools: Record<string, Saved[]>;
  favorites: Favorite[];
}
interface ThemeInfo {
  id: string;
  name: string;
  colors: string[];
  font: string | null;
  shadow: boolean;
  shadowOff: string[];
  builtIn: boolean;
}

/** The same style, whatever the key order. */
function same(a: unknown, b: unknown): boolean {
  return JSON.stringify(sorted(a)) === JSON.stringify(sorted(b));
}
function sorted(v: unknown): unknown {
  if (Array.isArray(v)) return v.map(sorted);
  if (v && typeof v === "object")
    return Object.fromEntries(
      Object.keys(v as object)
        .sort()
        .filter((k) => (v as Record<string, unknown>)[k] !== null)
        .map((k) => [k, sorted((v as Record<string, unknown>)[k])]),
    );
  return v;
}
function hashOf(v: unknown): string {
  const text = JSON.stringify(sorted(v));
  let h = 2166136261;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0).toString(36);
}

function sampleUrl(tool: string, which: string, style: unknown): string {
  return xsdocUrl(`sample/${tool}/${which}?v=${hashOf(style)}`);
}

/** Saved styles and themes, read again whenever the core says they changed. */
function useStylesData(): { data: StylesData | null; themes: ThemeInfo[]; active: string } {
  const version = useEditor((s) => s.themesVersion);
  const [data, setData] = useState<StylesData | null>(null);
  const [themes, setThemes] = useState<{ active: string; themes: ThemeInfo[] }>({
    active: "",
    themes: [],
  });
  useEffect(() => {
    let alive = true;
    void Promise.all([
      run<StylesData>("editor.styles"),
      run<{ active: string; themes: ThemeInfo[] }>("editor.themes"),
    ])
      .then(([d, t]) => {
        if (!alive) return;
        setData(d);
        setThemes(t);
      })
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [version]);
  return { data, themes: themes.themes, active: themes.active };
}

const styleRun = (params: Record<string, unknown>) =>
  run<Record<string, unknown>>("editor.style", params).catch((e) =>
    useEditor.getState().showError(e),
  );

/** Replace or Keep both, when an imported theme has the name of one of the user's. */
export function ThemeConflict() {
  const { t } = useTranslation();
  const conflict = useEditor((s) => s.themeConflict);
  if (!conflict) return null;
  const close = () => useEditor.setState({ themeConflict: null });
  const choose = async (onConflict: "replace" | "keep") => {
    close();
    try {
      const r = await themeRun({ action: "import", path: conflict.path, onConflict });
      announceImport(r, useEditor.getState().showToast);
    } catch (e) {
      useEditor.getState().showError(e);
    }
  };
  return (
    <Dialog
      title={t("ui.editor.themes.conflict_title")}
      onCancel={close}
      actions={
        <>
          <button type="button" className="btn primary" onClick={() => void choose("keep")}>
            {t("ui.editor.themes.keep_both")}
          </button>
          <button type="button" className="btn" onClick={() => void choose("replace")}>
            {t("ui.editor.themes.replace")}
          </button>
          <button type="button" className="btn" onClick={close}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <p>{t("ui.editor.themes.conflict_body", { name: conflict.name })}</p>
    </Dialog>
  );
}

type Ask =
  | { kind: "rename-style"; tool: string; id: string; name: string }
  | { kind: "rename-theme"; id: string; name: string }
  | { kind: "reset"; tool: string }
  | { kind: "edit-theme"; theme: ThemeInfo | null };

export function StylesTab() {
  const { t } = useTranslation();
  const tool = useEditor((s) => s.tool);
  const current = useEditor((s) => s.styles);
  const selection = useEditor((s) => s.selection);
  const doc = useEditor((s) => s.doc);
  const { data, themes, active } = useStylesData();
  const [ask, setAsk] = useState<Ask | null>(null);
  const [dragged, setDragged] = useState<string | null>(null);
  const theme = themes.find((x) => x.id === active);
  const styled = (STYLED_TOOLS as readonly string[]).includes(tool);
  const toolName = (k: string) => t(`ui.editor.tool.${k}`);

  const themeItems: MenuItem[] = themes.map((x) => ({
    id: `theme-${x.id}`,
    label: x.name,
    checked: x.id === active,
    onSelect: () => void themeRun({ action: "activate", id: x.id }).catch(fail),
  }));
  const more: MenuItem[] = [
    {
      id: "theme-new",
      label: t("ui.editor.themes.new"),
      onSelect: () => setAsk({ kind: "edit-theme", theme: null }),
    },
    {
      id: "theme-edit",
      label: t("ui.editor.themes.edit"),
      disabled: !theme || theme.builtIn,
      onSelect: () => theme && setAsk({ kind: "edit-theme", theme }),
    },
    {
      id: "theme-duplicate",
      label: t("ui.editor.themes.duplicate"),
      onSelect: () =>
        void themeRun({ action: "duplicate", id: active })
          .then((r) => themeRun({ action: "activate", id: String(r.id) }))
          .catch(fail),
    },
    {
      id: "theme-rename",
      label: t("ui.editor.themes.rename"),
      disabled: !theme || theme.builtIn,
      onSelect: () => theme && setAsk({ kind: "rename-theme", id: theme.id, name: theme.name }),
    },
    {
      id: "theme-delete",
      label: t("ui.editor.themes.delete"),
      disabled: !theme || theme.builtIn,
      onSelect: () => void themeRun({ action: "delete", id: active }).catch(fail),
    },
    {
      id: "theme-export",
      label: t("ui.editor.themes.export"),
      separator: true,
      onSelect: () => void exportTheme(theme),
    },
    {
      id: "theme-import",
      label: t("ui.editor.themes.import"),
      onSelect: () => void importTheme(),
    },
    {
      id: "styles-reset",
      label: t("ui.editor.styles.reset"),
      separator: true,
      disabled: !styled,
      onSelect: () => setAsk({ kind: "reset", tool }),
    },
  ];

  // The selected objects of this tool take a style clicked (EDT-015).
  const selectedOfTool = (doc?.objects ?? [])
    .filter((o) => selection.includes(o.id) && o.type === tool)
    .map((o) => o.id);
  const apply = (id: string) =>
    void styleRun({
      action: "apply",
      tool,
      styleId: id,
      ...(selectedOfTool.length && doc
        ? { documentId: doc.documentId, objectIds: selectedOfTool }
        : {}),
    });

  const favoriteOf = (k: string, style: unknown) =>
    data?.favorites.find((f) => f.tool === k && same(f.style, style));

  const onSampleMenu = (e: MouseEvent, s: Saved) => {
    e.preventDefault();
    const fav = favoriteOf(tool, s.style);
    useContextMenu.setState({
      open: true,
      x: e.clientX,
      y: e.clientY,
      label: t("ui.editor.styles.title"),
      items: [
        {
          id: "style-update",
          label: t("ui.editor.styles.update"),
          onSelect: () => void styleRun({ action: "update", tool, styleId: s.id }),
        },
        {
          id: "style-rename",
          label: t("ui.editor.styles.rename"),
          onSelect: () => setAsk({ kind: "rename-style", tool, id: s.id, name: s.name ?? "" }),
        },
        {
          id: "style-favorite",
          label: fav ? t("ui.editor.styles.unfavorite") : t("ui.editor.styles.favorite"),
          onSelect: () =>
            void (fav
              ? styleRun({ action: "unfavorite", favoriteId: fav.id })
              : styleRun({ action: "favorite", tool, styleId: s.id })),
        },
        {
          id: "style-delete",
          label: t("ui.editor.styles.delete"),
          separator: true,
          onSelect: () => void styleRun({ action: "delete", tool, styleId: s.id }),
        },
      ],
    });
  };

  // Drag to reorder (EDT-015, EDT-016).
  const dragProps = (id: string, move: (to: number) => void, index: number) => ({
    draggable: true,
    onDragStart: (e: DragEvent) => {
      setDragged(id);
      e.dataTransfer.effectAllowed = "move";
    },
    onDragOver: (e: DragEvent) => {
      if (dragged && dragged !== id) e.preventDefault();
    },
    onDrop: (e: DragEvent) => {
      e.preventDefault();
      if (dragged && dragged !== id) move(index);
      setDragged(null);
    },
    onDragEnd: () => setDragged(null),
  });

  const header = (
    <div className="styles-head">
      <Menu
        label={theme?.name ?? "…"}
        items={themeItems}
        triggerClass="btn small"
        ariaLabel={t("ui.editor.themes.theme")}
        title={t("ui.editor.themes.theme")}
      />
      <Menu
        label={<Icon name="more" />}
        items={more}
        align="end"
        chevron={false}
        triggerClass="btn icon small"
        ariaLabel={t("ui.editor.themes.more")}
        title={t("ui.editor.themes.more")}
      />
    </div>
  );

  let body;
  if (tool === "favorites") {
    const favs = data?.favorites ?? [];
    body = favs.length ? (
      <div className="style-grid" role="list" aria-label={t("ui.editor.tool.favorites")}>
        {favs.map((f, i) => (
          <div
            key={f.id}
            className="style-cell"
            role="listitem"
            {...dragProps(
              f.id,
              (to) => void styleRun({ action: "move_favorite", favoriteId: dragged, to }),
              i,
            )}
          >
            <button
              type="button"
              className="style-sample"
              title={toolName(f.tool)}
              onClick={() => void styleRun({ action: "apply_favorite", favoriteId: f.id })}
            >
              <img src={sampleUrl(f.tool, f.id, f.style)} alt={toolName(f.tool)} />
              <span className="style-tool">
                <Icon name={f.tool as IconName} />
              </span>
            </button>
            <button
              type="button"
              className="style-star"
              aria-pressed={true}
              aria-label={t("ui.editor.styles.unfavorite")}
              title={t("ui.editor.styles.unfavorite")}
              onClick={() => void styleRun({ action: "unfavorite", favoriteId: f.id })}
            >
              ★
            </button>
          </div>
        ))}
      </div>
    ) : (
      <p className="muted">{t("ui.editor.styles.no_favorites")}</p>
    );
  } else if (styled && data && current) {
    const saved = data.tools[tool] ?? [];
    const now = (current as unknown as Record<string, unknown>)[tool];
    const isSaved = saved.some((s) => same(s.style, now));
    body = (
      <div className="style-grid" role="list" aria-label={t("ui.editor.styles.title")}>
        {saved.map((s, i) => {
          const fav = favoriteOf(tool, s.style);
          const label = s.name ?? `${toolName(tool)} ${i + 1}`;
          return (
            <div
              key={s.id}
              className="style-cell"
              role="listitem"
              {...dragProps(
                s.id,
                (to) => void styleRun({ action: "move", tool, styleId: dragged, to }),
                i,
              )}
            >
              <button
                type="button"
                className="style-sample"
                aria-pressed={same(s.style, now)}
                title={label}
                onClick={() => apply(s.id)}
                onContextMenu={(e) => onSampleMenu(e, s)}
              >
                <img src={sampleUrl(tool, s.id, s.style)} alt={label} />
              </button>
              <button
                type="button"
                className="style-star"
                aria-pressed={Boolean(fav)}
                aria-label={fav ? t("ui.editor.styles.unfavorite") : t("ui.editor.styles.favorite")}
                title={fav ? t("ui.editor.styles.unfavorite") : t("ui.editor.styles.favorite")}
                onClick={() =>
                  void (fav
                    ? styleRun({ action: "unfavorite", favoriteId: fav.id })
                    : styleRun({ action: "favorite", tool, styleId: s.id }))
                }
              >
                {fav ? "★" : "☆"}
              </button>
            </div>
          );
        })}
        {!isSaved && (
          // The current style, not saved yet: dashed, with its + (EDT-015).
          <div className="style-cell temporary" role="listitem">
            <span className="style-sample" title={t("ui.editor.styles.current")}>
              <img src={sampleUrl(tool, "current", now)} alt={t("ui.editor.styles.current")} />
            </span>
            <button
              type="button"
              className="style-add"
              aria-label={t("ui.editor.styles.add")}
              title={t("ui.editor.styles.add")}
              onClick={() => void styleRun({ action: "save", tool })}
            >
              +
            </button>
          </div>
        )}
      </div>
    );
  } else if (tool !== "stamp") {
    body = <p className="muted">{t("ui.editor.styles.none")}</p>;
  }

  return (
    <div className="styles-tab">
      {header}
      {body}
      {ask?.kind === "rename-style" || ask?.kind === "rename-theme" ? (
        <RenameDialog
          title={
            ask.kind === "rename-style"
              ? t("ui.editor.styles.rename_title")
              : t("ui.editor.themes.rename_title")
          }
          value={ask.name}
          onCancel={() => setAsk(null)}
          onOk={(name) => {
            setAsk(null);
            if (ask.kind === "rename-style")
              void styleRun({ action: "rename", tool: ask.tool, styleId: ask.id, name });
            else void themeRun({ action: "rename", id: ask.id, name }).catch(fail);
          }}
        />
      ) : null}
      {ask?.kind === "reset" && (
        <Dialog
          title={t("ui.editor.styles.reset_title", { tool: toolName(ask.tool) })}
          onCancel={() => setAsk(null)}
          actions={
            <>
              <button
                type="button"
                className="btn primary"
                onClick={() => {
                  setAsk(null);
                  void styleRun({ action: "reset", tool: ask.tool });
                }}
              >
                {t("ui.editor.styles.reset")}
              </button>
              <button type="button" className="btn" onClick={() => setAsk(null)}>
                {t("ui.editor.cancel")}
              </button>
            </>
          }
        >
          <p>{t("ui.editor.styles.reset_body")}</p>
        </Dialog>
      )}
      {ask?.kind === "edit-theme" && <ThemeEditor theme={ask.theme} onClose={() => setAsk(null)} />}
    </div>
  );
}

function fail(e: unknown) {
  useEditor.getState().showError(e);
}

async function exportTheme(theme: ThemeInfo | undefined) {
  if (!theme) return;
  try {
    const path = await save({
      defaultPath: `${theme.name}.xtheme`,
      filters: [{ name: i18next.t("ui.editor.themes.file"), extensions: ["xtheme"] }],
    });
    if (!path) return;
    await themeRun({ action: "export", id: theme.id, path });
    useEditor.getState().showToast(i18next.t("ui.editor.themes.exported"));
  } catch (e) {
    fail(e);
  }
}

function RenameDialog(props: {
  title: string;
  value: string;
  onCancel: () => void;
  onOk: (name: string) => void;
}) {
  const { t } = useTranslation();
  const [value, setValue] = useState(props.value);
  return (
    <Dialog
      title={props.title}
      onCancel={props.onCancel}
      actions={
        <>
          <button type="button" className="btn primary" onClick={() => props.onOk(value)}>
            {t("ui.editor.ok")}
          </button>
          <button type="button" className="btn" onClick={props.onCancel}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <label className="prop-field">
        <span>{t("ui.editor.styles.name")}</span>
        <input
          type="text"
          data-autofocus
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") props.onOk(value);
          }}
        />
      </label>
    </Dialog>
  );
}

/** Tools whose shadow a theme can turn off (EDT-017, EDT-021). */
const SHADOW_TOOLS: Tool[] = ["arrow", "line", "text", "callout", "shape", "step", "pen", "stamp"];

/** Theme editor (EDT-017): name, colors, font, shadow, and a live preview. */
function ThemeEditor({ theme, onClose }: { theme: ThemeInfo | null; onClose: () => void }) {
  const { t } = useTranslation();
  const fonts = useEditor((s) => s.fonts);
  const [name, setName] = useState(theme?.name ?? "");
  const [colors, setColors] = useState<string[]>(
    theme?.colors ?? ["#1F2430", "#F2545B", "#2D7FF9"],
  );
  const [font, setFont] = useState(theme?.font ?? "");
  const [shadow, setShadow] = useState(theme?.shadow ?? true);
  const [shadowOff, setShadowOff] = useState<string[]>(
    theme?.shadowOff ?? ["text", "stamp", "pen"],
  );
  const [warning, setWarning] = useState<string | null>(null);
  const [preview, setPreview] = useState<Record<string, string>>({});
  const [confirm, setConfirm] = useState(false);
  useEffect(() => {
    void loadFonts();
  }, []);
  const settings = { name, colors, font: font || null, shadow, shadowOff };
  const key = JSON.stringify(settings);
  useEffect(() => {
    let alive = true;
    const timer = setTimeout(() => {
      void themeRun({ action: "preview", theme: settings })
        .then((r) => alive && setPreview((r.samples as Record<string, string>) ?? {}))
        .catch(() => undefined);
    }, 150);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);
  const norm = (c: string) => c.slice(0, 7).toUpperCase();
  const setColor = (i: number, c: string) => {
    // No color twice (EDT-017).
    if (colors.some((x, j) => j !== i && norm(x) === norm(c))) {
      setWarning(t("ui.editor.themes.color_twice"));
      return;
    }
    setWarning(null);
    setColors(colors.map((x, j) => (j === i ? c : x)));
  };
  const addColor = () => {
    const free = ["#888888", "#3FB950", "#A371F7", "#FF7B72", "#79C0FF", "#E3B341", "#000000"].find(
      (c) => !colors.some((x) => norm(x) === c),
    );
    if (free) setColors([...colors, free]);
  };
  const move = (i: number, d: -1 | 1) => {
    const j = i + d;
    if (j < 0 || j >= colors.length) return;
    const next = [...colors];
    [next[i], next[j]] = [next[j] as string, next[i] as string];
    setColors(next);
  };
  const doSave = async () => {
    try {
      await themeRun({ action: "save", ...(theme ? { id: theme.id } : {}), theme: settings });
      onClose();
    } catch (e) {
      if (isCommandError(e)) setWarning(e.message);
      else fail(e);
    }
  };
  if (confirm)
    return (
      <Dialog
        title={t("ui.editor.themes.save_title")}
        onCancel={() => setConfirm(false)}
        actions={
          <>
            <button type="button" className="btn primary" onClick={() => void doSave()}>
              {t("ui.editor.themes.save")}
            </button>
            <button type="button" className="btn" onClick={() => setConfirm(false)}>
              {t("ui.editor.cancel")}
            </button>
          </>
        }
      >
        <p>{t("ui.editor.themes.save_body")}</p>
      </Dialog>
    );
  return (
    <Dialog
      title={theme ? t("ui.editor.themes.edit_title") : t("ui.editor.themes.new_title")}
      onCancel={onClose}
      actions={
        <>
          <button
            type="button"
            className="btn primary"
            onClick={() => (theme ? setConfirm(true) : void doSave())}
          >
            {t("ui.editor.themes.save")}
          </button>
          <button type="button" className="btn" onClick={onClose}>
            {t("ui.editor.cancel")}
          </button>
        </>
      }
    >
      <div className="theme-editor">
        <label className="prop-field">
          <span>{t("ui.editor.styles.name")}</span>
          <input
            type="text"
            data-autofocus
            value={name}
            placeholder={t("ui.editor.themes.untitled")}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
        <fieldset className="theme-colors">
          <legend>{t("ui.editor.themes.colors")}</legend>
          {colors.map((c, i) => (
            <div key={i} className="theme-color">
              <ColorPicker
                label={`${t("ui.editor.prop.color")} ${i + 1}`}
                value={c}
                onCommit={(v) => v && setColor(i, v)}
              />
              <button
                type="button"
                className="btn icon small"
                aria-label={t("ui.editor.themes.up")}
                title={t("ui.editor.themes.up")}
                disabled={i === 0}
                onClick={() => move(i, -1)}
              >
                ↑
              </button>
              <button
                type="button"
                className="btn icon small"
                aria-label={t("ui.editor.themes.down")}
                title={t("ui.editor.themes.down")}
                disabled={i === colors.length - 1}
                onClick={() => move(i, 1)}
              >
                ↓
              </button>
              <button
                type="button"
                className="btn icon small"
                aria-label={t("ui.editor.themes.remove_color")}
                title={t("ui.editor.themes.remove_color")}
                disabled={colors.length <= 2}
                onClick={() => setColors(colors.filter((_, j) => j !== i))}
              >
                ×
              </button>
            </div>
          ))}
          <button
            type="button"
            className="btn small"
            disabled={colors.length >= 8}
            onClick={addColor}
          >
            {t("ui.editor.themes.add_color")}
          </button>
          {warning && (
            <p className="warning-text" role="alert">
              {warning}
            </p>
          )}
        </fieldset>
        <label className="prop-field">
          <span>{t("ui.editor.themes.font")}</span>
          <select value={font} onChange={(e) => setFont(e.target.value)}>
            <option value="">{t("ui.editor.themes.system_font")}</option>
            {fonts.map((f) => (
              <option key={f} value={f}>
                {f}
              </option>
            ))}
          </select>
        </label>
        <label className="panel-check">
          <input type="checkbox" checked={shadow} onChange={(e) => setShadow(e.target.checked)} />
          {t("ui.editor.themes.shadow")}
        </label>
        {shadow && (
          <div className="theme-shadow-tools">
            {SHADOW_TOOLS.map((k) => (
              <label key={k} className="panel-check">
                <input
                  type="checkbox"
                  checked={!shadowOff.includes(k)}
                  onChange={(e) =>
                    setShadowOff(
                      e.target.checked ? shadowOff.filter((x) => x !== k) : [...shadowOff, k],
                    )
                  }
                />
                {t(`ui.editor.tool.${k}`)}
              </label>
            ))}
          </div>
        )}
        <div className="theme-preview" aria-label={t("ui.editor.themes.preview")}>
          {["arrow", "callout", "shape", "step"].map((k) =>
            preview[k] ? <img key={k} src={preview[k]} alt={t(`ui.editor.tool.${k}`)} /> : null,
          )}
        </div>
      </div>
    </Dialog>
  );
}

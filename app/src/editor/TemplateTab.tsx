// The Template tab (EDT-097, EDT-099): the settings of a template and its sections.
// Every change is one command of the core (`document.template`) and one undo step.
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Menu } from "../components/Menu";
import { useIndicator } from "../components/useIndicator";
import { run } from "../lib/commands";
import { ColorPicker } from "./ColorPicker";
import { FontField, SelectField } from "./Fields";
import { Icon } from "./icons";
import { FillPicker } from "./FillPicker";
import { STEP_SHAPES, useEditor, type StepShapeKind } from "./state";
import { askSaveTemplate, setTemplate, templateCommand } from "./templateActions";
import { SECTION_KINDS, bands, templateOf, type TemplateSettings } from "./templateUi";
import { useDraft } from "./useDraft";

function useThemes(): { id: string; name: string }[] {
  const version = useEditor((s) => s.themesVersion);
  const [themes, setThemes] = useState<{ id: string; name: string }[]>([]);
  useEffect(() => {
    let alive = true;
    run<{ themes: { id: string; name: string }[] }>("editor.themes")
      .then((r) => alive && setThemes(r.themes))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [version]);
  return themes;
}

export function TemplateTab() {
  const { t } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const fonts = useEditor((s) => s.fonts);
  const themes = useThemes();
  const template = templateOf(doc);
  const seg = useRef<HTMLDivElement>(null);
  useIndicator(seg, template?.settings.scale ?? "fit");
  if (!doc || !template) {
    return <p className="muted">{t("ui.editor.template.not_template")}</p>;
  }
  const s = template.settings;
  const set = (changes: Partial<TemplateSettings>) => void setTemplate(changes);
  const check = (key: "title" | "captions" | "footer" | "numbering", label: string) => (
    <label className="panel-check">
      <input type="checkbox" checked={s[key]} onChange={(e) => set({ [key]: e.target.checked })} />
      {label}
    </label>
  );
  const sections = bands(template);
  return (
    <section className="tpl-tab" aria-label={t("ui.editor.tab_template")}>
      <h2 className="side-title">{t("ui.editor.tab_template")}</h2>
      <div className="prop-field">
        <span className="prop-label">{t("ui.editor.template.scale")}</span>
        <div className="seg small" role="radiogroup" ref={seg}>
          <span className="seg-indicator" aria-hidden="true" />
          {(["fit", "fill"] as const).map((v) => (
            <button
              key={v}
              type="button"
              role="radio"
              data-value={v}
              aria-checked={s.scale === v}
              onClick={() => s.scale !== v && set({ scale: v })}
            >
              {t(`ui.editor.template.scale_${v}`)}
            </button>
          ))}
        </div>
      </div>
      {check("title", t("ui.editor.template.show_title"))}
      {check("captions", t("ui.editor.template.captions"))}
      {check("footer", t("ui.editor.template.footer"))}
      {s.footer && (
        <>
          <SelectField
            label={t("ui.editor.template.author")}
            value={s.author}
            options={[
              { value: "system", label: t("ui.editor.template.author_system") },
              { value: "custom", label: t("ui.editor.template.author_custom") },
            ]}
            onCommit={(author) => set({ author })}
          />
          {s.author === "custom" && (
            <AuthorField value={s.authorText} onCommit={(authorText) => set({ authorText })} />
          )}
        </>
      )}
      <SelectField
        label={t("ui.editor.template.theme")}
        value={s.theme ?? ""}
        options={[
          { value: "", label: t("ui.editor.template.theme_active") },
          ...themes.map((th) => ({ value: th.id, label: th.name })),
        ]}
        onCommit={(theme) => set({ theme: theme || null })}
      />
      <div className="prop-field">
        <label htmlFor="font-field">{t("ui.editor.prop.font")}</label>
        <FontField value={s.font ?? ""} fonts={fonts} onCommit={(font) => set({ font })} />
      </div>
      <ColorPicker
        label={t("ui.editor.template.text_color")}
        value={s.textColor ?? "#1F2430"}
        onCommit={(textColor) => textColor && set({ textColor })}
      />
      <FillPicker
        label={t("ui.editor.template.background")}
        value={s.background}
        allowTransparent
        onCommit={(background) => set({ background })}
      />
      {check("numbering", t("ui.editor.template.numbering"))}
      {s.numbering && (
        <>
          <ColorPicker
            label={t("ui.editor.template.number_color")}
            value={s.numberColor ?? "#F2545B"}
            onCommit={(numberColor) => numberColor && set({ numberColor })}
          />
          <SelectField
            label={t("ui.editor.prop.step_shape")}
            value={s.numberShape}
            options={STEP_SHAPES.map((v) => ({ value: v, label: t(`ui.editor.step_shape.${v}`) }))}
            onCommit={(numberShape: StepShapeKind) => set({ numberShape })}
          />
        </>
      )}
      <div className="row">
        <button
          type="button"
          className="btn"
          aria-pressed={s.reversed}
          onClick={() => void templateCommand("reverse")}
        >
          {t("ui.editor.template.reverse")}
        </button>
      </div>

      <h3 className="side-subtitle">{t("ui.editor.template.sections")}</h3>
      <ol className="tpl-sections">
        {sections.map(({ section }, i) => (
          <li key={section.id} className="tpl-section">
            <span className="tpl-section-name">
              {t(`ui.editor.template.section.${section.kind}`, section.kind)}
            </span>
            <span className="muted">{Math.round(section.height)} px</span>
            <span className="tpl-section-tools">
              <button
                type="button"
                className="btn icon small"
                aria-label={t("ui.editor.template.move_up")}
                title={t("ui.editor.template.move_up")}
                disabled={i === 0}
                onClick={() =>
                  void templateCommand("move_section", {
                    sectionId: section.id,
                    index: Math.max(0, template.sections.indexOf(section) - 1),
                  })
                }
              >
                <Icon name="up" />
              </button>
              <button
                type="button"
                className="btn icon small"
                aria-label={t("ui.editor.template.move_down")}
                title={t("ui.editor.template.move_down")}
                disabled={i === sections.length - 1}
                onClick={() =>
                  void templateCommand("move_section", {
                    sectionId: section.id,
                    index: template.sections.indexOf(section) + 1,
                  })
                }
              >
                <Icon name="chevron" />
              </button>
              <button
                type="button"
                className="btn icon small"
                aria-label={t("ui.editor.template.remove_section")}
                title={t("ui.editor.template.remove_section")}
                onClick={() => void templateCommand("remove_section", { sectionId: section.id })}
              >
                <Icon name="trash" />
              </button>
            </span>
            <span className="tpl-add">
              <Menu
                label={<Icon name="plus" />}
                ariaLabel={t("ui.editor.template.add_section_after")}
                title={t("ui.editor.template.add_section_after")}
                triggerClass="btn icon small"
                chevron={false}
                items={SECTION_KINDS.map((kind) => ({
                  id: kind,
                  label: t(`ui.editor.template.section.${kind}`),
                  onSelect: () =>
                    void templateCommand("add_section", {
                      kind,
                      index: template.sections.indexOf(section) + 1,
                    }),
                }))}
              />
            </span>
          </li>
        ))}
      </ol>
      <p className="muted small">{t("ui.editor.template.sections_help")}</p>
      <div className="row">
        <button type="button" className="btn" onClick={askSaveTemplate}>
          {t("menu.save_as_template")}
        </button>
      </div>
    </section>
  );
}

function AuthorField(props: { value: string; onCommit: (v: string) => void }) {
  const { t } = useTranslation();
  const [draft, setDraft] = useDraft(props.value);
  const commit = () => {
    if (draft !== props.value) props.onCommit(draft);
    setDraft(null);
  };
  return (
    <div className="prop-field">
      <label htmlFor="tpl-author">{t("ui.editor.template.author_name")}</label>
      <input
        id="tpl-author"
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => e.key === "Enter" && commit()}
      />
    </div>
  );
}

// The steps of a guide (SMT-029, ADR 0009): its sections in small, to reorder,
// duplicate, delete (several with Delete) and add steps, texts or pictures after any;
// the pictures' scale, the author and the theme. Each change is one command of the core
// (`document.template`) and one undo step.
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Menu } from "../components/Menu";
import { run } from "../lib/commands";
import { NumberField, SelectField } from "./Fields";
import { Icon } from "./icons";
import { imageUrl, useEditor, type DocDetails } from "./state";
import { setTemplate, templateCommand } from "./templateActions";
import { exportGuide, GUIDE_FORMATS } from "./guideExport";
import { bands, plainText, templateOf, type TemplateSection } from "./templateUi";

const ADD_KINDS = ["guideStep", "guideText", "guideImage"] as const;

function textOf(doc: DocDetails, slot: string): string {
  const o = doc.objects.find((x) => (x as { slot?: string }).slot === slot);
  return o && o.type === "text" ? o.text : "";
}

function pictureOf(doc: DocDetails, sid: string): string | null {
  const o = doc.objects.find(
    (x) => x.type === "zone" && (x as { section?: string }).section === sid,
  );
  return o && o.type === "zone" ? (o.image ?? null) : null;
}

/** The guide's sections in the list, each step with a picture numbered in order
 * (SMT-028). */
function numbered(doc: DocDetails, template: NonNullable<ReturnType<typeof templateOf>>) {
  const out = [];
  let count = 0;
  for (const band of bands(template)) {
    const kind = band.section.kind;
    if (kind === "guideHeader" || kind === "footer") continue;
    const picture = kind === "guideText" ? null : pictureOf(doc, band.section.id);
    if (kind === "guideStep" && picture) count += 1;
    out.push({ ...band, picture, number: count });
  }
  return out;
}

/** Centers the canvas on a section. */
function show(doc: DocDetails, top: number, height: number) {
  const ed = useEditor.getState();
  const view = ed.views[doc.documentId];
  if (view) ed.setView(doc.documentId, { ...view, cx: doc.canvas.width / 2, cy: top + height / 2 });
}

export function GuideTab({ themes }: { themes: { id: string; name: string }[] }) {
  const { t } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const [chosen, setChosen] = useState<string[]>([]);
  const template = templateOf(doc);
  if (!doc || !template) return null;
  const s = template.settings;
  const rows = numbered(doc, template);
  const index = (section: TemplateSection) => template.sections.indexOf(section);
  const remove = (ids: string[]) => {
    setChosen([]);
    void templateCommand("remove_section", { sectionIds: ids });
  };
  return (
    <section className="tpl-tab guide-tab" aria-label={t("ui.guide.title")}>
      <h2 className="side-title">{t("ui.guide.title")}</h2>
      <ol
        className="tpl-sections guide-steps"
        aria-label={t("ui.guide.steps")}
        tabIndex={0}
        onKeyDown={(e) => {
          if ((e.key === "Delete" || e.key === "Backspace") && chosen.length > 0) {
            e.preventDefault();
            remove(chosen);
          }
        }}
      >
        {rows.map(({ section, top, picture, number }, i) => {
          const kind = section.kind;
          const text =
            kind === "guideStep"
              ? textOf(doc, `caption:${section.id}:0`)
              : kind === "guideText"
                ? textOf(doc, `text:${section.id}`)
                : "";
          const label =
            kind === "guideStep"
              ? `${picture ? `${number}. ` : ""}${plainText(text) || t("ui.guide.empty_step")}`
              : kind === "guideText"
                ? plainText(text) || t("ui.guide.kind.guideText")
                : t(picture ? "ui.guide.kind.guideImage" : "ui.guide.empty_image");
          const selected = chosen.includes(section.id);
          return (
            <li
              key={section.id}
              className={`tpl-section guide-step${selected ? " selected" : ""}`}
              aria-selected={selected}
            >
              <button
                type="button"
                className="guide-step-main"
                title={plainText(text, 400) || label}
                onClick={(e) => {
                  setChosen((c) =>
                    e.ctrlKey || e.metaKey
                      ? c.includes(section.id)
                        ? c.filter((x) => x !== section.id)
                        : [...c, section.id]
                      : [section.id],
                  );
                  show(doc, top, section.height);
                }}
              >
                {picture ? (
                  <img className="guide-thumb" src={imageUrl(doc.documentId, picture)} alt="" />
                ) : (
                  <span className="guide-thumb empty" aria-hidden="true">
                    <Icon name={kind === "guideText" ? "text" : "capture"} />
                  </span>
                )}
                <span className="guide-step-label">{label}</span>
              </button>
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
                      index: Math.max(0, index(section) - 1),
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
                  disabled={i === rows.length - 1}
                  onClick={() =>
                    void templateCommand("move_section", {
                      sectionId: section.id,
                      index: index(section) + 1,
                    })
                  }
                >
                  <Icon name="chevron" />
                </button>
                <button
                  type="button"
                  className="btn icon small"
                  aria-label={t("ui.guide.duplicate")}
                  title={t("ui.guide.duplicate")}
                  onClick={() =>
                    void templateCommand("duplicate_section", { sectionId: section.id })
                  }
                >
                  <Icon name="copy" />
                </button>
                <button
                  type="button"
                  className="btn icon small"
                  aria-label={t("ui.guide.remove")}
                  title={t("ui.guide.remove")}
                  onClick={() => remove(selected && chosen.length > 1 ? chosen : [section.id])}
                >
                  <Icon name="trash" />
                </button>
                <Menu
                  label={<Icon name="plus" />}
                  ariaLabel={t("ui.guide.add_after")}
                  title={t("ui.guide.add_after")}
                  triggerClass="btn icon small"
                  chevron={false}
                  items={[
                    ...ADD_KINDS.map((k) => ({
                      id: k,
                      label: t(`ui.guide.add.${k}`),
                      onSelect: () =>
                        void templateCommand("add_section", { kind: k, index: index(section) + 1 }),
                    })),
                    // The recorder again; its steps come in after this one (SMT-029).
                    {
                      id: "record",
                      label: t("ui.guide.record_here"),
                      separator: true,
                      onSelect: () =>
                        void run("smart.steps_start", {
                          into: { documentId: doc.documentId, after: section.id },
                        }).catch((e: unknown) => useEditor.getState().showError(e)),
                    },
                  ]}
                />
              </span>
            </li>
          );
        })}
      </ol>
      <p className="muted small">{t("ui.guide.help")}</p>

      {/* Word, PowerPoint, PDF, Markdown, HTML, pictures, clipboard (SMT-030 to SMT-034). */}
      <div className="row">
        <Menu
          label={t("ui.guide.export_as")}
          triggerClass="btn"
          items={GUIDE_FORMATS.map((f) => ({
            id: f,
            label: t(`ui.guide.export.${f}`),
            separator: f === "md" || f === "clipboard",
            onSelect: () => void exportGuide(f),
          }))}
        />
      </div>

      <h3 className="side-subtitle">{t("ui.guide.layout")}</h3>
      <NumberField
        label={t("ui.guide.scale")}
        value={s.imageScale ?? 100}
        min={10}
        max={100}
        step={5}
        unit="%"
        onCommit={(imageScale) => void setTemplate({ imageScale })}
      />
      <SelectField
        label={t("ui.editor.template.author")}
        value={s.author}
        options={[
          { value: "system", label: t("ui.editor.template.author_system") },
          { value: "custom", label: t("ui.editor.template.author_custom") },
        ]}
        onCommit={(author) => void setTemplate({ author })}
      />
      <SelectField
        label={t("ui.editor.template.theme")}
        value={s.theme ?? ""}
        options={[
          { value: "", label: t("ui.editor.template.theme_active") },
          ...themes.map((th) => ({ value: th.id, label: th.name })),
        ]}
        onCommit={(theme) => void setTemplate({ theme: theme || null })}
      />
    </section>
  );
}

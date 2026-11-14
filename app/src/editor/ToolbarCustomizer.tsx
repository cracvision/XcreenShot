// Customize the toolbar (EDT-004): what can go on it, to drag onto the bar (or click to
// add at the end); dropping a bar item here takes it off. Restore defaults and Done.
import { useEffect, useRef, type DragEvent } from "react";
import { useTranslation } from "react-i18next";
import { Icon, type IconName } from "./icons";
import {
  actionIcon,
  actionName,
  insertItem,
  removeAt,
  resetToolbar,
  saveToolbar,
  useToolbar,
} from "./toolbarLayout";
import type { Tool } from "./state";

const DRAG_TYPE = "application/x-xcreenshot-toolbar";

export function ToolbarCustomizer() {
  const { t } = useTranslation();
  const data = useToolbar((s) => s.data);
  const customizing = useToolbar((s) => s.customizing);
  const done = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (customizing) done.current?.focus();
  }, [customizing]);
  if (!customizing || !data) return null;
  const items = data.items;
  const onBar = new Set(items);
  const close = () => useToolbar.setState({ customizing: false });
  const add = (item: string) => void saveToolbar(insertItem(items, item, items.length));
  const chip = (item: string, label: string, icon: string) => (
    <button
      key={item}
      type="button"
      className="tbc-chip"
      draggable
      onDragStart={(e) => {
        e.dataTransfer.setData(DRAG_TYPE, JSON.stringify({ item }));
        e.dataTransfer.effectAllowed = "copyMove";
      }}
      onClick={() => add(item)}
      title={t("ui.editor.customize.add_hint")}
    >
      <Icon name={icon as IconName} />
      <span>{label}</span>
    </button>
  );
  const takeOff = (e: DragEvent) => {
    const raw = e.dataTransfer.getData(DRAG_TYPE);
    if (!raw) return;
    e.preventDefault();
    const from = JSON.parse(raw) as { index?: number };
    if (typeof from.index === "number") void saveToolbar(removeAt(items, from.index));
  };
  return (
    <section
      className="tb-customizer"
      aria-label={t("ui.editor.customize.title")}
      onDragOver={(e) => e.preventDefault()}
      onDrop={takeOff}
      onKeyDown={(e) => {
        if (e.key === "Escape") close();
      }}
    >
      <p className="muted">{t("ui.editor.customize.hint")}</p>
      <div className="tbc-groups">
        <div>
          <h3>{t("ui.editor.customize.tools")}</h3>
          <div className="tbc-chips">
            {data.available.tools
              .filter((id) => !onBar.has(`tool:${id}`))
              .map((id: Tool) => chip(`tool:${id}`, t(`ui.editor.tool.${id}`), id))}
          </div>
        </div>
        <div>
          <h3>{t("ui.editor.customize.commands")}</h3>
          <div className="tbc-chips">
            {data.available.actions
              .filter((a) => !onBar.has(`action:${a}`))
              .map((a) => chip(`action:${a}`, actionName(t, a), actionIcon(a)))}
            {chip("sep", t("ui.editor.customize.separator"), "minus")}
            {chip("space", t("ui.editor.customize.space"), "distribute_h")}
          </div>
        </div>
        <div>
          <h3>{t("ui.editor.customize.effects")}</h3>
          <div className="tbc-chips">
            {data.available.effects
              .filter((a) => !onBar.has(`action:${a}`))
              .map((a) => chip(`action:${a}`, actionName(t, a), "effects"))}
          </div>
        </div>
      </div>
      <div className="tbc-actions">
        <button type="button" className="btn" onClick={() => void resetToolbar()}>
          {t("ui.editor.customize.reset")}
        </button>
        <button type="button" className="btn primary" ref={done} onClick={close}>
          {t("ui.editor.image.done")}
        </button>
      </div>
    </section>
  );
}

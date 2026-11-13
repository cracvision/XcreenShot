// The part of the toolbar the user arranges (EDT-003, EDT-004): runs of tools, commands,
// effects, separators and flexible spaces, then the temporary place of a tool chosen in
// More, and More itself. While customizing, items are dragged in, around and out, or
// moved with Alt+arrows and removed with Delete.
import { useEffect, useRef, type DragEvent, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { Menu, type MenuItem } from "../components/Menu";
import { useIndicator } from "../components/useIndicator";
import { useTooltipLevel } from "../components/tooltips";
import { useSetting } from "../store";
import { performAction, selectTool } from "./actions";
import { Icon, type IconName } from "./icons";
import { useEditor, type Tool } from "./state";
import {
  actionIcon,
  actionName,
  insertItem,
  loadToolbar,
  moveItem,
  removeAt,
  saveToolbar,
  TOOL_GROUPS,
  useToolbar,
} from "./toolbarLayout";
import { hint, useShortcut } from "./tools";

/** Where a dragged toolbar item comes from. */
const DRAG_TYPE = "application/x-xcreenshot-toolbar";

export function ToolButton({ id, active }: { id: Tool; active: boolean }) {
  const { t } = useTranslation();
  const name = t(`ui.editor.tool.${id}`);
  const shortcut = useShortcut(`editor.tool_${id}`);
  // Detailed tooltips also say what the tool does (APP-014).
  const detailed = useTooltipLevel() === "detailed";
  // Image tools do nothing on a video: off, with what to do instead (VID-065).
  const video = useEditor((s) => !!s.doc?.video);
  return (
    <button
      type="button"
      className="tb-tool"
      aria-pressed={active && !video}
      disabled={video}
      title={
        video
          ? `${name}: ${t("errors.video_no_image_tools")}`
          : hint(name, shortcut, detailed ? t(`ui.editor.tool_detail.${id}`) : undefined)
      }
      onClick={() => selectTool(id)}
    >
      <Icon name={id} />
      <span className="tb-label">{name}</span>
    </button>
  );
}

/** A run of tools side by side: one segmented group with its indicator. */
function ToolRun({ ids, tool, extra }: { ids: Tool[]; tool: Tool; extra?: string }) {
  const { t } = useTranslation();
  const labels = useSetting("editor.toolbarLabels", true);
  const ref = useRef<HTMLDivElement>(null);
  // With a video no tool is chosen: the indicator goes (VID-065).
  const video = useEditor((s) => !!s.doc?.video);
  useIndicator(ref, `${tool}:${labels}:${ids.join(",")}:${video}`);
  return (
    <div
      className={`tb-group seg brand tb-tools${extra ? ` ${extra}` : ""}`}
      role="group"
      aria-label={t("ui.editor.tools")}
      ref={ref}
    >
      <span className="seg-indicator" aria-hidden="true" />
      {ids.map((id) => (
        <ToolButton key={id} id={id} active={tool === id} />
      ))}
    </div>
  );
}

function ActionButton({ action }: { action: string }) {
  const { t } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const shortcut = useShortcut(`editor.${action}`);
  const name = actionName(t, action);
  const needsDoc = action !== "new_from_clipboard";
  return (
    <button
      type="button"
      className="btn tb-btn"
      // Image actions (effects, turning, sizes) do not apply to a video (VID-065).
      disabled={needsDoc && (!doc || !!doc.video)}
      title={hint(name, shortcut)}
      onClick={() => performAction(action)}
    >
      <Icon name={actionIcon(action) as IconName} />
      <span className="tb-label tb-wide">{name}</span>
    </button>
  );
}

/** `hide`: how many items at the end of the bar wait in More, when the window is narrow. */
export function CustomBar({ hide = 0 }: { hide?: number }) {
  const { t } = useTranslation();
  const data = useToolbar((s) => s.data);
  const customizing = useToolbar((s) => s.customizing);
  const temp = useToolbar((s) => s.temp);
  const tool = useEditor((s) => s.tool);
  const available = useEditor((s) => s.tools);
  useEffect(() => {
    void loadToolbar();
  }, []);
  const all = data?.items ?? [];
  // While customizing, everything shows (EDT-004).
  const shown = customizing ? all.length : Math.max(0, all.length - hide);
  const items = all.slice(0, shown);
  const hiddenActions = all.slice(shown).filter((i) => i.startsWith("action:"));
  const onBar = new Set(items);
  const tempShown = temp && !onBar.has(`tool:${temp}`) && available.includes(temp) ? temp : null;

  const drop = (e: DragEvent, at: number) => {
    const raw = e.dataTransfer.getData(DRAG_TYPE);
    if (!raw) return;
    e.preventDefault();
    e.stopPropagation();
    const from = JSON.parse(raw) as { index?: number; item?: string };
    if (typeof from.index === "number") {
      void saveToolbar(moveItem(items, from.index, from.index < at ? at - 1 : at));
    } else if (from.item) {
      void saveToolbar(insertItem(items, from.item, at));
    }
  };
  const dropAt = (e: DragEvent, i: number) => {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    drop(e, e.clientX < r.left + r.width / 2 ? i : i + 1);
  };
  const onKey = (e: KeyboardEvent, i: number) => {
    if (!customizing) return;
    if (e.altKey && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
      e.preventDefault();
      e.stopPropagation();
      void saveToolbar(moveItem(items, i, i + (e.key === "ArrowLeft" ? -1 : 1)));
    } else if (e.key === "Delete") {
      e.preventDefault();
      e.stopPropagation();
      void saveToolbar(removeAt(items, i));
    }
  };

  // Consecutive tools form one group; everything else stands alone.
  const blocks: { start: number; items: string[] }[] = [];
  items.forEach((item, i) => {
    const last = blocks[blocks.length - 1];
    const isTool = item.startsWith("tool:");
    if (isTool && last && last.items[0]?.startsWith("tool:") && !customizing) last.items.push(item);
    else blocks.push({ start: i, items: [item] });
  });

  const more: MenuItem[] = [
    ...TOOL_GROUPS.map((g) => ({
      id: `more-${g.key}`,
      label: t(`ui.editor.more_group.${g.key}`),
      onSelect: () => undefined,
      submenu: g.tools
        .filter((id) => available.includes(id) && !onBar.has(`tool:${id}`))
        .map((id) => ({
          id: `more-${id}`,
          label: t(`ui.editor.tool.${id}`),
          checked: tool === id,
          onSelect: () => {
            useToolbar.setState({ temp: id });
            selectTool(id);
          },
        })),
    })).filter((g) => g.submenu.length > 0),
    // Commands that left a narrow bar wait here too.
    ...hiddenActions.map((item, i) => {
      const action = item.slice(7);
      return {
        id: `more-${item}`,
        label: actionName(t, action),
        separator: i === 0,
        onSelect: () => performAction(action),
      };
    }),
    {
      id: "more-customize",
      label: t("menu.customize_toolbar"),
      separator: true,
      onSelect: () => useToolbar.setState({ customizing: true }),
    },
  ];

  return (
    <div
      className={`tb-custom${customizing ? " customizing" : ""}`}
      onDragOver={(e) => customizing && e.preventDefault()}
      onDrop={(e) => drop(e, items.length)}
    >
      {blocks.map((b) => {
        const first = b.items[0] ?? "";
        const wrap = (child: React.ReactNode, key: string) =>
          customizing ? (
            <div
              key={key}
              className="tb-item"
              draggable
              tabIndex={-1}
              onDragStart={(e) => {
                e.dataTransfer.setData(DRAG_TYPE, JSON.stringify({ index: b.start }));
                e.dataTransfer.effectAllowed = "move";
              }}
              onDragEnd={(e) => {
                // Let go outside the bar: it leaves the bar (EDT-004).
                if (e.dataTransfer.dropEffect === "none")
                  void saveToolbar(removeAt(items, b.start));
              }}
              onDragOver={(e) => e.preventDefault()}
              onDrop={(e) => dropAt(e, b.start)}
              onKeyDown={(e) => onKey(e, b.start)}
            >
              {child}
            </div>
          ) : (
            <span key={key} className="tb-item-static" onKeyDown={(e) => onKey(e, b.start)}>
              {child}
            </span>
          );
        if (first.startsWith("tool:")) {
          return wrap(
            <ToolRun ids={b.items.map((i) => i.slice(5) as Tool)} tool={tool} />,
            `t${b.start}`,
          );
        }
        if (first.startsWith("action:"))
          return wrap(<ActionButton action={first.slice(7)} />, `a${b.start}`);
        if (first === "sep")
          return wrap(<span className="tb-sep" role="separator" />, `s${b.start}`);
        return wrap(<span className="tb-space" aria-hidden="true" />, `f${b.start}`);
      })}
      {tempShown && <ToolRun ids={[tempShown]} tool={tool} extra="tb-temp" />}
      <Menu
        triggerClass="btn tb-btn tb-more"
        title={t("ui.editor.more")}
        label={
          <>
            <Icon name="more" />
            <span className="tb-label tb-wide">{t("ui.editor.more")}</span>
          </>
        }
        items={more}
      />
    </div>
  );
}

// Editor toolbar (EDT-001, EDT-003, EDT-004): Capture, the items the user arranged
// (tools, commands, effects; the rest in More), Undo and Redo
// (they stay with the menus: Carlos, 2026-10-28), and the share area (Copy all,
// Share ▾). Tooltips show the current shortcut (APP-014).
//
// A narrow window never lets its items run over each other (Carlos, 2026-11-12): the
// bar gets compact by steps (`useFit`): first the tools' names go, then the secondary
// labels, then Capture's and the view switch's; then the language and Preferences fold
// into ⋯, then Open and Save; at last the items at the end of the bar go to More.
import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { fitClasses, useFit } from "../components/fit";
import { languageItems } from "../components/languageItems";
import { LanguageToggle } from "../components/LanguageToggle";
import { Menu } from "../components/Menu";
import { run } from "../lib/commands";
import {
  copyAll,
  exportImage,
  openCapturePanel,
  openFiles,
  redo,
  saveActive,
  saveAs,
  undo,
} from "./actions";
import { Icon } from "./icons";
import { useEditor } from "./state";
import { CustomBar } from "./CustomBar";
import { menuPoint, toolbarMenu } from "./menus";
import { useSetting } from "../store";
import { hint, useShortcut } from "./tools";
import { ViewSwitch } from "../library/ViewSwitch";
import { arrowFocus } from "../components/arrowFocus";
import { useToolbar } from "./toolbarLayout";

/** The steps of a narrow bar (see the top of this file). */
const FIT_PREFS = 4;
const FIT_FILES = 5;

export function Toolbar() {
  const { t, i18n } = useTranslation();
  const doc = useEditor((s) => s.doc);
  const hasDoc = doc !== null;
  const labels = useSetting("editor.toolbarLabels", true);
  const items = useToolbar((s) => s.data?.items ?? EMPTY);
  const bar = useRef<HTMLDivElement>(null);
  const level = useFit(
    bar,
    FIT_FILES + items.length,
    `${items.join(",")}|${labels}|${hasDoc}|${i18n.language}`,
  );
  const folded = level >= FIT_PREFS;
  const filesFolded = level >= FIT_FILES;
  const keys = {
    undo: useShortcut("editor.undo"),
    redo: useShortcut("editor.redo"),
    open: useShortcut("editor.open"),
    save: useShortcut("editor.save"),
    copyAll: useShortcut("editor.copy_all"),
  };

  return (
    <div
      ref={bar}
      className={`editor-toolbar${labels ? "" : " icons-only"}${fitClasses(level)}`}
      role="toolbar"
      aria-label={t("ui.editor.toolbar")}
      onKeyDown={arrowFocus("horizontal")}
      onContextMenu={(e) => {
        e.preventDefault();
        toolbarMenu(menuPoint(e));
      }}
      onClick={(e) => {
        // A mouse click leaves no focus on the button: a key pressed later would draw
        // its focus ring and an inactive tool would look selected (EDT-003). Enter and
        // Space (detail 0) keep it, and so do the drop-down menus.
        const button = (e.target as HTMLElement).closest("button");
        if (e.detail > 0 && button && !button.closest(".menu-anchor")) button.blur();
      }}
    >
      <ViewSwitch />
      <button type="button" className="btn primary tb-btn tb-capture" onClick={openCapturePanel}>
        <Icon name="capture" />
        <span className="tb-label">{t("ui.editor.capture")}</span>
      </button>
      <CustomBar hide={Math.max(0, level - FIT_FILES)} />
      <div className="tb-group">
        <button
          type="button"
          className="btn icon tb-undo"
          disabled={!doc?.canUndo}
          title={hint(
            doc?.undoLabel
              ? t("ui.editor.undo_named", { label: doc.undoLabel })
              : t("ui.editor.undo"),
            keys.undo,
          )}
          aria-label={t("ui.editor.undo")}
          onClick={undo}
        >
          <Icon name="undo" />
        </button>
        <button
          type="button"
          className="btn icon tb-redo"
          disabled={!doc?.canRedo}
          title={hint(
            doc?.redoLabel
              ? t("ui.editor.redo_named", { label: doc.redoLabel })
              : t("ui.editor.redo"),
            keys.redo,
          )}
          aria-label={t("ui.editor.redo")}
          onClick={redo}
        >
          <Icon name="redo" />
        </button>
      </div>
      <div className="tb-spacer" />
      {!filesFolded && (
        <div className="tb-group">
          <button
            type="button"
            className="btn tb-btn"
            title={hint(t("ui.editor.open"), keys.open)}
            onClick={() => void openFiles()}
          >
            <Icon name="open" />
            <span className="tb-label tb-wide">{t("ui.editor.open")}</span>
          </button>
          <button
            type="button"
            className="btn tb-btn"
            disabled={!hasDoc}
            title={hint(t("ui.editor.save"), keys.save)}
            onClick={() => void saveActive()}
          >
            <Icon name="save" />
            <span className="tb-label tb-wide">{t("ui.editor.save")}</span>
          </button>
        </div>
      )}
      <div className="tb-group share" role="group" aria-label={t("ui.editor.share_area")}>
        <button
          type="button"
          className="btn tb-btn"
          disabled={!hasDoc}
          title={hint(t("ui.editor.copy_all"), keys.copyAll)}
          onClick={() => void copyAll()}
        >
          <Icon name="copy" />
          <span className="tb-label tb-wide">{t("ui.editor.copy_all")}</span>
        </button>
        <Menu
          triggerClass="btn tb-btn"
          align="end"
          disabled={!hasDoc}
          title={t("ui.editor.share")}
          label={
            <>
              <Icon name="share" />
              <span className="tb-label tb-wide">{t("ui.editor.share")}</span>
            </>
          }
          items={[
            { id: "file", label: t("ui.editor.share_file"), onSelect: () => void exportImage() },
            {
              id: "clipboard",
              label: t("ui.editor.share_clipboard"),
              onSelect: () => void copyAll(),
            },
            {
              id: "save_as",
              label: t("ui.editor.save_as"),
              onSelect: () => void saveAs(),
              separator: true,
            },
          ]}
        />
      </div>
      {folded ? (
        <Menu
          triggerClass="btn icon tb-overflow"
          align="end"
          chevron={false}
          ariaLabel={t("ui.editor.bar_more")}
          title={t("ui.editor.bar_more")}
          label={<Icon name="more" />}
          items={[
            ...(filesFolded
              ? [
                  {
                    id: "open",
                    label: t("ui.editor.open"),
                    shortcut: keys.open ?? undefined,
                    onSelect: () => void openFiles(),
                  },
                  {
                    id: "save",
                    label: t("ui.editor.save"),
                    shortcut: keys.save ?? undefined,
                    disabled: !hasDoc,
                    onSelect: () => void saveActive(),
                  },
                ]
              : []),
            ...languageItems(t, i18n.language, filesFolded),
            {
              id: "preferences",
              label: t("ui.editor.preferences"),
              onSelect: () => void run("settings.open"),
            },
          ]}
        />
      ) : (
        <>
          <LanguageToggle />
          <button
            type="button"
            className="btn icon"
            title={t("ui.editor.preferences")}
            aria-label={t("ui.editor.preferences")}
            onClick={() => void run("settings.open")}
          >
            <Icon name="settings" />
          </button>
        </>
      )}
    </div>
  );
}

const EMPTY: string[] = [];

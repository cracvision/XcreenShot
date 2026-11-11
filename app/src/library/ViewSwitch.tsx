// Editor | Library switch at the top of the main window, always visible (LIB-001).
import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { useIndicator } from "../components/useIndicator";
import { Icon } from "../editor/icons";
import { hint, useShortcut } from "../editor/tools";
import { showView, useMainView, type MainView } from "./state";

const VIEWS: { id: MainView; icon: "editor" | "library" }[] = [
  { id: "editor", icon: "editor" },
  { id: "library", icon: "library" },
];

export function ViewSwitch() {
  const { t } = useTranslation();
  const view = useMainView((s) => s.view);
  const group = useRef<HTMLDivElement>(null);
  useIndicator(group, view);
  const keys = {
    editor: useShortcut("editor.view_editor"),
    library: useShortcut("editor.view_library"),
  };
  return (
    <div
      className="seg brand view-switch"
      role="group"
      aria-label={t("ui.library.view_switch")}
      ref={group}
    >
      <span className="seg-indicator" aria-hidden="true" />
      {VIEWS.map(({ id, icon }) => {
        const name = t(`menu.view_${id}`);
        return (
          <button
            key={id}
            type="button"
            aria-pressed={view === id}
            title={hint(name, keys[id])}
            onClick={() => showView(id)}
          >
            <Icon name={icon} />
            <span className="tb-label">{name}</span>
          </button>
        );
      })}
    </div>
  );
}

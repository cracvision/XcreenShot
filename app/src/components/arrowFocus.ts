// Arrow keys move the focus inside a group of buttons (APP-043): the editor toolbar
// (left and right), the section lists of Preferences and Help (up and down). Home and
// End go to the first and the last.
import type { KeyboardEvent } from "react";

const FOCUSABLE = "button:not(:disabled), [role='button']:not([aria-disabled='true'])";

/** onKeyDown of the group. */
export function arrowFocus(orientation: "horizontal" | "vertical") {
  const [back, forward] =
    orientation === "horizontal" ? ["ArrowLeft", "ArrowRight"] : ["ArrowUp", "ArrowDown"];
  return (e: KeyboardEvent<HTMLElement>) => {
    if (e.ctrlKey || e.altKey || e.metaKey) return;
    if (![back, forward, "Home", "End"].includes(e.key)) return;
    const items = Array.from(e.currentTarget.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
      (el) => el.offsetParent !== null,
    );
    const at = items.indexOf(document.activeElement as HTMLElement);
    if (at < 0) return;
    const next =
      e.key === "Home"
        ? 0
        : e.key === "End"
          ? items.length - 1
          : Math.min(items.length - 1, Math.max(0, at + (e.key === forward ? 1 : -1)));
    e.preventDefault();
    items[next]?.focus();
  };
}

/** Whether a key event happens in a toolbar, where the arrows move between buttons. */
export function inToolbar(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest('[role="toolbar"]') !== null;
}

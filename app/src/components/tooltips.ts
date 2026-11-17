// Tooltips (APP-014): the level the user chose in Preferences > General, and nothing at
// all when they are off.
import { useEffect } from "react";
import { settingValue, useApp } from "../store";

export type TooltipLevel = "off" | "brief" | "detailed";

/** The tooltip level the user chose in Preferences > General. */
export function useTooltipLevel(): TooltipLevel {
  return useApp((s) => {
    const v = settingValue(s.settings, "general.tooltips");
    return v === "off" || v === "brief" ? v : "detailed";
  });
}

/**
 * With tooltips off, the title of whatever the pointer reaches is taken away before
 * the system shows it. Screen readers keep the accessible names (APP-044).
 */
export function useTooltipsOff(): void {
  const level = useTooltipLevel();
  useEffect(() => {
    if (level !== "off") return;
    const strip = (e: Event) => {
      for (let el = e.target as Element | null; el; el = el.parentElement) {
        if (el.hasAttribute("title")) el.removeAttribute("title");
      }
    };
    window.addEventListener("pointerover", strip, true);
    return () => window.removeEventListener("pointerover", strip, true);
  }, [level]);
}

// In-app shortcuts (APP-017 "de la app" scope) are data from shortcuts.list: a key
// press that matches an app-scope row runs that row's command with its parameters.
import { useEffect } from "react";
import { useApp } from "../store";
import { fromEvent } from "./accelerator";
import { run } from "./commands";

export function useAppShortcuts(): void {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const { shortcuts, recording } = useApp.getState();
      if (recording || !shortcuts) return;
      const accel = fromEvent(e);
      if (!accel) return;
      const row = shortcuts.rows.find((r) => r.scope === "app" && r.shortcut === accel);
      if (!row) return;
      e.preventDefault();
      void run(row.runs, row.params, "shortcut");
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}

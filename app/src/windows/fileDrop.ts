// Files dropped on the capture panel or the edge launcher (CAP-005): images become
// captures (CAP-040), a PDF opens Import PDF (CAP-041), an Internet shortcut captures
// its web page (CAP-037) and an .xshot opens in the editor.
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useEffect, useState } from "react";
import { isCommandError, run } from "../lib/commands";
import { askImport } from "./presetActions";

/** Whether something is being dragged over the window; drops go to `capture.file`. */
export function useFileDrop(onError?: (message: string) => void, onOver?: () => void): boolean {
  const [over, setOver] = useState(false);
  useEffect(() => {
    let off: (() => void) | undefined;
    void getCurrentWebview()
      .onDragDropEvent((e) => {
        const kind = e.payload.type;
        if (kind === "enter" || kind === "over") {
          setOver(true);
          onOver?.();
        } else if (kind === "leave") setOver(false);
        else if (kind === "drop") {
          setOver(false);
          if (!e.payload.paths.length) return;
          // Presets (.xpreset) are imported (CAP-063); the rest become captures.
          const presets = e.payload.paths.filter((p) => p.toLowerCase().endsWith(".xpreset"));
          const files = e.payload.paths.filter((p) => !p.toLowerCase().endsWith(".xpreset"));
          if (presets.length) void askImport(presets);
          if (!files.length) return;
          void run("capture.file", { paths: files }).catch((err: unknown) => {
            if (isCommandError(err) && err.code === "cancelled") return;
            onError?.(isCommandError(err) ? err.message : String(err));
          });
        }
      })
      .then((f) => (off = f));
    return () => off?.();
  }, [onError, onOver]);
  return over;
}

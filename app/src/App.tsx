import { useTooltipsOff } from "./components/tooltips";
import { Overlay } from "./overlay/Overlay";
import { Editor } from "./editor/Editor";
import { CapturePanel } from "./windows/CapturePanel";
import { Help } from "./windows/Help";
import { Countdown } from "./windows/Countdown";
import { MultiCapture } from "./windows/MultiCapture";
import { Preferences } from "./windows/Preferences";
import { ScrollPanel } from "./windows/ScrollPanel";
import { WindowPicker } from "./windows/WindowPicker";
import { ClipboardPanel } from "./windows/ClipboardPanel";
import { TextResult } from "./windows/TextResult";
import { PdfImport } from "./windows/PdfImport";
import { Launcher } from "./windows/Launcher";
import { WebpagePrompt } from "./windows/WebpagePrompt";
import { AskText } from "./windows/AskText";
import { RecordBar, RecordFrame } from "./windows/RecordBar";
import { StepsBar, StepsNotice } from "./windows/StepsBar";
import { LiveDrawBar, LiveDrawOverlay } from "./windows/LiveDraw";
import { Working } from "./windows/Working";
import { Welcome } from "./windows/Welcome";
import { isOverlay, windowLabel } from "./windowKind";
import { useEffect } from "react";
import { run } from "./lib/commands";

/** Windows where Ctrl+/ shows the keyboard shortcuts sheet, in the main window (APP-023). */
const SHEET_FROM = ["preferences", "capture-panel", "help", "welcome", "multicapture"];

/** Ctrl+/ (Cmd+/) in the other windows: the shortcuts sheet, in the main window. */
function useShortcutSheetKey() {
  useEffect(() => {
    if (!SHEET_FROM.includes(windowLabel) && !windowLabel.startsWith("text-")) return;
    const onKey = (e: KeyboardEvent) => {
      if (!(e.ctrlKey || e.metaKey) || e.key !== "/") return;
      e.preventDefault();
      void run("app.show")
        .then(() => run("editor.action", { action: "shortcut_sheet" }))
        .catch(() => undefined);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}

/** Each webview window renders the view named by its label. */
export function App() {
  // Tooltips off (APP-014) holds in every window except the selection.
  useTooltipsOff();
  useShortcutSheetKey();
  if (isOverlay) return <Overlay />;
  // Extract text: one window per result (SMT-006).
  if (windowLabel.startsWith("text-")) return <TextResult id={windowLabel.slice(5)} />;
  // Import PDF: one window per file (CAP-041).
  if (windowLabel.startsWith("pdf-")) return <PdfImport id={windowLabel.slice(4)} />;
  switch (windowLabel) {
    case "preferences":
      return <Preferences />;
    case "capture-panel":
      return <CapturePanel />;
    case "countdown":
      return <Countdown />;
    case "multicapture":
      return <MultiCapture />;
    case "scroll":
      return <ScrollPanel />;
    case "picker":
      return <WindowPicker />;
    case "clipboard":
      return <ClipboardPanel />;
    case "launcher":
      return <Launcher />;
    case "webpage":
      return <WebpagePrompt />;
    case "ask-text":
      return <AskText />;
    case "stepsbar":
      return <StepsBar />;
    case "stepsnote":
      return <StepsNotice />;
    case "recbar":
      return <RecordBar />;
    case "recframe":
      return <RecordFrame />;
    case "livedraw":
      return <LiveDrawOverlay />;
    case "drawbar":
      return <LiveDrawBar />;
    case "working":
      return <Working />;
    case "help":
      return <Help />;
    case "welcome":
      return <Welcome />;
    default:
      return <Editor />;
  }
}

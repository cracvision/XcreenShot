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
import { TextResult } from "./windows/TextResult";
import { Welcome } from "./windows/Welcome";
import { isOverlay, windowLabel } from "./windowKind";

/** Each webview window renders the view named by its label. */
export function App() {
  // Tooltips off (APP-014) holds in every window except the selection.
  useTooltipsOff();
  if (isOverlay) return <Overlay />;
  // Extract text: one window per result (SMT-006).
  if (windowLabel.startsWith("text-")) return <TextResult id={windowLabel.slice(5)} />;
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
    case "help":
      return <Help />;
    case "welcome":
      return <Welcome />;
    default:
      return <Editor />;
  }
}

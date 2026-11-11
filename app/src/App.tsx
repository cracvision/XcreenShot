import { Overlay } from "./overlay/Overlay";
import { Editor } from "./editor/Editor";
import { CapturePanel } from "./windows/CapturePanel";
import { Help } from "./windows/Help";
import { Countdown } from "./windows/Countdown";
import { MultiCapture } from "./windows/MultiCapture";
import { Preferences } from "./windows/Preferences";
import { Welcome } from "./windows/Welcome";
import { isOverlay, windowLabel } from "./windowKind";

/** Each webview window renders the view named by its label. */
export function App() {
  if (isOverlay) return <Overlay />;
  switch (windowLabel) {
    case "preferences":
      return <Preferences />;
    case "capture-panel":
      return <CapturePanel />;
    case "countdown":
      return <Countdown />;
    case "multicapture":
      return <MultiCapture />;
    case "help":
      return <Help />;
    case "welcome":
      return <Welcome />;
    default:
      return <Editor />;
  }
}

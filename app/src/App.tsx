import { Overlay } from "./overlay/Overlay";
import { Editor } from "./editor/Editor";
import { CapturePanel } from "./windows/CapturePanel";
import { Countdown } from "./windows/Countdown";
import { MultiCapture } from "./windows/MultiCapture";
import { Preferences } from "./windows/Preferences";
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
    default:
      return <Editor />;
  }
}

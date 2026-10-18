import { Overlay } from "./overlay/Overlay";
import { CapturePanel } from "./windows/CapturePanel";
import { Countdown } from "./windows/Countdown";
import { Home } from "./windows/Home";
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
    default:
      return <Home />;
  }
}

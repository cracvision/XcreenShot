import { Overlay } from "./overlay/Overlay";
import { Home } from "./windows/Home";
import { Preferences } from "./windows/Preferences";
import { isOverlay, windowLabel } from "./windowKind";

/** Each webview window renders the view named by its label. */
export function App() {
  if (isOverlay) return <Overlay />;
  return windowLabel === "preferences" ? <Preferences /> : <Home />;
}

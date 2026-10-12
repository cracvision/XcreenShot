import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Home } from "./windows/Home";
import { Preferences } from "./windows/Preferences";

/** Each webview window renders the view named by its label. */
export function App() {
  return getCurrentWebviewWindow().label === "preferences" ? <Preferences /> : <Home />;
}

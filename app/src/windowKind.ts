import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

export const windowLabel = getCurrentWebviewWindow().label;

/** On-screen selection windows (CAP-013) are labelled `overlay-<monitor>`. */
export const isOverlay = windowLabel.startsWith("overlay-");

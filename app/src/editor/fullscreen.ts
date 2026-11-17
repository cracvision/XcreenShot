// Full screen (EDT-009): the main window on its monitor, without the recent tray;
// Esc leaves it before anything else it would do.
import { getCurrentWindow } from "@tauri-apps/api/window";
import { create } from "zustand";
import { useTray } from "../library/state";

export const useFullscreen = create<{ on: boolean; trayBefore: boolean }>(() => ({
  on: false,
  trayBefore: true,
}));

export async function setFullscreen(on: boolean): Promise<void> {
  const win = getCurrentWindow();
  await win.setFullscreen(on);
  if (on) {
    useFullscreen.setState({ on: true, trayBefore: useTray.getState().visible });
    if (useTray.getState().visible) useTray.getState().toggle();
  } else {
    const { trayBefore } = useFullscreen.getState();
    useFullscreen.setState({ on: false });
    if (trayBefore && !useTray.getState().visible) useTray.getState().toggle();
  }
}

export async function toggleFullscreen(): Promise<void> {
  try {
    await setFullscreen(!useFullscreen.getState().on);
  } catch {
    // The window could not change: it stays as it was.
  }
}

// Help > Move XcreenShot to another computer… (APP-055, LIB-032): what opens the
// assistant and what it says about where the library is.
import type { TFunction } from "i18next";
import { create } from "zustand";

export const useMoveComputer = create<{ open: boolean }>(() => ({ open: false }));

export function openMoveComputer(): void {
  useMoveComputer.setState({ open: true });
}

/** How the library gets to the other computer: copied by hand, or by its sync service. */
export function libraryStep(
  where: { place: string; service: string | null },
  t: TFunction,
): { synced: boolean; text: string } {
  if (where.place !== "synced") return { synced: false, text: t("ui.move_computer.library_copy") };
  return {
    synced: true,
    text: where.service
      ? t("ui.move_computer.library_synced", {
          service: t(`ui.library.service.${where.service}`),
        })
      : t("ui.move_computer.library_synced_generic"),
  };
}

/** The file name at the end of a path. */
export function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

// Remove background (SMT-020): of the image or of a floating image, then a notice that
// offers the other ways: Invert (keep the background, remove the subject) and As a
// floating image (the original stays, the subject goes on top). Each undoes the first
// and does it again that way.
import i18next from "i18next";
import { run } from "../lib/commands";
import { notify } from "../library/actions";
import { useEditor } from "./state";

const t = (key: string) => i18next.t(key);

interface Options {
  invert?: boolean;
  floating?: boolean;
}

/** Removes the background of the open image, or of the floating image `objectId`. */
export async function removeBackground(objectId?: string, options: Options = {}): Promise<boolean> {
  const doc = useEditor.getState().doc;
  if (!doc || doc.video) return false;
  const documentId = doc.documentId;
  try {
    await run("smart.remove_background", {
      documentId,
      ...(objectId ? { objectId } : {}),
      ...options,
    });
  } catch (e) {
    useEditor.getState().showError(e);
    return false;
  }
  // The other ways, offered after the first only.
  if (options.invert || options.floating) {
    notify(t("ui.remove_bg.removed"));
    return true;
  }
  const again = (o: Options) => async () => {
    try {
      await run("document.undo", { documentId });
    } catch (e) {
      useEditor.getState().showError(e);
      return;
    }
    await removeBackground(objectId, o);
  };
  notify(
    t("ui.remove_bg.removed"),
    { label: t("ui.remove_bg.invert"), run: again({ invert: true }) },
    [{ label: t("ui.remove_bg.floating"), run: again({ floating: true }) }],
  );
  return true;
}

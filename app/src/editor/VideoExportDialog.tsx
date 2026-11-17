// File > Export of a video (VID-066): MP4 with the same codec, MP4 H.264 (when the
// video has another codec, such as H.265) or an animated GIF (VID-068).
import { save } from "@tauri-apps/plugin-dialog";
import { useTranslation } from "react-i18next";
import { Dialog } from "../components/Dialog";
import { run } from "../lib/commands";
import { fail, saveAs } from "./actions";
import { useEditor } from "./state";
import { useVideoExport, videoAction } from "./videoState";

/** Asks where, then writes the H.264 copy; true once it is written. */
async function exportH264(): Promise<boolean> {
  const { active, doc } = useEditor.getState();
  if (!active || !doc) return false;
  const path = await save({
    defaultPath: `${doc.name} (H.264).mp4`,
    filters: [{ name: "MP4", extensions: ["mp4"] }],
  });
  if (!path) return false;
  try {
    await run("video.export", { documentId: active, path });
    return true;
  } catch (e) {
    fail(e);
    return false;
  }
}

export function VideoExportDialog() {
  const { t } = useTranslation();
  const open = useVideoExport((s) => s.open);
  const codec = useEditor((s) => s.doc?.video?.codec ?? "H.264");
  if (!open) return null;
  const close = () => useVideoExport.setState({ open: false });
  const choose = (f: () => unknown) => () => {
    close();
    void f();
  };
  const option = (label: string, help: string, onClick: () => void) => (
    <button type="button" className="video-export-option" onClick={onClick}>
      <strong>{label}</strong>
      <span className="muted">{help}</span>
    </button>
  );
  return (
    <Dialog
      title={t("ui.video.export_title")}
      onCancel={close}
      actions={
        <button type="button" className="btn" onClick={close}>
          {t("ui.editor.cancel")}
        </button>
      }
    >
      <div className="video-export">
        {option(t("ui.video.export_mp4"), t("ui.video.export_mp4_help", { codec }), choose(saveAs))}
        {codec !== "H.264" &&
          option(
            t("ui.video.export_h264"),
            t("ui.video.export_h264_help"),
            choose(async () => {
              if (await exportH264()) useEditor.getState().showToast(t("ui.video.exported"));
            }),
          )}
        {option(
          t("ui.video.export_gif"),
          t("ui.video.export_gif_help"),
          choose(() => videoAction("video_gif")),
        )}
      </div>
    </Dialog>
  );
}

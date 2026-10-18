import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { display, fromEvent, releasedCombination, type LayoutMap } from "../lib/accelerator";
import { run } from "../lib/commands";
import { useApp } from "../store";

interface Props {
  value: string | null;
  platform: "windows" | "macos";
  layout: LayoutMap | undefined;
  conflict: boolean;
  label: string;
  /** null removes the shortcut. */
  onCommit: (shortcut: string | null) => void;
}

/**
 * Records a combination (APP-018): click or Enter starts, the combination is taken
 * when the keys are released, Esc cancels and Backspace/Delete remove it. Global
 * shortcuts are paused while recording so they can be recorded.
 */
export function ShortcutRecorder({ value, platform, layout, conflict, label, onCommit }: Props) {
  const { t } = useTranslation();
  const setRecordingFlag = useApp((s) => s.setRecording);
  const [recording, setRecording] = useState(false);
  const [preview, setPreview] = useState<string | null>(null);
  const candidate = useRef<string | null>(null);
  const commit = useRef(onCommit);
  useEffect(() => {
    commit.current = onCommit;
  });

  useEffect(() => {
    if (!recording) return;
    // Only real start/stop transitions reach the core; mounting never does.
    setRecordingFlag(true);
    void run("shortcuts.recording", { active: true });
    candidate.current = null;
    const stop = () => {
      setRecording(false);
      setPreview(null);
    };
    const onDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const plain = !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey;
      if (plain && e.key === "Escape") return stop();
      if (plain && (e.key === "Backspace" || e.key === "Delete")) {
        stop();
        return commit.current(null);
      }
      const accel = fromEvent(e);
      candidate.current = accel;
      setPreview(accel);
    };
    const onUp = (e: KeyboardEvent) => {
      e.preventDefault();
      const accel = releasedCombination(candidate.current, e);
      if (accel) {
        stop();
        commit.current(accel);
      }
    };
    window.addEventListener("keydown", onDown, true);
    window.addEventListener("keyup", onUp, true);
    window.addEventListener("blur", stop);
    return () => {
      window.removeEventListener("keydown", onDown, true);
      window.removeEventListener("keyup", onUp, true);
      window.removeEventListener("blur", stop);
      setRecordingFlag(false);
      void run("shortcuts.recording", { active: false });
    };
  }, [recording, setRecordingFlag]);

  const text = recording
    ? preview
      ? display(preview, platform, layout)
      : t("ui.shortcuts.record_prompt")
    : value
      ? display(value, platform, layout)
      : t("ui.shortcuts.none");

  return (
    <button
      type="button"
      className={`recorder${recording ? " recording" : ""}${conflict ? " conflict" : ""}`}
      aria-label={`${label}: ${text}`}
      aria-describedby={recording ? "recorder-hint" : undefined}
      onClick={() => setRecording(true)}
    >
      {recording ? text : <kbd>{text}</kbd>}
      {recording && (
        <span id="recorder-hint" className="muted" style={{ display: "block", fontSize: 12 }}>
          {t("ui.shortcuts.record_hint")}
        </span>
      )}
    </button>
  );
}

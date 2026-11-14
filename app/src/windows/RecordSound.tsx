// The sound controls of the recording bar (VID-011): the system's sound and the
// microphone, each a switch with its meter; the microphone's menu (VID-039) and the
// warnings under the bar (VID-019, VID-043).
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Menu, type MenuItem } from "../components/Menu";
import { run } from "../lib/commands";
import { useSetting, useStringSetting } from "../store";
import { meterFill, warningActions, type BarWarning, type SoundLevels } from "./recSound";

/** A level meter (VID-011): it fills as the sound gets louder and turns red at the top. */
export function LevelMeter({
  level,
  clip,
  label,
}: {
  level: number;
  clip: boolean;
  label: string;
}) {
  const { t } = useTranslation();
  const fill = meterFill(level);
  return (
    <span
      className={`recbar-meter${clip ? " clip" : ""}`}
      role="meter"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(fill * 100)}
      aria-valuetext={clip ? t("ui.recbar.clip") : undefined}
      title={clip ? t("ui.recbar.clip") : label}
    >
      <span style={{ transform: `scaleX(${fill})` }} />
    </span>
  );
}

const sound = (p: Record<string, unknown>) => void run("record.sound", p).catch(() => undefined);

/** The system's sound (VID-040): a switch with its meter, also while recording when the
 * video has a sound track. */
export function SystemAudioControl({ levels }: { levels: SoundLevels | null }) {
  const { t } = useTranslation();
  const setting = useSetting("video.systemAudio", false);
  const s = levels?.systemAudio;
  const on = s?.on ?? setting;
  const locked = !on && levels?.track === false;
  return (
    <span className="recbar-split">
      <button
        type="button"
        className="btn small recbar-toggle"
        aria-pressed={on}
        disabled={locked}
        title={locked ? t("ui.recbar.no_track") : t("ui.recbar.sound")}
        onClick={() => sound({ systemAudio: !on })}
      >
        {t("ui.recbar.sound")}
      </button>
      {on && s && <LevelMeter level={s.level} clip={s.clip} label={t("ui.recbar.system_meter")} />}
    </span>
  );
}

interface Input {
  id: string;
  name: string;
  wireless: boolean;
}

/** The microphone (VID-039): a switch, its meter, and its menu: the device (in Ready or
 * paused), the automatic level and the system's sound settings. */
export function MicControl({
  levels,
  up,
  onMenu,
}: {
  levels: SoundLevels | null;
  /** The menu opens upwards: the bar sits low on its monitor. */
  up: boolean;
  /** The menu appears with its number of entries, or goes (0). */
  onMenu: (rows: number) => void;
}) {
  const { t } = useTranslation();
  const setting = useSetting("video.microphone", true);
  const chosen = useStringSetting("video.microphoneDevice", "");
  const auto = useSetting("video.microphoneAutoLevel", true);
  const denoise = useSetting("video.noiseReduction", false);
  const [inputs, setInputs] = useState<{ inputs: Input[]; default: string | null } | null>(null);
  const [asked, setAsked] = useState(0);
  // The inputs, again each time the menu opens (VID-043: what is plugged in now).
  useEffect(() => {
    let alive = true;
    void run<{ inputs: Input[]; default: string | null }>("record.devices")
      .then((r) => {
        if (alive) setInputs(r);
      })
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [asked]);
  const mic = levels?.microphone;
  const on = mic?.on ?? setting;
  const recording = levels?.status === "recording";
  const locked = !on && levels?.track === false;
  const deviceLocked = recording ? t("ui.recbar.mic_device_locked") : undefined;
  const defaultName = inputs?.inputs.find((i) => i.id === inputs.default)?.name;
  const items: MenuItem[] = [
    {
      id: "default",
      label: defaultName
        ? t("ui.recbar.mic_default_named", { name: defaultName })
        : t("ui.recbar.mic_default"),
      checked: chosen === "",
      disabled: recording,
      ...(deviceLocked ? { title: deviceLocked } : {}),
      onSelect: () => sound({ microphone: true, microphoneDevice: "" }),
    },
    ...(inputs?.inputs ?? []).map((i): MenuItem => ({
      id: i.id,
      label: i.name,
      checked: chosen === i.id,
      disabled: recording,
      ...(deviceLocked ? { title: deviceLocked } : {}),
      ...(i.wireless
        ? { trailing: <span className="muted">{t("ui.recbar.mic_wireless")}</span> }
        : {}),
      onSelect: () => sound({ microphone: true, microphoneDevice: i.id }),
    })),
    {
      id: "denoise",
      label: t("ui.recbar.mic_denoise"),
      checked: denoise,
      separator: true,
      onSelect: () => sound({ noiseReduction: !denoise }),
    },
    {
      id: "auto",
      label: t("ui.recbar.mic_auto"),
      checked: auto,
      onSelect: () => sound({ microphoneAutoLevel: !auto }),
    },
    {
      id: "settings",
      label: t("ui.recbar.sound_settings"),
      separator: true,
      onSelect: () => void run("record.sound_settings").catch(() => undefined),
    },
  ];
  return (
    <span className="recbar-split">
      <button
        type="button"
        className="btn small recbar-toggle"
        aria-pressed={on}
        disabled={locked}
        title={
          locked
            ? t("ui.recbar.no_track")
            : mic?.device
              ? `${t("ui.recbar.mic")}: ${mic.device}`
              : t("ui.recbar.mic")
        }
        onClick={() => sound({ microphone: !on })}
      >
        {t("ui.recbar.mic")}
      </button>
      {on && mic && (
        <LevelMeter level={mic.level} clip={mic.clip} label={t("ui.recbar.mic_meter")} />
      )}
      <Menu
        label={null}
        items={items}
        align="end"
        side={up ? "top" : "bottom"}
        triggerClass="btn small icon recbar-chev"
        ariaLabel={t("ui.recbar.mic_options")}
        title={t("ui.recbar.mic_options")}
        onShown={(shown) => {
          if (shown) setAsked((n) => n + 1);
          onMenu(shown ? items.length : 0);
        }}
      />
    </span>
  );
}

/** A warning under the bar (VID-019, VID-038, VID-043, VID-077), with its ways out;
 * Esc or × closes it. */
export function SoundStrip({ warning, onDismiss }: { warning: BarWarning; onDismiss: () => void }) {
  const { t } = useTranslation();
  const esc = (e: React.KeyboardEvent) => {
    if (e.key !== "Escape") return;
    e.preventDefault();
    e.stopPropagation();
    onDismiss();
  };
  return (
    <div className="recbar-strip" role="alert">
      <span className="recbar-strip-dot" aria-hidden="true" />
      <span className="recbar-strip-text">{t(`ui.recbar.${warning}`)}</span>
      {warningActions(warning).map((a) => (
        <button
          key={a.label}
          type="button"
          className="btn small"
          onKeyDown={esc}
          onClick={() => void run(a.command, a.params ?? {}).catch(() => undefined)}
        >
          {t(`ui.recbar.${a.label}`)}
        </button>
      ))}
      <button
        type="button"
        className="btn icon small"
        aria-label={t("ui.recbar.dismiss")}
        title={t("ui.recbar.dismiss")}
        onKeyDown={esc}
        onClick={onDismiss}
      >
        ×
      </button>
    </div>
  );
}

// The microphone to record (VID-039): the system's default or one of the inputs that
// are plugged in; a chosen one that is not plugged in stays listed as such (VID-043).
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { run } from "../lib/commands";

interface Input {
  id: string;
  name: string;
  wireless: boolean;
}

export function MicDeviceSelect({
  id,
  value,
  disabled,
  describedBy,
  onChange,
}: {
  id: string;
  /** The input's id; empty: the system's default. */
  value: string;
  disabled?: boolean;
  describedBy?: string;
  onChange: (id: string) => void;
}) {
  const { t } = useTranslation();
  const [inputs, setInputs] = useState<Input[] | null>(null);
  useEffect(() => {
    let alive = true;
    const load = () =>
      void run<{ inputs: Input[] }>("record.devices")
        .then((r) => {
          if (alive) setInputs(r.inputs);
        })
        .catch(() => undefined);
    load();
    // What is plugged in changes (VID-043): looked at again when the window comes back.
    window.addEventListener("focus", load);
    return () => {
      alive = false;
      window.removeEventListener("focus", load);
    };
  }, []);
  const missing = value !== "" && inputs !== null && !inputs.some((i) => i.id === value);
  return (
    <select
      id={id}
      value={value}
      disabled={disabled}
      aria-describedby={describedBy}
      onChange={(e) => onChange(e.target.value)}
    >
      <option value="">{t("ui.recbar.mic_default")}</option>
      {(inputs ?? []).map((i) => (
        <option key={i.id} value={i.id}>
          {i.wireless ? `${i.name} — ${t("ui.recbar.mic_wireless")}` : i.name}
        </option>
      ))}
      {missing && <option value={value}>{t("ui.prefs.mic_not_connected")}</option>}
    </select>
  );
}

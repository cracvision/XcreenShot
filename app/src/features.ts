// Smart features switched on or off (SMT-002): a feature off loses its buttons, menu
// entries and capture modes. The core keeps the same list (`xs_core::features`) and
// refuses its commands.
import type { SettingEntry } from "./lib/commands";
import { settingValue, useApp } from "./store";

export type FeatureName =
  | "extractText"
  | "autoRedact"
  | "steps"
  | "textOnly"
  | "autoSimplify"
  | "editText"
  | "separate"
  | "qr";

export const FEATURE_SETTINGS: Record<FeatureName, string> = {
  extractText: "smart.extractText.enabled",
  autoRedact: "smart.autoRedact.enabled",
  steps: "smart.steps.enabled",
  textOnly: "smart.textOnly.enabled",
  autoSimplify: "smart.autoSimplify.enabled",
  editText: "smart.editText.enabled",
  separate: "smart.separate.enabled",
  qr: "smart.qr.enabled",
};

/** The editor actions of each feature. */
const ACTIONS: Record<string, FeatureName> = {
  extract_text: "extractText",
  video_extract_text: "extractText",
  auto_redact: "autoRedact",
  edit_text: "editText",
  insert_qr: "qr",
};

/** On unless switched off; on too before the settings arrive. */
export function featureOn(settings: SettingEntry[], name: FeatureName): boolean {
  return settingValue(settings, FEATURE_SETTINGS[name]) !== false;
}

/** As it is now, outside React. */
export function isOn(name: FeatureName): boolean {
  return featureOn(useApp.getState().settings, name);
}

export function useFeature(name: FeatureName): boolean {
  return useApp((s) => featureOn(s.settings, name));
}

/** An editor action whose feature is switched off. */
export function actionOff(action: string): boolean {
  const name = ACTIONS[action];
  return name !== undefined && !isOn(name);
}

/** The capture modes of a feature switched off leave the list, unless chosen. */
export function captureModes<T extends string>(
  modes: readonly T[],
  settings: SettingEntry[],
  current: string,
): T[] {
  return modes.filter(
    (m) =>
      m === current ||
      ((m !== "text" || featureOn(settings, "extractText")) &&
        (m !== "steps" || featureOn(settings, "steps"))),
  );
}

import { invoke, isTauriRuntime } from "./runtime";

export type HistoryRetention = "one_month" | "three_months" | "six_months" | "twelve_months" | "always";
export interface PrivacySettings {
  history_retention: HistoryRetention;
  allow_popups: boolean;
  smart_screen_supported: boolean;
  smart_screen_enabled: boolean;
}

const defaults: PrivacySettings = {
  history_retention: "three_months",
  allow_popups: false,
  smart_screen_supported: false,
  smart_screen_enabled: true,
};

export async function fetchPrivacySettings(): Promise<PrivacySettings> {
  return isTauriRuntime() ? invoke<PrivacySettings>("get_privacy_settings") : defaults;
}

export async function setHistoryRetention(retention: HistoryRetention): Promise<void> {
  if (isTauriRuntime()) await invoke("set_history_retention", { retention });
}

export async function setPopupSetting(enabled: boolean): Promise<void> {
  if (isTauriRuntime()) await invoke("set_popup_setting", { enabled });
}

export async function setSmartScreen(enabled: boolean): Promise<void> {
  if (isTauriRuntime()) await invoke("set_smart_screen", { enabled });
}

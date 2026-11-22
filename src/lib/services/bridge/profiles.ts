import { invoke } from "./runtime";
import type { PickerState, ProfileInfo, ProfileList } from "../../types/profiles";

export const listProfiles = () => invoke<ProfileList>("list_profiles");
export const createProfile = (name: string) => invoke<ProfileInfo>("create_profile", { name });
export const renameProfile = (id: string, name: string) => invoke<ProfileInfo>("rename_profile", { id, name });
export const deleteProfile = (id: string) => invoke<void>("delete_profile", { id });
/** Opens the profile in its own window: the running one when it is open, a new process otherwise. */
export const openProfile = (id: string) => invoke<void>("open_profile", { id });

/** What this process is for: the browser, or only the profile picker shown when Spark starts. */
export const getLaunchState = () => invoke<{ mode: "browser" | "picker"; locked: boolean }>("get_launch_state");
export const getPickerState = () => invoke<PickerState>("get_picker_state");
/** Opens the chosen profile in its own process, remembers the choice, and ends the picker. */
export const chooseProfile = (id: string, askEveryTime: boolean) => invoke<void>("choose_profile", { id, askEveryTime });
export const getLauncherPrefs = () => invoke<{ ask_every_time: boolean }>("get_launcher_prefs");
export const setAskProfileAtStartup = (enabled: boolean) => invoke<void>("set_ask_profile_at_startup", { enabled });

import { invoke } from "./runtime";
import type { ProfileInfo, ProfileList } from "../../types/profiles";

export const listProfiles = () => invoke<ProfileList>("list_profiles");
export const createProfile = (name: string) => invoke<ProfileInfo>("create_profile", { name });
export const renameProfile = (id: string, name: string) => invoke<ProfileInfo>("rename_profile", { id, name });
export const deleteProfile = (id: string) => invoke<void>("delete_profile", { id });
/** Opens the profile in its own window: the running one when it is open, a new process otherwise. */
export const openProfile = (id: string) => invoke<void>("open_profile", { id });

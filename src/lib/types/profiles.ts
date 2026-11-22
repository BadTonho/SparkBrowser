export interface ProfileInfo {
  id: string;
  /** Empty for the Default profile: the interface translates "Default". */
  name: string;
  is_default: boolean;
  created_at: number;
}

export interface ProfileList {
  current: string;
  items: ProfileInfo[];
  max: number;
}

/** What the profile picker shows when Spark starts. */
export interface PickerState {
  profiles: ProfileInfo[];
  ask_every_time: boolean;
  last_profile: string | null;
}

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

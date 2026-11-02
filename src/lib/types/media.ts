export interface TabMediaState {
  tab_id: string;
  muted: boolean;
  audible: boolean;
  mute_available: boolean;
}
export interface MediaSnapshot { revision: number; tabs: TabMediaState[] }

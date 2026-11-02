export interface TabMediaState {
  tab_id: string;
  muted: boolean;
  audible: boolean;
  mute_available: boolean;
  volume: number;
  applied_volume: number;
  volume_status: "ready" | "limited" | "pending" | "unavailable";
  document_key: string;
  frames: {context: string; document: string; count: number; applied_volume: number; status: string}[];
}
export interface MediaSnapshot { revision: number; tabs: TabMediaState[] }

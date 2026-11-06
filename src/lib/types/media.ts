export interface MediaCapabilities { mute: boolean; volume: boolean; picture_in_picture: boolean }
export interface MediaVideoCandidate { id:string; label:string; width:number; height:number; eligible:boolean }
export interface FrameMedia {context:string;document:string;count:number;applied_volume:number;status:string;videos:MediaVideoCandidate[];playing:boolean;picture_in_picture:boolean;pip_video:string|null;pip_mode?:"custom"|"native"|null;pip_session?:string|null}
export interface TabMediaState {
  tab_id: string;
  muted: boolean;
  audible: boolean;
  mute_available: boolean;
  volume: number;
  applied_volume: number;
  volume_status: "ready" | "limited" | "pending" | "unavailable";
  document_key: string;
  frames: FrameMedia[];
  capabilities: MediaCapabilities;
}
export interface MediaSnapshot { revision: number; tabs: TabMediaState[]; frame_tracking_supported?: boolean }

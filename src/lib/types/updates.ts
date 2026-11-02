export type UpdateChannel = "stable" | "beta";
export interface UpdateInfo {
  version: string;
  tag: string;
  notes: string;
  published_at: string | null;
}
export interface UpdateState {
  revision: number;
  supported: boolean;
  message: string;
  current_version: string;
  channel: UpdateChannel;
  last_checked: number | null;
  status: "idle" | "unavailable" | "checking" | "available" | "up_to_date" | "downloading" | "ready" | "installing" | "error";
  update: UpdateInfo | null;
  received: number;
  total: number | null;
  error: string | null;
  retry_after: number | null;
}

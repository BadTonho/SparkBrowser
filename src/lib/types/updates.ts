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
  /** Download in the background and install at the next start without asking. */
  auto_update: boolean;
  /** The downloaded update is saved on disk and installs the next time Spark opens. */
  staged: boolean;
  update: UpdateInfo | null;
  received: number;
  total: number | null;
  error: string | null;
  retry_after: number | null;
}

export interface SavedSession { id: number; name: string; pages: { title: string; url: string; is_pinned: boolean; is_selected: boolean }[]; }

export interface SiteSearchShortcut { keyword: string; name: string; template: string; }

export interface PageNote { url: string; title: string; text: string; updated_at: number; }

export interface SplitLayout { workspace_id: string; left_tab_id: string; right_tab_id: string; ratio: number; focused_tab_id: string; }

export interface SitePermission { origin: string; kind: "camera" | "microphone" | "location" | "notifications"; state: "ask" | "allow" | "block"; }
export interface PermissionRequest { id: string; tab_id: string; origin: string; top_origin: string; frame_origin: string | null; kind: SitePermission["kind"]; }
export interface PermissionSnapshot { supported: boolean; message: string; settings: SitePermission[]; requests: PermissionRequest[]; }

export interface ViewportBounds {
  split_ratio?: number;
  split_key?: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface QuickLink {
  id: string;
  title: string;
  url: string;
  icon: string;
  accent: string;
}

export interface WorkspaceItem {
  id: string;
  name: string;
  icon: string;
  accent_primary: string;
  accent_secondary: string;
}

export interface WorkspaceDeleteResult {
  workspaces: WorkspaceItem[];
  tabs: TabSnapshot;
  active_workspace_id: string;
}

export interface TabItem {
  id: string;
  title: string;
  url: string;
  workspace_id: string;
  is_active: boolean;
  is_visible?: boolean;
  is_hibernated: boolean;
  last_active_epoch?: number;
  navigation?: NavigationState;
  is_pinned?: boolean;
  activity?: { dirty_form: boolean; playing_media: boolean; downloading: boolean };
}

export interface HistoryEntry {
  id: number;
  title: string;
  url: string;
  visited_at: number;
}

export interface BookmarkItem {
  id: number;
  title: string;
  url: string;
  icon: string;
  folder_id?: number | null;
}

export interface BookmarkFolder {
  id: number;
  name: string;
}

export interface BookmarkData {
  bookmarks: BookmarkItem[];
  folders: BookmarkFolder[];
}

export interface DownloadItem {
  id: number;
  file_name: string;
  url: string;
  save_path: string;
  status: string;
  created_at: number;
}

export interface DownloadTask {
  id: string; revision: number; file_name: string; url: string; source: "url" | "native";
  source_page_url: string | null;
  status: "Queued" | "Downloading" | "Pausing" | "Paused" | "Interrupted" | "RestartRequired" | "Verifying" | "VerificationFailed" | "Finalizing" | "Cancelling" | "Completed" | "Cancelled" | "Failed";
  received: number; total: number | null; error: string | null; cancellable: boolean;
  can_pause: boolean; can_resume: boolean; can_restart: boolean; can_verify: boolean;
  save_path: string | null; expected_sha256: string | null; actual_sha256: string | null; verification_received: number;
}

export interface PrivacyStats {
  shield_enabled: boolean;
  blocked_count: number;
  hibernated_tabs_count: number;
  supports_live_policy_updates: boolean;
}

export interface CommandItem {
  id: string;
  title: string;
  subtitle: string;
  icon: string;
  badge: string;
  action: () => void | Promise<void>;
}

/** Browser preview uses arrays; the native runtime sends revisioned snapshots. */
export type TabSnapshot = TabItem[] | { revision: number; tabs: TabItem[]; split_layout?: SplitLayout | null };

export interface NavigationState {
  revision: number;
  loading: boolean;
  error: string | null;
  can_go_back: boolean | null;
  can_go_forward: boolean | null;
}
export interface NavigationPatch { activity?: TabItem["activity"]; id: string; url: string; title: string; navigation: NavigationState; }

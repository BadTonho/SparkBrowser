export interface ViewportBounds {
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
  is_hibernated: boolean;
  last_active_epoch?: number;
  navigation?: NavigationState;
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

export interface PrivacyStats {
  shield_enabled: boolean;
  blocked_count: number;
  hibernated_tabs_count: number;
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
export type TabSnapshot = TabItem[] | { revision: number; tabs: TabItem[] };

export interface NavigationState {
  revision: number;
  loading: boolean;
  error: string | null;
  can_go_back: boolean | null;
  can_go_forward: boolean | null;
}
export interface NavigationPatch { id: string; url: string; title: string; navigation: NavigationState; }

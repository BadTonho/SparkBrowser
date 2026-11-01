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
  tabs: TabItem[];
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

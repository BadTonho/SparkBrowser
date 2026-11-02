import type { TabSnapshot } from "./browser";
export interface ClosedTab { id: number; url: string; title: string; workspace_id: string; workspace_name: string; position: number; is_pinned: boolean; closed_at: number; }
export interface ReopenClosedTabResult { snapshot: TabSnapshot; workspace_missing: boolean; }

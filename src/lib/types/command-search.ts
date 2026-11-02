export interface CommandSearchResult {
  notes: { url: string; title: string; preview: string; characters: number }[];
  sessions: { id: number; name: string; preview: string; page_count: number }[];
  notes_count: number; sessions_count: number; notes_has_more: boolean; sessions_has_more: boolean;
}

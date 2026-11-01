import { invoke } from "./runtime";
import type { TabSnapshot } from "../../types/browser";
export interface BulkCloseResult { snapshot: TabSnapshot; closed: string[]; preserved: { id: string; reason: string }[]; }
export const closeTabsReviewed = (tabIds: string[]) => invoke<BulkCloseResult>("close_tabs_reviewed", { tabIds });

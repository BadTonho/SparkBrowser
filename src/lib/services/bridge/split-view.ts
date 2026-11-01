import { invoke } from "./runtime";
import type { TabSnapshot } from "../../types/browser";
export const setSplitView = (leftTabId: string | null, rightTabId: string | null) => invoke<TabSnapshot>("set_split_view", { leftTabId, rightTabId });
export const commitSplitRatio = (key: string, ratio: number) => invoke<TabSnapshot>("commit_split_ratio", { key, ratio });

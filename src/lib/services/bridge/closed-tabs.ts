import { invoke, isTauriRuntime } from "./runtime";
import { listen } from "@tauri-apps/api/event";
import type { ClosedTab, ReopenClosedTabResult } from "../../types/closed-tabs";
export const fetchClosedTabs = () => invoke<ClosedTab[]>("get_closed_tabs");
export const reopenClosedTab = (id: number | null) => invoke<ReopenClosedTabResult | null>("reopen_closed_tab", { id });
export const clearClosedTabs = () => invoke<void>("clear_closed_tabs");
export const onClosedTabsUpdated = (callback: (rows: ClosedTab[]) => void) => isTauriRuntime() ? listen<ClosedTab[]>("closed-tabs-updated", event => callback(event.payload)) : Promise.resolve(() => {});

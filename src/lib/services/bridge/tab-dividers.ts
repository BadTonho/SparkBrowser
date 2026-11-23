import { invoke } from "./runtime";
import { listen } from "@tauri-apps/api/event";
import type { DividerEdit, TabDivider } from "../../stores/tabDividersStore";
export const fetchTabDividers = () => invoke<TabDivider[]>("get_tab_dividers");
export const editTabDivider = (edit: DividerEdit) => invoke<TabDivider[]>("set_tab_dividers", { edit });
export const onTabDividersUpdated = (refresh: () => void) => listen("tab-dividers-updated", refresh);

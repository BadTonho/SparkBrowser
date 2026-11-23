import { invoke } from "./runtime";
import type { TabGroups } from "../../stores/tabGroupsStore";
export const fetchTabGroups = () => invoke<TabGroups>("get_tab_groups");
export const saveTabGroups = (groups: TabGroups) => invoke<void>("set_tab_groups", { groups });

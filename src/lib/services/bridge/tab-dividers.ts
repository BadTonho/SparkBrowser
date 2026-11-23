import { invoke } from "./runtime";
import type { TabDivider } from "../../stores/tabDividersStore";
export const fetchTabDividers = () => invoke<TabDivider[]>("get_tab_dividers");
export const saveTabDividers = (dividers: TabDivider[]) => invoke<void>("set_tab_dividers", { dividers });

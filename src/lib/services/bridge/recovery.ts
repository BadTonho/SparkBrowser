import { invoke } from "./runtime";
import type { RecoveryStatus } from "../../types/recovery";
export const fetchRecoveryStatus = () => invoke<RecoveryStatus>("get_recovery_status");

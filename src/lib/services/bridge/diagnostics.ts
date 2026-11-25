export type ProtectionReason = "selection" | "split" | "pinned" | "loading" | "form" | "media" | "picture_in_picture" | "download";
export interface TabDiagnostic { id: string; title: string; url: string; workspace_id: string; resident: boolean; seconds_since_activation: number; protection_reasons: ProtectionReason[] }
import { invoke, isTauriRuntime } from "./runtime";
export interface NativeDiagnostics {
  application_identifier: string;
  metrics: {
    samples: { operation: string; duration_ms: number; affected_rows: number; success: boolean }[];
    totals: Record<string, { calls: number; failures: number; affected_rows: number }>;
  };
  tabs: number;
  resident_webviews: number;
  protected_tabs: number;
  hibernated_tabs: number;
  resident_limit: number;
  tab_details: TabDiagnostic[];
}
export interface RecentErrorEntry { time: string; level: "INFO" | "WARN" | "ERROR" | "CRITICAL"; text: string }
export interface RecentErrors { path: string; entries: RecentErrorEntry[] }
/** The latest entries of the local error log, newest first. */
export async function fetchRecentErrors(): Promise<RecentErrors | null> {
  return isTauriRuntime() ? invoke<RecentErrors>("get_recent_errors") : null;
}
export async function fetchDiagnostics(): Promise<NativeDiagnostics | null> {
  return isTauriRuntime() ? invoke<NativeDiagnostics>("get_diagnostics") : null;
}

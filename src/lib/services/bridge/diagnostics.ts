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
export async function fetchDiagnostics(): Promise<NativeDiagnostics | null> {
  return isTauriRuntime() ? invoke<NativeDiagnostics>("get_diagnostics") : null;
}

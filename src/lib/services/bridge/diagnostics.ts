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
}
export async function fetchDiagnostics(): Promise<NativeDiagnostics | null> {
  return isTauriRuntime() ? invoke<NativeDiagnostics>("get_diagnostics") : null;
}

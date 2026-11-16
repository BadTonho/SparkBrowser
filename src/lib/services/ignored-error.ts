import { invoke, isTauriRuntime } from "./bridge/runtime";

/**
 * Use instead of an empty `.catch(() => {})`: the failure is still ignored, but it is written to
 * the error log as a warning, so nothing fails invisibly. Warnings never show a notification.
 */
export function logIgnored(error: unknown): void {
  if (!isTauriRuntime()) return;
  const failure = error as { message?: string; stack?: string } | undefined;
  void invoke("report_frontend_error", {
    summary: `Ignored failure: ${failure?.message ?? String(error)}`,
    details: failure?.stack ?? "",
    level: "warning",
  }).catch(() => {});
}

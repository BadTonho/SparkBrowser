

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
import { invoke as nativeInvoke, type InvokeArgs } from "@tauri-apps/api/core";
import { ipcDiagnostics } from "../diagnostics.js";
import { UserError } from "../user-errors.js";

// The same failure is written to the log once every ten seconds (the log keeps the same window), so a
// command that is retried in a loop does not turn into a flood of log calls.
const lastFailureLogged = new Map<string, number>();

/** Commands that answer before start-up is ready (the first screen needs them), so they never wait for it. */
const ANSWERS_BEFORE_READY = new Set([
  "get_diagnostics", "get_launch_state", "get_picker_state", "choose_profile",
  "get_lock_state", "app_unlock_password", "app_unlock_vault_password", "app_unlock_hello", "request_exit", "erase_locked_profile",
]);

let startup: Promise<void> | undefined;
function waitForStartup(): Promise<void> {
  return startup ??= (async () => {
    const deadline = Date.now() + 30_000;
    while (Date.now() < deadline) {
      const diagnostics = await nativeInvoke<{ startup_ready: boolean }>("get_diagnostics");
      if (diagnostics.startup_ready) return;
      await new Promise(resolve => setTimeout(resolve, 50));
    }
    throw new UserError("startup.failed", "Application startup did not finish");
  })().catch(error => { startup = undefined; throw error; });
}

/**
 * Failures that are an expected answer, not a problem, so they are not written to the error log: a page
 * with no media (`media.unavailable`), and the settings tab that is still loading, which the caller asks
 * again every 40 ms until it is ready (see `setSettingsTab`). Anything else a command returns is logged.
 */
const EXPECTED_FAILURES = new Set([
  "refresh_tab_media:media.unavailable",
  "get_reader_article:reader.unsupported",
  "set_settings_overlay:settings_tab.not_ready",
  "set_settings_overlay:settings_tab.invalid_tab",
]);

function isExpectedFailure(command: string, error: unknown): boolean {
  const text = typeof error === "string" ? error : "";
  return EXPECTED_FAILURES.has(`${command}:${text.split(":", 1)[0]}`);
}

export async function invoke<T>(command: string, args?: InvokeArgs): Promise<T> {
  const started = performance.now();
  let success = false;
  try {
    if (!ANSWERS_BEFORE_READY.has(command)) await waitForStartup();
    const result = await nativeInvoke<T>(command, args);
    success = true;
    return result;
  } catch (error) {
    // A failed command is always written to the error log (as a warning), even when the caller
    // handles it, so a bug never leaves without a trace. The log command itself is skipped to avoid a loop.
    const failure = `${command}:${typeof error === "string" ? error : (error as Error)?.message ?? String(error)}`;
    const recent = (lastFailureLogged.get(failure) ?? -Infinity) > performance.now() - 10_000;
    if (!recent && command !== "report_frontend_error" && command !== "get_diagnostics" && !isExpectedFailure(command, error)) {
      lastFailureLogged.set(failure, performance.now());
      void nativeInvoke("report_frontend_error", {
        summary: `Command "${command}" failed: ${typeof error === "string" ? error : (error as Error)?.message ?? String(error)}`,
        details: (error as Error)?.stack ?? "",
        level: "warning",
      }).catch(() => {});
    }
    // New native domains return stable dotted codes, optionally followed by diagnostic details.
    if (typeof error === "string" && /^(backup|closed_tabs|recovery|downloads|shutdown|zoom|command_search|media|runtime|security|settings_tab|vault)\.[a-z_]+(?::|$)/.test(error)) {
      throw new UserError(error.split(":", 1)[0], error);
    }
    throw new UserError(`${command}.failed`, error);
  } finally {
    ipcDiagnostics.record(command, performance.now() - started, success);
  }
}

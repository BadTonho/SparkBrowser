

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
import { invoke as nativeInvoke, type InvokeArgs } from "@tauri-apps/api/core";
import { ipcDiagnostics } from "../diagnostics.js";
import { UserError } from "../user-errors.js";

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

export async function invoke<T>(command: string, args?: InvokeArgs): Promise<T> {
  const started = performance.now();
  let success = false;
  try {
    if (command !== "get_diagnostics") await waitForStartup();
    const result = await nativeInvoke<T>(command, args);
    success = true;
    return result;
  } catch (error) {
    // A failed command is always written to the error log (as a warning), even when the caller
    // handles it, so a bug never leaves without a trace. The log command itself is skipped to avoid a loop.
    if (command !== "report_frontend_error" && command !== "get_diagnostics") {
      void nativeInvoke("report_frontend_error", {
        summary: `Command "${command}" failed: ${typeof error === "string" ? error : (error as Error)?.message ?? String(error)}`,
        details: (error as Error)?.stack ?? "",
        level: "warning",
      }).catch(() => {});
    }
    // New native domains return stable dotted codes, optionally followed by diagnostic details.
    if (typeof error === "string" && /^(backup|closed_tabs|recovery|downloads|shutdown|zoom|command_search|media|runtime|security|settings_tab)\.[a-z_]+(?::|$)/.test(error)) {
      throw new UserError(error.split(":", 1)[0], error);
    }
    throw new UserError(`${command}.failed`, error);
  } finally {
    ipcDiagnostics.record(command, performance.now() - started, success);
  }
}

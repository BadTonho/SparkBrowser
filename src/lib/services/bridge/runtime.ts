

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
import { invoke as nativeInvoke, type InvokeArgs } from "@tauri-apps/api/core";
import { ipcDiagnostics } from "../diagnostics.js";

export async function invoke<T>(command: string, args?: InvokeArgs): Promise<T> {
  const started = performance.now();
  let success = false;
  try {
    const result = await nativeInvoke<T>(command, args);
    success = true;
    return result;
  } finally {
    ipcDiagnostics.record(command, performance.now() - started, success);
  }
}

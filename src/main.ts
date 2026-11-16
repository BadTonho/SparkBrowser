import { mount } from "svelte";
import "./styles/theme.css";
import { invoke, isTauriRuntime } from "./lib/services/bridge/runtime";

// Script errors in any browser interface (shell, settings, tools, downloads) go to the error log
// and to the notification in the bottom-right corner.
let reported = 0;
function reportScriptError(summary: string, details = ""): void {
  if (!isTauriRuntime() || ++reported > 10) return;
  void invoke("report_frontend_error", { summary, details }).catch(() => {});
}
window.addEventListener("error", (event) => {
  reportScriptError(event.message || "Script error", `${event.filename}:${event.lineno}:${event.colno}\n${event.error?.stack ?? ""}`);
});
window.addEventListener("unhandledrejection", (event) => {
  const reason = event.reason as { message?: string; stack?: string } | undefined;
  reportScriptError(`Unhandled rejection: ${reason?.message ?? String(event.reason)}`, reason?.stack ?? "");
});

if (
  typeof navigator !== "undefined" &&
  (navigator.userAgent.includes("Linux") || navigator.platform.includes("Linux"))
) {
  document.documentElement.dataset.platform = "linux";
}

const component = new URLSearchParams(location.search).get("surface") === "tools"
  ? import("./lib/components/tools/ToolsSurface.svelte")
  : new URLSearchParams(location.search).get("surface") === "downloads"
    ? import("./lib/components/downloads/DownloadsSurface.svelte")
    : new URLSearchParams(location.search).get("surface") === "settings-quick"
      ? import("./lib/components/settings/SettingsQuickSurface.svelte")
      : ["settings", "settings-tab"].includes(new URLSearchParams(location.search).get("surface") ?? "")
        ? import("./lib/components/settings/SettingsSurface.svelte")
        : import("./App.svelte");
void component.then(({ default: view }) => mount(view, { target: document.getElementById("app")! }));

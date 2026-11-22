import { mount } from "svelte";
import "./styles/theme.css";
import { invoke, isTauriRuntime } from "./lib/services/bridge/runtime";

// Script errors in any browser interface (shell, settings, tools, downloads) go to the error log
// and to the notification in the bottom-right corner.
let reported = 0;
function reportScriptError(summary: string, details = "", level?: "warning"): void {
  if (!isTauriRuntime() || ++reported > 50) return;
  void invoke("report_frontend_error", { summary, details, level }).catch(() => {});
}
// Errors the interface prints to the console (including framework warnings) are logged too.
const consoleError = console.error.bind(console);
console.error = (...args: unknown[]) => {
  consoleError(...args);
  reportScriptError(`console.error: ${args.map((a) => (a instanceof Error ? a.message : String(a))).join(" ").slice(0, 300)}`,
    args.find((a): a is Error => a instanceof Error)?.stack ?? "", "warning");
};
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
      : new URLSearchParams(location.search).get("surface") === "resources"
      ? import("./lib/components/resources/ResourcesSurface.svelte")
      : new URLSearchParams(location.search).get("surface") === "commands"
        ? import("./lib/components/commandbar/CommandsSurface.svelte")
        : new URLSearchParams(location.search).get("surface") === "suggestions"
        ? import("./lib/components/titlebar/SuggestionsSurface.svelte")
        : ["settings", "settings-tab"].includes(new URLSearchParams(location.search).get("surface") ?? "")
        ? import("./lib/components/settings/SettingsSurface.svelte")
        : import("./LaunchGate.svelte");
void component.then(({ default: view }) => mount(view, { target: document.getElementById("app")! }));

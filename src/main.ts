import { mount } from "svelte";
import "./styles/theme.css";

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
      : import("./App.svelte");
void component.then(({ default: view }) => mount(view, { target: document.getElementById("app")! }));

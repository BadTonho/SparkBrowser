import { mount } from "svelte";
import "./styles/theme.css";

const component = new URLSearchParams(location.search).get("surface") === "tools"
  ? import("./lib/components/tools/ToolsSurface.svelte")
  : import("./App.svelte");
void component.then(({ default: view }) => mount(view, { target: document.getElementById("app")! }));

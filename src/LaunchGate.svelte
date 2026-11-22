<script lang="ts">
  import { onMount } from "svelte";
  import type { Component } from "svelte";
  import ProfilePicker from "./lib/components/profile/ProfilePicker.svelte";
  import { getLaunchState } from "./lib/services/bridge/profiles";
  import { isTauriRuntime } from "./lib/services/bridge/runtime";
  import { logIgnored } from "./lib/services/ignored-error";

  // The first thing the main window shows. A start that only asks which profile to open shows the
  // picker (a process with no profile of its own, so nothing of the browser may load); every other
  // start is the browser. The browser is only loaded once this is known, so none of its stores run
  // (and none of its commands are called) in the picker.
  let mode = $state<"loading" | "picker" | "browser">("loading");
  let Browser = $state<Component | null>(null);

  async function showBrowser(): Promise<void> {
    mode = "browser";
    Browser = (await import("./App.svelte")).default as unknown as Component;
  }

  onMount(() => {
    if (!isTauriRuntime()) { void showBrowser(); return; }
    getLaunchState()
      .then(state => { if (state.mode === "picker") mode = "picker"; else return showBrowser(); })
      .catch(error => { logIgnored(error); return showBrowser(); });
  });
</script>

{#if mode === "picker"}
  <ProfilePicker />
{:else if Browser}
  <Browser />
{/if}

<script lang="ts">
  import { focusDialog } from "../../actions/focusDialog";
  import { isSitePermissionsOpen, closeSitePermissions } from "../../stores/sitePermissionsStore";
  import { currentUrl, triggerReload, tabs } from "../../stores/browserStore";
  import PermissionSettings from "./PermissionSettings.svelte";
  import "../tools/tools.css";
  let confirmingReload = $state(false); let error = $state("");
  const origin = $derived.by(() => { try { const url = new URL($currentUrl); return ["http:", "https:"].includes(url.protocol) ? url.origin : null; } catch { return null; } });
  const protectedPage = $derived($tabs.find(tab => tab.is_active)?.activity);
  $effect(() => { void origin; void $isSitePermissionsOpen; confirmingReload = false; error = ""; });
  async function reload() { try { await closeSitePermissions(); await triggerReload(); confirmingReload = false; } catch (e) { error = String(e); } }
</script>
{#if $isSitePermissionsOpen}
  <div class="tools-backdrop" role="presentation" onclick={(event) => { if (event.target === event.currentTarget) void closeSitePermissions(); }}>
    <div class="tools-panel" role="dialog" aria-modal="true" aria-label="Site permissions" tabindex="-1" use:focusDialog={{ onEscape: closeSitePermissions }}>
      <header><h2>Current site</h2><button onclick={closeSitePermissions}>Close</button></header>
      <div class="tools-content">
        {#if origin}<PermissionSettings {origin} />{:else}<p>This page has no supported site origin.</p>{/if}
        {#if error}<p role="alert">{error}</p>{/if}
        {#if confirmingReload}<p>Reloading may discard forms or interrupt media/downloads. Reload this page?</p><button onclick={reload}>Confirm reload</button><button onclick={() => confirmingReload = false}>Cancel</button>
        {:else}<button onclick={() => { if (protectedPage?.dirty_form || protectedPage?.playing_media || protectedPage?.downloading) confirmingReload = true; else void reload(); }}>Reload when ready</button>{/if}
      </div>
    </div>
  </div>
{/if}

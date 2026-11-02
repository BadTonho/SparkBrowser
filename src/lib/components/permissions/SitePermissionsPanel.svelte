<script lang="ts">
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { t } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { isSitePermissionsOpen, closeSitePermissions } from "../../stores/sitePermissionsStore";
  import { currentUrl, triggerReload, tabs } from "../../stores/browserStore";
  import PermissionSettings from "./PermissionSettings.svelte";
  import "../tools/tools.css";
  let confirmingReload = $state(false); let error = $state("");
  const origin = $derived.by(() => { try { const url = new URL($currentUrl); return ["http:", "https:"].includes(url.protocol) ? url.origin : null; } catch { return null; } });
  const protectedPage = $derived($tabs.find(tab => tab.is_active)?.activity);
  $effect(() => { void origin; void $isSitePermissionsOpen; confirmingReload = false; error = ""; });
  async function reload() { try { await closeSitePermissions(); await triggerReload(); confirmingReload = false; } catch (e) { error = describeError(e); } }
</script>
{#if $isSitePermissionsOpen}
  <div class="tools-backdrop" role="presentation" onclick={(event) => { if (event.target === event.currentTarget) void closeSitePermissions(); }}>
    <div class="tools-panel" role="dialog" aria-modal="true" aria-label={$t("permissions.site_permissions_a688")} tabindex="-1" use:focusDialog={{ onEscape: closeSitePermissions }}>
      <header><h2>{$t("permissions.current_site_334f")}</h2><button onclick={closeSitePermissions}>{$t("permissions.close_7d9e")}</button></header>
      <div class="tools-content">
        {#if origin}<PermissionSettings {origin} />{:else}<p>{$t("permissions.this_page_has_no_supported_site_origin_5c79")}</p>{/if}
        {#if error}<ErrorMessage error={error} />{/if}
        {#if confirmingReload}<p>{$t("permissions.reloading_may_discard_forms_or_interrupt_media_downloads_reload_this_pag_5db0")}</p><button onclick={reload}>{$t("permissions.confirm_reload_a5f7")}</button><button onclick={() => confirmingReload = false}>{$t("permissions.cancel_1976")}</button>
        {:else}<button onclick={() => { if (protectedPage?.dirty_form || protectedPage?.playing_media || protectedPage?.downloading) confirmingReload = true; else void reload(); }}>{$t("permissions.reload_when_ready_3a1a")}</button>{/if}
      </div>
    </div>
  </div>
{/if}

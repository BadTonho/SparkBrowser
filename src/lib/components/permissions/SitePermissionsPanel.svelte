<script lang="ts">
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { t } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { permissionPopover } from "../../actions/permissionPopover";
  import { isSitePermissionsOpen, closeSitePermissions } from "../../stores/sitePermissionsStore";
  import { currentUrl, triggerReload, tabs } from "../../stores/browserStore";
  import PermissionSettings from "./PermissionSettings.svelte";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  const native = isTauriRuntime();
  let confirmingReload = $state(false); let error = $state("");
  const origin = $derived.by(() => { try { const url = new URL($currentUrl); return ["http:", "https:"].includes(url.protocol) ? url.origin : null; } catch { return null; } });
  const protectedPage = $derived($tabs.find(tab => tab.is_active)?.activity);
  $effect(() => { void origin; void $isSitePermissionsOpen; confirmingReload = false; error = ""; });
  async function reload() { try { await closeSitePermissions(); await triggerReload(); confirmingReload = false; } catch (e) { error = describeError(e); } }
</script>
{#if !native && $isSitePermissionsOpen}
  <div class="permissions-dismiss" role="presentation" onclick={(event) => { if (event.target === event.currentTarget) void closeSitePermissions(); }}>
    <div class="permission-settings-popover" role="dialog" aria-modal="true" aria-label={$t("permissions.site_permissions_a688")} tabindex="-1" use:permissionPopover use:focusDialog={{ onEscape: closeSitePermissions }}>
      <header><h2>{$t("permissions.current_site_334f")}</h2><button class="close-button" aria-label={$t("permissions.close_7d9e")} onclick={closeSitePermissions}>×</button></header>
      <div class="popover-content">
        {#if origin}<PermissionSettings {origin} />{:else}<p>{$t("permissions.this_page_has_no_supported_site_origin_5c79")}</p>{/if}
        {#if error}<ErrorMessage error={error} />{/if}
        {#if confirmingReload}<p>{$t("permissions.reloading_may_discard_forms_or_interrupt_media_downloads_reload_this_pag_5db0")}</p><div class="reload-actions"><button onclick={reload}>{$t("permissions.confirm_reload_a5f7")}</button><button onclick={() => confirmingReload = false}>{$t("permissions.cancel_1976")}</button></div>
        {:else}<button class="reload-button" onclick={() => { if (protectedPage?.dirty_form || protectedPage?.playing_media || protectedPage?.picture_in_picture || protectedPage?.downloading) confirmingReload = true; else void reload(); }}>{$t("permissions.reload_when_ready_3a1a")}</button>{/if}
      </div>
    </div>
  </div>
{/if}
<style>
  .permissions-dismiss { position: fixed; inset: 0; z-index: 1500; }
  .permission-settings-popover { position: fixed; top: var(--permission-popover-top, 54px); left: var(--permission-popover-left, 8px); width: min(360px, calc(100vw - 16px)); max-height: var(--permission-popover-max-height, calc(100vh - 62px)); overflow: auto; display: flex; flex-direction: column; gap: 10px; padding: 12px; border: 1px solid var(--border-subtle, #ffffff20); border-radius: 10px; background: var(--bg-permission-popover, #1c1c1c); color: var(--text-primary, #f5f5f5); box-shadow: 0 12px 36px #0008; font-size: 12px; line-height: 1.45; }
  header { display:flex; align-items:center; justify-content:space-between; gap:8px; }
  h2 { margin:0; font-size:13px; font-weight:650; }
  button { min-height:28px; padding:5px 8px; border:1px solid var(--border-subtle, #ffffff20); border-radius:6px; background:#ffffff08; color:inherit; font:inherit; cursor:pointer; }
  .close-button { width:28px; padding:0; font-size:18px; line-height:1; }
  .popover-content { display:flex; flex-direction:column; gap:8px; min-width:0; }
  .reload-button { width:100%; }
  .reload-actions { display:flex; justify-content:flex-end; gap:6px; }
  :global(.permission-settings-popover :focus-visible) { outline:2px solid var(--accent-primary); outline-offset:2px; }
</style>

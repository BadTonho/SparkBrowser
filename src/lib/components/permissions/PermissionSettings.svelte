<script lang="ts">
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { t } from "../../i18n";
  import { onMount } from "svelte";
  import type { SitePermission, PermissionSnapshot } from "../../types/browser";
  import { fetchSitePermissions, setSitePermission } from "../../services/bridge/site-permissions";
  let { origin = null }: { origin?: string | null } = $props();
  let snapshot = $state<PermissionSnapshot | null>(null); let error = $state(""); let busy = $state(false);
  const kinds: SitePermission["kind"][] = ["camera", "microphone", "location", "notifications"];
  async function refresh() { try { snapshot = await fetchSitePermissions(); } catch (e) { error = describeError(e); } }
  async function change(site: string, kind: SitePermission["kind"], state: SitePermission["state"]) { busy = true; error = ""; try { await setSitePermission(site, kind, state); await refresh(); } catch (e) { error = describeError(e); } finally { busy = false; } }
  onMount(() => { void refresh(); });
</script>
<section class="permission-settings" aria-labelledby="site-permissions-heading">
  <h2 id="site-permissions-heading" class="group-title">{$t("permissions.site_permissions_a688")}</h2>
  {#if error}<ErrorMessage error={error} />{/if}
  {#if !snapshot}<p>{$t("permissions.loading_native_permissions_cf31")}</p>
  {:else if !snapshot.supported}<p role="status">{$t("common.permissions_unavailable")}</p>
  {:else}
    <p>{$t("permissions.decisions_are_shared_across_workspaces_changes_apply_to_new_requests_act_7bc3")}</p>
    <button disabled={busy} onclick={refresh}>{$t("permissions.refresh_decisions_a478")}</button>
    {#if origin}
      <p>{origin}</p>
      {#each kinds as kind}
        <label>{$t(("common." + kind) as "common.camera" | "common.microphone" | "common.location" | "common.notifications")}<select aria-label={$t("permissions.p0_permission_6a1e", { p0: $t(("common." + kind) as "common.camera" | "common.microphone" | "common.location" | "common.notifications") })} disabled={busy} value={snapshot.settings.find(row => row.origin === origin && row.kind === kind)?.state ?? "ask"} onchange={(event) => change(origin!, kind, event.currentTarget.value as SitePermission["state"])}><option value="ask">{$t("permissions.ask_b8c2")}</option><option value="allow">{$t("permissions.allow_e213")}</option><option value="block">{$t("permissions.block_211d")}</option></select></label>
      {/each}
    {:else}
      {#if !snapshot.settings.length}<p>{$t("permissions.no_saved_site_decisions_6caf")}</p>{/if}
      {#each snapshot.settings as row (`${row.origin}:${row.kind}`)}
        <label>{row.origin} · {$t(("common." + row.kind) as "common.camera" | "common.microphone" | "common.location" | "common.notifications")}<select aria-label={$t("permissions.p0_p1_permission_68b4", { p0: row.origin, p1: $t(("common." + row.kind) as "common.camera" | "common.microphone" | "common.location" | "common.notifications") })} disabled={busy} value={row.state} onchange={(event) => change(row.origin, row.kind, event.currentTarget.value as SitePermission["state"])}><option value="ask">{$t("permissions.ask_b8c2")}</option><option value="allow">{$t("permissions.allow_e213")}</option><option value="block">{$t("permissions.block_211d")}</option></select></label>
      {/each}
    {/if}
  {/if}
</section>
<style>
  .permission-settings { display: flex; flex-direction: column; gap: 8px; }
  .permission-settings h2 { display:block; margin:12px 0 2px; font-size:11px; font-weight:700; text-transform:uppercase; letter-spacing:.7px; color:var(--text-muted); }
  .permission-settings > p { margin:0; font-size:11px; }
  .permission-settings label { display: flex; justify-content: space-between; align-items: center; gap: 10px; min-height:30px; overflow-wrap: anywhere; font-size:12px; }
  .permission-settings select, .permission-settings button { max-width: 150px; padding: 5px 7px; background: #ffffff10; color: var(--text-primary, #eee); border: 1px solid var(--border-subtle, #555); border-radius: 6px; font:inherit; }
</style>

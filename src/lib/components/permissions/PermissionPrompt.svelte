<script lang="ts">
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { t, plural } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { permissionRequests } from "../../stores/sitePermissionsStore";
  import { respondSitePermission } from "../../services/bridge/site-permissions";
  import "../tools/tools.css";
  let busy = $state(false); let rememberBlock = $state(false); let error = $state("");
  const request = $derived($permissionRequests[0]);
  $effect(() => { void request?.id; rememberBlock = false; error = ""; });
  async function respond(decision: string) { if (!request || busy) return; busy = true; error = ""; try { await respondSitePermission(request.id, decision); rememberBlock = false; } catch (e) { error = describeError(e); } finally { busy = false; } }
</script>
{#if request}
  {#key request.id}
    <div class="permission-backdrop">
      <div data-testid="permission-request" class="tools-panel" role="dialog" aria-modal="true" aria-label={$t("permissions.site_permission_request_1192")} tabindex="-1" use:focusDialog={{ onEscape: () => respond("block") }}>
        <h2>{$t("permissions.site_permission_request_1192")}</h2>
        <p><strong>{request.frame_origin ?? request.origin}</strong> {$t("permissions.wants_to_use_183f")} <strong>{$t(("common." + request.kind) as "common.camera" | "common.microphone" | "common.location" | "common.notifications")}</strong>.</p>
        {#if request.frame_origin || request.origin !== request.top_origin}<p>{$t("permissions.embedded_in_58f8")} {request.top_origin}</p>{/if}
        {#if request.frame_origin && request.frame_origin !== request.origin}<p>{$t("permissions.webview2_applies_this_permission_decision_to_a0ef")} {request.origin}.</p>{/if}
        <p>{$t("permissions.requests_expire_after_60_seconds_5d7d")} {$plural($permissionRequests.length, $t("common.pending_one"), $t("common.pending_other"))}</p>
        {#if error}<ErrorMessage error={error} />{/if}
        <label><input type="checkbox" bind:checked={rememberBlock} disabled={busy} /> {$t("permissions.remember_blocking_for_this_site_76fa")}</label>
        <div class="tool-actions"><button disabled={busy} onclick={() => respond("allow_once")}>{$t("permissions.allow_once_1685")}</button><button disabled={busy} onclick={() => respond("allow_always")}>{$t("permissions.always_allow_9776")}</button><button disabled={busy} onclick={() => respond(rememberBlock ? "block_always" : "block")}>{$t("permissions.block_211d")}</button></div>
      </div>
    </div>
  {/key}
{/if}
<style>.permission-backdrop { position: fixed; inset: 0; z-index: 1500; background: #000a; display: grid; place-items: center; padding: 24px; }</style>

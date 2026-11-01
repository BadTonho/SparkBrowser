<script lang="ts">
  import { focusDialog } from "../../actions/focusDialog";
  import { permissionRequests } from "../../stores/sitePermissionsStore";
  import { respondSitePermission } from "../../services/bridge/site-permissions";
  import "../tools/tools.css";
  let busy = $state(false); let rememberBlock = $state(false); let error = $state("");
  const request = $derived($permissionRequests[0]);
  $effect(() => { void request?.id; rememberBlock = false; error = ""; });
  async function respond(decision: string) { if (!request || busy) return; busy = true; error = ""; try { await respondSitePermission(request.id, decision); rememberBlock = false; } catch (e) { error = String(e); } finally { busy = false; } }
</script>
{#if request}
  {#key request.id}
    <div class="permission-backdrop">
      <div class="tools-panel" role="dialog" aria-modal="true" aria-label="Site permission request" tabindex="-1" use:focusDialog={{ onEscape: () => respond("block") }}>
        <h2>Site permission request</h2>
        <p><strong>{request.frame_origin ?? request.origin}</strong> wants to use <strong>{request.kind}</strong>.</p>
        {#if request.frame_origin || request.origin !== request.top_origin}<p>Embedded in {request.top_origin}</p>{/if}
        {#if request.frame_origin && request.frame_origin !== request.origin}<p>WebView2 applies this permission decision to {request.origin}.</p>{/if}
        <p>Requests expire after 60 seconds. {$permissionRequests.length} pending.</p>
        {#if error}<p role="alert">{error}</p>{/if}
        <label><input type="checkbox" bind:checked={rememberBlock} disabled={busy} /> Remember blocking for this site</label>
        <div class="tool-actions"><button disabled={busy} onclick={() => respond("allow_once")}>Allow once</button><button disabled={busy} onclick={() => respond("allow_always")}>Always allow</button><button disabled={busy} onclick={() => respond(rememberBlock ? "block_always" : "block")}>Block</button></div>
      </div>
    </div>
  {/key}
{/if}
<style>.permission-backdrop { position: fixed; inset: 0; z-index: 1500; background: #000a; display: grid; place-items: center; padding: 24px; }</style>

<script lang="ts">
  import VideoControls from "./VideoControls.svelte";
  import { t, formatNumber } from "../../i18n";
  import { onMount } from "svelte";
  import { focusDialog } from "../../actions/focusDialog";
  import { tabs } from "../../stores/browserStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { isMediaOpen, closeMedia, mediaState, mediaBusy, volumeDrafts, queueVolume, refreshMedia, toggleTabMute } from "../../stores/mediaStore";
  import "../tools/tools.css";
  const active = $derived($tabs.find(t=>t.is_active));
  const media = $derived($mediaState.tabs.find(t=>t.tab_id===active?.id));
  $effect(()=> { const id=active?.id; if ($isMediaOpen && id) void refreshMedia(id); });
  onMount(()=> { const timer=setInterval(()=> {if($isMediaOpen && active)void refreshMedia(active.id);},1000);return()=>clearInterval(timer); });
</script>
{#if $isMediaOpen}
  <div class="tools-backdrop" role="presentation" onclick={e=>{if(e.target===e.currentTarget)void closeMedia();}}>
    <div class="tools-panel media-panel" role="dialog" aria-modal="true" aria-label={$t("media.heading")} tabindex="-1" use:focusDialog={{onEscape:closeMedia}}>
      <header><h2>{$t("media.heading")}</h2><button onclick={closeMedia}>{$t("media.close")}</button></header>
      <div class="tools-content">
        <h3>{active?.title}</h3><p class="page-url">{active?.url}</p>
        {#if !isTauriRuntime() || !media?.mute_available}<p>{$t("media.unavailable")}</p>
        {:else if active}
          <button data-testid="media-toggle-mute" disabled={$mediaBusy.has(active.id)} aria-pressed={media.muted} onclick={()=>toggleTabMute(active.id)}>{$t(media.muted?"media.unmute":"media.mute")}</button>
          <label for="page-volume">{$t("media.volume")} — {$formatNumber(media.applied_volume)}%</label>
          <input id="page-volume" data-testid="media-volume" type="range" min="0" max="200" step="1"
            value={$volumeDrafts[active.id]??media.volume} disabled={!media.document_key || media.volume_status==='unavailable'}
            oninput={e=>queueVolume(active.id,media.document_key,Number(e.currentTarget.value))} />
          <button data-testid="media-reset" disabled={!media.document_key} onclick={()=>queueVolume(active.id,media.document_key,100)}>{$t("media.reset")}</button>
          {#if media.volume_status==='limited'}<p role="status">{$t("media.limited")}</p>
          {:else if media.volume_status==='pending'}<p role="status">{$t("media.pending")}</p>
          {:else if media.volume_status==='unavailable'}<p role="status">{$t("media.volume_unavailable")}</p>{/if}
          <VideoControls {media} />
        {/if}
      </div>
    </div>
  </div>
{/if}
<style>
  .media-panel { max-width:480px; height:auto; max-height:80vh; margin:auto; }
  .tools-content { display:grid; gap:14px; }
  .page-url { overflow-wrap:anywhere; color:var(--text-secondary); }
  input { width:100%; accent-color:var(--accent-primary); }
</style>

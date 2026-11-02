<script lang="ts">
  import { t } from "../../i18n";
  import { tabs, selectTab } from "../../stores/browserStore";
  import { mediaState, pipBusy, enterVideo, exitVideo, closeMedia } from "../../stores/mediaStore";
  import type { TabMediaState } from "../../types/media";
  let { media }: {media: TabMediaState} = $props();
  const videos = $derived(media.frames.flatMap(frame=>frame.videos.filter(v=>v.eligible).map(video=>({frame,video}))));
  const owner = $derived($mediaState.tabs.find(t=>t.frames.some(f=>f.picture_in_picture)));
  async function returnToTab(id:string) {await closeMedia();await selectTab(id);}
</script>
<div class="video-controls">
  <h3>{$t("media.video")}</h3>
  {#if owner}
    <p>{$t("media.floating")}: {$tabs.find(t=>t.id===owner.tab_id)?.title}</p>
    <button data-testid="pip-return" disabled={$pipBusy} onclick={()=>returnToTab(owner.tab_id)}>{$t("media.return")}</button>
    <button data-testid="pip-exit" disabled={$pipBusy} onclick={exitVideo}>{$t("media.exit_pip")}</button>
  {/if}
  {#each videos as item, index (`${item.frame.context}:${item.video.id}`)}
    <div class="video-candidate">
      <span>{item.video.label || $t("media.video")} {videos.length>1?`(${index+1})`:""} — {item.video.width} × {item.video.height}</span>
      <button data-testid="pip-enter" disabled={$pipBusy || item.frame.pip_video===item.video.id} onclick={()=>enterVideo(media.tab_id,item.frame.context,item.frame.document,item.video.id)}>{$t("media.enter_pip")}</button>
    </div>
  {:else}<p>{$t("media.no_video")}</p>{/each}
</div>
<style>.video-controls{display:grid;gap:12px;border-top:1px solid var(--border-subtle);padding-top:12px;}.video-candidate{display:grid;gap:6px;}span{overflow-wrap:anywhere;}</style>

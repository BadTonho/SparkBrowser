<script lang="ts">
  import { downloadProgress } from "../../stores/downloadsStore";
  let outline = $state<SVGRectElement>();
  let pausedOffset = $state(0);
  $effect.pre(() => {
    if ($downloadProgress.kind === "paused" && $downloadProgress.percent === null && outline)
      pausedOffset = Number.parseFloat(getComputedStyle(outline).strokeDashoffset) || 0;
  });
</script>
{#if $downloadProgress.kind !== "idle"}
  <svg class="download-progress-ring" class:indeterminate={$downloadProgress.kind === "indeterminate"} class:paused={$downloadProgress.kind === "paused"}
    class:completed={$downloadProgress.kind === "completed"} class:error={$downloadProgress.kind === "error"}
    data-testid="download-progress-ring" data-kind={$downloadProgress.kind} data-percent={$downloadProgress.percent ?? undefined}
    viewBox="0 0 32 32" aria-hidden="true" focusable="false">
    <rect bind:this={outline} x="1" y="1" width="30" height="30" rx="7" pathLength="100" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
      stroke-dasharray={$downloadProgress.kind === "indeterminate" || ($downloadProgress.kind === "paused" && $downloadProgress.percent === null) ? "24 76" : "100"}
      stroke-dashoffset={$downloadProgress.kind === "indeterminate" ? 0 : $downloadProgress.kind === "paused" && $downloadProgress.percent === null ? pausedOffset : 100 - ($downloadProgress.percent ?? 25)} />
  </svg>
{/if}
<style>
  .download-progress-ring { position: absolute; inset: 0; width: 100%; height: 100%; color: var(--accent-primary); pointer-events: none; overflow: visible; }
  rect { transition: stroke-dashoffset 180ms linear, color 180ms ease; }
  .indeterminate rect { animation: download-border 1.4s linear infinite; }
  .paused { color: var(--download-status-paused, #eab308); }
  .paused rect { transition: none; }
  .completed { color: var(--download-status-completed, #22c55e); }
  .completed rect { animation: download-completed 320ms ease-out; }
  .error { color: var(--download-status-error, #ef4444); }
  .error rect { transition: none; }
  @keyframes download-completed { from { opacity: .4; } to { opacity: 1; } }
  @keyframes download-border { to { stroke-dashoffset: -100; } }
  @media (prefers-reduced-motion: reduce) { rect { transition: none; } .indeterminate rect, .completed rect { animation: none; } }
</style>

<script lang="ts">
  import { downloadProgress } from "../../stores/downloadsStore";
</script>
{#if $downloadProgress.kind !== "idle"}
  <svg class="download-progress-ring" class:indeterminate={$downloadProgress.kind === "indeterminate"} class:paused={$downloadProgress.kind === "paused"}
    data-testid="download-progress-ring" data-kind={$downloadProgress.kind} data-percent={$downloadProgress.percent ?? undefined}
    viewBox="0 0 32 32" aria-hidden="true" focusable="false">
    <rect x="1" y="1" width="30" height="30" rx="7" pathLength="100" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
      stroke-dasharray={$downloadProgress.kind === "indeterminate" ? "24 76" : "100"}
      stroke-dashoffset={$downloadProgress.kind === "indeterminate" ? 0 : 100 - ($downloadProgress.percent ?? 25)} />
  </svg>
{/if}
<style>
  .download-progress-ring { position: absolute; inset: 0; width: 100%; height: 100%; color: var(--accent-primary); pointer-events: none; overflow: visible; }
  rect { transition: stroke-dashoffset 180ms linear; }
  .indeterminate rect { animation: download-border 1.4s linear infinite; }
  .paused { opacity: .42; }
  @keyframes download-border { to { stroke-dashoffset: -100; } }
  @media (prefers-reduced-motion: reduce) { rect { transition: none; } .indeterminate rect { animation: none; } }
</style>

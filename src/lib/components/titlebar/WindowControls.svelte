<script lang="ts">
  import { t } from "../../i18n";
  import { minimizeWindow, toggleMaximizeWindow, closeWindow } from "../../services/tauri-bridge";
</script>

<div class="window-controls" data-testid="window-controls">
  <button class="win-btn" onclick={minimizeWindow} title={$t("titlebar.minimize_d72e")} aria-label={$t("titlebar.minimize_d72e")}>
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><rect x="2" y="5.5" width="8" height="1" fill="currentColor" /></svg>
  </button>
  <button class="win-btn" onclick={toggleMaximizeWindow} title={$t("titlebar.maximize_fba6")} aria-label={$t("titlebar.maximize_fba6")}>
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><rect x="2.5" y="2.5" width="7" height="7" rx="1" stroke="currentColor" stroke-width="1.2" fill="none" /></svg>
  </button>
  <button class="win-btn close-btn" onclick={closeWindow} title={$t("titlebar.close_7d9e")} aria-label={$t("titlebar.close_7d9e")}>
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path d="M3 3L9 9M9 3L3 9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" /></svg>
  </button>
</div>

<style>
  .window-controls { display:flex; align-items:center; gap:2px; flex-shrink:0; }
  .win-btn { width:34px; height:26px; border-radius:6px; display:flex; align-items:center; justify-content:center; color:var(--text-secondary); transition:background .15s ease,color .15s ease; }
  .win-btn:hover { background:var(--bg-glass-hover); color:var(--text-primary); }
  .close-btn:hover { background:#e81123; color:#fff; }
  .win-btn:focus-visible { outline:2px solid var(--accent-primary); outline-offset:-2px; }
</style>

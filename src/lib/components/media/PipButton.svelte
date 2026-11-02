<script lang="ts">
  import { t } from "../../i18n";
  import { tabs } from "../../stores/browserStore";
  import { mediaState, pipBusy } from "../../stores/mediaStore";
  import { pipButtonBusy, toggleVideoPip } from "../../stores/pipStore";
  import { selectPipVideo } from "../../services/pip-controller.js";
  import { isTauriRuntime } from "../../services/bridge/runtime";

  const native = isTauriRuntime();
  const active = $derived($tabs.find(tab => tab.is_active));
  const media = $derived($mediaState.tabs.find(tab => tab.tab_id === active?.id));
  const floating = $derived($mediaState.tabs.some(tab => tab.frames.some(frame => frame.picture_in_picture)));
  const candidate = $derived(selectPipVideo(media));
  const busy = $derived($pipButtonBusy || $pipBusy);
  const label = $derived($t(!native ? "media.unavailable" : floating ? "media.exit_pip" : candidate ? "media.enter_pip" : "media.no_video"));
</script>

<button data-testid="toggle-pip" aria-label={label} title={label} aria-pressed={floating} aria-busy={busy}
  class:active={floating} disabled={!native || busy || (!floating && !candidate)} onclick={toggleVideoPip}>
  <svg width="15" height="15" viewBox="0 0 24 24" stroke="currentColor" fill="none" stroke-width="1.8" aria-hidden="true">
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <rect x="12" y="12" width="7" height="6" rx=".5" fill="currentColor" stroke="none" />
  </svg>
</button>

<style>
  button { width: 25px; height: 25px; display: grid; place-items: center; flex-shrink: 0; color: var(--text-secondary); border-radius: 4px; }
  button:hover:not(:disabled) { background: var(--bg-glass-hover); }
  button.active { color: var(--accent-primary); background: var(--bg-glass-hover); }
  button:focus-visible { outline: 2px solid var(--accent-primary); }
  button:disabled { opacity: .45; cursor: default; }
</style>

<script lang="ts">
  import { t } from "../../i18n";
  import { isPrivateWindow } from "../../stores/privateWindowStore";
  import AppIcon from "../icons/AppIcon.svelte";
</script>

{#if $isPrivateWindow}
  <span class="private-badge" data-tauri-drag-region data-testid="private-badge" role="status" title={$t("titlebar.private_badge_hint")}>
    <AppIcon name="eye-off" size={14} />
    <span data-tauri-drag-region>{$t("titlebar.private_badge")}</span>
  </span>
{/if}

<style>
  .private-badge { display: inline-flex; flex: 0 0 auto; align-items: center; gap: 6px; padding: 4px 11px 4px 9px; border: 1px solid rgba(167, 139, 250, 0.5); border-radius: 999px; background: rgba(124, 58, 237, 0.24); color: #ddd6fe; font-size: 12px; font-weight: 700; letter-spacing: 0.2px; line-height: 1.2; white-space: nowrap; user-select: none; }
  :global(html[data-theme="light"]) .private-badge { color: #5b21b6; background: rgba(124, 58, 237, 0.14); }
</style>

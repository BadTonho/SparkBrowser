<script lang="ts">
  import { t, plural, formatNumber } from "../../i18n";
  import { privacyStats, flipAdblockShield } from "../../stores/privacyStore";
  import { tabs, suspendInactiveTabs } from "../../stores/browserStore";

  let hibernatedCount = $derived($tabs.filter((t) => t.is_hibernated).length);
  let backgroundAwakeCount = $derived(
    $tabs.filter((t) => !t.is_active && !t.is_hibernated).length
  );
</script>

<div class="shield-card" class:disabled={!$privacyStats.shield_enabled}>
  <div class="shield-top">
    <div class="shield-info">
      <span class="shield-icon">
        {$privacyStats.shield_enabled ? "🛡️" : "⚪"}
      </span>
      <div class="shield-labels">
        <span class="shield-title">
          {$privacyStats.shield_enabled ? $t("privacy.privacy_shield_active_3f20") : $t("privacy.shield_paused_01b0")}
        </span>
        <span class="shield-sub">
          {$formatNumber($privacyStats.blocked_count)} {$t("privacy.trackers_blocked_d400")} {$formatNumber(hibernatedCount)} {$t("privacy.saved_f3d3")}
        </span>
      </div>
    </div>

    <button
      class="shield-toggle"
      class:on={$privacyStats.shield_enabled}
      onclick={flipAdblockShield}
      title={$t("privacy.toggle_privacy_protection_reloads_the_active_page_and_hibernates_backgro_79e9")}
      aria-label={$t("privacy.toggle_adblock_shield_6c4c")}
    >
      <span class="toggle-knob"></span>
    </button>
  </div>

  <span class="shield-sub">{$t("privacy.changing_protection_reloads_tabs_and_resets_page_state_5a3a")}</span>

  {#if backgroundAwakeCount > 0}
    <button
      class="ram-saver-btn"
      onclick={() => suspendInactiveTabs(0)}
      title={$t("privacy.hibernate_all_background_tabs_to_free_ram_immediately_dea9")}
    >
      <span>{$plural(backgroundAwakeCount, $t("common.hibernate_one"), $t("common.hibernate_other"))}</span>
    </button>
  {/if}
</div>

<style>
  .shield-card {
    padding: 10px 11px;
    border-radius: var(--radius-md);
    background: linear-gradient(
      135deg,
      rgba(34, 197, 94, 0.1) 0%,
      color-mix(in srgb, var(--accent-primary) 8%, transparent) 100%
    );
    border: 1px solid rgba(34, 197, 94, 0.28);
    display: flex;
    flex-direction: column;
    gap: 8px;
    transition: border-color 0.2s ease, background 0.2s ease;
  }

  .shield-card.disabled {
    background: rgba(255, 255, 255, 0.03);
    border-color: var(--border-subtle);
  }

  .shield-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .shield-info {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .shield-icon {
    font-size: 16px;
  }

  .shield-labels {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .shield-title {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .shield-sub {
    font-size: 10px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .shield-toggle {
    width: 34px;
    height: 19px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.14);
    padding: 2px;
    display: flex;
    align-items: center;
    transition: background 0.2s ease;
    flex-shrink: 0;
  }

  .shield-toggle.on {
    background: #22c55e;
  }

  .toggle-knob {
    width: 15px;
    height: 15px;
    border-radius: 50%;
    background: #ffffff;
    transition: transform 0.2s cubic-bezier(0.22, 1, 0.36, 1);
  }

  .shield-toggle.on .toggle-knob {
    transform: translateX(15px);
  }

  .ram-saver-btn {
    width: 100%;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--accent-primary) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--accent-primary) 30%, transparent);
    color: var(--text-primary);
    font-size: 10.5px;
    font-weight: 600;
    text-align: center;
    transition:
      background 0.18s ease,
      transform 0.15s ease;
  }

  .ram-saver-btn:hover {
    background: color-mix(in srgb, var(--accent-primary) 26%, transparent);
    transform: translateY(-1px);
  }
</style>


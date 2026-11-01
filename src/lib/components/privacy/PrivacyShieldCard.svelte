<script lang="ts">
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
          {$privacyStats.shield_enabled ? "Privacy Shield Active" : "Shield Paused"}
        </span>
        <span class="shield-sub">
          {$privacyStats.blocked_count} trackers blocked · {hibernatedCount} 💤 saved
        </span>
      </div>
    </div>

    <button
      class="shield-toggle"
      class:on={$privacyStats.shield_enabled}
      onclick={flipAdblockShield}
      title="Toggle privacy protection. Reloads the active page and hibernates background tabs."
      aria-label="Toggle AdBlock Shield"
    >
      <span class="toggle-knob"></span>
    </button>
  </div>

  <span class="shield-sub">Changing protection reloads tabs and resets page state.</span>

  {#if backgroundAwakeCount > 0}
    <button
      class="ram-saver-btn"
      onclick={() => suspendInactiveTabs(0)}
      title="Hibernate all background tabs to free RAM immediately"
    >
      <span>💤 Hibernate {backgroundAwakeCount} Inactive {backgroundAwakeCount === 1 ? "Tab" : "Tabs"} (Free RAM)</span>
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


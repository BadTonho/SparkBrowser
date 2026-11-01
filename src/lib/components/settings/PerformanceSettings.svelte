<script lang="ts">
import {
  autoHibernateMode,
  residentTabLimit,
  setResidentTabLimit,
  setAutoHibernateMode,
} from "../../stores/settingsStore";
import { privacyStats, flipAdblockShield } from "../../stores/privacyStore";
const autoHibernateOptions = [5, 10, 30, "auto", "off"] as const;
</script>

<section class="setting-group" aria-labelledby="performance-heading">
  <h2 class="group-title" id="performance-heading">🚀 Performance & Privacy</h2>

  <div class="setting-row" id="setting-resident-limit" tabindex="-1">
    <div class="row-text"><span class="row-label">Resident Tab Limit</span><span class="row-desc">Keep up to this many WebViews awake. Active, pinned, loading, media, download and edited-form tabs are protected.</span></div>
    <input type="number" min="1" max="100" step="1" value={$residentTabLimit} aria-label="Resident tab limit" onchange={event => setResidentTabLimit(Number(event.currentTarget.value))} />
  </div>

  <div class="setting-row" id="setting-adblock-shield" tabindex="-1">
    <div class="row-text">
      <span class="row-label">Privacy Shield</span>
      <span class="row-desc">Filters known tracker hosts. {$privacyStats.supports_live_policy_updates ? "Changing protection preserves page state." : "Changing protection reloads tabs and resets page state."}</span>
    </div>
    <button
      class="pill-action"
      class:active={$privacyStats.shield_enabled}
      onclick={flipAdblockShield}
    >
      {$privacyStats.shield_enabled ? "Enabled" : "Paused"}
    </button>
  </div>

  <div class="setting-row" id="setting-auto-hibernate" tabindex="-1">
    <div class="row-text">
      <span class="row-label">Auto-Hibernate Inactive Tabs</span>
      <span class="row-desc">
        {#if $autoHibernateMode === "off"}
          Automatic hibernation is off; manual hibernation remains available.
        {:else if $autoHibernateMode === "auto"}
          Hibernates regular tabs after 10 min and frequently used tabs after 5 hours.
        {:else}
          Suspends background WebViews after {$autoHibernateMode} min of inactivity.
        {/if}
      </span>
    </div>
    <div class="timer-pills">
      {#each autoHibernateOptions as mode}
        <button
          class="mini-pill"
          class:selected={$autoHibernateMode === mode}
          aria-pressed={$autoHibernateMode === mode}
          onclick={() => setAutoHibernateMode(mode)}
        >
          {typeof mode === "number" ? `${mode}m` : mode === "auto" ? "Auto" : "Off"}
        </button>
      {/each}
    </div>
  </div>
</section>

<script lang="ts">
  import { t } from "../../i18n";
  import { currentUrl } from "../../stores/browserStore";
  import { isNewTab } from "../../services/new-tab";
  import { openReaderMode, isReaderOpen, readerLoading } from "../../stores/readerStore";

  const isEligible = $derived(
    !isNewTab($currentUrl) &&
    ($currentUrl.startsWith("http://") || $currentUrl.startsWith("https://"))
  );
</script>

{#if isEligible}
  <button
    type="button"
    class="reader-mode-btn"
    class:active={$isReaderOpen}
    disabled={$readerLoading}
    onclick={() => openReaderMode()}
    title={$t("reader.button_tooltip")}
    aria-label={$t("reader.title")}
  >
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
      <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
      <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
    </svg>
  </button>
{/if}

<style>
  .reader-mode-btn {
    width: 25px;
    height: 25px;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    color: var(--text-secondary);
    border-radius: 4px;
    background: transparent;
    border: none;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .reader-mode-btn:hover:not(:disabled) {
    background: var(--bg-glass-hover);
    color: var(--text-primary);
  }

  .reader-mode-btn.active {
    color: var(--accent-primary);
    background: var(--bg-glass-hover);
  }

  .reader-mode-btn:focus-visible {
    outline: 2px solid var(--accent-primary);
  }

  .reader-mode-btn:disabled {
    opacity: 0.45;
    cursor: default;
  }
</style>


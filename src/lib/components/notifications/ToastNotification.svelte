<script lang="ts">
  import { activeToasts, dismissToast } from "../../stores/notificationStore";
</script>

{#if $activeToasts.length > 0}
  <div class="toast-container" role="region" aria-label="Notificações">
    {#each $activeToasts as toast (toast.id)}
      <div class="toast-card {toast.type}" role="status" aria-live="polite">
        <div class="toast-icon">
          {#if toast.type === "success"}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" width="16" height="16">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          {:else if toast.type === "error"}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" width="16" height="16">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="8" x2="12" y2="12" />
              <line x1="12" y1="16" x2="12.01" y2="16" />
            </svg>
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" width="16" height="16">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="16" x2="12" y2="12" />
              <line x1="12" y1="8" x2="12.01" y2="8" />
            </svg>
          {/if}
        </div>

        <span class="toast-message">{toast.message}</span>

        {#if toast.actionLabel && toast.onAction}
          <button
            type="button"
            class="toast-action-btn"
            onclick={() => {
              toast.onAction?.();
              dismissToast(toast.id);
            }}
          >
            {toast.actionLabel}
          </button>
        {/if}

        <button
          type="button"
          class="toast-close-btn"
          aria-label="Fechar notificação"
          onclick={() => dismissToast(toast.id)}
        >
          ×
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    top: 46px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: center;
    pointer-events: none;
    max-width: 90vw;
  }

  .toast-card {
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 14px;
    border-radius: 8px;
    background: rgba(24, 24, 28, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.14);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    color: #f0f0f4;
    font-size: 13px;
    animation: toast-slide-in 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .toast-card.success {
    border-color: rgba(52, 199, 89, 0.4);
  }
  .toast-card.success .toast-icon {
    color: #34c759;
  }

  .toast-card.error {
    border-color: rgba(255, 69, 58, 0.4);
  }
  .toast-card.error .toast-icon {
    color: #ff453a;
  }

  .toast-card.info .toast-icon {
    color: #0a84ff;
  }

  .toast-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .toast-message {
    line-height: 1.4;
    font-weight: 450;
  }

  .toast-action-btn {
    background: rgba(255, 255, 255, 0.12);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #fff;
    padding: 3px 10px;
    border-radius: 5px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s;
    margin-left: 4px;
  }
  .toast-action-btn:hover {
    background: rgba(255, 255, 255, 0.22);
  }

  .toast-close-btn {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.5);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
    padding: 0 4px;
    margin-left: 4px;
    display: flex;
    align-items: center;
    transition: color 0.15s;
  }
  .toast-close-btn:hover {
    color: #fff;
  }

  @keyframes toast-slide-in {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>


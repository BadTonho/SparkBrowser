<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import { t } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { activeWorkspaceId, workspaces } from "../../stores/workspaceStore";
  import { isTauriRuntime } from "../../services/tauri-bridge";
  import {
    chooseWorkspaceFromPicker,
    dismissWorkspacePicker,
    workspaceSwitcherOverlay,
  } from "../../stores/workspaceSwitcherStore";

  let pickerEl: HTMLElement | undefined = $state();
  let lastFocusedElement: HTMLElement | null = null;
  let hadOverlay = false;
  let pickerTop = $state(12);
  const inTauri = isTauriRuntime();

  $effect(() => {
    const overlay = $workspaceSwitcherOverlay;

    if (overlay && !hadOverlay) {
      lastFocusedElement = document.activeElement as HTMLElement | null;
      requestAnimationFrame(() => {
        pickerEl
          ?.querySelector<HTMLButtonElement>(".workspace-option")
          ?.focus({ preventScroll: true });
      });
    }

    if (!overlay && hadOverlay && lastFocusedElement?.isConnected) {
      requestAnimationFrame(() => {
        lastFocusedElement?.focus({ preventScroll: true });
        lastFocusedElement = null;
      });
    }

    hadOverlay = overlay !== null;
  });

  $effect(() => {
    if (!$workspaceSwitcherOverlay) return;

    const updatePickerPosition = () => {
      const viewport = document.querySelector<HTMLElement>(".viewport-frame");
      const top = viewport?.getBoundingClientRect().top;
      pickerTop = Math.max(8, Math.round((top ?? 0) + 10));
    };

    updatePickerPosition();
    window.addEventListener("resize", updatePickerPosition);
    const observer = new ResizeObserver(updatePickerPosition);
    const viewport = document.querySelector<HTMLElement>(".viewport-frame");
    if (viewport) observer.observe(viewport);

    return () => {
      window.removeEventListener("resize", updatePickerPosition);
      observer.disconnect();
    };
  });

  function handleBackdropClick(): void {
    if ($workspaceSwitcherOverlay?.kind === "picker") {
      dismissWorkspacePicker();
    }
  }
</script>

{#if $workspaceSwitcherOverlay && !inTauri}
  <div
    class="workspace-switcher-backdrop"
    style={`--workspace-picker-top: ${pickerTop}px`}
    transition:fade={{ duration: 100 }}
    onclick={handleBackdropClick}
    role="presentation"
  >
    {#if $workspaceSwitcherOverlay.kind === "picker"}
      <div
        bind:this={pickerEl}
        class="workspace-picker"
        in:fly={{ y: -6, duration: 140, easing: cubicOut }}
        out:fade={{ duration: 80 }}
        onclick={(event) => event.stopPropagation()}
        onkeydown={(event) => event.stopPropagation()}
        use:focusDialog={{ onEscape: dismissWorkspacePicker }}
      role="dialog"
        aria-modal="true"
        aria-label={$t("workspaces.choose_a_workspace_995c")}
        tabindex="-1"
      >
        <div class="workspace-list">
          {#each $workspaces as workspace (workspace.id)}
            <button
              class="workspace-option"
              class:active={$activeWorkspaceId === workspace.id}
              aria-pressed={$activeWorkspaceId === workspace.id}
              aria-label={workspace.name}
              onclick={() => chooseWorkspaceFromPicker(workspace.id)}
            >
              <span class="workspace-icon" aria-hidden="true"><AppIcon name={workspace.icon} /></span>
              <span class="workspace-name">{workspace.name}</span>
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .workspace-switcher-backdrop {
    position: fixed;
    inset: 0;
    z-index: 2000;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: var(--workspace-picker-top, 12px) 12px 12px;
    background: transparent;
  }

  .workspace-picker {
    width: max-content;
    max-width: min(760px, 100%);
    overflow: hidden;
    border: 1px solid color-mix(in srgb, var(--accent-primary) 55%, var(--border-subtle));
    border-radius: 12px;
    background: color-mix(in srgb, var(--bg-glass-elevated) 96%, transparent);
    box-shadow:
      0 12px 32px rgba(0, 0, 0, 0.4),
      0 0 24px color-mix(in srgb, var(--accent-primary) 14%, transparent);
    backdrop-filter: blur(var(--overlay-backdrop-blur, 16px));
  }

  .workspace-list {
    display: flex;
    max-width: 100%;
    gap: 4px;
    overflow-x: auto;
    overscroll-behavior-x: contain;
    padding: 5px;
    scrollbar-width: thin;
  }

  .workspace-option {
    min-width: 0;
    max-width: 190px;
    display: flex;
    flex: 0 0 auto;
    align-items: center;
    gap: 7px;
    padding: 7px 10px;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary);
    transition:
      background 0.14s ease,
      border-color 0.14s ease,
      color 0.14s ease;
  }

  .workspace-option:hover,
  .workspace-option:focus-visible {
    outline: none;
    border-color: var(--border-subtle);
    background: rgba(255, 255, 255, 0.07);
    color: var(--text-primary);
  }

  .workspace-option.active {
    border-color: color-mix(in srgb, var(--accent-primary) 72%, transparent);
    background: color-mix(in srgb, var(--accent-primary) 20%, transparent);
    color: var(--text-primary);
  }

  .workspace-icon {
    flex: 0 0 auto;
    font-size: 15px;
    line-height: 1;
  }

  .workspace-name {
    min-width: 0;
    overflow: hidden;
    font-size: 11px;
    font-weight: 650;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

</style>

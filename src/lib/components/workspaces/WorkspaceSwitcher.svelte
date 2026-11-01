<script lang="ts">
  import {
    workspaces,
    activeWorkspaceId,
    activateWorkspace,
  } from "../../stores/workspaceStore";
  import { changeTabWorkspace } from "../../stores/browserStore";

  let { horizontal = false }: { horizontal?: boolean } = $props();

  let dropTargetWsId = $state<string | null>(null);

  function handleDragOver(event: DragEvent, wsId: string) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = "move";
    }
    dropTargetWsId = wsId;
  }

  function handleDragLeave(event: DragEvent, wsId: string) {
    const related = event.relatedTarget as Node | null;
    if (!related || !(event.currentTarget as HTMLElement).contains(related)) {
      if (dropTargetWsId === wsId) {
        dropTargetWsId = null;
      }
    }
  }

  function handleDrop(event: DragEvent, wsId: string) {
    event.preventDefault();
    const tabId =
      event.dataTransfer?.getData("application/x-veloce-tab") ||
      event.dataTransfer?.getData("text/plain");

    if (tabId) {
      changeTabWorkspace(tabId, wsId);
    }
    dropTargetWsId = null;
  }
</script>

<div class="workspace-bar" class:horizontal>
  {#each $workspaces as ws, index (ws.id)}
    <button
      class="ws-pill"
      data-ws-id={ws.id}
      class:active={$activeWorkspaceId === ws.id}
      class:drop-hover={dropTargetWsId === ws.id}
      onclick={() => activateWorkspace(ws.id)}
      ondragover={(e) => handleDragOver(e, ws.id)}
      ondragleave={(e) => handleDragLeave(e, ws.id)}
      ondrop={(e) => handleDrop(e, ws.id)}
      title={index < 9
        ? `Switch to ${ws.name} Workspace (Alt+${index + 1})`
        : `Switch to ${ws.name} Workspace`}
      style="--ws-primary: {ws.accent_primary}; --ws-secondary: {ws.accent_secondary};"
    >
      <span class="ws-icon">{ws.icon}</span>
      <span class="ws-name">{ws.name}</span>
    </button>
  {/each}
</div>

<style>
  .workspace-bar {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    max-height: 150px;
    overflow-y: auto;
    gap: 5px;
    padding: 4px;
    border-radius: var(--radius-md);
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border-subtle);
  }

  .workspace-bar.horizontal {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 2px;
    flex: 0 1 auto;
    min-width: 0;
    max-width: min(36vw, 460px);
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: thin;
    scrollbar-color: color-mix(in srgb, var(--accent-primary) 45%, transparent) transparent;
    background: transparent;
    border: 0;
  }

  .workspace-bar.horizontal::-webkit-scrollbar { height: 4px; }
  .workspace-bar.horizontal::-webkit-scrollbar-thumb {
    border-radius: 99px;
    background: color-mix(in srgb, var(--accent-primary) 45%, transparent);
  }

  .workspace-bar.horizontal .ws-pill { flex: 0 0 auto; }

  .ws-pill {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    padding: 6px 4px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition:
      background 0.2s ease,
      color 0.2s ease,
      transform 0.16s cubic-bezier(0.22, 1, 0.36, 1),
      box-shadow 0.2s ease;
  }

  .ws-pill:hover {
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-primary);
    transform: translateY(-1px);
  }

  .ws-pill.drop-hover {
    background: color-mix(in srgb, var(--ws-primary) 30%, transparent);
    color: var(--text-primary);
    box-shadow: 0 0 0 2px var(--ws-primary), 0 0 12px var(--ws-primary);
    transform: scale(1.05);
  }

  .ws-pill.active {
    background: linear-gradient(
      135deg,
      color-mix(in srgb, var(--ws-primary) 28%, transparent) 0%,
      color-mix(in srgb, var(--ws-secondary) 22%, transparent) 100%
    );
    color: var(--text-primary);
    box-shadow: inset 0 0 0 1px var(--ws-primary);
  }

  .workspace-bar.horizontal .ws-pill {
    flex-direction: row;
    gap: 5px;
    min-width: 0;
    padding: 5px 7px;
    white-space: nowrap;
  }

  .ws-icon {
    font-size: 13px;
  }

  .ws-name {
    font-size: 9.5px;
    font-weight: 600;
    letter-spacing: 0.2px;
  }

  @media (max-width: 800px) {
    .workspace-bar.horizontal .ws-name {
      display: none;
    }

    .workspace-bar.horizontal .ws-pill {
      width: 30px;
      justify-content: center;
      padding: 5px 4px;
    }
  }
</style>


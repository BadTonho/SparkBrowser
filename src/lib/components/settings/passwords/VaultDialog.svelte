<script lang="ts">
  import { onMount, type Snippet } from "svelte";

  // A native modal dialog: it opens in the top layer, centered on the window, with the page dimmed and
  // inert behind it. Escape and a click outside close it.
  let { title, onclose, children, testid }: { title: string; onclose: () => void; children: Snippet; testid?: string } = $props();

  let dialog: HTMLDialogElement;
  // Taking the dialog down from the page must not report a second close to the parent.
  let disposed = false;

  onMount(() => {
    dialog.showModal();
    return () => {
      disposed = true;
      if (dialog.open) dialog.close();
    };
  });

  function outside(event: MouseEvent): void {
    // The body fills the dialog, so only a click on the dimmed area itself reaches the dialog element.
    if (event.target === dialog) onclose();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
<dialog bind:this={dialog} class="vault-dialog" aria-labelledby="vault-dialog-title" onclose={() => { if (!disposed) onclose(); }} onclick={outside} data-testid={testid}>
  <div class="vault-dialog-body">
    <h3 id="vault-dialog-title">{title}</h3>
    {@render children()}
  </div>
</dialog>

<style>
  .vault-dialog {
    width: min(420px, calc(100vw - 32px));
    max-height: calc(100dvh - 32px);
    margin: auto;
    padding: 0;
    border: var(--modal-surface-border, 1px solid var(--border-subtle));
    border-radius: var(--radius-lg, 14px);
    background: var(--modal-surface-bg, #1b1b1f);
    color: var(--text-primary);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.5);
    overflow: auto;
  }
  .vault-dialog::backdrop { background: rgba(0, 0, 0, 0.55); }
  .vault-dialog-body { display: flex; flex-direction: column; gap: 12px; padding: 18px; }
  .vault-dialog-body h3 { margin: 0; font-size: 16px; line-height: 1.35; color: var(--text-primary); }
</style>

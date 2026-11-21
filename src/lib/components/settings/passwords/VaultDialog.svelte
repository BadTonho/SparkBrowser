<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import { t } from "../../../i18n";

  // A modal made of plain elements, not a native <dialog>: the Settings tab shows a projection of the
  // interface that only carries an allowed set of tags (see services/panel-presentation.ts), and a
  // <dialog> is not one of them. The backdrop is a button so a click outside closes it even there,
  // where only buttons, forms and inputs can be clicked.
  let { title, onclose, children, testid }: { title: string; onclose: () => void; children: Snippet; testid?: string } = $props();

  let layer: HTMLDivElement;

  onMount(() => {
    layer.querySelector<HTMLElement>("input:not([disabled]), button.primary-pill:not([disabled])")?.focus();
  });

  function key(event: KeyboardEvent): void {
    if (event.key === "Escape") onclose();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="vault-dialog-layer" bind:this={layer} onkeydown={key} data-testid={testid}>
  <button class="vault-dialog-backdrop" type="button" tabindex="-1" aria-label={$t("passwords.cancel")} onclick={onclose}></button>
  <div class="vault-dialog" role="dialog" aria-modal="true" aria-labelledby="vault-dialog-title">
    <div class="vault-dialog-body">
      <h3 id="vault-dialog-title">{title}</h3>
      {@render children()}
    </div>
  </div>
</div>

<style>
  .vault-dialog-layer { position: fixed; inset: 0; z-index: 1000; display: grid; place-items: center; padding: 16px; }
  .vault-dialog-backdrop { position: absolute; inset: 0; width: 100%; height: 100%; padding: 0; border: 0; background: rgba(0, 0, 0, 0.55); cursor: default; }
  .vault-dialog {
    position: relative;
    width: min(420px, 100%);
    max-height: 100%;
    overflow: auto;
    border: var(--modal-surface-border, 1px solid var(--border-subtle));
    border-radius: var(--radius-lg, 14px);
    background: var(--modal-surface-bg, #1b1b1f);
    color: var(--text-primary);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.5);
  }
  .vault-dialog-body { display: flex; flex-direction: column; gap: 12px; padding: 18px; }
  .vault-dialog-body h3 { margin: 0; font-size: 16px; line-height: 1.35; color: var(--text-primary); }
</style>

<script lang="ts">
  import type { Snippet } from "svelte";
  import AppIcon from "../../icons/AppIcon.svelte";

  let {
    id,
    icon,
    title,
    description = "",
    tone,
    children,
    actions,
  }: {
    id: string;
    icon: string;
    title: string;
    description?: string;
    /** Colors the description, e.g. to confirm an action. */
    tone?: "ok";
    children?: Snippet;
    actions?: Snippet;
  } = $props();
</script>

<!--
  The shared card of Data & Backup: icon tile + title, optional body, buttons pinned to the bottom.
  Children inherit the form vocabulary below (.field, .hint, .pair) so every card looks the same.
-->
<article class="card" aria-labelledby={id}>
  <header>
    <span class="tile" aria-hidden="true"><AppIcon name={icon} size={18} /></span>
    <div class="titles">
      <h3 {id}>{title}</h3>
      {#if description}<p class:ok={tone === "ok"}>{description}</p>{/if}
    </div>
  </header>
  {#if children}<div class="body">{@render children()}</div>{/if}
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
</article>

<style>
  .card { display: flex; box-sizing: border-box; width: 100%; min-width: 0; flex-direction: column; gap: 14px; padding: 16px; border: 1px solid var(--border-subtle); border-radius: 14px; background: var(--bg-glass, rgba(255, 255, 255, 0.025)); }
  header { display: flex; align-items: flex-start; gap: 11px; min-width: 0; }
  .tile { display: grid; flex: 0 0 auto; width: 34px; height: 34px; place-items: center; border-radius: 10px; background: color-mix(in srgb, var(--accent-primary) 16%, transparent); color: var(--accent-primary); }
  .titles { min-width: 0; }
  h3 { margin: 0; color: var(--text-primary); font-size: 14px; font-weight: 700; line-height: 1.3; }
  .titles p { margin: 3px 0 0; color: var(--text-secondary); font-size: 11.5px; line-height: 1.5; overflow-wrap: anywhere; }
  .titles p.ok { color: #86efac; }

  .body { display: flex; min-width: 0; flex-direction: column; gap: 12px; }
  .body :global(.field) { display: flex; min-width: 0; flex-direction: column; gap: 6px; color: var(--text-primary); font-size: 11.5px; font-weight: 600; }
  .body :global(.field input), .body :global(.field select) { box-sizing: border-box; width: 100%; min-width: 0; padding: 8px 10px; border: 1px solid var(--border-subtle); border-radius: 8px; background: var(--bg-glass-elevated); color: var(--text-primary); font-size: 12px; }
  .body :global(.field input:focus-visible), .body :global(.field select:focus-visible) { outline: 2px solid var(--accent-primary); outline-offset: 1px; }
  .body :global(.pair) { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 12px; }
  .body :global(.hint) { margin: 0; color: var(--text-secondary); font-size: 11.5px; line-height: 1.55; overflow-wrap: anywhere; }

  /* Equal-width buttons that never wrap, so rows of cards line up. */
  .actions { display: grid; grid-template-columns: repeat(auto-fit, minmax(0, 1fr)); gap: 8px; margin-top: auto; }
  .actions :global(button) { box-sizing: border-box; width: 100%; min-width: 0; padding: 8px 10px; overflow: hidden; text-align: center; text-overflow: ellipsis; white-space: nowrap; }
  .card :global(.primary-pill) { border-color: rgba(66, 133, 244, 0.42); background: rgba(66, 133, 244, 0.18); color: var(--text-primary); font-weight: 600; }
  .card :global(.primary-pill:hover:not(:disabled)) { background: rgba(66, 133, 244, 0.28); }
</style>

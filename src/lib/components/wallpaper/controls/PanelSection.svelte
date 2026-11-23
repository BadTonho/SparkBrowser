<script lang="ts">
  import type { Snippet } from "svelte";
  let { title, resetLabel, onreset, children }: { title: string; resetLabel?: string; onreset?: () => void; children: Snippet } = $props();
</script>

<section>
  <header>
    <h2>{title}</h2>
    {#if onreset}<button type="button" class="reset" onclick={onreset}>{resetLabel}</button>{/if}
  </header>
  {@render children()}
</section>

<style>
  section { margin: 0 0 18px; }
  header { display: flex; align-items: center; justify-content: space-between; margin: 0 0 8px; }
  h2 { margin: 0; color: var(--muted); font-size: 10px; font-weight: 700; letter-spacing: 0.7px; text-transform: uppercase; }
  .reset { padding: 2px 0; border: 0; background: transparent; color: var(--muted); font: inherit; font-size: 10px; font-weight: 700; letter-spacing: 0.7px; text-transform: uppercase; cursor: pointer; }
  .reset:hover { color: var(--text); }
  .reset:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; border-radius: 4px; }
</style>

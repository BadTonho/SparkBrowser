<script lang="ts">
  import { onMount } from "svelte";
  import { accountContainers, loadAccountContainers } from "../../stores/accountContainersStore";
  let { id }: { id: string } = $props();
  const account = $derived($accountContainers.find(item => item.id === id));
  onMount(() => { void loadAccountContainers().catch(() => {}); });
</script>
<span class="container-badge" title={account?.name ?? id} aria-label={account?.name ?? id}>◉ {account?.name ?? "…"}</span>
<style>
  .container-badge { color: #b699ff; font-size: 10px; max-width: 85px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex-shrink: 1; }
</style>

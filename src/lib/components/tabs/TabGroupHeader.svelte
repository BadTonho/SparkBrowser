<script lang="ts">
  import { t } from "../../i18n";
  import { slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { showTabGroupContextMenu } from "../../stores/persistenceStore";
  import { editingGroupId, renameGroup, toggleGroupCollapsed, type TabGroup } from "../../stores/tabGroupsStore";

  let { group, order, count, onopen }: { group: TabGroup; order: number; count: number; onopen: (group: TabGroup, anchor: HTMLElement) => void } = $props();
  let node: HTMLElement;
  let editing = $derived($editingGroupId === group.id);
  let draft = $state("");
  $effect(() => { if (editing) draft = group.name; });

  function focusInput(input: HTMLInputElement) { input.focus(); input.select(); }
  function commit() {
    if ($editingGroupId !== group.id) return;
    renameGroup(group.id, draft);
    editingGroupId.set(null);
  }
  function onKey(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Enter") { event.preventDefault(); commit(); }
    else if (event.key === "Escape") { event.preventDefault(); editingGroupId.set(null); }
  }
  /** Double-clicking the name renames the group. The first click of the pair already folded or opened
   *  the group, so that is undone first. */
  function startRename(event: MouseEvent) {
    event.stopPropagation();
    if (group.mode === "collapse") toggleGroupCollapsed(group.id);
    editingGroupId.set(group.id);
  }
  function activate(event?: MouseEvent) {
    // The second click of a double-click does nothing, so the pair never folds the group twice.
    if (editing || (event && event.detail > 1)) return;
    if (group.mode === "popup") onopen(group, node);
    else toggleGroupCollapsed(group.id);
  }
</script>

<div
  bind:this={node}
  class="tab-group-header"
  class:collapsed={group.mode === "collapse" && group.collapsed}
  data-group-id={group.id}
  data-color={group.color}
  style:order={order}
  role="button"
  tabindex="0"
  aria-expanded={group.mode === "collapse" ? !group.collapsed : undefined}
  aria-haspopup={group.mode === "popup" ? "true" : undefined}
  onclick={(event) => activate(event)}
  onkeydown={(event) => { if (event.target === node && (event.key === "Enter" || event.key === " ")) { event.preventDefault(); activate(); } }}
  oncontextmenu={(event) => showTabGroupContextMenu(event, group.id)}
  transition:slide={{ duration: 160, easing: cubicOut }}
>
  <span class="dot"></span>
  {#if editing}
    <input
      class="name-input"
      maxlength="40"
      aria-label={$t("tabs.group_name")}
      placeholder={$t("tabs.group_name")}
      bind:value={draft}
      use:focusInput
      onkeydown={onKey}
      onblur={commit}
      onclick={(event) => event.stopPropagation()}
    />
  {:else}
    <span class="name" role="presentation" ondblclick={startRename}>{group.name || $t("tabs.group_unnamed")}</span>
  {/if}
  <span class="count">{count}</span>
  <span class="chevron" aria-hidden="true">{group.mode === "popup" ? "›" : "⌄"}</span>
</div>

<style>
  .tab-group-header {
    --group-color: #9aa0a6;
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 30px;
    padding: 4px 10px;
    border: 1px solid color-mix(in srgb, var(--group-color) 30%, transparent);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--group-color) 13%, transparent);
    color: var(--text-primary);
    font-size: 11.5px;
    font-weight: 700;
    cursor: pointer;
    user-select: none;
    transition: background 0.15s ease, border-color 0.15s ease;
  }
  .tab-group-header:hover, :global(.tab-group-header.drop-hover) {
    background: color-mix(in srgb, var(--group-color) 24%, transparent);
    border-color: color-mix(in srgb, var(--group-color) 60%, transparent);
  }
  .tab-group-header:focus-visible { outline: 2px solid var(--group-color); outline-offset: 1px; }
  [data-color="grey"] { --group-color: #9aa0a6; }
  [data-color="blue"] { --group-color: #5b9cf5; }
  [data-color="red"] { --group-color: #f26b6b; }
  [data-color="yellow"] { --group-color: #f2c14e; }
  [data-color="green"] { --group-color: #4cc38a; }
  [data-color="pink"] { --group-color: #f472b6; }
  [data-color="purple"] { --group-color: #a78bfa; }
  [data-color="cyan"] { --group-color: #4cc9f0; }
  .dot { flex: none; width: 10px; height: 10px; border-radius: 50%; background: var(--group-color); }
  .name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .name-input {
    flex: 1;
    min-width: 0;
    padding: 2px 6px;
    border: 1px solid color-mix(in srgb, var(--group-color) 60%, transparent);
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-primary);
    font: inherit;
    outline: none;
  }
  .count { flex: none; min-width: 18px; padding: 1px 6px; border-radius: 99px; background: color-mix(in srgb, var(--group-color) 25%, transparent); font-size: 10px; text-align: center; }
  .chevron { flex: none; color: var(--text-muted); transition: transform 0.18s ease; }
  .collapsed .chevron { transform: rotate(-90deg); }
</style>

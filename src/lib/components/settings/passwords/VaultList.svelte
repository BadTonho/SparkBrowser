<script lang="ts">
  import { t, formatDate } from "../../../i18n";
  import { avatarHue, groupEntries } from "../../../services/vault/groups.js";
  import { copyEntryPassword, copyVaultSecret, removeEntry, revealEntry, vaultBusy, vaultEntries, vaultUnreadable } from "../../../stores/vaultStore";
  import type { VaultEntryData } from "../../../types/vault";
  import EntryEditor from "./EntryEditor.svelte";

  const blank: VaultEntryData = { title: "", url: "", username: "", password: "", notes: "" };

  let query = $state("");
  let editing = $state<{ id: string | null; data: VaultEntryData } | null>(null);
  let deleting = $state<string | null>(null);
  let opened = $state<string[]>([]);
  /** Only success is shown here; a failure appears as the vault error message. */
  let copied = $state(false);

  const groups = $derived(groupEntries($vaultEntries, query));
  const searching = $derived(query.trim().length > 0);

  function isOpen(site: string): boolean {
    return searching || opened.includes(site);
  }

  function toggle(site: string): void {
    opened = opened.includes(site) ? opened.filter(item => item !== site) : [...opened, site];
  }

  async function copyPassword(id: string): Promise<void> {
    copied = await copyEntryPassword(id);
  }

  async function copyUsername(username: string): Promise<void> {
    copied = await copyVaultSecret(username);
  }

  async function edit(id: string): Promise<void> {
    const entry = await revealEntry(id);
    if (entry) editing = { id, data: { title: entry.title, url: entry.url, username: entry.username, password: entry.password, notes: entry.notes } };
  }

  async function confirmDelete(id: string): Promise<void> {
    deleting = null;
    await removeEntry(id);
  }
</script>

{#if editing}
  {#key editing.id}
    <EntryEditor id={editing.id} initial={editing.data} onclose={() => (editing = null)} />
  {/key}
{:else}
  <div class="vault-toolbar">
    <input class="vault-search" type="search" bind:value={query} placeholder={$t("passwords.search")} aria-label={$t("passwords.search")} data-testid="vault-search" />
    <button class="pill-action primary-pill" type="button" onclick={() => (editing = { id: null, data: { ...blank } })} data-testid="vault-add">{$t("passwords.add")}</button>
  </div>
  {#if $vaultUnreadable > 0}<p class="vault-error" role="alert">{$t("passwords.unreadable", { count: $vaultUnreadable })}</p>{/if}
  {#if copied}<p class="vault-ok" role="status">{$t("passwords.copied")}</p>{/if}
  {#if $vaultEntries.length === 0}
    <p class="vault-note" data-testid="vault-empty">{$t("passwords.empty")}</p>
  {:else if groups.length === 0}
    <p class="vault-note">{$t("passwords.no_results")}</p>
  {:else}
    <ul class="vault-groups" data-testid="vault-list">
      {#each groups as group (group.site)}
        <li class="vault-group">
          <button class="vault-group-head" type="button" aria-expanded={isOpen(group.site)} aria-label={$t("passwords.group_open", { site: group.site })}
            onclick={() => toggle(group.site)} data-testid="vault-group">
            <span class="vault-avatar" style={`--hue: ${avatarHue(group.site)}`} aria-hidden="true">{group.site.charAt(0).toUpperCase()}</span>
            <span class="vault-group-name">{group.site}</span>
            {#if group.entries.length > 1}<span class="vault-group-count">{$t("passwords.accounts", { count: group.entries.length })}</span>{/if}
            <span class="vault-chevron" class:open={isOpen(group.site)} aria-hidden="true">›</span>
          </button>
          {#if isOpen(group.site)}
            <ul class="vault-accounts">
              {#each group.entries as entry (entry.id)}
                <li class="vault-item" data-testid="vault-item">
                  <div class="vault-item-text">
                    <span class="vault-item-title">{entry.username || entry.title}</span>
                    <span class="vault-item-sub">{[entry.username ? entry.title : "", entry.url].filter(Boolean).join(" · ")}</span>
                    <span class="vault-item-sub">{$t("passwords.updated", { date: $formatDate(entry.updated_at * 1000, { dateStyle: "medium" }) })}</span>
                  </div>
                  {#if deleting === entry.id}
                    <div class="vault-actions" role="alertdialog" aria-label={$t("passwords.delete_confirm")}>
                      <span class="vault-note">{$t("passwords.delete_confirm")}</span>
                      <button class="pill-action" type="button" disabled={$vaultBusy} onclick={() => confirmDelete(entry.id)} data-testid="vault-delete-confirm">{$t("passwords.delete")}</button>
                      <button class="pill-action" type="button" onclick={() => (deleting = null)}>{$t("passwords.cancel")}</button>
                    </div>
                  {:else}
                    <div class="vault-actions">
                      <button class="pill-action" type="button" disabled={$vaultBusy} onclick={() => copyPassword(entry.id)} data-testid="vault-copy-password">{$t("passwords.copy_password")}</button>
                      {#if entry.username}<button class="pill-action" type="button" disabled={$vaultBusy} onclick={() => copyUsername(entry.username)}>{$t("passwords.copy_username")}</button>{/if}
                      <button class="pill-action" type="button" disabled={$vaultBusy} onclick={() => edit(entry.id)} data-testid="vault-edit">{$t("passwords.edit")}</button>
                      <button class="pill-action" type="button" onclick={() => (deleting = entry.id)} data-testid="vault-delete">{$t("passwords.delete")}</button>
                    </div>
                  {/if}
                </li>
              {/each}
            </ul>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
{/if}

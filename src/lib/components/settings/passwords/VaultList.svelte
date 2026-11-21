<script lang="ts">
  import { t, formatDate } from "../../../i18n";
  import { copyEntryPassword, copyVaultSecret, removeEntry, revealEntry, vaultBusy, vaultEntries, vaultUnreadable } from "../../../stores/vaultStore";
  import type { VaultEntryData } from "../../../types/vault";
  import EntryEditor from "./EntryEditor.svelte";

  const blank: VaultEntryData = { title: "", url: "", username: "", password: "", notes: "" };

  let query = $state("");
  let editing = $state<{ id: string | null; data: VaultEntryData } | null>(null);
  let deleting = $state<string | null>(null);
  /** Only success is shown here; a failure appears as the vault error message. */
  let copied = $state(false);

  const shown = $derived.by(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return $vaultEntries;
    return $vaultEntries.filter(entry => `${entry.title} ${entry.url} ${entry.username}`.toLowerCase().includes(needle));
  });

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
  {:else if shown.length === 0}
    <p class="vault-note">{$t("passwords.no_results")}</p>
  {:else}
    <ul class="vault-list" data-testid="vault-list">
      {#each shown as entry (entry.id)}
        <li class="vault-item" data-testid="vault-item">
          <div class="vault-item-text">
            <span class="vault-item-title">{entry.title}</span>
            <span class="vault-item-sub">{[entry.username, entry.url].filter(Boolean).join(" · ")}</span>
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
{/if}

<script lang="ts">
  import { t, formatDate } from "../../../i18n";
  import { avatarSlot, groupEntries, siteOf } from "../../../services/vault/groups.js";
  import { watchIdle } from "../../../services/vault/idle.js";
  import { removeEntry, revealEntry, vaultEntries, vaultUnreadable } from "../../../stores/vaultStore";
  import type { VaultEntryData, VaultRevealed, VaultSummary } from "../../../types/vault";
  import EntryEditor from "./EntryEditor.svelte";
  import VaultConfirmDialog from "./VaultConfirmDialog.svelte";
  import VaultDetail from "./VaultDetail.svelte";
  import VaultIcon from "./VaultIcon.svelte";

  const blank: VaultEntryData = { title: "", url: "", username: "", password: "", notes: "", totp: "" };

  let query = $state("");
  let opened = $state<string[]>([]);
  /** The login the person clicked; they are being asked to confirm who they are. */
  let confirming = $state<VaultSummary | null>(null);
  /** A login shown in full. It exists only after a confirmation and is dropped after five minutes away. */
  let detail = $state<VaultRevealed | null>(null);
  let editing = $state<{ id: string | null; data: VaultEntryData } | null>(null);
  let expired = $state(false);

  const groups = $derived(groupEntries($vaultEntries, query));
  const searching = $derived(query.trim().length > 0);

  // While a password is on screen (shown, or being edited) the window is watched: after five minutes
  // away it is closed and has to be confirmed again.
  $effect(() => {
    if (!detail && !(editing && editing.id)) return;
    return watchIdle(() => {
      detail = null;
      editing = null;
      expired = true;
    });
  });

  function isOpen(site: string): boolean {
    return searching || opened.includes(site);
  }

  function toggle(site: string): void {
    opened = opened.includes(site) ? opened.filter(item => item !== site) : [...opened, site];
  }

  /** One account opens at once; several are listed first. Either way the password needs a confirmation. */
  function pick(entries: VaultSummary[], site: string): void {
    if (entries.length === 1) ask(entries[0]);
    else toggle(site);
  }

  function ask(entry: VaultSummary): void {
    expired = false;
    confirming = entry;
  }

  function confirmed(revealed: VaultRevealed): void {
    confirming = null;
    detail = revealed;
  }

  function edit(): void {
    if (!detail) return;
    const { id, title, url, username, password, notes, totp } = detail;
    editing = { id, data: { title, url, username, password, notes, totp } };
    detail = null;
  }

  async function remove(): Promise<void> {
    const id = detail?.id;
    detail = null;
    if (id) await removeEntry(id);
  }
</script>

{#if editing}
  {#key editing.id}
    <EntryEditor id={editing.id} initial={editing.data} onclose={() => (editing = null)} />
  {/key}
{:else if detail}
  <VaultDetail entry={detail} site={siteOf(detail)} onclose={() => (detail = null)} onedit={edit} ondelete={remove} />
{:else}
  <div class="vault-toolbar">
    <div class="vault-search-field">
      <span class="vault-search-icon"><VaultIcon name="search" /></span>
      <input class="vault-search" type="search" bind:value={query} placeholder={$t("passwords.search")} aria-label={$t("passwords.search")} data-testid="vault-search" />
    </div>
    <button class="vault-button vault-primary vault-add" type="button" onclick={() => (editing = { id: null, data: { ...blank } })} data-testid="vault-add">
      <VaultIcon name="plus" /> {$t("passwords.add")}
    </button>
  </div>
  {#if expired}<p class="vault-note" role="status" data-testid="vault-expired">{$t("passwords.afk_closed")}</p>{/if}
  {#if $vaultUnreadable > 0}<p class="vault-error" role="alert">{$t("passwords.unreadable", { count: $vaultUnreadable })}</p>{/if}
  {#if $vaultEntries.length === 0}
    <p class="vault-note vault-list-empty" data-testid="vault-empty">{$t("passwords.empty")}</p>
  {:else if groups.length === 0}
    <p class="vault-note vault-list-empty" role="status">{$t("passwords.no_results")}</p>
  {:else}
    <ul class="vault-groups" data-testid="vault-list">
      {#each groups as group (group.site)}
        <li class="vault-group">
          <button class="vault-group-head" type="button" aria-expanded={group.entries.length > 1 ? isOpen(group.site) : undefined}
            aria-label={$t("passwords.group_open", { site: group.site })} onclick={() => pick(group.entries, group.site)} data-testid="vault-group">
            <span class={`vault-avatar hue-${avatarSlot(group.site)}`} aria-hidden="true">{group.site.charAt(0).toUpperCase()}</span>
            <span class="vault-group-label">
              <span class="vault-group-name">{group.site}</span>
              {#if group.entries.length > 1}<span class="vault-group-count">{$t("passwords.accounts", { count: group.entries.length })}</span>{/if}
            </span>
            <span class="vault-chevron" class:open={group.entries.length > 1 && isOpen(group.site)}><VaultIcon name="chevron" /></span>
          </button>
          {#if group.entries.length > 1 && isOpen(group.site)}
            <ul class="vault-accounts">
              {#each group.entries as entry (entry.id)}
                <li>
                  <button class="vault-account" type="button" onclick={() => ask(entry)} data-testid="vault-item">
                    <span class="vault-account-label">
                      <span class="vault-item-title">{entry.username || entry.title}</span>
                      <span class="vault-item-sub">{$t("passwords.updated", { date: $formatDate(entry.updated_at * 1000, { dateStyle: "medium" }) })}</span>
                    </span>
                    <span class="vault-chevron"><VaultIcon name="chevron" /></span>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
{/if}

{#if confirming}
  <VaultConfirmDialog reveal={secret => revealEntry(confirming!.id, secret)} site={siteOf(confirming)} onconfirmed={confirmed} oncancel={() => (confirming = null)} />
{/if}

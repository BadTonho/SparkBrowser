<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../../i18n";
  import { watchIdle } from "../../../services/vault/idle.js";
  import {
    autofillCards, autofillContacts, autofillUnreadable, loadAutofill, removeAutofill, revealAutofill,
  } from "../../../stores/autofillStore";
  import { vaultBusy } from "../../../stores/vaultStore";
  import type { AutofillCard, AutofillContact, AutofillKind, AutofillRevealed, AutofillSummary } from "../../../types/autofill";
  import VaultConfirmDialog from "../passwords/VaultConfirmDialog.svelte";
  import VaultDialog from "../passwords/VaultDialog.svelte";
  import VaultIcon from "../passwords/VaultIcon.svelte";
  import CardEditor from "./CardEditor.svelte";
  import ContactEditor from "./ContactEditor.svelte";

  // One list for cards ("payments") and one for contact details. A card shows only its last four digits;
  // opening it asks who is there first, because the editor shows the whole number.
  let { kind, onback }: { kind: AutofillKind; onback: () => void } = $props();

  const blankCard: AutofillCard = { kind: "card", nickname: "", holder: "", number: "", exp_month: 0, exp_year: 0 };
  const blankContact: AutofillContact = {
    kind: "contact", name: "", organization: "", email: "", phone: "", street: "", street2: "", city: "", region: "", postal_code: "", country: "",
  };

  let query = $state("");
  let confirming = $state<AutofillSummary | null>(null);
  let editing = $state<{ id: string | null; record: AutofillRevealed | null } | null>(null);
  let deleting = $state(false);
  let expired = $state(false);

  const records = $derived(kind === "card" ? $autofillCards : $autofillContacts);
  const shown = $derived(records.filter(row => `${row.title} ${row.subtitle}`.toLowerCase().includes(query.trim().toLowerCase())));

  onMount(() => { void loadAutofill(kind); });

  // A card on screen is closed after five minutes away, like a password, and has to be confirmed again.
  $effect(() => {
    if (!(editing && editing.id && kind === "card")) return;
    return watchIdle(() => { editing = null; expired = true; });
  });

  function open(row: AutofillSummary): void {
    expired = false;
    if (kind === "card") confirming = row;
    else void openContact(row);
  }

  async function openContact(row: AutofillSummary): Promise<void> {
    const record = await revealAutofill(row.id, null);
    if (record) editing = { id: row.id, record };
  }

  function confirmed(record: AutofillRevealed): void {
    confirming = null;
    editing = { id: record.id, record };
  }

  async function remove(): Promise<void> {
    const id = editing?.id;
    deleting = false;
    editing = null;
    if (id) await removeAutofill(id, kind);
  }
</script>

<div class="vault-detail-head">
  <button class="vault-icon-button" type="button" onclick={onback} aria-label={$t("passwords.detail_back")} data-testid="autofill-back"><VaultIcon name="back" /></button>
  <h3>{$t(kind === "card" ? "autofill.payments" : "autofill.contacts")}</h3>
</div>

{#if editing}
  {#key editing.id}
    {#if kind === "card"}
      <CardEditor id={editing.id} initial={editing.record?.kind === "card" ? editing.record : blankCard} onclose={() => (editing = null)} ondelete={editing.id ? () => (deleting = true) : undefined} />
    {:else}
      <ContactEditor id={editing.id} initial={editing.record?.kind === "contact" ? editing.record : blankContact} onclose={() => (editing = null)} ondelete={editing.id ? () => (deleting = true) : undefined} />
    {/if}
  {/key}
{:else}
  <p class="vault-note autofill-intro">{$t(kind === "card" ? "autofill.payments_description" : "autofill.contacts_description")}</p>
  <div class="vault-toolbar">
    <div class="vault-search-field">
      <span class="vault-search-icon"><VaultIcon name="search" /></span>
      <input class="vault-search" type="search" bind:value={query} placeholder={$t("passwords.search")} aria-label={$t("passwords.search")} data-testid="autofill-search" />
    </div>
    <button class="vault-button vault-primary vault-add" type="button" onclick={() => (editing = { id: null, record: null })} data-testid="autofill-add">
      <VaultIcon name="plus" /> {$t("passwords.add")}
    </button>
  </div>
  {#if expired}<p class="vault-note" role="status">{$t("passwords.afk_closed")}</p>{/if}
  {#if $autofillUnreadable > 0}<p class="vault-error" role="alert">{$t("passwords.unreadable", { count: $autofillUnreadable })}</p>{/if}
  {#if records.length === 0}
    <p class="vault-note vault-list-empty" data-testid="autofill-empty">{$t(kind === "card" ? "autofill.payments_empty" : "autofill.contacts_empty")}</p>
  {:else if shown.length === 0}
    <p class="vault-note vault-list-empty" role="status">{$t("passwords.no_results")}</p>
  {:else}
    <ul class="vault-groups" data-testid="autofill-list">
      {#each shown as row (row.id)}
        <li class="vault-group">
          <button class="vault-group-head" type="button" onclick={() => open(row)} data-testid="autofill-item">
            <span class="vault-group-label">
              <span class="vault-group-name">{row.title}</span>
              {#if row.subtitle}<span class="vault-group-count">{row.subtitle}</span>{/if}
            </span>
            <span class="vault-chevron"><VaultIcon name="chevron" /></span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
{/if}

{#if confirming}
  <VaultConfirmDialog reveal={secret => revealAutofill(confirming!.id, secret)} site={confirming.title} onconfirmed={confirmed} oncancel={() => (confirming = null)} />
{/if}

{#if deleting}
  <VaultDialog title={$t("passwords.delete_confirm")} onclose={() => (deleting = false)} testid="autofill-delete-dialog">
    <div class="vault-actions">
      <button class="pill-action primary-pill" type="button" disabled={$vaultBusy} onclick={remove} data-testid="autofill-delete-confirm">{$t("passwords.delete")}</button>
      <button class="pill-action" type="button" onclick={() => (deleting = false)}>{$t("passwords.cancel")}</button>
    </div>
  </VaultDialog>
{/if}

<style>
  .autofill-intro { margin-top: 10px; }
</style>

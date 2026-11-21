<script lang="ts">
  import { t } from "../../../i18n";
  import { saveAutofill } from "../../../stores/autofillStore";
  import { vaultBusy } from "../../../stores/vaultStore";
  import type { AutofillCard } from "../../../types/autofill";

  // The security code (CVV) is deliberately not here: it is never stored, the person types it on the site.
  let { id, initial, onclose, ondelete }: {
    id: string | null;
    initial: AutofillCard;
    onclose: () => void;
    ondelete?: () => void;
  } = $props();

  /* svelte-ignore state_referenced_locally */
  let nickname = $state(initial.nickname);
  /* svelte-ignore state_referenced_locally */
  let holder = $state(initial.holder);
  /* svelte-ignore state_referenced_locally */
  let number = $state(initial.number);
  /* svelte-ignore state_referenced_locally */
  let month = $state(initial.exp_month ? String(initial.exp_month) : "");
  /* svelte-ignore state_referenced_locally */
  let year = $state(initial.exp_year ? String(initial.exp_year) : "");

  const thisYear = new Date().getFullYear();
  const years = Array.from({ length: 21 }, (_, index) => thisYear - 1 + index);

  function clear(): void {
    number = "";
  }

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const saved = await saveAutofill(id, {
      kind: "card", nickname, holder, number, exp_month: Number(month), exp_year: Number(year),
    });
    if (saved) {
      clear();
      onclose();
    }
  }

  function cancel(): void {
    clear();
    onclose();
  }
</script>

<form class="vault-card" onsubmit={submit} data-testid="autofill-card-editor">
  <h3>{$t(id ? "autofill.card_edit" : "autofill.card_add")}</h3>
  <label class="vault-field">
    <span>{$t("autofill.card_number")}</span>
    <input bind:value={number} inputmode="numeric" autocomplete="off" maxlength="23" required disabled={$vaultBusy} data-testid="autofill-card-number" />
  </label>
  <label class="vault-field">
    <span>{$t("autofill.card_holder")}</span>
    <input bind:value={holder} maxlength="200" autocomplete="off" disabled={$vaultBusy} data-testid="autofill-card-holder" />
  </label>
  <div class="vault-row">
    <label class="vault-field">
      <span>{$t("autofill.card_month")}</span>
      <select bind:value={month} required disabled={$vaultBusy} data-testid="autofill-card-month">
        <option value="" disabled></option>
        {#each Array.from({ length: 12 }, (_, index) => index + 1) as value (value)}<option value={String(value)}>{String(value).padStart(2, "0")}</option>{/each}
      </select>
    </label>
    <label class="vault-field">
      <span>{$t("autofill.card_year")}</span>
      <select bind:value={year} required disabled={$vaultBusy} data-testid="autofill-card-year">
        <option value="" disabled></option>
        {#each years as value (value)}<option value={String(value)}>{value}</option>{/each}
      </select>
    </label>
  </div>
  <label class="vault-field">
    <span>{$t("autofill.card_nickname")}</span>
    <input bind:value={nickname} maxlength="200" autocomplete="off" disabled={$vaultBusy} data-testid="autofill-card-nickname" />
  </label>
  <p class="vault-note">{$t("autofill.card_cvv_note")}</p>
  <div class="vault-actions">
    <button class="pill-action primary-pill" type="submit" disabled={$vaultBusy || !number.trim() || !month || !year} data-testid="autofill-save">{$t("passwords.save")}</button>
    <button class="pill-action" type="button" onclick={cancel}>{$t("passwords.cancel")}</button>
    {#if ondelete}<button class="pill-action" type="button" onclick={ondelete} data-testid="autofill-delete">{$t("passwords.delete")}</button>{/if}
  </div>
</form>

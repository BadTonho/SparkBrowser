<script lang="ts">
  import { t } from "../../../i18n";
  import { saveAutofill } from "../../../stores/autofillStore";
  import { vaultBusy } from "../../../stores/vaultStore";
  import type { AutofillContact } from "../../../types/autofill";

  let { id, initial, onclose, ondelete }: {
    id: string | null;
    initial: AutofillContact;
    onclose: () => void;
    ondelete?: () => void;
  } = $props();

  /* svelte-ignore state_referenced_locally */
  let form = $state<AutofillContact>({ ...initial });

  const fields = [
    { key: "name", label: "autofill.contact_name", autocomplete: "name" },
    { key: "organization", label: "autofill.contact_organization", autocomplete: "organization" },
    { key: "email", label: "autofill.contact_email", autocomplete: "email", type: "email" },
    { key: "phone", label: "autofill.contact_phone", autocomplete: "tel", type: "tel" },
    { key: "street", label: "autofill.contact_street", autocomplete: "address-line1" },
    { key: "street2", label: "autofill.contact_street2", autocomplete: "address-line2" },
    { key: "city", label: "autofill.contact_city", autocomplete: "address-level2" },
    { key: "region", label: "autofill.contact_region", autocomplete: "address-level1" },
    { key: "postal_code", label: "autofill.contact_postal_code", autocomplete: "postal-code" },
    { key: "country", label: "autofill.contact_country", autocomplete: "country-name" },
  ] as const;

  const empty = $derived(fields.every(field => !form[field.key].trim()));

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (await saveAutofill(id, form)) onclose();
  }
</script>

<form class="vault-card" onsubmit={submit} data-testid="autofill-contact-editor">
  <h3>{$t(id ? "autofill.contact_edit" : "autofill.contact_add")}</h3>
  {#each fields as field (field.key)}
    <label class="vault-field">
      <span>{$t(field.label)}</span>
      <input bind:value={form[field.key]} type={"type" in field ? field.type : "text"} maxlength="500" autocomplete="off" disabled={$vaultBusy} data-testid={`autofill-contact-${field.key}`} />
    </label>
  {/each}
  <div class="vault-actions">
    <button class="pill-action primary-pill" type="submit" disabled={$vaultBusy || empty} data-testid="autofill-save">{$t("passwords.save")}</button>
    <button class="pill-action" type="button" onclick={onclose}>{$t("passwords.cancel")}</button>
    {#if ondelete}<button class="pill-action" type="button" onclick={ondelete} data-testid="autofill-delete">{$t("passwords.delete")}</button>{/if}
  </div>
</form>

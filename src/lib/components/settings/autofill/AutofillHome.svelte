<script lang="ts">
  import { t } from "../../../i18n";
  import VaultIcon from "../passwords/VaultIcon.svelte";

  import type { AutofillPage } from "../../../types/autofill";
  let { onopen }: { onopen: (page: Exclude<AutofillPage, "home">) => void } = $props();

  const rows = [
    { page: "passwords", label: "autofill.passwords", hint: "autofill.passwords_hint", testid: "autofill-open-passwords" },
    { page: "payments", label: "autofill.payments", hint: "autofill.payments_hint", testid: "autofill-open-payments" },
    { page: "contacts", label: "autofill.contacts", hint: "autofill.contacts_hint", testid: "autofill-open-contacts" },
  ] as const;
</script>

<ul class="vault-groups autofill-home" data-testid="autofill-home">
  {#each rows as row (row.page)}
    <li class="vault-group">
      <button class="vault-group-head" type="button" onclick={() => onopen(row.page)} data-testid={row.testid}>
        <span class="vault-group-label">
          <span class="vault-group-name">{$t(row.label)}</span>
          <span class="vault-group-count">{$t(row.hint)}</span>
        </span>
        <span class="vault-chevron"><VaultIcon name="chevron" /></span>
      </button>
    </li>
  {/each}
</ul>

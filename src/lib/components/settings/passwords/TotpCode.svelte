<script lang="ts">
  import { t } from "../../../i18n";
  import { getVaultTotpCode } from "../../../services/bridge/vault";
  import { vaultError } from "../../../services/vault/errors.js";
  import { copyVaultSecret, vaultBusy } from "../../../stores/vaultStore";
  import type { VaultTotpCode } from "../../../types/vault";
  import VaultIcon from "./VaultIcon.svelte";

  // The six-digit 2FA code of a login. The backend computes it; this only shows it and counts down.
  // `preview` is the editor: a wrong secret shows a hint instead of a failure message.
  let { secret, preview = false }: { secret: string; preview?: boolean } = $props();

  let current = $state<VaultTotpCode | null>(null);
  let left = $state(0);
  let invalid = $state(false);
  let copied = $state(false);

  async function load(text: string): Promise<void> {
    try {
      const next = await getVaultTotpCode(text);
      if (text !== secret) return;
      current = next;
      left = next.seconds_left;
      invalid = false;
    } catch (error) {
      if (text !== secret) return;
      current = null;
      invalid = vaultError(error).key === "passwords.error_invalid_totp" || preview;
    }
  }

  // A new secret (typed in the editor) is checked after a short pause; an unchanged one just ticks.
  $effect(() => {
    const text = secret.trim();
    copied = false;
    if (!text) {
      current = null;
      invalid = false;
      return;
    }
    const wait = setTimeout(() => void load(text), preview ? 400 : 0);
    return () => clearTimeout(wait);
  });

  $effect(() => {
    if (!current) return;
    const text = secret.trim();
    const tick = setInterval(() => {
      left -= 1;
      if (left <= 0) void load(text);
    }, 1000);
    return () => clearInterval(tick);
  });

  const grouped = $derived(current ? `${current.code.slice(0, Math.ceil(current.code.length / 2))} ${current.code.slice(Math.ceil(current.code.length / 2))}` : "");

  async function copy(): Promise<void> {
    if (current) copied = await copyVaultSecret(current.code);
  }
</script>

{#if invalid}
  <p class="vault-hint" role="status" data-testid="vault-totp-invalid">{$t("passwords.totp_invalid")}</p>
{:else if current}
  <div class="vault-detail-field vault-totp" data-testid="vault-totp">
    <span class="vault-detail-value vault-totp-code" data-testid="vault-totp-code">{grouped}</span>
    <span class="vault-totp-left" aria-label={$t("passwords.totp_expires", { seconds: left })}>{left}s</span>
    <button class="vault-icon-button" type="button" disabled={$vaultBusy} onclick={copy} aria-label={$t("passwords.totp_copy")} data-testid="vault-totp-copy"><VaultIcon name="copy" /></button>
  </div>
  {#if copied}<p class="vault-ok" role="status">{$t("passwords.copied")}</p>{/if}
{/if}

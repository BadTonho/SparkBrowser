<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { eraseForgotten, loadLockState, lockBusy, lockFailure, lockState, quitLocked, unlock } from "../../stores/appLockStore";

  // The first screen of a profile that has an app lock. Nothing of the profile exists behind it (no
  // tab, page or window) until the secret is right; then the browser takes its place.
  let { onunlocked }: { onunlocked: () => void } = $props();

  let secret = $state("");
  let useVault = $state(false);
  let forgetting = $state(false);
  let erasing = $state(false);
  let field = $state<HTMLInputElement>();
  /** When the brake lets the next attempt through (epoch milliseconds, 0 when it is not on). */
  let waitUntil = $state(0);
  let now = $state(Date.now());

  const waitLeft = $derived(Math.max(0, Math.ceil((waitUntil - now) / 1000)));
  const lock = $derived($lockState);
  const way = $derived(useVault && lock?.vault ? "vault" : lock?.password ? "password" : lock?.vault ? "vault" : "password");

  onMount(() => {
    void loadLockState();
    const tick = setInterval(() => { now = Date.now(); }, 500);
    return () => clearInterval(tick);
  });

  // A wait reported by the backend (a wrong guess, or a brake still on from before) starts the countdown.
  $effect(() => {
    const seconds = Math.max($lockFailure?.seconds ?? 0, $lockState?.retry_after_seconds ?? 0);
    if (seconds > 0) { waitUntil = Date.now() + seconds * 1000; now = Date.now(); }
  });

  $effect(() => { if (lock && !forgetting && !erasing) field?.focus(); });

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!secret || $lockBusy || waitLeft > 0) return;
    const opened = await unlock(way === "vault" ? "vault" : "password", secret);
    secret = "";
    if (opened) onunlocked();
  }

  async function hello(): Promise<void> {
    if ($lockBusy || waitLeft > 0) return;
    if (await unlock("hello")) onunlocked();
  }

  async function erase(): Promise<void> {
    if (await eraseForgotten()) erasing = true;
  }
</script>

<main class="lock-root" data-testid="app-lock-screen">
  <header class="lock-bar" data-tauri-drag-region>
    <span class="lock-brand" data-tauri-drag-region>Spark</span>
    <button class="lock-close" type="button" aria-label={$t("lock.close")} onclick={() => void quitLocked()} data-testid="app-lock-close">×</button>
  </header>
  <section class="lock-body">
    <h1 class="lock-title">{$t("lock.title")}</h1>
    {#if erasing}
      <p class="lock-note" role="status" data-testid="app-lock-erasing">{$t("lock.erasing")}</p>
    {:else if forgetting}
      <div class="lock-card" data-testid="app-lock-forgot-panel">
        <h2 class="lock-card-title">{$t("lock.forgot_title")}</h2>
        <p class="lock-note">{$t("lock.forgot_warning")}</p>
        {#if $lockFailure}<p class="lock-error" role="alert" data-testid="app-lock-error">{$t($lockFailure.key, $lockFailure.values)}</p>{/if}
        <div class="lock-actions">
          <button class="lock-danger" type="button" disabled={$lockBusy} onclick={() => void erase()} data-testid="app-lock-forgot-confirm">{$t("lock.forgot_confirm")}</button>
          <button class="lock-secondary" type="button" onclick={() => (forgetting = false)} data-testid="app-lock-forgot-cancel">{$t("lock.forgot_cancel")}</button>
        </div>
      </div>
    {:else if lock}
      <p class="lock-note">{$t("lock.subtitle")}</p>
      <form class="lock-card" onsubmit={submit}>
        {#if lock.password || lock.vault}
          <label class="lock-label" for="app-lock-secret">{$t(way === "vault" ? "lock.vault_label" : "lock.password_label")}</label>
          <input id="app-lock-secret" class="lock-input" type="password" autocomplete="off" spellcheck="false" maxlength="256"
            bind:value={secret} bind:this={field} disabled={$lockBusy} data-testid="app-lock-password" />
          <button class="lock-primary" type="submit" disabled={$lockBusy || !secret || waitLeft > 0} data-testid="app-lock-submit">{$t("lock.unlock")}</button>
        {/if}
        {#if lock.hello && lock.hello_available}
          <button class="lock-secondary" type="button" disabled={$lockBusy || waitLeft > 0} onclick={() => void hello()} data-testid="app-lock-hello">{$t("lock.hello")}</button>
        {/if}
        {#if lock.password && lock.vault}
          <button class="lock-link" type="button" onclick={() => { useVault = !useVault; secret = ""; }} data-testid="app-lock-switch">{$t(way === "vault" ? "lock.use_password" : "lock.use_vault")}</button>
        {/if}
        {#if waitLeft > 0}<p class="lock-error" role="status" data-testid="app-lock-wait">{$t("lock.wait", { seconds: waitLeft })}</p>{/if}
        {#if $lockFailure}<p class="lock-error" role="alert" data-testid="app-lock-error">{$t($lockFailure.key, $lockFailure.values)}</p>{/if}
      </form>
      <div class="lock-actions">
        <button class="lock-link" type="button" onclick={() => { forgetting = true; }} data-testid="app-lock-forgot">{$t("lock.forgot")}</button>
        <button class="lock-link" type="button" onclick={() => void quitLocked()} data-testid="app-lock-quit">{$t("lock.quit")}</button>
      </div>
      <p class="lock-fine">{$t("lock.not_encrypted")}</p>
    {/if}
  </section>
</main>

<style>
  .lock-root { display: flex; flex-direction: column; width: 100vw; height: 100vh; box-sizing: border-box; background: var(--bg-viewport-frame, #101010); color: var(--text-primary); font-family: inherit; }
  .lock-bar { display: flex; align-items: center; justify-content: space-between; height: 40px; padding: 0 8px 0 16px; flex: none; }
  .lock-brand { font-size: 13px; font-weight: 700; letter-spacing: 0.02em; }
  .lock-close { width: 32px; height: 28px; border: 0; border-radius: 8px; background: none; color: var(--text-secondary); font-size: 20px; line-height: 1; cursor: pointer; }
  .lock-close:hover { background: var(--bg-glass-hover); color: var(--text-primary); }
  .lock-body { display: flex; flex-direction: column; align-items: center; gap: 14px; padding: 36px 24px 24px; overflow: auto; }
  .lock-title { margin: 0; font-size: 24px; font-weight: 700; text-align: center; }
  .lock-note { margin: 0; color: var(--text-secondary); font-size: 13px; text-align: center; max-width: 440px; line-height: 1.5; }
  .lock-fine { margin: 8px 0 0; color: var(--text-muted); font-size: 12px; text-align: center; max-width: 440px; }
  .lock-card { display: flex; flex-direction: column; gap: 10px; width: min(380px, 100%); padding: 18px; border: 1px solid var(--border-subtle); border-radius: 14px; background: var(--bg-glass-elevated); }
  .lock-card-title { margin: 0; font-size: 16px; }
  .lock-label { color: var(--text-secondary); font-size: 12px; font-weight: 600; }
  .lock-input { min-width: 0; padding: 10px 12px; border-radius: 9px; border: 1px solid var(--border-subtle); background: var(--bg-glass-primary, rgba(0, 0, 0, 0.4)); color: var(--text-primary); font-size: 14px; font-family: inherit; }
  .lock-primary, .lock-secondary, .lock-danger { padding: 10px 14px; border-radius: 10px; border: 1px solid var(--border-subtle); font-size: 13px; font-weight: 600; cursor: pointer; }
  .lock-primary { background: var(--accent-primary); color: var(--accent-foreground); border-color: transparent; }
  .lock-secondary { background: var(--bg-glass-hover); color: var(--text-primary); }
  .lock-danger { background: #7f1d1d; color: #fee2e2; border-color: transparent; }
  .lock-primary:disabled, .lock-secondary:disabled, .lock-danger:disabled { opacity: 0.5; cursor: default; }
  .lock-link { padding: 4px 6px; border: 0; background: none; color: var(--text-secondary); font-size: 12.5px; text-decoration: underline; cursor: pointer; }
  .lock-actions { display: flex; gap: 12px; justify-content: center; flex-wrap: wrap; }
  .lock-error { margin: 0; color: #fca5a5; font-size: 13px; }
</style>

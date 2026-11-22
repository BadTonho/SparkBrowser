<script lang="ts">
  import { onMount } from "svelte";
  import { focusDialog } from "../../actions/focusDialog";
  import { t, type MessageKey } from "../../i18n";
  import { profileThumbnail } from "../../services/profile/create-form";
  import type { CreateProfileForm } from "../../types/profiles";
  import AppIcon from "../icons/AppIcon.svelte";
  import ProfileAvatar from "./ProfileAvatar.svelte";
  import "./profile-avatar.css";

  let { onclose, oncreate, onphotofailure }: {
    onclose: () => void;
    oncreate: (form: CreateProfileForm) => Promise<MessageKey | null>;
    onphotofailure: () => void;
  } = $props();

  let dialog: HTMLDialogElement;
  let nameInput: HTMLInputElement;
  let fileInput: HTMLInputElement;
  let name = $state("");
  let password = $state("");
  let avatar = $state<string | null>(null);
  let failure = $state<MessageKey | null>(null);
  let photoFailure = $state(false);
  let showPassword = $state(false);
  let busy = $state(false);
  let readingPhoto = $state(false);

  function dismiss() {
    if (!busy) onclose();
  }

  onMount(() => {
    const element = dialog;
    const focus = focusDialog(element, { onEscape: dismiss });
    element.showModal();
    queueMicrotask(() => nameInput?.focus());
    return () => {
      password = "";
      element.close();
      focus.destroy();
    };
  });

  async function selectPhoto(event: Event) {
    const field = event.currentTarget as HTMLInputElement;
    const file = field.files?.[0];
    field.value = "";
    if (!file) return;
    photoFailure = false;
    readingPhoto = true;
    try {
      avatar = await profileThumbnail(file);
    } catch {
      photoFailure = true;
      onphotofailure();
    } finally {
      readingPhoto = false;
    }
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (busy || readingPhoto || !name.trim()) return;
    busy = true;
    failure = null;
    const form = { name, avatar, password };
    password = "";
    showPassword = false;
    try {
      failure = await oncreate(form);
      if (!failure) onclose();
    } finally {
      form.password = "";
      busy = false;
    }
  }
</script>

<dialog bind:this={dialog} data-testid="profile-create-dialog" aria-labelledby="profile-create-heading"
  oncancel={event => { event.preventDefault(); dismiss(); }}>
  <form onsubmit={submit} aria-busy={busy}>
    <header><h2 id="profile-create-heading">{$t("profiles.create")}</h2></header>

    <div class="photo">
      <input bind:this={fileInput} type="file" accept="image/png,image/jpeg,image/webp" hidden
        onchange={selectPhoto} data-testid="profile-photo-input" />
      <button type="button" class="avatar-picker" disabled={busy || readingPhoto} onclick={() => fileInput.click()}
        aria-label={$t(avatar ? "profiles.change_photo" : "profiles.choose_photo")} data-testid="profile-photo-picker">
        <ProfileAvatar {name} {avatar} />
        <span class="avatar-brush" aria-hidden="true"><AppIcon name="brush" size={22} /></span>
      </button>
      {#if avatar}
        <button type="button" class="remove" disabled={busy || readingPhoto} data-testid="profile-photo-remove"
          onclick={() => { avatar = null; photoFailure = false; }}>{$t("profiles.remove_photo")}</button>
      {/if}
    </div>
    {#if photoFailure}<p class="error" role="alert">{$t("profiles.error_invalid_avatar")}</p>{/if}

    <div class="field">
      <label for="profile-create-name">{$t("profiles.name_label")}</label>
      <input bind:this={nameInput} id="profile-create-name" type="text" autocomplete="off" spellcheck="false"
        placeholder={$t("profiles.new_placeholder")} required maxlength="80" bind:value={name} disabled={busy}
        oninput={() => { if (failure === "profiles.error_invalid_name" || failure === "profiles.error_duplicate") failure = null; }}
        data-testid="profile-new-input" />
    </div>
    <div class="field">
      <label for="profile-create-password">{$t("profiles.optional_password")}</label>
      <div class="password-row">
        <input id="profile-create-password" type={showPassword ? "text" : "password"} bind:value={password}
          autocomplete="new-password" maxlength="512" disabled={busy}
          oninput={() => { if (failure === "profiles.error_invalid_password") failure = null; }}
          data-testid="profile-new-password" aria-describedby="profile-create-hint" />
        <button type="button" class="password-toggle" disabled={busy} onclick={() => (showPassword = !showPassword)}
          aria-controls="profile-create-password" aria-label={$t(showPassword ? "profiles.hide_password" : "profiles.show_password")}
          data-testid="profile-password-toggle">
          <AppIcon name={showPassword ? "eye-off" : "eye"} size={18} />
        </button>
      </div>
      <p id="profile-create-hint">{$t("profiles.password_hint")}</p>
    </div>
    {#if failure}<p class="error" role="alert" data-testid="profile-create-error">{$t(failure)}</p>{/if}

    <footer>
      <button type="button" class="secondary" disabled={busy} onclick={dismiss} data-testid="profile-create-cancel">{$t("profiles.cancel")}</button>
      <button type="submit" class="primary" disabled={busy || readingPhoto || !name.trim()} data-testid="profile-create-submit">
        {$t(busy ? "profiles.creating" : "profiles.create")}
      </button>
    </footer>
  </form>
</dialog>

<style>
  dialog {
    box-sizing: border-box;
    width: min(440px, calc(100vw - 32px));
    max-height: calc(100dvh - 32px);
    margin: auto;
    padding: clamp(16px, 5vw, 26px);
    overflow: auto;
    border: var(--modal-surface-border, 1px solid var(--border-subtle));
    border-radius: 18px;
    background: var(--modal-surface-bg, #161616);
    backdrop-filter: var(--modal-surface-filter, blur(24px));
    box-shadow: var(--modal-surface-shadow);
    color: var(--text-primary);
    color-scheme: var(--settings-control-scheme, dark);
  }
  dialog::backdrop { background: rgb(0 0 0 / 52%); backdrop-filter: blur(5px); }
  form { display: grid; gap: 18px; }
  h2 { margin: 0; font-size: 21px; line-height: 1.2; }
  .photo { --profile-avatar-size: 72px; display: grid; justify-items: center; gap: 4px; }
  footer { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
  .field { display: grid; gap: 7px; min-width: 0; font-size: 13px; }
  label { font-weight: 600; }
  .password-row { position: relative; }
  input:not([type="file"]) {
    box-sizing: border-box;
    width: 100%;
    min-width: 0;
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    padding: 9px 10px;
    background: var(--bg-glass-elevated);
    color: inherit;
    font: inherit;
  }
  p { margin: 0; font-size: 12px; line-height: 1.4; color: var(--text-secondary); }
  .error { color: var(--status-danger, #db6969); }
  footer { justify-content: flex-end; }
  button {
    min-height: 34px;
    border: 1px solid transparent;
    border-radius: 9px;
    padding: 6px 12px;
    background: transparent;
    color: inherit;
    font-family: inherit;
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
    cursor: pointer;
  }
  button:disabled { opacity: 0.55; cursor: default; }
  .secondary { background: var(--bg-glass-elevated); border-color: var(--border-subtle); }
  .secondary:hover:not(:disabled) { background: var(--bg-glass-hover); }
  .avatar-picker {
    position: relative;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 50%;
    padding: 0;
    font-size: 25px;
  }
  .avatar-brush {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    border-radius: inherit;
    background: rgb(0 0 0 / 58%);
    color: white;
    opacity: 0;
    pointer-events: none;
  }
  .avatar-picker:hover:enabled .avatar-brush, .avatar-picker:focus-visible .avatar-brush { opacity: 1; }
  @media (hover: none) {
    .avatar-picker:enabled .avatar-brush { opacity: 1; }
  }
  .remove { min-height: 28px; padding: 2px 8px; color: var(--text-secondary); }
  .remove:hover:enabled { color: var(--text-primary); text-decoration: underline; }
  .password-row input { padding-inline-end: 46px; }
  .password-row input::-ms-reveal { display: none; }
  .password-toggle {
    position: absolute;
    inset-block: 3px;
    inset-inline-end: 4px;
    display: grid;
    place-items: center;
    width: 34px;
    min-height: 0;
    padding: 0;
    border-radius: 6px;
    color: var(--text-secondary);
  }
  .password-toggle:hover:enabled { background: var(--bg-glass-hover); color: var(--text-primary); }
  .primary { background: var(--accent-primary); color: var(--accent-foreground); }
  .primary:hover:not(:disabled) { filter: brightness(1.12); }
  :focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 2px; }
</style>

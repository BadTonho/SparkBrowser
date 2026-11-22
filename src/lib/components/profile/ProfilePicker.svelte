<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { chooseProfile, getPickerState } from "../../services/bridge/profiles";
  import { logIgnored } from "../../services/ignored-error";
  import { profileError } from "../../services/profile/errors.js";
  import type { MessageKey } from "../../i18n";
  import type { PickerState } from "../../types/profiles";
  import ProfileAvatar from "./ProfileAvatar.svelte";
  import "./profile-avatar.css";

  // The first screen of Spark when more than one profile exists and the person has not chosen one yet
  // (or asked to be asked every time). It runs in a process with no profile of its own: choosing one
  // starts that profile in its own process and ends this one.
  let picker = $state<PickerState | null>(null);
  let askEveryTime = $state(false);
  let busy = $state(false);
  let failure = $state<MessageKey | null>(null);

  onMount(() => {
    void getPickerState()
      .then(loaded => { picker = loaded; askEveryTime = loaded.ask_every_time; })
      .catch(error => { failure = profileError(error).key as MessageKey; });
  });

  const label = (profile: PickerState["profiles"][number]) => (profile.is_default ? $t("profiles.default_name") : profile.name);

  async function choose(id: string): Promise<void> {
    if (busy) return;
    busy = true;
    failure = null;
    try {
      await chooseProfile(id, askEveryTime);
    } catch (error) {
      failure = profileError(error).key as MessageKey;
      busy = false;
    }
  }

  function closeWindow(): void {
    void getCurrentWindow().close().catch(logIgnored);
  }
</script>

<main class="picker-root" data-testid="profile-picker">
  <header class="picker-bar" data-tauri-drag-region>
    <span class="picker-brand" data-tauri-drag-region>Spark</span>
    <button class="picker-close" type="button" aria-label={$t("profiles.picker_close")} onclick={closeWindow} data-testid="profile-picker-close">×</button>
  </header>
  <div class="picker-body">
    <section class="picker-content" aria-labelledby="picker-title">
      <h1 class="picker-title" id="picker-title">{$t("profiles.picker_title")}</h1>
      <p class="picker-hint">{$t("profiles.picker_hint")}</p>
      {#if failure}<p class="picker-error" role="alert" data-testid="profile-picker-error">{$t(failure)}</p>{/if}
      {#if picker}
        <ul class="picker-list" data-testid="profile-picker-list">
          {#each picker.profiles as profile (profile.id)}
            {@const name = label(profile)}
            <li>
              <button class="picker-item" type="button" disabled={busy} onclick={() => void choose(profile.id)}
                data-testid="profile-picker-item" data-profile-id={profile.id}>
                <ProfileAvatar {name} avatar={profile.avatar} />
                <span class="picker-caption">
                  <span class="picker-name" title={name}>{name}</span>
                  {#if picker.last_profile === profile.id}<span class="picker-last">{$t("profiles.picker_last")}</span>{/if}
                </span>
              </button>
            </li>
          {/each}
        </ul>
        <label class="picker-ask">
          <input type="checkbox" bind:checked={askEveryTime} disabled={busy} data-testid="profile-picker-ask" />
          <span>{$t("profiles.picker_ask")}</span>
        </label>
      {/if}
    </section>
  </div>
</main>

<style>
  .picker-root { display: flex; flex-direction: column; width: 100vw; height: 100vh; box-sizing: border-box; background: var(--bg-viewport-frame, #101010); color: var(--text-primary); font-family: inherit; }
  .picker-bar { display: flex; align-items: center; justify-content: space-between; height: 40px; padding: 0 8px 0 16px; flex: none; }
  .picker-brand { font-size: 13px; font-weight: 700; letter-spacing: 0.02em; }
  .picker-close { width: 32px; height: 28px; border: 0; border-radius: 8px; background: none; color: var(--text-secondary); font-size: 20px; line-height: 1; cursor: pointer; }
  .picker-close:hover { background: var(--bg-glass-hover); color: var(--text-primary); }
  .picker-body { flex: 1; min-height: 0; display: flex; flex-direction: column; padding: 24px; overflow: auto; }
  .picker-content { flex: none; display: flex; flex-direction: column; align-items: center; margin-block: auto; text-align: center; }
  .picker-title { margin: 0; font-size: 28px; font-weight: 700; line-height: 1.2; }
  .picker-hint { max-width: 440px; margin: 9px 0 0; color: var(--text-secondary); font-size: 13px; line-height: 1.45; }
  .picker-error { margin: 0; color: #fca5a5; font-size: 13px; }
  .picker-list {
    --card-width: 152px;
    --card-gap: 16px;
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--card-gap);
    width: min(100%, calc(4 * var(--card-width) + 3 * var(--card-gap)));
    margin: 24px 0 0;
    padding: 0;
    list-style: none;
  }
  .picker-list li { width: min(100%, var(--card-width)); min-width: 0; }
  .picker-item {
    --profile-avatar-size: 64px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    gap: 12px;
    width: 100%;
    height: 100%;
    min-height: 166px;
    padding: 18px 12px 14px;
    border: 1px solid var(--border-subtle);
    border-radius: 16px;
    background: var(--bg-glass-elevated);
    color: var(--text-primary);
    font: inherit;
    font-size: 22px;
    text-align: center;
    cursor: pointer;
  }
  .picker-item:hover:enabled { border-color: var(--text-secondary); background: var(--bg-glass-hover); }
  .picker-item:disabled { opacity: .6; cursor: default; }
  .picker-caption { display: grid; gap: 4px; width: 100%; }
  .picker-name { display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 2; line-clamp: 2; overflow: hidden; overflow-wrap: anywhere; font-size: 14px; font-weight: 600; line-height: 1.3; }
  .picker-last { color: var(--text-secondary); font-size: 12px; line-height: 1.3; }
  .picker-ask { display: flex; align-items: center; gap: 8px; margin-top: 22px; color: var(--text-secondary); font-size: 13px; text-align: start; cursor: pointer; }
  .picker-ask input { flex: none; width: 14px; height: 14px; margin: 0; accent-color: var(--accent-primary); }
  :is(.picker-item, .picker-close, .picker-ask input):focus-visible { outline: 2px solid var(--text-primary); outline-offset: 3px; }
</style>

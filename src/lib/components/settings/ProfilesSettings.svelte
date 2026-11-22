<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import {
    addProfile, askProfileAtStartup, changeAskAtStartup, changeProfileName, loadProfiles, loadStartupPrefs, openProfileWindow, profiles, profilesBusy, profilesFailure, profilesNotice, removeProfile,
  } from "../../stores/profilesStore";
  import type { ProfileInfo } from "../../types/profiles";
  import AppLockCard from "./AppLockCard.svelte";

  let newName = $state("");
  /** The profile being renamed, with the text typed so far. */
  let renaming = $state<{ id: string; value: string } | null>(null);
  /** The profile about to be deleted, with the name typed to confirm. */
  let deleting = $state<{ id: string; typed: string } | null>(null);

  const list = $derived($profiles);
  const current = $derived(list?.items.find(item => item.id === list.current));

  onMount(() => { void loadProfiles(); void loadStartupPrefs(); });

  const label = (profile: ProfileInfo) => (profile.is_default ? $t("profiles.default_name") : profile.name);

  async function create(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (await addProfile(newName)) newName = "";
  }

  async function saveName(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (renaming && await changeProfileName(renaming.id, renaming.value)) renaming = null;
  }

  async function confirmDelete(profile: ProfileInfo): Promise<void> {
    if (!deleting || deleting.typed.trim() !== profile.name) return;
    if (await removeProfile(profile.id)) deleting = null;
  }
</script>

<section class="setting-group" aria-labelledby="profiles-heading">
  <h2 id="profiles-heading" class="group-title">{$t("profiles.category")}</h2>
  <p class="row-desc" data-testid="profiles-intro">{$t("profiles.intro")}</p>
  {#if current && !current.is_default}<p class="row-desc" data-testid="profiles-updates-note">{$t("profiles.updates_note")}</p>{/if}
  {#if $profilesFailure}<p class="row-desc" role="alert" data-testid="profiles-error">{$t($profilesFailure)}</p>{/if}
  {#if $profilesNotice}<p class="row-desc" role="status" data-testid="profiles-notice">{$t("profiles.opened")}</p>{/if}

  {#if list}
    <div data-testid="profiles-list">
      {#each list.items as profile (profile.id)}
        <div class="setting-row" id={`setting-profile-${profile.id}`} tabindex="-1" data-testid="profile-row" data-profile-id={profile.id}>
          {#if renaming?.id === profile.id}
            <form class="row-text" onsubmit={saveName}>
              <label class="row-label" for={`profile-name-${profile.id}`}>{$t("profiles.name_label")}</label>
              <input id={`profile-name-${profile.id}`} type="text" maxlength="40" autocomplete="off" spellcheck="false" bind:value={renaming.value}
                disabled={$profilesBusy} data-testid="profile-rename-input" />
              <div>
                <button class="pill-action" type="submit" disabled={$profilesBusy || !renaming.value.trim()} data-testid="profile-rename-save">{$t("profiles.save")}</button>
                <button class="pill-action" type="button" onclick={() => (renaming = null)}>{$t("profiles.cancel")}</button>
              </div>
            </form>
          {:else}
            <div class="row-text">
              <span class="row-label" data-testid="profile-name">{label(profile)}</span>
              {#if profile.id === list.current}<span class="row-desc" data-testid="profile-current">{$t("profiles.current_badge")}</span>{/if}
            </div>
            <div>
              <button class="pill-action" type="button" disabled={$profilesBusy} onclick={() => void openProfileWindow(profile.id)} data-testid="profile-open">{$t("profiles.open")}</button>
              {#if !profile.is_default}
                <button class="pill-action" type="button" disabled={$profilesBusy} onclick={() => { deleting = null; renaming = { id: profile.id, value: profile.name }; }} data-testid="profile-rename">{$t("profiles.rename")}</button>
                {#if profile.id !== list.current}
                  <button class="pill-action" type="button" disabled={$profilesBusy} onclick={() => { renaming = null; deleting = { id: profile.id, typed: "" }; }} data-testid="profile-delete">{$t("profiles.delete")}</button>
                {/if}
              {/if}
            </div>
          {/if}
        </div>
        {#if deleting?.id === profile.id}
          <div class="setting-row" data-testid="profile-delete-confirm">
            <div class="row-text">
              <span class="row-label">{$t("profiles.delete_title", { name: profile.name })}</span>
              <span class="row-desc">{$t("profiles.delete_warning")}</span>
              <label class="row-desc" for={`profile-delete-${profile.id}`}>{$t("profiles.delete_type")}</label>
              <input id={`profile-delete-${profile.id}`} type="text" maxlength="40" autocomplete="off" spellcheck="false" bind:value={deleting.typed}
                disabled={$profilesBusy} data-testid="profile-delete-input" />
              <div>
                <button class="pill-action" type="button" disabled={$profilesBusy || deleting.typed.trim() !== profile.name} onclick={() => void confirmDelete(profile)} data-testid="profile-delete-submit">{$t("profiles.delete_confirm")}</button>
                <button class="pill-action" type="button" onclick={() => (deleting = null)}>{$t("profiles.cancel")}</button>
              </div>
            </div>
          </div>
        {/if}
      {/each}
    </div>

    <div class="setting-row" id="setting-profile-startup" tabindex="-1">
      <div class="row-text">
        <span class="row-label">{$t("profiles.startup_label")}</span>
        <span class="row-desc">{$t("profiles.startup_description")}</span>
      </div>
      <input type="checkbox" checked={$askProfileAtStartup} disabled={$profilesBusy} aria-label={$t("profiles.startup_label")}
        onchange={event => void changeAskAtStartup(event.currentTarget.checked)} data-testid="profile-ask-startup" />
    </div>

    <form class="setting-row" id="setting-profile-new" tabindex="-1" onsubmit={create} data-testid="profile-new">
      <div class="row-text">
        <label class="row-label" for="profile-new-name">{$t("profiles.new_label")}</label>
        <span class="row-desc">{$t("profiles.new_description", { max: list.max })}</span>
        <input id="profile-new-name" type="text" maxlength="40" autocomplete="off" spellcheck="false" placeholder={$t("profiles.new_placeholder")}
          bind:value={newName} disabled={$profilesBusy} data-testid="profile-new-input" />
      </div>
      <button class="pill-action" type="submit" disabled={$profilesBusy || !newName.trim() || list.items.length >= list.max} data-testid="profile-create">{$t("profiles.create")}</button>
    </form>
  {/if}
  <AppLockCard />
</section>

<style>
  input[type="text"] { min-width: 0; width: min(320px, 100%); padding: 7px 9px; border-radius: 7px; border: 1px solid var(--border-subtle); background: var(--bg-glass-elevated); color: var(--text-primary); font-size: 12px; font-family: inherit; }
  input[type="text"]:disabled { opacity: 0.6; }
  form.row-text, .row-text { display: flex; flex-direction: column; gap: 6px; align-items: flex-start; }
</style>

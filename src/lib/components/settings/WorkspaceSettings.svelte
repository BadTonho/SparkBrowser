<script lang="ts">
  import { onMount } from "svelte";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { t, plural } from "../../i18n";
  import { tabs } from "../../stores/browserStore";
  import {
    activeWorkspaceId,
    createWorkspaceDraft,
    removeWorkspaceProfile,
    saveWorkspaceProfile,
    workspaces,
  } from "../../stores/workspaceStore";
  import {
    loadWorkspaceIsolation,
    setWorkspaceIsolationEnabled,
    workspaceIsolation,
  } from "../../stores/settingsStore";
  import type { WorkspaceItem } from "../../types/browser";

  let selectedWorkspaceId = $state("");
  let draft = $state<WorkspaceItem | null>(null);
  let isCreating = $state(false);
  let saving = $state(false);
  let errorMessage = $state("");
  let isDeleteDialogOpen = $state(false);
  let deleteAction = $state<"move" | "close">("move");
  let deleteTargetId = $state("");
  let isolationError = $state("");
  let applyingIsolation = $state(false);

  const selectedWorkspace = $derived(
    $workspaces.find((workspace) => workspace.id === selectedWorkspaceId)
  );
  const tabsInSelectedWorkspace = $derived(
    $tabs.filter((tab) => tab.workspace_id === selectedWorkspaceId)
  );

  function editWorkspace(workspace: WorkspaceItem): void {
    selectedWorkspaceId = workspace.id;
    draft = { ...workspace };
    isCreating = false;
    errorMessage = "";
    isDeleteDialogOpen = false;
  }

  async function applyIsolation(enabled: boolean): Promise<void> {
    applyingIsolation = true;
    isolationError = "";
    try {
      await setWorkspaceIsolationEnabled(enabled);
    } catch (error) {
      isolationError = describeError(error);
    } finally {
      applyingIsolation = false;
    }
  }

  onMount(() => {
    void loadWorkspaceIsolation();
  });

  function startNewWorkspace(): void {
    const workspace = createWorkspaceDraft();
    selectedWorkspaceId = workspace.id;
    draft = workspace;
    isCreating = true;
    errorMessage = "";
    isDeleteDialogOpen = false;
  }

  function cancelEditing(): void {
    if (isCreating) {
      selectedWorkspaceId = "";
      draft = null;
      isCreating = false;
    } else if (selectedWorkspace) {
      draft = { ...selectedWorkspace };
    }
    errorMessage = "";
  }

  async function saveDraft(): Promise<void> {
    if (!draft) return;
    saving = true;
    errorMessage = "";
    try {
      const saved = await saveWorkspaceProfile({ ...draft }, isCreating);
      selectedWorkspaceId = saved.id;
      draft = { ...saved };
      isCreating = false;
    } catch (error) {
      errorMessage = describeError(error);
    } finally {
      saving = false;
    }
  }

  function openDeleteDialog(): void {
    deleteAction = "move";
    deleteTargetId =
      $workspaces.find((workspace) => workspace.id !== selectedWorkspaceId)?.id ?? "";
    errorMessage = "";
    isDeleteDialogOpen = true;
  }

  async function confirmDelete(): Promise<void> {
    if (!selectedWorkspace || selectedWorkspace.id === "ws-personal") return;
    if (tabsInSelectedWorkspace.length > 0 && deleteAction === "move" && !deleteTargetId) {
      errorMessage = "Choose a destination workspace.";
      return;
    }

    saving = true;
    errorMessage = "";
    try {
      await removeWorkspaceProfile(
        selectedWorkspace.id,
        tabsInSelectedWorkspace.length > 0 ? deleteAction : "close",
        deleteAction === "move" ? deleteTargetId : undefined
      );
      isDeleteDialogOpen = false;
      selectedWorkspaceId = "";
      draft = null;
      isCreating = false;
    } catch (error) {
      errorMessage = describeError(error);
    } finally {
      saving = false;
    }
  }
</script>

<section class="setting-group" aria-labelledby="workspaces-heading">
  <div class="section-heading">
    <h2 class="group-title" id="workspaces-heading">{$t("settings.workspaces_theme_0497")}</h2>
    <button class="primary-action" onclick={startNewWorkspace}>{$t("settings.create_workspace_a862")}</button>
  </div>

  <div class="setting-row" id="setting-workspace-isolation" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.content_isolation_9f15")}</span>
      <span class="row-desc">{$t("settings.content_isolation_desc_fea8")}</span>
    </div>
    <button
      class="pill-action"
      class:active={$workspaceIsolation}
      aria-pressed={$workspaceIsolation}
      aria-label={$t("settings.content_isolation_9f15")}
      disabled={applyingIsolation}
      onclick={() => applyIsolation(!$workspaceIsolation)}
    >
      {$workspaceIsolation ? $t("settings.enabled_92c1") : $t("settings.paused_e159")}
    </button>
  </div>
  {#if isolationError}
    <ErrorMessage error={isolationError} />
  {/if}

  <div
    class="workspace-list"
    id="setting-workspace-profiles"
    tabindex="-1"
    aria-label={$t("settings.workspaces_1377")}
  >
    {#each $workspaces as workspace (workspace.id)}
      <button
        class="workspace-card"
        class:selected={selectedWorkspaceId === workspace.id}
        class:active={$activeWorkspaceId === workspace.id}
        aria-pressed={selectedWorkspaceId === workspace.id}
        onclick={() => editWorkspace(workspace)}
        style="--c1: {workspace.accent_primary}; --c2: {workspace.accent_secondary};"
      >
        <span class="workspace-icon">{workspace.icon || "◌"}</span>
        <span class="workspace-name">{workspace.name}</span>
        {#if $activeWorkspaceId === workspace.id}
          <span class="active-badge">{$t("settings.active_9234")}</span>
        {/if}
      </button>
    {/each}
  </div>

  {#if draft}
    <div class="editor" aria-label={$t("settings.edit_workspace_52e3")}>
      <div class="editor-heading">
        <div>
          <h3>{isCreating ? $t("settings.new_workspace_df0c") : $t("settings.edit_p0_58bd", { p0: selectedWorkspace?.name ?? "workspace" })}</h3>
          <p>{isCreating ? "This workspace will be added to the end of the list." : "Changes apply when you save."}</p>
        </div>
        {#if !isCreating && selectedWorkspace?.id === "ws-personal"}
          <span class="protected-note">{$t("settings.personal_is_protected_f8ff")}</span>
        {/if}
      </div>

      <div class="form-grid">
        <label class="field name-field">
          <span>{$t("settings.name_dcd1")}</span>
          <input bind:value={draft.name} maxlength="40" autocomplete="off" />
        </label>
        <label class="field icon-field">
          <span>{$t("settings.emoji_61ad")}</span>
          <input bind:value={draft.icon} maxlength="12" autocomplete="off" aria-label={$t("settings.workspace_emoji_c456")} />
        </label>
        <label class="field color-field">
          <span>{$t("settings.primary_color_2b03")}</span>
          <input bind:value={draft.accent_primary} type="color" />
        </label>
        <label class="field color-field">
          <span>{$t("settings.secondary_color_413d")}</span>
          <input bind:value={draft.accent_secondary} type="color" />
        </label>
      </div>

      {#if errorMessage}
        <ErrorMessage error={errorMessage} />
      {/if}

      {#if isDeleteDialogOpen && !isCreating}
        <div class="delete-panel" role="alertdialog" aria-labelledby="delete-heading">
          <h3 id="delete-heading">{$t("settings.delete_e2d0")} {selectedWorkspace?.name}?</h3>
          {#if tabsInSelectedWorkspace.length > 0}
            <p>{$plural(tabsInSelectedWorkspace.length, $t("common.workspace_tabs_one"), $t("common.workspace_tabs_other"))}</p>
            <label class="choice-row">
              <input type="radio" name="delete-tabs" value="move" bind:group={deleteAction} />
              <span>{$t("settings.move_tabs_to_438f")}</span>
              <select bind:value={deleteTargetId} disabled={deleteAction !== "move"}>
                {#each $workspaces.filter((workspace) => workspace.id !== selectedWorkspaceId) as destination (destination.id)}
                  <option value={destination.id}>{destination.icon} {destination.name}</option>
                {/each}
              </select>
            </label>
            <label class="choice-row">
              <input type="radio" name="delete-tabs" value="close" bind:group={deleteAction} />
              <span>{$t("settings.close_all_tabs_their_history_will_be_kept_c6e1")}</span>
            </label>
          {:else}
            <p>{$plural(tabsInSelectedWorkspace.length, $t("common.workspace_tabs_one"), $t("common.workspace_tabs_other"))}</p>
          {/if}
          <div class="dialog-actions">
            <button class="secondary-action" onclick={() => (isDeleteDialogOpen = false)}>{$t("settings.cancel_1976")}</button>
            <button class="danger-action" onclick={confirmDelete} disabled={saving}>{$t("settings.delete_workspace_55f0")}</button>
          </div>
        </div>
      {/if}

      {#if !isDeleteDialogOpen}
        <div class="editor-actions">
          {#if !isCreating && selectedWorkspace?.id !== "ws-personal"}
            <button class="danger-action" onclick={openDeleteDialog}>{$t("settings.delete_workspace_55f0")}</button>
          {/if}
          <span class="action-spacer"></span>
          <button class="secondary-action" onclick={cancelEditing} disabled={saving}>{$t("settings.cancel_1976")}</button>
          <button class="primary-action" onclick={saveDraft} disabled={saving}>
            {saving ? "Saving…" : isCreating ? $t("settings.create_workspace_4b89") : $t("settings.save_changes_dd0a")}
          </button>
        </div>
      {/if}
    </div>
  {:else}
    <p class="empty-state">{$t("settings.select_a_workspace_to_edit_its_name_emoji_and_theme_a3ee")}</p>
  {/if}
</section>

<style>
  .setting-group { display: flex; flex-direction: column; gap: 14px; }
  .section-heading, .editor-heading, .editor-actions, .dialog-actions {
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
  }
  .group-title { margin: 0; font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: .7px; color: var(--text-muted); }
  .workspace-list { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 8px; }
  .workspace-card {
    min-width: 0; min-height: 42px; display: flex; align-items: center; gap: 8px; padding: 8px 10px;
    border: 1px solid var(--border-subtle); border-radius: var(--radius-md); background: rgba(255,255,255,.04);
    color: var(--text-primary); text-align: left; transition: border-color .15s ease, background .15s ease;
  }
  .workspace-card:hover { background: rgba(255,255,255,.07); }
  .workspace-card.selected { border-color: var(--c1); background: color-mix(in srgb, var(--c1) 12%, transparent); }
  .workspace-card.active:not(.selected) { box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--c1) 55%, transparent); }
  .workspace-icon { flex: 0 0 auto; font-size: 15px; }
  .workspace-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; font-weight: 600; }
  .active-badge { flex: 0 0 auto; margin-left: auto; color: var(--c1); font-size: 9px; font-weight: 700; text-transform: uppercase; }
  .editor { display: flex; flex-direction: column; gap: 14px; padding: 16px; border: 1px solid var(--border-subtle); border-radius: var(--radius-md); background: rgba(255,255,255,.025); }
  .editor-heading h3, .delete-panel h3 { margin: 0; color: var(--text-primary); font-size: 13px; }
  .editor-heading p, .delete-panel p { margin: 4px 0 0; color: var(--text-secondary); font-size: 11px; line-height: 1.45; }
  .protected-note { flex: 0 0 auto; color: var(--text-muted); font-size: 10px; }
  .form-grid { display: grid; grid-template-columns: minmax(130px, 1fr) minmax(90px, .6fr) minmax(110px, .7fr) minmax(110px, .7fr); gap: 10px; }
  .field { min-width: 0; display: flex; flex-direction: column; gap: 6px; color: var(--text-secondary); font-size: 10px; font-weight: 600; }
  .field input:not([type="color"]) { width: 100%; height: 34px; box-sizing: border-box; padding: 0 9px; border: 1px solid var(--border-subtle); border-radius: 7px; outline: none; background: rgba(0,0,0,.16); color: var(--text-primary); font: inherit; font-size: 12px; }
  .field input:focus { border-color: var(--accent-primary); }
  .color-field input { width: 100%; height: 34px; padding: 3px; border: 1px solid var(--border-subtle); border-radius: 7px; background: rgba(0,0,0,.16); cursor: pointer; }
  .primary-action, .secondary-action, .danger-action { min-height: 32px; padding: 7px 11px; border: 1px solid transparent; border-radius: 7px; font-size: 11px; font-weight: 650; white-space: nowrap; }
  .primary-action { background: var(--accent-primary); color: var(--accent-foreground); }
  .primary-action:hover:not(:disabled) { filter: brightness(1.08); }
  .secondary-action { border-color: var(--border-subtle); background: rgba(255,255,255,.04); color: var(--text-secondary); }
  .danger-action { border-color: rgba(244,63,94,.25); background: rgba(244,63,94,.1); color: #fb7185; }
  button:disabled { opacity: .55; cursor: default; }
  .action-spacer { flex: 1 1 auto; }
  .empty-state { margin: 2px 0; color: var(--text-secondary); font-size: 11px; }
  .delete-panel { display: flex; flex-direction: column; gap: 10px; padding: 13px; border: 1px solid rgba(244,63,94,.35); border-radius: 8px; background: rgba(244,63,94,.055); }
  .choice-row { display: flex; align-items: center; gap: 8px; color: var(--text-secondary); font-size: 11px; }
  .choice-row input { accent-color: var(--accent-primary); }
  .choice-row select { min-width: 120px; max-width: 220px; height: 29px; margin-left: auto; padding: 0 7px; border: 1px solid var(--border-subtle); border-radius: 6px; background: #202020; color: var(--text-primary); }
  .dialog-actions { justify-content: flex-end; margin-top: 2px; }
  @media (max-width: 700px) {
    .form-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .section-heading, .editor-heading { align-items: flex-start; }
  }
  @media (max-width: 480px) {
    .workspace-list { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .editor { padding: 12px; }
    .choice-row { align-items: flex-start; flex-wrap: wrap; }
    .choice-row select { margin-left: 24px; }
  }
</style>

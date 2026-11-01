<script lang="ts">
  import { get } from "svelte/store";
  import {
    bookmarkEditorState,
    bookmarkFolders,
    closeBookmarkEditor,
    createBookmarkFolder,
    deleteBookmarkFolder,
    renameBookmarkFolder,
    saveBookmark,
    removeBookmark,
  } from "../../stores/persistenceStore";

  let title = $state("");
  let url = $state("");
  let icon = $state("⭐");
  let selectedFolderId = $state<number | null>(null);
  let folderName = $state("");
  let newFolderName = $state("");
  let folderDrafts = $state<Record<number, string>>({});
  let errorMessage = $state("");
  let isSaving = $state(false);
  let pendingRemovalBookmarkId = $state<number | null>(null);

  $effect(() => {
    const editor = $bookmarkEditorState;
    pendingRemovalBookmarkId = null;
    if (editor?.kind === "save") {
      title = editor.title;
      url = editor.url;
      icon = editor.icon;
      selectedFolderId = editor.bookmark?.folder_id ?? null;
      errorMessage = "";
    } else if (editor?.kind === "rename-folder") {
      folderName = editor.name;
      errorMessage = "";
    }
  });

  $effect(() => {
    folderDrafts = Object.fromEntries($bookmarkFolders.map((folder) => [folder.id, folder.name]));
  });

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape" && get(bookmarkEditorState)) {
      event.preventDefault();
      void closeBookmarkEditor();
    }
  }

  function handleBackdropClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) void closeBookmarkEditor();
  }

  async function handleSave(): Promise<void> {
    const editor = get(bookmarkEditorState);
    if (editor?.kind !== "save") return;
    isSaving = true;
    errorMessage = "";
    try {
      await saveBookmark(editor.bookmark?.id ?? null, title, url, icon, selectedFolderId);
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    } finally {
      isSaving = false;
    }
  }

  async function handleRemove(): Promise<void> {
    const editor = get(bookmarkEditorState);
    if (editor?.kind !== "save" || !editor.bookmark) return;
    if (pendingRemovalBookmarkId !== editor.bookmark.id) {
      pendingRemovalBookmarkId = editor.bookmark.id;
      errorMessage = "";
      return;
    }
    isSaving = true;
    errorMessage = "";
    try {
      await removeBookmark(editor.bookmark.id);
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    } finally {
      isSaving = false;
    }
  }

  async function handleCreateFolder(): Promise<void> {
    errorMessage = "";
    try {
      await createBookmarkFolder(newFolderName);
      const createdFolder = get(bookmarkFolders).at(-1);
      if (createdFolder && get(bookmarkEditorState)?.kind === "save") {
        selectedFolderId = createdFolder.id;
      }
      newFolderName = "";
      if (get(bookmarkEditorState)?.kind === "create-folder") {
        await closeBookmarkEditor();
      }
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    }
  }

  async function handleRenameFolder(folderId: number): Promise<void> {
    errorMessage = "";
    try {
      await renameBookmarkFolder(folderId, folderDrafts[folderId] ?? "");
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    }
  }

  async function handleDeleteFolder(folderId: number): Promise<void> {
    const folder = get(bookmarkFolders).find((item) => item.id === folderId);
    if (!folder || !window.confirm(`Delete folder “${folder.name}”? Its bookmarks will be moved to the root.`)) return;
    errorMessage = "";
    try {
      await deleteBookmarkFolder(folderId);
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    }
  }

  async function handleRenameCurrentFolder(): Promise<void> {
    const editor = get(bookmarkEditorState);
    if (editor?.kind !== "rename-folder") return;
    errorMessage = "";
    try {
      await renameBookmarkFolder(editor.folderId, folderName);
      await closeBookmarkEditor();
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if $bookmarkEditorState}
  <div class="bookmark-dialog-backdrop" onclick={handleBackdropClick} role="presentation">
    <div
      class="bookmark-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="bookmark-dialog-title"
      onclick={(event) => event.stopPropagation()}
      onkeydown={(event) => event.stopPropagation()}
      tabindex="-1"
    >
      <header class="dialog-header">
        <h2 id="bookmark-dialog-title">
          {$bookmarkEditorState.kind === "folders"
            ? "Manage Bookmark Folders"
            : $bookmarkEditorState.kind === "create-folder"
              ? "Create Bookmark Folder"
            : $bookmarkEditorState.kind === "rename-folder"
              ? "Rename Folder"
              : $bookmarkEditorState.bookmark ? "Edit Bookmark" : "Save Bookmark"}
        </h2>
        <button class="icon-button" type="button" aria-label="Close" onclick={() => void closeBookmarkEditor()}>×</button>
      </header>

      {#if $bookmarkEditorState.kind === "save"}
        <div class="dialog-content">
          <label>
            <span>Title</span>
            <input bind:value={title} maxlength="200" autocomplete="off" />
          </label>
          <label>
            <span>Address</span>
            <input bind:value={url} maxlength="4096" autocomplete="off" />
          </label>
          <label>
            <span>Folder</span>
            <select bind:value={selectedFolderId}>
              <option value={null}>Bookmarks bar</option>
              {#each $bookmarkFolders as folder (folder.id)}
                <option value={folder.id}>{folder.name}</option>
              {/each}
            </select>
          </label>
          <details class="create-folder-inline">
            <summary>Create a folder</summary>
            <div class="create-folder-row">
              <input bind:value={newFolderName} maxlength="64" placeholder="Folder name" />
              <button class="secondary-button" type="button" onclick={handleCreateFolder}>Create</button>
            </div>
          </details>
          {#if errorMessage}<p class="error-message" role="alert">{errorMessage}</p>{/if}
          <footer class="dialog-actions">
            {#if $bookmarkEditorState.bookmark}
              <button
                class="danger-button"
                type="button"
                disabled={isSaving}
                aria-label={pendingRemovalBookmarkId === $bookmarkEditorState.bookmark.id ? "Confirm bookmark removal" : "Remove bookmark"}
                onclick={handleRemove}
              >
                {pendingRemovalBookmarkId === $bookmarkEditorState.bookmark.id ? "Confirm removal" : "Remove bookmark"}
              </button>
            {/if}
            <span class="action-spacer"></span>
            <button class="secondary-button" type="button" onclick={() => void closeBookmarkEditor()}>Cancel</button>
            <button class="primary-button" type="button" disabled={isSaving || !title.trim() || !url.trim()} onclick={handleSave}>
              {$bookmarkEditorState.bookmark ? "Save changes" : "Save bookmark"}
            </button>
          </footer>
        </div>
      {:else if $bookmarkEditorState.kind === "folders"}
        <div class="dialog-content">
          <div class="create-folder-row">
            <input bind:value={newFolderName} maxlength="64" placeholder="New folder name" />
            <button class="primary-button" type="button" onclick={handleCreateFolder}>Create folder</button>
          </div>
          {#if $bookmarkFolders.length === 0}
            <p class="empty-message">No folders yet.</p>
          {:else}
            <div class="folder-manager-list">
              {#each $bookmarkFolders as folder (folder.id)}
                <div class="folder-manager-row">
                  <span aria-hidden="true">📁</span>
                  <input
                    value={folderDrafts[folder.id] ?? folder.name}
                    maxlength="64"
                    aria-label={`Folder name: ${folder.name}`}
                    oninput={(event) => folderDrafts[folder.id] = event.currentTarget.value}
                  />
                  <button class="secondary-button" type="button" onclick={() => handleRenameFolder(folder.id)}>Rename</button>
                  <button class="danger-button" type="button" onclick={() => handleDeleteFolder(folder.id)}>Delete</button>
                </div>
              {/each}
            </div>
          {/if}
          {#if errorMessage}<p class="error-message" role="alert">{errorMessage}</p>{/if}
          <footer class="dialog-actions">
            <span class="action-spacer"></span>
            <button class="primary-button" type="button" onclick={() => void closeBookmarkEditor()}>Done</button>
          </footer>
        </div>
      {:else if $bookmarkEditorState.kind === "create-folder"}
        <div class="dialog-content">
          <label>
            <span>Folder name</span>
            <input bind:value={newFolderName} maxlength="64" autocomplete="off" />
          </label>
          {#if errorMessage}<p class="error-message" role="alert">{errorMessage}</p>{/if}
          <footer class="dialog-actions">
            <span class="action-spacer"></span>
            <button class="secondary-button" type="button" onclick={() => void closeBookmarkEditor()}>Cancel</button>
            <button class="primary-button" type="button" disabled={!newFolderName.trim()} onclick={handleCreateFolder}>Create folder</button>
          </footer>
        </div>
      {:else}
        <div class="dialog-content">
          <label>
            <span>Folder name</span>
            <input bind:value={folderName} maxlength="64" autocomplete="off" />
          </label>
          {#if errorMessage}<p class="error-message" role="alert">{errorMessage}</p>{/if}
          <footer class="dialog-actions">
            <span class="action-spacer"></span>
            <button class="secondary-button" type="button" onclick={() => void closeBookmarkEditor()}>Cancel</button>
            <button class="primary-button" type="button" disabled={!folderName.trim()} onclick={handleRenameCurrentFolder}>Save changes</button>
          </footer>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .bookmark-dialog-backdrop {
    position: fixed;
    inset: 0;
    z-index: 2100;
    display: grid;
    place-items: center;
    padding: 20px;
    background: rgba(0, 0, 0, 0.42);
    backdrop-filter: blur(var(--overlay-backdrop-blur, 16px));
  }

  .bookmark-dialog {
    width: min(480px, 100%);
    max-height: min(680px, 90vh);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    background: var(--bg-glass-primary);
    box-shadow: 0 24px 80px rgba(0, 0, 0, 0.42);
    color: var(--text-primary);
  }

  .dialog-header,
  .dialog-actions,
  .create-folder-row,
  .folder-manager-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .dialog-header {
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border-subtle);
  }

  h2 { margin: 0; font-size: 14px; font-weight: 700; }

  .dialog-content {
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow: auto;
    padding: 16px;
  }

  label { display: grid; gap: 6px; color: var(--text-secondary); font-size: 11px; }
  input, select {
    width: 100%;
    min-width: 0;
    padding: 9px 10px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    background: var(--bg-glass-secondary);
    color: var(--text-primary);
    font-size: 12px;
  }

  select, select option { color-scheme: dark; background: #202127; color: #f4f4f5; }
  .create-folder-inline summary { color: var(--text-secondary); cursor: pointer; font-size: 11px; }
  .create-folder-row { margin-top: 9px; }
  .folder-manager-list { display: grid; gap: 8px; max-height: 300px; overflow: auto; }
  .folder-manager-row { padding: 8px; border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); }
  .folder-manager-row input { flex: 1; }
  .dialog-actions { margin-top: 4px; }
  .action-spacer { flex: 1; }
  button { flex: 0 0 auto; padding: 7px 10px; border-radius: var(--radius-sm); font-size: 11px; }
  .icon-button { padding: 2px 6px; color: var(--text-muted); font-size: 20px; }
  .primary-button { background: var(--accent-primary); color: var(--accent-foreground); font-weight: 700; }
  .secondary-button { border: 1px solid var(--border-subtle); background: var(--bg-glass-hover); color: var(--text-primary); }
  .danger-button { border: 1px solid rgba(239, 68, 68, 0.38); background: rgba(239, 68, 68, 0.12); color: #fca5a5; }
  button:disabled { opacity: 0.55; cursor: not-allowed; }
  .empty-message { margin: 8px 0; color: var(--text-muted); font-size: 12px; }
  .error-message { margin: 0; color: #fca5a5; font-size: 11px; }

  @media (max-width: 520px) {
    .folder-manager-row { flex-wrap: wrap; }
    .folder-manager-row input { flex-basis: 100%; }
    .dialog-actions { flex-wrap: wrap; }
  }
</style>

<script lang="ts">
  import { updateState, updateBusy, updateActionError, checkUpdates, selectUpdateChannel, startUpdateDownload, startUpdateInstall } from "../../stores/updatesStore";
  import type { UpdateChannel } from "../../types/updates";
  const active = $derived($updateBusy || ["checking", "downloading", "installing"].includes($updateState.status));
  const status = $derived.by(() => {
    switch ($updateState.status) {
      case "checking": return "Checking GitHub Releases…";
      case "up_to_date": return "No newer version is available for this channel.";
      case "available": return "An update is available.";
      case "downloading": return "Downloading and verifying the update…";
      case "ready": return "The signed update is ready to install.";
      case "installing": return "Saving your session and starting the installer…";
      case "error": return "The update could not be completed.";
      case "unavailable": return $updateState.message;
      default: return "Check for a newer version of Veloce Browser.";
    }
  });
  function date(timestamp: number) { return new Date(timestamp * 1000).toLocaleString(); }
</script>

<section class="setting-group" aria-labelledby="updates-heading">
  <h2 id="updates-heading" class="group-title">About & Updates</h2>
  <p class="version">Veloce Browser {$updateState.current_version}</p>
  <div class="setting-row">
    <div class="row-text">
      <label for="update-channel" class="row-label">Release channel</label>
      <span class="row-desc">Beta includes previews and newer stable releases. Both channels share your local data.</span>
    </div>
    <select id="update-channel" class="engine-select" value={$updateState.channel} disabled={!$updateState.supported || active}
      onchange={event => { void selectUpdateChannel(event.currentTarget.value as UpdateChannel); }}>
      <option value="stable">Stable</option><option value="beta">Beta</option>
    </select>
  </div>
  {#if $updateState.channel === "stable" && $updateState.current_version.includes("-beta.")}
    <p>You will stay on this beta until a newer stable release is available. Changing channels never installs an older version.</p>
  {/if}
  <p role="status" aria-live="polite">{status}</p>
  <p>Last check: {$updateState.last_checked ? date($updateState.last_checked) : "Not checked yet"}</p>
  {#if $updateState.error || $updateActionError}<p role="alert">{$updateActionError || $updateState.error}</p>{/if}
  {#if $updateState.retry_after}<p>Try again after {date($updateState.retry_after)}.</p>{/if}
  <div class="actions">
    <button class="pill-action" disabled={!$updateState.supported || active} onclick={() => { void checkUpdates(); }}>Check for updates</button>
    {#if $updateState.update}
      {#if $updateState.status === "ready"}
        <button class="pill-action" disabled={active} onclick={() => { void startUpdateInstall(); }}>Install and restart</button>
      {:else}
        <button class="pill-action" disabled={active} onclick={() => { void startUpdateDownload(); }}>Download update</button>
      {/if}
    {/if}
  </div>
  {#if $updateState.status === "downloading"}
    <progress aria-label="Update download" max={$updateState.total ?? 1} value={$updateState.total ? $updateState.received : undefined}></progress>
    <p>{($updateState.received / 1048576).toFixed(1)} MB{$updateState.total ? ` / ${($updateState.total / 1048576).toFixed(1)} MB` : ""}</p>
  {/if}
  {#if $updateState.update}
    <h3>Version {$updateState.update.version}</h3>
    <pre>{$updateState.update.notes || "No release notes provided."}</pre>
  {/if}
  <p>Updates are checked when the browser opens. Downloading and installing require your confirmation. Installing restarts the browser; page forms and media are not restored.</p>
</section>

<style>
  p, pre { color: var(--text-secondary); font-size: 12px; line-height: 1.6; }
  .version { color: var(--text-primary); font-size: 16px; }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin: 12px 0; }
  pre { white-space: pre-wrap; overflow-wrap: anywhere; font-family: inherit; max-height: 260px; overflow: auto; }
  progress { width: 100%; }
  [role="alert"] { color: #ef9a9a; }
</style>

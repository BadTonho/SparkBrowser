<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { invoke, isTauriRuntime } from "../../services/bridge/runtime";
  import { describeError } from "../../services/user-errors.js";
  let exceptions = $state<string[] | null>(null);
  let domain = $state("");
  let busy = $state(false);
  let error = $state("");
  onMount(() => { if (isTauriRuntime()) void invoke<string[]>("get_shield_exceptions").then(value => exceptions = value).catch(cause => error = describeError(cause)); });
  async function save(next: string[]) {
    if (!window.confirm($t("settings.gpc_reload_confirm"))) return;
    busy = true; error = "";
    try { exceptions = await invoke<string[]>("set_shield_exceptions", { exceptions: next }); domain = ""; }
    catch (cause) { error = describeError(cause); }
    finally { busy = false; }
  }
</script>
<div class="exception-editor" role="group" aria-labelledby="shield-exceptions-title">
  <label class="exception-label" id="shield-exceptions-title" for="shield-exception-input">{$t("settings.shield_exceptions")}</label>
  <div class="exception-input-row">
    <input id="shield-exception-input" bind:value={domain} placeholder={$t("settings.tracking_cleaning_placeholder")} disabled={!exceptions || busy} autocomplete="off" />
    <button class="pill-action" type="button" disabled={!exceptions || busy || !domain.trim()} onclick={() => { if (exceptions) void save([...exceptions, domain.trim()]); }}>{$t("settings.tracking_cleaning_add")}</button>
  </div>
  {#if exceptions?.length}
    <ul class="exception-list">{#each exceptions as exception (exception)}<li>{exception}<button class="exception-remove" type="button" disabled={busy} aria-label={$t("settings.tracking_cleaning_remove", { domain: exception })} onclick={() => { if (exceptions) void save(exceptions.filter(value => value !== exception)); }}>×</button></li>{/each}</ul>
  {/if}
  {#if error}<p class="settings-message" role="alert">{error}</p>{/if}
</div>

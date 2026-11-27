<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { invoke, isTauriRuntime } from "../../services/bridge/runtime";
  import { describeError } from "../../services/user-errors.js";
  type Settings = { enabled: boolean; exceptions: string[] };
  let settings = $state<Settings | null>(null);
  let domain = $state("");
  let busy = $state(false);
  let error = $state("");
  onMount(() => { if (isTauriRuntime()) void invoke<Settings>("get_tracking_cleaning").then(value => settings = value).catch(cause => error = describeError(cause)); });
  async function save(value: Settings) {
    if (!window.confirm($t("settings.gpc_reload_confirm"))) return;
    busy = true; error = "";
    try { settings = await invoke<Settings>("set_tracking_cleaning", { settings: value }); }
    catch (cause) { error = describeError(cause); }
    finally { busy = false; }
  }
  async function add() {
    if (!settings || !domain.trim()) return;
    const before = settings;
    await save({ ...settings, exceptions: [...settings.exceptions, domain.trim()] });
    if (settings !== before) domain = "";
  }
  function changeEnabled(event: Event) {
    if (!settings) return;
    const input = event.currentTarget as HTMLInputElement;
    const enabled = input.checked;
    input.checked = settings.enabled;
    void save({ ...settings, enabled });
  }
</script>
<section class="setting-card" id="setting-tracking-cleaning" aria-labelledby="tracking-cleaning-title">
  <label class="setting-card-head">
    <span class="row-text"><span class="row-label" id="tracking-cleaning-title">{$t("settings.tracking_cleaning")}</span><span class="row-desc">{$t("settings.tracking_cleaning_description")}</span></span>
    <input type="checkbox" role="switch" aria-labelledby="tracking-cleaning-title" checked={settings?.enabled ?? false} disabled={!settings || busy} onchange={changeEnabled} />
  </label>
  <div class="exception-editor">
    <label class="exception-label" for="tracking-exception-input">{$t("settings.tracking_cleaning_exceptions")}</label>
    <div class="exception-input-row">
      <input id="tracking-exception-input" bind:value={domain} placeholder={$t("settings.tracking_cleaning_placeholder")} disabled={!settings || busy} autocomplete="off" />
      <button class="pill-action" type="button" disabled={!settings || busy || !domain.trim()} onclick={add}>{$t("settings.tracking_cleaning_add")}</button>
    </div>
    {#if settings?.exceptions.length}
      <ul class="exception-list">{#each settings.exceptions as exception (exception)}<li>{exception}<button class="exception-remove" type="button" disabled={busy} aria-label={$t("settings.tracking_cleaning_remove", { domain: exception })} onclick={() => { if (settings) void save({ ...settings, exceptions: settings.exceptions.filter(value => value !== exception) }); }}>×</button></li>{/each}</ul>
    {/if}
  </div>
  {#if error}<p class="settings-message" role="alert">{error}</p>{/if}
</section>

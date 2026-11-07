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
<section class="tracking-cleaning" aria-labelledby="tracking-cleaning-title">
  <label class="tracking-option">
    <span><strong id="tracking-cleaning-title">{$t("settings.tracking_cleaning")}</strong><small>{$t("settings.tracking_cleaning_description")}</small></span>
    <input type="checkbox" role="switch" checked={settings?.enabled ?? false} disabled={!settings || busy} onchange={changeEnabled} />
  </label>
  <label class="tracking-field">{$t("settings.tracking_cleaning_exceptions")}
    <input bind:value={domain} placeholder={$t("settings.tracking_cleaning_placeholder")} disabled={!settings || busy} autocomplete="off" />
  </label>
  <button type="button" disabled={!settings || busy || !domain.trim()} onclick={add}>{$t("settings.tracking_cleaning_add")}</button>
  <ul>{#each settings?.exceptions ?? [] as exception (exception)}<li>{exception} <button type="button" disabled={busy} aria-label={$t("settings.tracking_cleaning_remove", { domain: exception })} onclick={() => { if (settings) void save({ ...settings, exceptions: settings.exceptions.filter(value => value !== exception) }); }}>×</button></li>{/each}</ul>
  {#if error}<p role="alert">{error}</p>{/if}
</section>
<style>
  .tracking-cleaning { padding: 12px 14px; border-bottom: 1px solid var(--border-subtle); font-size: 12px; }
  .tracking-option { display:flex; justify-content:space-between; gap:18px; align-items:center; }
  .tracking-option span, .tracking-field { display:grid; gap:5px; }
  small { color:var(--text-secondary); line-height:1.45; }
  .tracking-field { margin-top:12px; color:var(--text-secondary); }
  .tracking-field input { max-width:250px; padding:6px; color:var(--text-primary); background:rgba(255,255,255,.04); border:1px solid var(--border-subtle); border-radius:4px; }
  button { margin-top:6px; padding:5px 9px; color:var(--text-primary); background:rgba(255,255,255,.04); border:1px solid var(--border-subtle); border-radius:4px; }
  ul { padding-left:18px; } p { color:var(--text-secondary); }
</style>

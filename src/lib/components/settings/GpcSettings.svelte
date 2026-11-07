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
  onMount(() => { if (isTauriRuntime()) void invoke<Settings>("get_gpc").then(value => settings = value).catch(cause => error = describeError(cause)); });
  async function save(value: Settings) {
    if (!window.confirm($t("settings.gpc_reload_confirm"))) return;
    busy = true; error = "";
    try { settings = await invoke<Settings>("set_gpc", { settings: value }); }
    catch (cause) { error = describeError(cause); }
    finally { busy = false; }
  }
  function toggle(event: Event) {
    if (!settings) return;
    const input = event.currentTarget as HTMLInputElement;
    const enabled = input.checked; input.checked = settings.enabled;
    void save({ ...settings, enabled });
  }
</script>
<section class="gpc" aria-labelledby="gpc-title">
  <label class="option"><span><strong id="gpc-title">{$t("settings.gpc")}</strong><small>{$t("settings.gpc_coverage")}</small></span><input type="checkbox" role="switch" checked={settings?.enabled ?? false} disabled={!settings || busy} onchange={toggle} /></label>
  <label class="field">{$t("settings.tracking_cleaning_exceptions")}<input bind:value={domain} placeholder={$t("settings.tracking_cleaning_placeholder")} disabled={!settings || busy} autocomplete="off" /></label>
  <button type="button" disabled={!settings || busy || !domain.trim()} onclick={() => { if (settings) void save({ ...settings, exceptions: [...settings.exceptions, domain.trim()] }); }} >{$t("settings.tracking_cleaning_add")}</button>
  <ul>{#each settings?.exceptions ?? [] as exception (exception)}<li>{exception} <button type="button" disabled={busy} aria-label={$t("settings.tracking_cleaning_remove", { domain: exception })} onclick={() => { if (settings) void save({ ...settings, exceptions: settings.exceptions.filter(value => value !== exception) }); }}>×</button></li>{/each}</ul>
  {#if error}<p role="alert">{error}</p>{/if}
</section>
<style>
  .gpc { padding:12px 14px; border-bottom:1px solid var(--border-subtle); font-size:12px; }
  .option { display:flex; justify-content:space-between; gap:18px; align-items:center; }
  .option span, .field { display:grid; gap:5px; } small { color:var(--text-secondary); line-height:1.45; }
  .field { margin-top:12px; color:var(--text-secondary); }
  .field input { max-width:250px; padding:6px; color:var(--text-primary); background:rgba(255,255,255,.04); border:1px solid var(--border-subtle); border-radius:4px; }
  button { margin-top:6px; padding:5px 9px; color:var(--text-primary); background:rgba(255,255,255,.04); border:1px solid var(--border-subtle); border-radius:4px; }
  ul { padding-left:18px; } p { color:var(--text-secondary); }
</style>

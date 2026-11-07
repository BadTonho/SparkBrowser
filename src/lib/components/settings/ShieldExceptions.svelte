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
<section aria-labelledby="shield-exceptions-title">
  <label id="shield-exceptions-title" class="field">{$t("settings.shield_exceptions")}
    <input bind:value={domain} placeholder={$t("settings.tracking_cleaning_placeholder")} disabled={!exceptions || busy} autocomplete="off" />
  </label>
  <button type="button" disabled={!exceptions || busy || !domain.trim()} onclick={() => { if (exceptions) void save([...exceptions, domain.trim()]); }}>{$t("settings.tracking_cleaning_add")}</button>
  <ul>{#each exceptions ?? [] as exception (exception)}<li>{exception} <button type="button" disabled={busy} aria-label={$t("settings.tracking_cleaning_remove", { domain: exception })} onclick={() => { if (exceptions) void save(exceptions.filter(value => value !== exception)); }}>×</button></li>{/each}</ul>
  {#if error}<p role="alert">{error}</p>{/if}
</section>
<style>
  section { padding:12px 14px; border-bottom:1px solid var(--border-subtle); font-size:12px; }
  .field { display:grid; gap:5px; color:var(--text-secondary); }
  input { max-width:250px; padding:6px; color:var(--text-primary); background:rgba(255,255,255,.04); border:1px solid var(--border-subtle); border-radius:4px; }
  button { margin-top:6px; padding:5px 9px; color:var(--text-primary); background:rgba(255,255,255,.04); border:1px solid var(--border-subtle); border-radius:4px; }
  ul { padding-left:18px; } p { color:var(--text-secondary); }
</style>

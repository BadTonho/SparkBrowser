<script lang="ts">
  import {t} from "../../i18n";
  import {pipWindowShadow,pipShadowAvailable,pipShadowBusy,pipShadowError,setPipWindowShadow} from "../../stores/pipAppearanceStore";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import {logIgnored} from "../../services/ignored-error";
</script>

<div class="setting-row" id="setting-pip-window-shadow" tabindex="-1">
  <div class="row-text">
    <span class="row-label">{$t("settings.pip_window_shadow")}</span>
    <span class="row-desc">{$t("settings.pip_window_shadow_description")}</span>
    {#if !$pipShadowAvailable && !$pipShadowBusy}<span class="row-desc">{$t("settings.pip_window_shadow_unavailable")}</span>{/if}
  </div>
  <button class="pill-action" class:active={$pipWindowShadow} data-testid="setting-pip-shadow" aria-label={$t("settings.pip_window_shadow")} aria-pressed={$pipWindowShadow} disabled={!$pipShadowAvailable || $pipShadowBusy} onclick={()=>{void setPipWindowShadow(!$pipWindowShadow).catch(logIgnored);}}>
    {$pipWindowShadow ? $t("settings.on") : $t("settings.off_ca79")}
  </button>
</div>
<ErrorMessage error={$pipShadowError} />

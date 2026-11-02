<script lang="ts">
  import { t } from "../../i18n";
  import { errorText, technicalError } from "../../i18n/errors";
  let { error }: { error: unknown } = $props();
</script>
{#if error}
  <div class="error-message">
    <p role="alert">{$errorText(error)}</p>
    <details><summary>{$t("common.technical_details")}</summary><pre>{technicalError(error)}</pre></details>
  </div>
{/if}
<style>
  .error-message { color: #ef9a9a; font-size: 12px; }
  pre { white-space: pre-wrap; overflow-wrap: anywhere; max-height: 180px; overflow: auto; }
</style>

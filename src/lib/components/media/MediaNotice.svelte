<script lang="ts">
  import { t } from "../../i18n";
  import { errorText } from "../../i18n/errors";
  import { mediaError } from "../../stores/mediaStore";
</script>
{#if $mediaError}
  <div class="media-notice" role="alert"><span>{$errorText($mediaError)}</span><button onclick={() => mediaError.set("")} aria-label={$t("media.close")}>×</button></div>
{/if}
<style>.media-notice { display:flex; justify-content:space-between; padding:6px 12px; background:var(--bg-glass-primary); color:var(--text-primary); }</style>

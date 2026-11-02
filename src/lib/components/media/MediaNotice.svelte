<script lang="ts">
  import { t } from "../../i18n";
  import { errorText } from "../../i18n/errors";
  import { mediaError } from "../../stores/mediaStore";
</script>
{#if $mediaError}
  <div class="media-notice" role="alert"><span>{$errorText($mediaError)}</span><button onclick={() => mediaError.set("")} aria-label={$t("media.close")}>×</button></div>
{/if}
<style>.media-notice { position:fixed;top:8px;left:50%;transform:translateX(-50%);z-index:3000;max-width:min(640px,85vw);display:flex;gap:12px;justify-content:space-between;padding:8px 12px;border:1px solid var(--border-subtle);border-radius:8px;background:var(--bg-glass-elevated);backdrop-filter:blur(16px);color:var(--text-primary); }</style>

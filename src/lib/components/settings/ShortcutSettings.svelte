<script lang="ts">
import { errorText } from "../../i18n/errors";
  import { t, labelText } from "../../i18n";
import {
  SHORTCUT_DEFINITIONS,
  formatShortcut,
  getShortcutBinding,
  keyboardShortcuts,
  recordingShortcutId,
} from "../../stores/shortcutsStore";
let { error, onStart }: { error: string; onStart: (id: string) => void } = $props();
</script>

<section class="setting-group" aria-labelledby="shortcuts-heading">
  <h2 class="group-title" id="shortcuts-heading">{$t("settings.keyboard_shortcuts_4e09")}</h2>
  <div class="shortcuts-grid">
    {#each SHORTCUT_DEFINITIONS as item}
      {@const binding = getShortcutBinding($keyboardShortcuts, item.id)}
      <div class="shortcut-row" id={`shortcut-${item.id}`} tabindex="-1">
        <span class="sc-desc">{$labelText(item.label)}</span>
        <div class="shortcut-editor">
          <button
            type="button"
            class="sc-keys"
            class:recording={$recordingShortcutId === item.id}
            aria-label={$t("settings.change_shortcut_for_p0_c56e", { p0: $labelText(item.label) })}
            aria-pressed={$recordingShortcutId === item.id}
            onclick={() => onStart(item.id)}
          >
            {$recordingShortcutId === item.id
              ? $t("settings.press_a_key_or_mouse_4_5_esc_cancels_4126")
              : item.hold
                ? $t("common.hold_shortcut", { key: binding?.key === "Control" ? "Ctrl" : binding?.key ?? "Alt" })
                : binding
                  ? formatShortcut(binding, item.sequence)
                  : $t("settings.unassigned_14d3")}
          </button>
          {#if $recordingShortcutId === item.id && error}
            <span class="shortcut-error" role="status">{$errorText(error)}</span>
          {/if}
        </div>
      </div>
    {/each}
  </div>
</section>

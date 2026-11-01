<script lang="ts">
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
  <h2 class="group-title" id="shortcuts-heading">⌨️ Keyboard Shortcuts</h2>
  <div class="shortcuts-grid">
    {#each SHORTCUT_DEFINITIONS as item}
      {@const binding = getShortcutBinding($keyboardShortcuts, item.id)}
      <div class="shortcut-row" id={`shortcut-${item.id}`} tabindex="-1">
        <span class="sc-desc">{item.label}</span>
        <div class="shortcut-editor">
          <button
            type="button"
            class="sc-keys"
            class:recording={$recordingShortcutId === item.id}
            aria-label={`Change shortcut for ${item.label}`}
            aria-pressed={$recordingShortcutId === item.id}
            onclick={() => onStart(item.id)}
          >
            {$recordingShortcutId === item.id
              ? "Press a key or Mouse 4/5… (Esc cancels)"
              : item.hold
                ? `Hold ${binding?.key === "Control" ? "Ctrl" : binding?.key ?? "Alt"} (0.4 sec)`
                : binding
                  ? formatShortcut(binding, item.sequence)
                  : "Unassigned"}
          </button>
          {#if $recordingShortcutId === item.id && error}
            <span class="shortcut-error" role="status">{error}</span>
          {/if}
        </div>
      </div>
    {/each}
  </div>
</section>

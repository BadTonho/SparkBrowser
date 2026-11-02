<script lang="ts">
  import { t } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { isToolsOpen, toolsSection, closeTools, type ToolsSection } from "../../stores/toolsStore";
  import TabsPanel from "./TabsPanel.svelte";
  import SessionsPanel from "./SessionsPanel.svelte";
  import NotesPanel from "./NotesPanel.svelte";
  import "./tools.css";
  const sections: { id: ToolsSection; label: string }[] = $derived([{ id: "sessions", label: $t("tools.sessions_6fa3") }, { id: "tabs", label: $t("tools.tabs_8e5e") }, { id: "notes", label: $t("tools.notes_8a75") }]);
</script>

{#if $isToolsOpen}
  <div class="tools-backdrop" role="presentation" onclick={(event) => { if (event.target === event.currentTarget) void closeTools(); }}>
    <div data-testid="tools-dialog" class="tools-panel" role="dialog" aria-modal="true" aria-label={$t("tools.tools_ea93")} tabindex="-1" use:focusDialog={{ onEscape: closeTools }}>
      <header><h2>{$t("tools.tools_ea93")}</h2><button onclick={closeTools} data-testid="close-tools" aria-label={$t("tools.close_tools_3f06")}>{$t("tools.close_7d9e")}</button></header>
      <nav aria-label={$t("tools.tools_sections_9c6a")}>
        {#each sections as section}<button data-testid={`tools-${section.id}`} aria-pressed={$toolsSection === section.id} onclick={() => toolsSection.set(section.id)}>{section.label}</button>{/each}
      </nav>
      <div class="tools-content">
        {#if $toolsSection === "sessions"}<SessionsPanel />
        {:else if $toolsSection === "notes"}<NotesPanel />
        {:else}<TabsPanel />{/if}
      </div>
    </div>
  </div>
{/if}

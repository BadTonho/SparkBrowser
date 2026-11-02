<script lang="ts">
  import { focusDialog } from "../../actions/focusDialog";
  import { isToolsOpen, toolsSection, closeTools, type ToolsSection } from "../../stores/toolsStore";
  import TabsPanel from "./TabsPanel.svelte";
  import SessionsPanel from "./SessionsPanel.svelte";
  import NotesPanel from "./NotesPanel.svelte";
  import "./tools.css";
  const sections: { id: ToolsSection; label: string }[] = [{ id: "sessions", label: "Sessions" }, { id: "tabs", label: "Tabs" }, { id: "notes", label: "Notes" }];
</script>

{#if $isToolsOpen}
  <div class="tools-backdrop" role="presentation" onclick={(event) => { if (event.target === event.currentTarget) void closeTools(); }}>
    <div data-testid="tools-dialog" class="tools-panel" role="dialog" aria-modal="true" aria-label="Tools" tabindex="-1" use:focusDialog={{ onEscape: closeTools }}>
      <header><h2>Tools</h2><button onclick={closeTools} data-testid="close-tools" aria-label="Close tools">Close</button></header>
      <nav aria-label="Tools sections">
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

<script lang="ts">
import {
  isZenMode,
  toggleZenMode,
  overlayBlurPx,
  setOverlayBlurPx,
  tabLayout,
  setTabLayout,
  showBookmarksBar,
  setShowBookmarksBar,
  workspaceSwitcherDisplay,
  setWorkspaceSwitcherDisplay,
} from "../../stores/settingsStore";
import { formatShortcut, getShortcutBinding, keyboardShortcuts } from "../../stores/shortcutsStore";
</script>

<section class="setting-group" aria-labelledby="appearance-heading">
  <h2 class="group-title" id="appearance-heading">🖥️ Appearance & Layout</h2>

  <div class="setting-row" id="setting-browser-layout" tabindex="-1">
    <div class="row-text">
      <span class="row-label">Browser Layout</span>
      <span class="row-desc">Choose a horizontal browser layout or a vertical sidebar layout</span>
    </div>
    <div class="timer-pills">
      <button
        class="mini-pill"
        class:selected={$tabLayout === "horizontal"}
        onclick={() => setTabLayout("horizontal")}
        title="Tabs at the top, workspaces beside search, and bookmarks below"
      >
        ↔ Horizontal
      </button>
      <button
        class="mini-pill"
        class:selected={$tabLayout === "vertical"}
        onclick={() => setTabLayout("vertical")}
        title="Workspaces, bookmarks, and tabs in the sidebar"
      >
        ↕ Vertical
      </button>
    </div>
  </div>

  <div class="setting-row" id="setting-background-blur" tabindex="-1">
    <div class="row-text">
      <span class="row-label">Background Blur</span>
      <span class="row-desc">Adjust blur behind browser overlays</span>
    </div>
    <div class="range-control">
      <input
        type="range"
        min="0"
        max="24"
        step="1"
        value={$overlayBlurPx}
        oninput={(event) => setOverlayBlurPx(Number(event.currentTarget.value))}
        aria-label="Background blur intensity"
        aria-valuetext={`${$overlayBlurPx} pixels`}
      />
      <output>{ $overlayBlurPx } px</output>
    </div>
  </div>

  <div class="setting-row" id="setting-bookmarks-bar" tabindex="-1">
    <div class="row-text">
      <span class="row-label">Bookmarks Bar</span>
      <span class="row-desc">Show or hide the horizontal bookmarks bar without removing saved bookmarks</span>
    </div>
    <button
      class="pill-action"
      class:active={$showBookmarksBar}
      aria-pressed={$showBookmarksBar}
      onclick={() => setShowBookmarksBar(!$showBookmarksBar)}
    >
      {$showBookmarksBar ? "Visible" : "Hidden"}
    </button>
  </div>

  <div class="setting-row" id="setting-workspace-switcher-display" tabindex="-1">
    <div class="row-text">
      <span class="row-label">Workspace Switcher</span>
      <span class="row-desc">Choose horizontal buttons or a floating picker; vertical sidebar buttons stay visible</span>
    </div>
    <div class="timer-pills">
      <button
        class="mini-pill"
        class:selected={$workspaceSwitcherDisplay === "toolbar"}
        aria-pressed={$workspaceSwitcherDisplay === "toolbar"}
        onclick={() => setWorkspaceSwitcherDisplay("toolbar")}
      >
        Toolbar buttons
      </button>
      <button
        class="mini-pill"
        class:selected={$workspaceSwitcherDisplay === "popup"}
        aria-pressed={$workspaceSwitcherDisplay === "popup"}
        onclick={() => setWorkspaceSwitcherDisplay("popup")}
      >
        Popup
      </button>
    </div>
  </div>

  <div class="setting-row" id="setting-zen-mode" tabindex="-1">
    <div class="row-text">
      <span class="row-label">Zen / Focus Mode ({$keyboardShortcuts && formatShortcut(getShortcutBinding($keyboardShortcuts, "zen")!)})</span>
      <span class="row-desc">Hides all browser chrome so websites use 100% of the viewport</span>
    </div>
    <button
      class="pill-action"
      class:active={$isZenMode}
      onclick={toggleZenMode}
    >
      {$isZenMode ? "Active" : "Off"}
    </button>
  </div>
</section>

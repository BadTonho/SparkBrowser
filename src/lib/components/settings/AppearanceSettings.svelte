<script lang="ts">
import LanguageSettings from "./LanguageSettings.svelte";
  import { t } from "../../i18n";
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
  <h2 class="group-title" id="appearance-heading">{$t("settings.appearance_layout_a2b1")}</h2>

  <LanguageSettings />

  <div class="setting-row" id="setting-browser-layout" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.browser_layout_28d1")}</span>
      <span class="row-desc">{$t("settings.choose_a_horizontal_browser_layout_or_a_vertical_sidebar_layout_a4a5")}</span>
    </div>
    <div class="timer-pills">
      <button
        class="mini-pill"
        class:selected={$tabLayout === "horizontal"}
        onclick={() => setTabLayout("horizontal")}
        title={$t("settings.tabs_at_the_top_workspaces_beside_search_and_bookmarks_below_e1af")}
      >
        {$t("settings.horizontal_9b55")}
      </button>
      <button
        class="mini-pill"
        class:selected={$tabLayout === "vertical"}
        onclick={() => setTabLayout("vertical")}
        title={$t("settings.workspaces_bookmarks_and_tabs_in_the_sidebar_5cff")}
      >
        {$t("settings.vertical_b6d6")}
      </button>
    </div>
  </div>

  <div class="setting-row" id="setting-background-blur" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.background_blur_c21c")}</span>
      <span class="row-desc">{$t("settings.adjust_blur_behind_browser_overlays_4f2a")}</span>
    </div>
    <div class="range-control">
      <input
        type="range"
        min="0"
        max="24"
        step="1"
        value={$overlayBlurPx}
        oninput={(event) => setOverlayBlurPx(Number(event.currentTarget.value))}
        aria-label={$t("settings.background_blur_intensity_9873")}
        aria-valuetext={$t("settings.p0_pixels_b580", { p0: $overlayBlurPx })}
      />
      <output>{ $overlayBlurPx } px</output>
    </div>
  </div>

  <div class="setting-row" id="setting-bookmarks-bar" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.bookmarks_bar_d670")}</span>
      <span class="row-desc">{$t("settings.show_or_hide_the_horizontal_bookmarks_bar_without_removing_saved_bookmar_85a7")}</span>
    </div>
    <button
      class="pill-action"
      class:active={$showBookmarksBar}
      aria-pressed={$showBookmarksBar}
      onclick={() => setShowBookmarksBar(!$showBookmarksBar)}
    >
      {$showBookmarksBar ? $t("settings.visible_8411") : $t("settings.hidden_7e6f")}
    </button>
  </div>

  <div class="setting-row" id="setting-workspace-switcher-display" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.workspace_switcher_43f1")}</span>
      <span class="row-desc">{$t("settings.choose_horizontal_buttons_or_a_floating_picker_vertical_sidebar_buttons__8411")}</span>
    </div>
    <div class="timer-pills">
      <button
        class="mini-pill"
        class:selected={$workspaceSwitcherDisplay === "toolbar"}
        aria-pressed={$workspaceSwitcherDisplay === "toolbar"}
        onclick={() => setWorkspaceSwitcherDisplay("toolbar")}
      >
        {$t("settings.toolbar_buttons_1962")}
      </button>
      <button
        class="mini-pill"
        class:selected={$workspaceSwitcherDisplay === "popup"}
        aria-pressed={$workspaceSwitcherDisplay === "popup"}
        onclick={() => setWorkspaceSwitcherDisplay("popup")}
      >
        {$t("settings.popup_4c14")}
      </button>
    </div>
  </div>

  <div class="setting-row" id="setting-zen-mode" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.zen_focus_mode_092d")}{$keyboardShortcuts && formatShortcut(getShortcutBinding($keyboardShortcuts, "zen")!)})</span>
      <span class="row-desc">{$t("settings.hides_all_browser_chrome_so_websites_use_100_of_the_viewport_3078")}</span>
    </div>
    <button
      class="pill-action"
      class:active={$isZenMode}
      onclick={toggleZenMode}
    >
      {$isZenMode ? $t("settings.active_9234") : $t("settings.off_ca79")}
    </button>
  </div>
</section>

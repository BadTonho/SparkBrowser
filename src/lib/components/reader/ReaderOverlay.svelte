<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { t } from "../../i18n";
  import { onOverlayOpening } from "../../services/bridge/overlays";
  import {
    isReaderOpen,
    readerArticle,
    readerLoading,
    readerError,
    readerTheme,
    readerFont,
    readerFontSize,
    readerWidth,
    closeReaderMode,
    changeReaderFontSize,
  } from "../../stores/readerStore";

  onMount(() => {
    return onOverlayOpening((id) => {
      if (id !== "reader-mode") void closeReaderMode();
    });
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      void closeReaderMode();
    }
  }

  const widthMap = {
    narrow: "640px",
    medium: "760px",
    wide: "940px",
  };

  const fontMap = {
    serif: "'Merriweather', 'Georgia', 'Cambria', serif",
    sans: "'Inter', system-ui, -apple-system, sans-serif",
    mono: "'Fira Code', 'Cascadia Code', monospace",
  };
</script>

<svelte:window onkeydown={handleKeydown} />

{#if $isReaderOpen}
  <div
    class="reader-overlay theme-{$readerTheme}"
    role="dialog"
    aria-modal="true"
    aria-label={$t("reader.title")}
    transition:fade={{ duration: 180 }}
  >
    <!-- Top toolbar -->
    <header class="reader-toolbar">
      <div class="toolbar-left">
        <button
          type="button"
          class="back-btn"
          onclick={closeReaderMode}
          title="{$t('reader.close')} (Esc)"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
          <span>{$t("reader.close")}</span>
        </button>

        {#if $readerArticle}
          <div class="meta-pills">
            <span class="read-time-pill">
              <AppIcon name="timer" /> {$t("reader.read_time", { mins: $readerArticle.reading_time_mins })}
            </span>
            <span class="word-count-pill">
              {$t("reader.words", { words: $readerArticle.word_count })}
            </span>
          </div>
        {/if}
      </div>

      <div class="toolbar-controls">
        <!-- Font family buttons -->
        <div class="control-group">
          <button
            type="button"
            class="control-btn"
            class:active={$readerFont === "serif"}
            onclick={() => ($readerFont = "serif")}
            title={$t("reader.font_serif")}
          >
            Serif
          </button>
          <button
            type="button"
            class="control-btn"
            class:active={$readerFont === "sans"}
            onclick={() => ($readerFont = "sans")}
            title={$t("reader.font_sans")}
          >
            Sans
          </button>
          <button
            type="button"
            class="control-btn"
            class:active={$readerFont === "mono"}
            onclick={() => ($readerFont = "mono")}
            title={$t("reader.font_mono")}
          >
            Mono
          </button>
        </div>

        <!-- Font size controls -->
        <div class="control-group">
          <button
            type="button"
            class="icon-btn"
            disabled={$readerFontSize <= 14}
            onclick={() => changeReaderFontSize(-1)}
            title={$t("reader.decrease_font")}
          >
            A-
          </button>
          <span class="font-size-label">{$readerFontSize}px</span>
          <button
            type="button"
            class="icon-btn"
            disabled={$readerFontSize >= 28}
            onclick={() => changeReaderFontSize(1)}
            title={$t("reader.increase_font")}
          >
            A+
          </button>
        </div>

        <!-- Column width -->
        <div class="control-group">
          <button
            type="button"
            class="control-btn"
            class:active={$readerWidth === "narrow"}
            onclick={() => ($readerWidth = "narrow")}
            title={$t("reader.width_narrow")}
          >
            S
          </button>
          <button
            type="button"
            class="control-btn"
            class:active={$readerWidth === "medium"}
            onclick={() => ($readerWidth = "medium")}
            title={$t("reader.width_medium")}
          >
            M
          </button>
          <button
            type="button"
            class="control-btn"
            class:active={$readerWidth === "wide"}
            onclick={() => ($readerWidth = "wide")}
            title={$t("reader.width_wide")}
          >
            L
          </button>
        </div>

        <!-- Themes -->
        <div class="theme-picker">
          <button
            type="button"
            class="theme-dot theme-paper"
            class:active={$readerTheme === "paper"}
            onclick={() => ($readerTheme = "paper")}
            title={$t("reader.theme_paper")}
          ></button>
          <button
            type="button"
            class="theme-dot theme-sepia"
            class:active={$readerTheme === "sepia"}
            onclick={() => ($readerTheme = "sepia")}
            title={$t("reader.theme_sepia")}
          ></button>
          <button
            type="button"
            class="theme-dot theme-dark"
            class:active={$readerTheme === "dark"}
            onclick={() => ($readerTheme = "dark")}
            title={$t("reader.theme_dark")}
          ></button>
          <button
            type="button"
            class="theme-dot theme-oled"
            class:active={$readerTheme === "oled"}
            onclick={() => ($readerTheme = "oled")}
            title={$t("reader.theme_oled")}
          ></button>
        </div>
      </div>
    </header>

    <!-- Reading Canvas Body -->
    <main class="reader-scroll-canvas">
      <div
        class="reader-article-container"
        style:max-width={widthMap[$readerWidth]}
        style:font-family={fontMap[$readerFont]}
        style:font-size="{$readerFontSize}px"
        transition:fly={{ y: 15, duration: 220, easing: cubicOut }}
      >
        {#if $readerLoading}
          <div class="loading-state">
            <div class="reader-spinner"></div>
            <p>{$t("reader.loading")}</p>
          </div>
        {:else if $readerError}
          <div class="error-state">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="40" height="40">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="8" x2="12" y2="12" />
              <line x1="12" y1="16" x2="12.01" y2="16" />
            </svg>
            <h3>{$t("reader.error_title")}</h3>
            <p>{$readerError}</p>
            <p class="error-hint">{$t("reader.error_hint")}</p>
            <button type="button" class="close-error-btn" onclick={closeReaderMode}>
              {$t("reader.close")}
            </button>
          </div>
        {:else if $readerArticle}
          <!-- Article Header -->
          <div class="article-header">
            {#if $readerArticle.site_name}
              <div class="article-site-tag">{$readerArticle.site_name}</div>
            {/if}
            <h1 class="article-title">{$readerArticle.title}</h1>
            {#if $readerArticle.byline}
              <div class="article-byline">Por {$readerArticle.byline}</div>
            {/if}
            <div class="article-divider"></div>
          </div>

          <!-- Article Content Body -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <article class="article-body">
            {@html $readerArticle.content_html}
          </article>
        {/if}
      </div>
    </main>
  </div>
{/if}

<style>
  .reader-overlay {
    position: fixed;
    inset: 0;
    z-index: 9998;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    user-select: text;
    transition: background-color 0.25s ease, color 0.25s ease;
  }

  /* Themes */
  .theme-paper {
    background-color: #fbfbf9;
    color: #2b2b2b;
  }
  .theme-paper .reader-toolbar {
    background: rgba(251, 251, 249, 0.92);
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  }

  .theme-sepia {
    background-color: #f6efe2;
    color: #3e3328;
  }
  .theme-sepia .reader-toolbar {
    background: rgba(246, 239, 226, 0.92);
    border-bottom: 1px solid rgba(62, 51, 40, 0.1);
  }

  .theme-dark {
    background-color: #1e1e24;
    color: #dedee4;
  }
  .theme-dark .reader-toolbar {
    background: rgba(30, 30, 36, 0.92);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .theme-oled {
    background-color: #000000;
    color: #e5e5e5;
  }
  .theme-oled .reader-toolbar {
    background: rgba(0, 0, 0, 0.92);
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
  }

  /* Toolbar */
  .reader-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 24px;
    backdrop-filter: blur(12px);
    z-index: 10;
    flex-shrink: 0;
    gap: 16px;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(128, 128, 128, 0.12);
    border: 1px solid rgba(128, 128, 128, 0.2);
    color: inherit;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s, transform 0.1s;
  }

  .back-btn:hover {
    background: rgba(128, 128, 128, 0.22);
  }

  .meta-pills {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    opacity: 0.75;
  }

  .read-time-pill,
  .word-count-pill {
    padding: 2px 8px;
    background: rgba(128, 128, 128, 0.1);
    border-radius: 12px;
  }

  .toolbar-controls {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .control-group {
    display: flex;
    align-items: center;
    background: rgba(128, 128, 128, 0.1);
    border-radius: 6px;
    padding: 2px;
    gap: 2px;
  }

  .control-btn {
    background: transparent;
    border: none;
    color: inherit;
    opacity: 0.7;
    font-size: 12px;
    font-weight: 500;
    padding: 4px 8px;
    border-radius: 4px;
    cursor: pointer;
    transition: opacity 0.15s, background 0.15s;
  }

  .control-btn:hover {
    opacity: 1;
    background: rgba(128, 128, 128, 0.15);
  }

  .control-btn.active {
    opacity: 1;
    background: rgba(128, 128, 128, 0.25);
    font-weight: 600;
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: inherit;
    font-size: 13px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 4px;
    cursor: pointer;
    opacity: 0.8;
  }

  .icon-btn:hover:not(:disabled) {
    background: rgba(128, 128, 128, 0.18);
  }

  .icon-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .font-size-label {
    font-size: 11.5px;
    padding: 0 4px;
    opacity: 0.75;
  }

  .theme-picker {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-left: 4px;
  }

  .theme-dot {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    transition: transform 0.15s, border-color 0.15s;
  }

  .theme-dot:hover {
    transform: scale(1.15);
  }

  .theme-dot.active {
    border-color: #3b82f6;
    transform: scale(1.1);
  }

  .theme-dot.theme-paper {
    background-color: #fbfbf9;
    border: 1px solid rgba(0, 0, 0, 0.2);
  }
  .theme-dot.theme-sepia {
    background-color: #f6efe2;
    border: 1px solid rgba(0, 0, 0, 0.2);
  }
  .theme-dot.theme-dark {
    background-color: #1e1e24;
    border: 1px solid rgba(255, 255, 255, 0.2);
  }
  .theme-dot.theme-oled {
    background-color: #000000;
    border: 1px solid rgba(255, 255, 255, 0.3);
  }

  /* Main Canvas */
  .reader-scroll-canvas {
    flex: 1;
    overflow-y: auto;
    padding: 40px 24px 100px;
    display: flex;
    justify-content: center;
    scroll-behavior: smooth;
  }

  .reader-article-container {
    width: 100%;
    line-height: 1.75;
    letter-spacing: 0.01em;
  }

  .article-header {
    margin-bottom: 36px;
  }

  .article-site-tag {
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    opacity: 0.6;
    margin-bottom: 8px;
  }

  .article-title {
    margin: 0 0 12px;
    font-size: 2.2em;
    line-height: 1.25;
    font-weight: 700;
  }

  .article-byline {
    font-size: 0.95em;
    opacity: 0.7;
    margin-bottom: 24px;
  }

  .article-divider {
    height: 1px;
    background: currentColor;
    opacity: 0.12;
  }

  .article-body {
    word-break: break-word;
  }

  .article-body :global(p) {
    margin: 1.3em 0;
  }

  .article-body :global(h2),
  .article-body :global(h3),
  .article-body :global(h4) {
    margin: 1.6em 0 0.6em;
    line-height: 1.3;
    font-weight: 600;
  }

  .article-body :global(blockquote) {
    margin: 1.5em 0;
    padding: 0.5em 0 0.5em 1.2em;
    border-left: 3px solid currentColor;
    opacity: 0.85;
    font-style: italic;
  }

  .article-body :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
    margin: 1.6em auto;
    display: block;
  }

  .article-body :global(pre),
  .article-body :global(code) {
    font-family: 'Fira Code', 'Cascadia Code', monospace;
    background: rgba(128, 128, 128, 0.12);
    border-radius: 4px;
    font-size: 0.9em;
  }

  .article-body :global(code) {
    padding: 2px 6px;
  }

  .article-body :global(pre) {
    padding: 14px 18px;
    overflow-x: auto;
    margin: 1.4em 0;
  }

  .article-body :global(ul),
  .article-body :global(ol) {
    padding-left: 1.8em;
    margin: 1.2em 0;
  }

  .article-body :global(li) {
    margin: 0.5em 0;
  }

  /* States */
  .loading-state,
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 80px 20px;
    gap: 16px;
  }

  .reader-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(128, 128, 128, 0.2);
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .error-state svg {
    opacity: 0.6;
    color: #ef4444;
  }

  .error-state h3 {
    margin: 0;
    font-size: 20px;
  }

  .error-state p {
    margin: 0;
    opacity: 0.75;
    max-width: 480px;
  }

  .error-hint {
    font-size: 13px;
    opacity: 0.5;
  }

  .close-error-btn {
    margin-top: 12px;
    background: rgba(128, 128, 128, 0.15);
    border: 1px solid rgba(128, 128, 128, 0.25);
    color: inherit;
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
  }
</style>


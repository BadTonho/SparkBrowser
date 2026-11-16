<script lang="ts">
  import { t, locale } from "../../i18n";
  import { updateState } from "../../stores/updatesStore";
  import { changelogEntries } from "../../services/changelog";
  import { changelogFiles } from "../../services/changelog-files";

  const entries = $derived(changelogEntries(changelogFiles, $locale === "pt-BR" ? "pt-BR" : "en"));
  // A development build runs from a version that may not be listed yet, so match loosely.
  const running = $derived(String($updateState.current_version ?? "").split("-")[0]);
</script>

<section class="setting-group" aria-labelledby="changelog-heading">
  <h2 id="changelog-heading" class="group-title">{$t("settings.changelog_5f3a")}</h2>
  <p class="intro">{$t("settings.changelog_intro_8c1d")}</p>
  {#if entries.length === 0}
    <p role="status">{$t("settings.changelog_empty_2e7b")}</p>
  {:else}
    <div class="releases">
      {#each entries as entry, index (entry.version)}
        <details class="release" open={index === 0}>
          <summary>
            <span class="release-title">{entry.title}</span>
            {#if entry.version === running}<span class="badge">{$t("settings.changelog_running_a94c")}</span>{/if}
          </summary>
          {#if entry.note}<p class="note">{entry.note}</p>{/if}
          {#each entry.sections as section (section.heading)}
            <h3>{section.heading}</h3>
            <ul>
              {#each section.items as item}
                <li>{#each item as part}{#if part.bold}<strong>{part.text}</strong>{:else if part.code}<code>{part.text}</code>{:else}{part.text}{/if}{/each}</li>
              {/each}
            </ul>
          {/each}
        </details>
      {/each}
    </div>
  {/if}
</section>

<style>
  .intro { margin: 0 0 14px; color: var(--text-secondary); font-size: 12.5px; line-height: 1.5; }
  .releases { display: grid; gap: 10px; }
  .release { padding: 12px 14px; border: 1px solid var(--border-subtle); border-radius: var(--radius-md); background: rgba(255, 255, 255, 0.03); }
  .release > summary { display: flex; align-items: center; gap: 10px; cursor: pointer; list-style: none; font-weight: 600; }
  .release > summary::-webkit-details-marker { display: none; }
  .release > summary::before { content: "›"; display: inline-block; width: 12px; color: var(--text-muted); transition: transform 0.15s ease; }
  .release[open] > summary::before { transform: rotate(90deg); }
  .release > summary:focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 3px; border-radius: 4px; }
  .release-title { font-size: 14px; }
  .badge { padding: 2px 8px; border-radius: 999px; background: color-mix(in srgb, var(--accent-primary) 22%, transparent); color: var(--accent-primary); font-size: 11px; font-weight: 600; }
  .note { margin: 10px 0 0; color: var(--text-muted); font-size: 12px; }
  h3 { margin: 14px 0 6px; color: var(--text-muted); font-size: 11px; font-weight: 700; letter-spacing: 0.6px; text-transform: uppercase; }
  ul { margin: 0; padding-left: 18px; display: grid; gap: 6px; }
  li { color: var(--text-primary); font-size: 12.5px; line-height: 1.5; overflow-wrap: anywhere; }
  code { padding: 1px 5px; border-radius: 4px; background: rgba(255, 255, 255, 0.08); font-size: 11.5px; }
  @media (prefers-reduced-motion: reduce) { .release > summary::before { transition: none; } }
</style>

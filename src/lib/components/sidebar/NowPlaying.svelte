<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { marquee } from "../../actions/marquee";
  import { controlNowPlaying, fetchNowPlaying, onNowPlaying, type NowPlaying, type NowPlayingOperation } from "../../services/bridge/now-playing";
  import { elapsedSeconds, formatClock } from "../../services/media-time";

  // The same per-device choice the new-tab card uses (Customize new tab → Music and video → YouTube).
  const THUMBNAIL_KEY = "spark_youtube_thumbnail";
  const readCover = () => { try { return localStorage.getItem(THUMBNAIL_KEY) === "cover"; } catch { return false; } };

  let video = $state<NowPlaying | null>(null);
  let sampledAt = $state(Date.now());
  let now = $state(Date.now());
  let cover = $state(readCover());
  let busy = $state(false);

  let duration = $derived(video?.duration && video.duration > 0 && video.position !== null ? video.duration : 0);
  let elapsed = $derived(video && duration ? elapsedSeconds(video.position ?? 0, duration, video.playing, sampledAt, now) : 0);
  let showCover = $derived(cover && Boolean(video?.image));
  let label = $derived(video?.source === "music" ? "YouTube Music" : "YouTube");

  function apply(next: NowPlaying | null) {
    video = next;
    sampledAt = now = Date.now();
  }
  async function run(operation: NowPlayingOperation) {
    if (busy) return;
    busy = true;
    try { await controlNowPlaying(operation); } catch { /* The card refreshes itself; a closed tab simply disappears. */ }
    finally { busy = false; }
  }

  onMount(() => {
    let disposed = false;
    let stop: (() => void) | undefined;
    void onNowPlaying(next => { if (!disposed) apply(next); }).then(unlisten => { if (disposed) unlisten(); else stop = unlisten; }).catch(() => {});
    void fetchNowPlaying().then(next => { if (!disposed) apply(next); }).catch(() => {});
    const onStorage = (event: StorageEvent) => { if (event.key === THUMBNAIL_KEY) cover = readCover(); };
    window.addEventListener("storage", onStorage);
    // Between backend samples the bar advances by itself, but only while a video plays.
    const timer = setInterval(() => { if (video?.playing) now = Date.now(); }, 1000);
    return () => { disposed = true; stop?.(); window.removeEventListener("storage", onStorage); clearInterval(timer); };
  });
</script>

{#if video}
  <section class="now-playing" class:cover={showCover} aria-label={$t("sidebar.now_playing_region")}>
    {#if showCover}<img class="backdrop" src={video.image} alt="" />{/if}
    <button type="button" class="logo" onclick={() => run("open")} disabled={busy} title={$t("sidebar.now_playing_open")} aria-label={`${label}: ${$t("sidebar.now_playing_open")}`}>
      {#if video.source === "music"}
        <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="11" fill="#ff0000" /><circle cx="12" cy="12" r="6.3" fill="none" stroke="#fff" stroke-width="1.6" /><path d="M10.2 9v6l5-3Z" fill="#fff" /></svg>
      {:else}
        <svg width="26" height="18" viewBox="0 0 28 20" aria-hidden="true"><rect width="28" height="20" rx="5.5" fill="#ff0000" /><path d="M11 5.5 18.5 10 11 14.5Z" fill="#fff" /></svg>
      {/if}
    </button>
    <div class="row">
      {#if !showCover && video.image}<img class="thumb" src={video.image} alt="" />{/if}
      <div class="meta">
        <div class="marquee" use:marquee={video.title} title={video.title}><span>{video.title}</span></div>
        {#if duration}
          <progress max={duration} value={elapsed} aria-label={$t("sidebar.now_playing_progress")}></progress>
          <div class="times"><span>{formatClock(elapsed)}</span><span>{formatClock(duration)}</span></div>
        {/if}
      </div>
    </div>
    <div class="controls">
      <button type="button" onclick={() => run("previous")} disabled={busy} title={$t("sidebar.now_playing_previous")} aria-label={$t("sidebar.now_playing_previous")}>⏮</button>
      <button type="button" onclick={() => run("toggle")} disabled={busy} title={video.playing ? $t("sidebar.now_playing_pause") : $t("sidebar.now_playing_play")} aria-label={video.playing ? $t("sidebar.now_playing_pause") : $t("sidebar.now_playing_play")}>{video.playing ? "⏸" : "▶"}</button>
      <button type="button" onclick={() => run("next")} disabled={busy} title={$t("sidebar.now_playing_next")} aria-label={$t("sidebar.now_playing_next")}>⏭</button>
    </div>
  </section>
{/if}

<style>
  .now-playing {
    position: relative;
    isolation: isolate;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 9px 10px 10px;
    overflow: hidden;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.035);
    color: var(--text-primary);
  }
  .now-playing.cover { color: #fff; }
  .now-playing.cover::after { content: ""; position: absolute; inset: 0; z-index: -1; background: linear-gradient(180deg, rgba(0, 0, 0, 0.4), rgba(0, 0, 0, 0.8)); }
  .backdrop { position: absolute; inset: 0; z-index: -2; width: 100%; height: 100%; object-fit: cover; }
  .logo { align-self: flex-start; display: grid; place-items: center; height: 22px; padding: 0; border-radius: 5px; }
  .logo:not(:disabled):hover { filter: brightness(1.2); }
  .logo:focus-visible, .controls button:focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 2px; }
  .row { display: flex; align-items: center; gap: 10px; min-width: 0; }
  .thumb { flex: none; width: 64px; height: 36px; border-radius: 5px; object-fit: cover; }
  .meta { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
  .marquee { overflow: hidden; white-space: nowrap; font-size: 12px; font-weight: 600; }
  .marquee > span { display: inline-block; }
  .marquee:global([data-scrolling="true"]) { -webkit-mask-image: linear-gradient(90deg, transparent, #000 8px, #000 calc(100% - 8px), transparent); mask-image: linear-gradient(90deg, transparent, #000 8px, #000 calc(100% - 8px), transparent); }
  .marquee:global([data-scrolling="true"]) > span { animation: now-playing-marquee var(--duration, 10s) ease-in-out infinite alternate; }
  @keyframes now-playing-marquee { 0%, 12% { transform: translateX(0); } 88%, 100% { transform: translateX(var(--shift, 0)); } }
  progress { display: block; appearance: none; -webkit-appearance: none; width: 100%; height: 3px; border: 0; border-radius: 2px; overflow: hidden; background: rgba(255, 255, 255, 0.16); }
  progress::-webkit-progress-bar { background: rgba(255, 255, 255, 0.16); }
  progress::-webkit-progress-value { background: #ff4d4d; transition: width 900ms linear; }
  progress::-moz-progress-bar { background: #ff4d4d; }
  .times { display: flex; justify-content: space-between; margin-top: -2px; color: var(--text-muted); font-size: 10px; font-variant-numeric: tabular-nums; }
  .cover .times { color: rgba(255, 255, 255, 0.78); }
  .controls { display: flex; justify-content: center; gap: 8px; }
  .controls button { width: 30px; height: 28px; display: grid; place-items: center; border: 1px solid var(--border-subtle); border-radius: 8px; background: rgba(255, 255, 255, 0.05); color: inherit; font-size: 11px; }
  .cover .controls button { border-color: rgba(255, 255, 255, 0.28); background: rgba(0, 0, 0, 0.4); }
  .controls button:not(:disabled):hover { background: var(--bg-glass-hover); }
  button:disabled { opacity: 0.6; }
  @media (prefers-reduced-motion: reduce) {
    .marquee:global([data-scrolling="true"]) > span { animation: none; max-width: 100%; overflow: hidden; text-overflow: ellipsis; vertical-align: top; }
    .marquee:global([data-scrolling="true"]) { -webkit-mask-image: none; mask-image: none; }
    progress::-webkit-progress-value { transition: none; }
  }
</style>

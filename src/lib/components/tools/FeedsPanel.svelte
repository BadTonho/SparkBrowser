<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { invoke } from "../../services/bridge/runtime";
  import { describeError } from "../../services/user-errors.js";
  import { addTab } from "../../stores/browserStore";
  import { closeTools } from "../../stores/toolsStore";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  type Feed = { id: number; title: string; url: string; interval_minutes: number; last_error: string | null };
  type Entry = { title: string; url: string; text: string };
  let feeds = $state<Feed[]>([]); let entries = $state<Entry[]>([]); let selected = $state<number | null>(null);
  let id = $state<number | null>(null); let title = $state(""); let url = $state(""); let interval = $state(60); let error = $state(""); let busy = $state(false);
  let compact = $state(false);
  async function load() { feeds = await invoke<Feed[]>("get_feeds"); if (selected !== null) entries = await invoke<Entry[]>("get_feed_entries", { id: selected }); }
  onMount(() => { void load().catch(e => error = describeError(e)); void invoke<{compact_feed_cards: boolean}>("get_experimental_flags").then(value => compact = value.compact_feed_cards).catch(() => {}); });
  async function save() {
    busy = true; error = "";
    try { await invoke("save_feed", { id, title, url, intervalMinutes: Number(interval) }); id = null; title = ""; url = ""; interval = 60; await load(); }
    catch (e) { error = describeError(e); } finally { busy = false; }
  }
  async function select(feed: Feed) { selected = feed.id; try { await load(); } catch (e) { error = describeError(e); } }
  async function refresh(feed: Feed) { busy = true; error = ""; try { await invoke("refresh_feed", { id: feed.id }); await select(feed); } catch (e) { error = describeError(e); await load(); } finally { busy = false; } }
  async function remove(feed: Feed) { if (!window.confirm($t("tools.feeds_delete_confirm", { title: feed.title }))) return; try { await invoke("delete_feed", { id: feed.id }); if (selected === feed.id) { selected = null; entries = []; } await load(); } catch (e) { error = describeError(e); } }
</script>
<h3>{$t("tools.feeds")}</h3>
<p>{$t("tools.feeds_description")}</p>
{#if error}<ErrorMessage {error} />{/if}
<label>{$t("tools.feeds_title")} <input bind:value={title} maxlength="120" /></label>
<label>{$t("tools.feeds_url")} <input bind:value={url} type="url" /></label>
<label>{$t("tools.feeds_interval")} <input bind:value={interval} type="number" min="15" max="1440" /></label>
<div class="tool-actions"><button disabled={busy || !title.trim() || !url.trim()} onclick={save}>{$t("tools.feeds_save")}</button>{#if id !== null}<button onclick={() => { id = null; title = ""; url = ""; interval = 60; }}>{$t("tools.cancel_1976")}</button>{/if}</div>
{#each feeds as feed (feed.id)}
  <article class="tool-card"><strong>{feed.title}</strong><span>{feed.url}</span>{#if feed.last_error}<p role="status">{$t("tools.feeds_update_failed")}</p>{/if}
    <div class="tool-actions"><button onclick={() => select(feed)}>{$t("tools.feeds_read")}</button><button disabled={busy} onclick={() => refresh(feed)}>{$t("tools.feeds_refresh")}</button><button onclick={() => { id = feed.id; title = feed.title; url = feed.url; interval = feed.interval_minutes; }}>{$t("tools.feeds_edit")}</button><button onclick={() => remove(feed)}>{$t("tools.feeds_delete")}</button></div>
  </article>
{/each}
{#if selected !== null}{#each entries as entry (entry.url)}<article class="tool-card"><strong>{entry.title || entry.url}</strong>{#if !compact}<p>{entry.text}</p>{/if}<button onclick={async () => { await closeTools(); await addTab(entry.url); }}>{$t("tools.open_page_af03")}</button></article>{/each}{/if}

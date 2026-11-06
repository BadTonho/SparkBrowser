<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { t } from "../../i18n";
  import { tabs } from "../../stores/browserStore";
  import { closeQrCode, isQrCodeOpen } from "../../stores/qrCodeStore";
  import { copyPngImage, makeQrPng, savePngImage } from "../../services/qr-code";
  import { focusDialog } from "../../actions/focusDialog";

  let png = $state("");
  let error = $state("");
  let notice = $state("");
  let busy = $state(false);
  let sourceUrl = $derived($tabs.find(tab => tab.is_active)?.url ?? "");
  let validUrl = $derived.by(() => {
    try { const parsed = new URL(sourceUrl); return parsed.protocol === "http:" || parsed.protocol === "https:"; }
    catch { return false; }
  });
  let generation = 0;

  $effect(() => {
    if (!$isQrCodeOpen) return;
    const url = sourceUrl;
    const valid = validUrl;
    const current = ++generation;
    png = ""; error = ""; notice = "";
    if (!valid) { error = $t("qr_code.http_only"); return; }
    void makeQrPng(url).then(value => {
      if (current === generation) png = value;
    }).catch(() => {
      if (current === generation) error = $t("qr_code.too_long");
    });
  });

  async function copyImage() {
    if (!png || busy) return;
    busy = true; error = ""; notice = "";
    try { await copyPngImage(png); notice = $t("qr_code.copied"); }
    catch { error = $t("qr_code.copy_failed"); }
    finally { busy = false; }
  }

  async function saveImage() {
    if (!png || busy) return;
    busy = true; error = ""; notice = "";
    try {
      if (await savePngImage(png)) notice = $t("qr_code.saved");
    } catch { error = $t("qr_code.save_failed"); }
    finally { busy = false; }
  }
</script>

{#if $isQrCodeOpen}
  <div class="backdrop" transition:fade={{ duration: 140 }} onclick={closeQrCode} role="presentation">
    <div class="panel" transition:fly={{ y: -12, duration: 180, easing: cubicOut }} onclick={event => event.stopPropagation()} onkeydown={event => event.stopPropagation()} use:focusDialog={{ onEscape: closeQrCode }} role="dialog" aria-modal="true" aria-labelledby="qr-heading" tabindex="-1">
      <header><h2 id="qr-heading">{$t("qr_code.title")}</h2><button class="close" onclick={closeQrCode} aria-label={$t("qr_code.close")}>×</button></header>
      <p class="url">{sourceUrl}</p>
      <div class="image-slot" aria-live="polite">
        {#if png}<img src={png} alt={$t("qr_code.image_alt")} />{:else if !error}<span>{$t("qr_code.generating")}</span>{/if}
      </div>
      {#if error}<p class="message error" role="alert">{error}</p>{/if}
      {#if notice}<p class="message" role="status">{notice}</p>{/if}
      <footer>
        <button onclick={copyImage} disabled={!png || busy}>{$t("qr_code.copy_image")}</button>
        <button onclick={saveImage} disabled={!png || busy}>{$t("qr_code.save_png")}</button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .backdrop { position: fixed; inset: 0; z-index: 1100; display: grid; place-items: start center; padding-top: 12vh; background: #080808a8; backdrop-filter: blur(12px); }
  .panel { width: min(390px, 92vw); padding: 18px; border: 1px solid var(--border-subtle); border-radius: var(--radius-lg); background: var(--bg-panel, #171b27); color: var(--text-primary); box-shadow: 0 22px 64px #0009; }
  header, footer { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
  h2 { margin: 0; font-size: 16px; } .close { font-size: 22px; color: var(--text-secondary); }
  .url { margin: 14px 0; overflow-wrap: anywhere; font-size: 11px; color: var(--text-secondary); }
  .image-slot { min-height: 300px; display: grid; place-items: center; border-radius: var(--radius-md); background: white; color: #555; font-size: 13px; }
  img { width: 288px; height: 288px; image-rendering: pixelated; }
  footer { justify-content: center; margin-top: 16px; } footer button { padding: 8px 12px; border-radius: var(--radius-sm); background: var(--bg-glass-hover); }
  button:disabled { opacity: .45; cursor: not-allowed; } .message { color: var(--text-secondary); font-size: 12px; text-align: center; } .error { color: var(--danger, #ef6b6b); }
</style>

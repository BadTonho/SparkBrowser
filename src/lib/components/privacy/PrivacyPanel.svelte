<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { locale, t } from "../../i18n";
  import { tabs } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceSelectionStore";
  import { privacyPopover, closePrivacy } from "../../stores/privacyPopoverStore";
  import { mediaPopover } from "../../stores/mediaPopoverStore";
  import { privacyStats, privacyBusy, privacyError, flipAdblockShield, setAutoSkipAds } from "../../stores/privacyStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { onOverlayOpening } from "../../services/bridge/overlays";
  import { onPrivacyPopoverAction, setPrivacyPopover } from "../../services/bridge/privacy-popover";
  import { describeError } from "../../services/user-errors.js";

  const native = isTauriRuntime();
  const active = $derived($tabs.find(tab => tab.is_active));
  let anchor = $state({ left: 0, top: 0, width: 0, height: 0 });
  let sending = false;
  function measure() {
    const r = $privacyPopover?.anchor.getBoundingClientRect();
    if (r && (r.left !== anchor.left || r.top !== anchor.top || r.width !== anchor.width || r.height !== anchor.height))
      anchor = { left: r.left, top: r.top, width: r.width, height: r.height };
  }
  async function change(action: string, value: boolean) {
    if (sending || $privacyBusy) return;
    sending = true;
    try {
      if (action === "auto-skip") await setAutoSkipAds(value);
      else if ($privacyStats.shield_enabled !== value) await flipAdblockShield();
    } catch (error) { privacyError.set(describeError(error)); }
    finally { sending = false; }
  }
  $effect(() => {
    const session = $privacyPopover;
    if (session && (active?.id !== session.tabId || active.url !== session.url || $activeWorkspaceId !== session.workspaceId || $mediaPopover)) void closePrivacy();
  });
  $effect(() => {
    const session = $privacyPopover;
    if (!session) return;
    untrack(measure);
    let frame = 0;
    const follow = () => {
      if (!session.anchor.isConnected || !session.anchor.getClientRects().length) { void closePrivacy(); return; }
      measure(); frame = requestAnimationFrame(follow);
    };
    frame = requestAnimationFrame(follow);
    return () => cancelAnimationFrame(frame);
  });
  $effect(() => {
    const session = $privacyPopover;
    if (!native || !session || !anchor.width) return;
    const style = getComputedStyle(document.documentElement);
    void setPrivacyPopover(true, session.id, {
      tabId: session.tabId, url: session.url, anchor,
      shield: $privacyStats.shield_enabled, autoSkip: $privacyStats.auto_skip_ads_enabled,
      busy: $privacyBusy, error: $privacyError,
      label: $t("privacy.controls"), adblockLabel: $t("privacy.adblock"), autoSkipLabel: $t("privacy.auto_skip"),
      blocked: $t("privacy.blocked", { count: $privacyStats.blocked_count }), description: $t("privacy.auto_skip_description"), locale: $locale,
      theme: { background: style.getPropertyValue("--bg-downloads-panel").trim() || "#1c1c1c", border: style.getPropertyValue("--border-subtle").trim(), accent: style.getPropertyValue("--accent-primary").trim() },
    }).catch(error => { if ($privacyPopover?.id === session.id) { privacyError.set(describeError(error)); void closePrivacy(); } });
  });
  onMount(() => {
    let disposed = false, stopActions: (() => void) | undefined;
    const stopOverlays = onOverlayOpening(() => { void closePrivacy(); });
    void onPrivacyPopoverAction(event => {
      const session = $privacyPopover;
      if (!session || session.id !== event.sessionId || session.tabId !== event.tabId || session.url !== event.url) return;
      if (event.action === "dismiss") void closePrivacy(["escape", "tab", "trigger"].includes(event.reason));
      else if (typeof event.value === "boolean") void change(event.action, event.value);
    }).then(stop => { if (disposed) stop(); else stopActions = stop; });
    return () => { disposed = true; stopActions?.(); stopOverlays(); void closePrivacy(); };
  });
</script>

<svelte:window onresize={measure} onkeydown={event => { if (event.key === "Escape") void closePrivacy(true); }} />

{#if !native && $privacyPopover}
  <div class="preview-dismiss" role="presentation" onclick={() => closePrivacy()}>
    <div class="privacy-preview" role="dialog" aria-label={$t("privacy.controls")} tabindex="-1"
      style:left={`${Math.max(8, Math.min(anchor.left + anchor.width - 300, window.innerWidth - 308))}px`}
      style:top={`${anchor.top + anchor.height + 6}px`} onclick={event => event.stopPropagation()} onkeydown={event => { if (event.key === "Escape") void closePrivacy(true); }}>
      <strong>{$t("privacy.controls")}</strong>
      <label>{$t("privacy.adblock")}<input type="checkbox" role="switch" checked={$privacyStats.shield_enabled} disabled={$privacyBusy} onchange={event => change("shield", event.currentTarget.checked)} /></label>
      <small>{$t("privacy.blocked", { count: $privacyStats.blocked_count })}</small>
      <label>{$t("privacy.auto_skip")}<input type="checkbox" role="switch" checked={$privacyStats.auto_skip_ads_enabled} disabled={$privacyBusy} onchange={event => change("auto-skip", event.currentTarget.checked)} /></label>
      <small>{$t("privacy.auto_skip_description")}</small>
      {#if $privacyError}<p role="alert">{$privacyError}</p>{/if}
    </div>
  </div>
{/if}

<style>
  .preview-dismiss { position:fixed;inset:0;z-index:2000; }
  .privacy-preview { position:absolute;box-sizing:border-box;width:300px;max-width:calc(100vw - 16px);padding:16px;background:var(--bg-downloads-panel, #1c1c1c);border:1px solid var(--border-subtle);border-radius:10px;box-shadow:0 4px 16px #0005; }
  label { display:flex;align-items:center;justify-content:space-between;margin-top:16px; }
  small { display:block;color:#b9b9c2;font-size:12px; }
  input { accent-color:var(--accent-primary); }
  p { color:#ffb4b4; }
</style>

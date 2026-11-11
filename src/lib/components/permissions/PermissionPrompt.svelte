<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { t, plural } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { permissionPopover } from "../../actions/permissionPopover";
  import { permissionRequests } from "../../stores/sitePermissionsStore";
  import { respondSitePermission } from "../../services/bridge/site-permissions";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  const native = isTauriRuntime();

  let busy = $state(false);
  let rememberBlock = $state(false);
  let error = $state("");
  const request = $derived($permissionRequests[0]);

  $effect(() => {
    void request?.id;
    rememberBlock = false;
    error = "";
  });

  async function respond(decision: string) {
    if (!request || busy) return;
    busy = true;
    error = "";
    try {
      await respondSitePermission(request.id, decision);
      rememberBlock = false;
    } catch (e) {
      error = describeError(e);
    } finally {
      busy = false;
    }
  }

  function getKindIcon(kind: string): string {
    switch (kind) {
      case "notifications":
        return "bell";
      case "camera":
        return "camera";
      case "microphone":
        return "mic";
      case "location":
        return "map-pin";
      default:
        return "lock";
    }
  }

  function cleanOrigin(urlStr: string): string {
    try {
      return new URL(urlStr).hostname;
    } catch {
      return urlStr;
    }
  }
</script>

{#if !native && request}
  {#key request.id}
    <div
      data-testid="permission-request"
      class="permission-popover"
      role="dialog"
      aria-modal="true"
      aria-label={$t("permissions.site_permission_request_1192")}
      tabindex="-1"
      use:permissionPopover
      use:focusDialog={{ onEscape: () => respond("block") }}
    >
      <div class="popover-header">
        <span class="kind-icon"><AppIcon name={getKindIcon(request.kind)} /></span>
        <div class="header-titles">
          <h3>{$t("permissions.site_permission_request_1192")}</h3>
          <span class="origin-host">{cleanOrigin(request.frame_origin ?? request.origin)}</span>
        </div>
      </div>

      <div class="popover-body">
        <p class="request-desc">
          <strong>{cleanOrigin(request.frame_origin ?? request.origin)}</strong> {$t("permissions.wants_to_use_183f")} <strong>{$t(("common." + request.kind) as "common.camera" | "common.microphone" | "common.location" | "common.notifications")}</strong>.
        </p>

        <details class="request-details">
          <summary>{$t("permissions.more_details")}</summary>
          {#if request.frame_origin || request.origin !== request.top_origin}
            <p class="sub-desc">{$t("permissions.embedded_in_58f8")} {request.top_origin}</p>
          {/if}
          {#if request.frame_origin && request.frame_origin !== request.origin}
            <p class="sub-desc">{$t("permissions.webview2_applies_this_permission_decision_to_a0ef")} {request.origin}.</p>
          {/if}
          <p class="expiry-desc">{$t("permissions.requests_expire_after_60_seconds_5d7d")} {$plural($permissionRequests.length, $t("common.pending_one"), $t("common.pending_other"))}</p>
        </details>

        {#if error}
          <ErrorMessage {error} />
        {/if}

        <label class="remember-label">
          <input type="checkbox" bind:checked={rememberBlock} disabled={busy} />
          <span>{$t("permissions.remember_blocking_for_this_site_76fa")}</span>
        </label>
      </div>

      <div class="popover-actions">
        <button
          class="btn-block"
          type="button"
          disabled={busy}
          onclick={() => respond(rememberBlock ? "block_always" : "block")}
        >
          {$t("permissions.block_211d")}
        </button>
        <button
          class="btn-secondary"
          type="button"
          disabled={busy}
          onclick={() => respond("allow_once")}
        >
          {$t("permissions.allow_once_1685")}
        </button>
        <button
          class="btn-primary"
          type="button"
          disabled={busy}
          onclick={() => respond("allow_always")}
        >
          {$t("permissions.always_allow_9776")}
        </button>
      </div>
    </div>
  {/key}
{/if}

<style>
  .permission-popover {
    position: fixed;
    top: var(--permission-popover-top, 54px);
    left: var(--permission-popover-left, 8px);
    z-index: 1500;
    width: min(360px, calc(100vw - 16px));
    max-height: var(--permission-popover-max-height, calc(100vh - 62px));
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: var(--modal-surface-bg, var(--bg-permission-popover, #1c1c1c));
    backdrop-filter: var(--modal-surface-filter);
    -webkit-backdrop-filter: var(--modal-surface-filter);
    color: var(--text-primary, #f5f5f5);
    border: var(--modal-surface-border, 1px solid rgba(255, 255, 255, 0.12));
    border-radius: var(--radius-md, 12px);
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.55), 0 0 0 1px rgba(255, 255, 255, 0.05);
    pointer-events: auto;
    font-size: 12px;
    line-height: 1.5;
    animation: popover-slide-in 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes popover-slide-in {
    from {
      opacity: 0;
      transform: translateY(-8px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .popover-header {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .kind-icon {
    font-size: 18px;
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    flex-shrink: 0;
  }

  .header-titles {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .header-titles h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #fff);
  }

  .origin-host {
    font-size: 11px;
    color: var(--text-muted, #8e8e93);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .popover-body {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }

  .request-desc {
    margin: 0;
    font-size: 12.5px;
    color: var(--text-primary, #f5f5f5);
    line-height: 1.4;
  }

  .sub-desc {
    margin: 0;
    font-size: 11px;
    color: var(--text-muted, #8e8e93);
  }

  .expiry-desc {
    margin: 0;
    font-size: 11px;
    color: var(--text-secondary, #a1a1aa);
  }

  .request-details { display:flex; flex-direction:column; gap:4px; color:var(--text-muted, #8e8e93); font-size:11px; }
  .request-details summary { cursor:pointer; width:fit-content; }
  .request-details p { margin:0; }

  .remember-label {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
    font-size: 11.5px;
    color: var(--text-secondary, #d4d4d8);
    cursor: pointer;
    user-select: none;
  }

  .remember-label input[type="checkbox"] {
    accent-color: var(--accent-primary, #3b82f6);
    cursor: pointer;
  }

  .popover-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    margin-top: 2px;
    flex-wrap: wrap;
  }

  .popover-actions button {
    padding: 5px 8px;
    min-height: 28px;
    font-size: 11.5px;
    font-weight: 500;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.14s ease, border-color 0.14s ease, transform 0.1s ease;
  }

  .btn-block {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.12));
    color: var(--text-secondary, #a1a1aa);
  }

  .btn-block:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.18);
    border-color: rgba(239, 68, 68, 0.4);
    color: #fca5a5;
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.12));
    color: var(--text-primary, #f5f5f5);
  }

  .btn-secondary:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.14);
    border-color: rgba(255, 255, 255, 0.22);
  }

  .btn-primary {
    background: var(--accent-primary, #3b82f6);
    border: 1px solid transparent;
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    filter: brightness(1.12);
  }

  .popover-actions button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  @media (prefers-reduced-motion: reduce) {
    .permission-popover {
      animation: none;
    }
    .popover-actions button {
      transition: none;
    }
  }
</style>

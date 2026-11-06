<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { get } from "svelte/store";
  import { locale, plural, t, type MessageKey } from "../../i18n";
  import { tabs, currentUrl, triggerReload } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceSelectionStore";
  import { isSitePermissionsOpen, closeSitePermissions, permissionRequests } from "../../stores/sitePermissionsStore";
  import { fetchSitePermissions, setSitePermission, respondSitePermission } from "../../services/bridge/site-permissions";
  import { onPermissionPopoverAction, setPermissionPopover, type PermissionPopoverPresentation, type PermissionPopoverAction } from "../../services/bridge/permission-popover";
  import { describeError } from "../../services/user-errors.js";
  import { isTauriRuntime } from "../../services/bridge/runtime";

  const native = isTauriRuntime();
  let anchor = $state({ left: 0, top: 0, width: 0, height: 0 });
  let settings = $state<Record<string, string>>({});
  let error = $state("");
  let busy = $state(false);
  let confirmingReload = $state(false);
  let sessionId = $state(0);
  let sessionKey = "";
  let lastPresentation = "";
  const activeTab = $derived($tabs.find(tab => tab.is_active));
  const origin = $derived.by(() => { try { const url = new URL($currentUrl); return ["http:", "https:"].includes(url.protocol) ? url.origin : ""; } catch { return ""; } });

  function measure() {
    const rect = document.querySelector<HTMLElement>(".omnibox-icon")?.getBoundingClientRect();
    if (rect && (rect.left !== anchor.left || rect.top !== anchor.top || rect.width !== anchor.width || rect.height !== anchor.height))
      anchor = { left: rect.left, top: rect.top, width: rect.width, height: rect.height };
  }
  function text(key: MessageKey, params?: Record<string, string | number>) { return get(t)(key, params); }
  function host(value: string) { try { return new URL(value).hostname; } catch { return value; } }
  function theme() {
    const style = getComputedStyle(document.documentElement);
    const light = document.documentElement.dataset.theme === "light";
    return {
      surface: style.getPropertyValue("--bg-permission-popover").trim() || (light ? "#ffffff" : "#1c1c1c"),
      border: style.getPropertyValue("--border-subtle").trim(), text: style.getPropertyValue("--text-primary").trim(),
      muted: style.getPropertyValue("--text-secondary").trim(), accent: style.getPropertyValue("--accent-primary").trim(),
      accentText: style.getPropertyValue("--accent-foreground").trim(), control: light ? "#f4f4f5" : "#292929",
    };
  }
  function labels(kindLabels: Record<string, string>) {
    return {
      title: text("permissions.site_permissions_a688"), currentSite: text("permissions.current_site_334f"),
      requestTitle: text("permissions.site_permission_request_1192"), close: text("permissions.close_7d9e"),
      note: text("permissions.decisions_are_shared_across_workspaces_changes_apply_to_new_requests_act_7bc3"),
      refresh: text("permissions.refresh_decisions_a478"), reload: text("permissions.reload_when_ready_3a1a"),
      confirmReload: text("permissions.confirm_reload_a5f7"), cancel: text("permissions.cancel_1976"),
      reloadWarning: text("permissions.reloading_may_discard_forms_or_interrupt_media_downloads_reload_this_pag_5db0"),
      kinds: kindLabels, states: { ask: text("permissions.ask_b8c2"), allow: text("permissions.allow_e213"), block: text("permissions.block_211d") },
      wantsToUse: text("permissions.wants_to_use_183f"), moreDetails: text("permissions.more_details"),
      rememberBlock: text("permissions.remember_blocking_for_this_site_76fa"), block: text("permissions.block_211d"),
      allowOnce: text("permissions.allow_once_1685"), allowAlways: text("permissions.always_allow_9776"),
      expiry: text("permissions.requests_expire_after_60_seconds_5d7d"),
      pending: $plural($permissionRequests.length, text("common.pending_one"), text("common.pending_other")),
    };
  }
  function currentPresentation(): PermissionPopoverPresentation | null {
    const request = $permissionRequests[0];
    const mode = request ? "request" : $isSitePermissionsOpen ? "settings" : null;
    if (!mode || !activeTab || !anchor.width) return null;
    const key = mode === "request" ? `request:${request!.id}` : `settings:${activeTab.id}:${$currentUrl}`;
    if (key !== sessionKey) { sessionKey = key; sessionId += 1; lastPresentation = ""; }
    const kinds = ["camera", "microphone", "location", "notifications"];
    const kindLabels = Object.fromEntries(kinds.map(kind => [kind, text(`common.${kind}` as MessageKey)]));
    const payload: PermissionPopoverPresentation = {
      mode, tabId: activeTab.id, url: $currentUrl, origin, anchor, pendingCount: $permissionRequests.length,
      request: request ? {
        id: request.id, tabId: request.tab_id, origin: request.origin, topOrigin: request.top_origin,
        frameOrigin: request.frame_origin, kind: request.kind,
        displayOrigin: host(request.frame_origin ?? request.origin),
        details: [
          ...(request.frame_origin || request.origin !== request.top_origin ? [`${text("permissions.embedded_in_58f8")} ${request.top_origin}`] : []),
          ...(request.frame_origin && request.frame_origin !== request.origin ? [`${text("permissions.webview2_applies_this_permission_decision_to_a0ef")} ${request.origin}.`] : []),
        ],
      } : null,
      settings: { ...settings }, labels: labels(kindLabels), theme: theme(), locale: $locale,
      error, busy, confirmingReload,
    };
    return payload;
  }
  async function push() {
    if (!native) return;
    const payload = currentPresentation();
    if (!payload) {
      if (sessionKey) { await setPermissionPopover(false, sessionId); sessionKey = ""; lastPresentation = ""; }
      return;
    }
    const serialized = JSON.stringify(payload);
    if (serialized === lastPresentation) return;
    lastPresentation = serialized;
    try { await setPermissionPopover(true, sessionId, payload); } catch (reason) { error = describeError(reason); lastPresentation = ""; }
  }
  async function refresh() {
    try {
      const snapshot = await fetchSitePermissions();
      settings = Object.fromEntries(["camera", "microphone", "location", "notifications"].map(kind => [kind, snapshot.settings.find(row => row.origin === origin && row.kind === kind)?.state ?? "ask"]));
      error = snapshot.supported ? "" : snapshot.message;
    } catch (reason) { error = describeError(reason); }
    await push();
  }
  async function handleAction(action: PermissionPopoverAction) {
    const request = $permissionRequests[0];
    if (action.sessionId !== sessionId || !activeTab || action.tabId !== activeTab.id || action.url !== $currentUrl || action.origin !== origin) return;
    error = "";
    try {
      switch (action.action) {
        case "dismiss": await closeSitePermissions(); break;
        case "refresh": await refresh(); break;
        case "set-permission":
          if (!origin || action.origin !== origin) return;
          busy = true; await push();
          await setSitePermission(origin, action.kind as "camera" | "microphone" | "location" | "notifications", action.value as "ask" | "allow" | "block");
          await refresh(); break;
        case "reload": {
          const activity = activeTab.activity;
          if (activity?.dirty_form || activity?.playing_media || activity?.picture_in_picture || activity?.downloading) confirmingReload = true;
          else { await closeSitePermissions(); await triggerReload(); }
          break;
        }
        case "confirm-reload": await closeSitePermissions(); await triggerReload(); confirmingReload = false; break;
        case "cancel-reload": confirmingReload = false; break;
        case "respond":
          if (!request || action.requestId !== request.id || request.tab_id !== activeTab.id) return;
          await respondSitePermission(request.id, action.decision);
          await setPermissionPopover(false, action.sessionId);
          break;
      }
    } catch (reason) {
      error = describeError(reason);
      if (action.action === "respond") { await setPermissionPopover(false, action.sessionId); sessionKey = ""; }
    }
    finally { busy = false; await push(); }
  }

  $effect(() => {
    void $isSitePermissionsOpen; void $permissionRequests; void $currentUrl; void $tabs; void $activeWorkspaceId;
    void $locale; void error; void busy; void settings; void confirmingReload; void anchor;
    untrack(() => { void push(); });
  });
  $effect(() => {
    const request = $permissionRequests[0];
    if (request && $isSitePermissionsOpen) untrack(() => { void closeSitePermissions(); });
    if (!request && !$isSitePermissionsOpen) { confirmingReload = false; error = ""; }
  });
  $effect(() => {
    if (!$isSitePermissionsOpen && !$permissionRequests.length) return;
    untrack(measure);
    let raf = 0;
    const frame = () => { measure(); raf = requestAnimationFrame(frame); };
    raf = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(raf);
  });
  onMount(() => {
    let disposed = false, stop: (() => void) | undefined;
    void onPermissionPopoverAction(action => { void handleAction(action); }).then(unlisten => { if (disposed) unlisten(); else stop = unlisten; });
    void refresh();
    return () => { disposed = true; stop?.(); if (sessionKey) void setPermissionPopover(false, sessionId); };
  });
</script>

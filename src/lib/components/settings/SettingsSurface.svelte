<script lang="ts">
  import { onMount, tick } from "svelte";
  import PanelViewNode from "../panels/PanelViewNode.svelte";
  import type { PanelPresentation, PanelViewAction } from "../../services/panel-presentation";
  import { languagePreference, t, type MessageKey } from "../../i18n";
  import CreateProfileDialog from "../profile/CreateProfileDialog.svelte";
  import { sealProfileForm } from "../../services/profile/create-form";
  import type { CreateProfileForm } from "../../types/profiles";
  import "./settings.css";
  import "../panels/popover-motion.css";
  import "./SettingsModal.svelte";

  type Payload = PanelPresentation & { visible: boolean; token: string; sessionId: number; animate?: boolean; document?: string; focus?: boolean };
  const tabMode = new URLSearchParams(location.search).get("surface") === "settings-tab";
  let presentation = $state<Payload | null>(null);
  let closing = $state(false);
  let loadTimedOut = $state(false);
  let root = $state<HTMLDivElement>();
  const pending = new Map<number, number>();
  let sequence = 0, closedSession = 0, inFlight: number | null = null;
  let highlighted = "";
  let searchWasShown = false;
  const outbox: PanelViewAction[] = [];
  let createControl = $state<{ target: number; key: string } | null>(null);
  let response: { request: number; finish: (error: MessageKey | null) => void } | null = null;
  let requestNumber = crypto.getRandomValues(new Uint32Array(1))[0] & 0x00ffffff;

  function reportCreateFailure(packet: "photo-invalid" | "seal-failed" | "response-timeout") {
    if (createControl) send({ kind: "profilecreate", target: createControl.target, request: 0, packet });
  }

  async function createProfile(form: CreateProfileForm): Promise<MessageKey | null> {
    const control = createControl;
    const request = ++requestNumber;
    if (!control) return "profiles.error_failed";
    let packet: string;
    try {
      packet = await sealProfileForm(form, control.key, request);
    } catch {
      reportCreateFailure("seal-failed");
      return "profiles.error_failed";
    } finally {
      form.password = "";
    }
    if (control !== createControl) return "profiles.error_failed";
    return new Promise(resolve => {
      const timer = setTimeout(() => {
        reportCreateFailure("response-timeout");
        response?.finish("profiles.error_timeout");
      }, 45_000);
      response = { request, finish(error) { clearTimeout(timer); response = null; resolve(error); } };
      send({ kind: "profilecreate", target: control.target, request, packet });
    });
  }

  function resetProfileDialog() {
    createControl = null;
    response?.finish("profiles.error_failed");
  }

  function acceptProfileResponse() {
    if (!response || !createControl || !root) return;
    const trigger = root.querySelector<HTMLButtonElement>(`button[data-panel-control="${createControl.target}"][data-profile-create-key]`);
    if (trigger?.dataset.profileCreateRequest === String(response.request)) {
      response.finish((trigger.dataset.profileCreateError as MessageKey | undefined) ?? null);
    }
  }
  function pump() {
    if (!presentation || inFlight !== null || !outbox.length) return;
    const action = outbox.shift()!;
    const next = ++sequence;
    inFlight = next;
    if (action.kind === "input" || action.kind === "change") pending.set(action.target, next);
    const url = new URL(tabMode ? "spark-internal://settings-tab/action" : "spark-internal://settings-popover/action");
    url.searchParams.set("token", presentation.token);
    if (tabMode) url.searchParams.set("document", presentation.document ?? "");
    url.searchParams.set("sequence", String(next));
    url.searchParams.set("payload", JSON.stringify(action));
    const transport = (window as unknown as { __sparkPostContent?: (url: string) => void }).__sparkPostContent;
    if (tabMode && transport) transport(url.toString()); else location.href = url.toString();
  }
  function send(action: PanelViewAction) { if (!presentation || closing) return; if (action.kind === "input" || action.kind === "change") pending.set(action.target, Infinity); outbox.push(action); pump(); }
  onMount(() => {
    const loadTimer = tabMode ? setTimeout(() => { loadTimedOut = true; }, 8500) : undefined;
    const set = async (value: Payload) => {
      if (!value.visible) {
        closedSession = Math.max(closedSession, value.sessionId);
        if (presentation && presentation.sessionId > value.sessionId) return;
        resetProfileDialog();
        outbox.length = 0; inFlight = null;
        if (value.animate && presentation?.sessionId === value.sessionId) closing = true;
        else { closing = false; presentation = null; }
        return;
      }
      if (value.sessionId <= closedSession || (presentation && value.sessionId < presentation.sessionId) || (presentation?.sessionId === value.sessionId && value.revision < presentation.revision)) return;
      const opening = presentation?.sessionId !== value.sessionId;
      if (opening) { resetProfileDialog(); closing = false; sequence = 0; highlighted = ""; pending.clear(); outbox.length = 0; inFlight = null; }
      presentation = value;
      if (inFlight !== null && value.ack >= inFlight) inFlight = null;
      document.documentElement.lang = value.locale;
      if (value.locale === "en" || value.locale === "pt-BR") languagePreference.set(value.locale);
      for (const [key, color] of Object.entries(value.theme)) document.documentElement.style.setProperty(key, color);
      pump();
      await tick();
      acceptProfileResponse();
      if (createControl) return;
      // The search is one icon until it is opened: focus the icon on opening, and the field once it appears.
      const searchField = root?.querySelector<HTMLInputElement>("#settings-search-input");
      const searchShown = Boolean(searchField && searchField.getClientRects().length);
      if (opening && (!tabMode || value.focus)) (searchShown ? searchField : root?.querySelector<HTMLElement>("#settings-search-open"))?.focus();
      else if (searchShown && !searchWasShown && (!tabMode || value.focus)) searchField?.focus();
      searchWasShown = searchShown;
      const target = root?.querySelector<HTMLElement>(".search-target-highlight");
      if (target && target.id !== highlighted) {
        highlighted = target.id;
        target.scrollIntoView({ block: "center" });
        if (!tabMode || value.focus) target.querySelector<HTMLElement>("button, input, select, [tabindex='0']")?.focus({ preventScroll: true });
      } else if (!target) highlighted = "";
    };
    (window as unknown as Record<string, typeof set>).__sparkSetSettingsPopover = set;
    const click = (event: MouseEvent) => {
      const target = event.target instanceof Element ? event.target.closest<HTMLElement>("button[data-panel-control],summary[data-panel-control]") : null;
      if (!target || target.matches(":disabled")) return;
      event.preventDefault(); event.stopPropagation();
      if (target.dataset.profileCreateKey) {
        target.focus();
        createControl = { target: Number(target.dataset.panelControl), key: target.dataset.profileCreateKey };
        return;
      }
      send({ kind: "click", target: Number(target.dataset.panelControl) });
    };
    const edit = (event: Event) => {
      const target = event.target;
      if (!(target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement)) return;
      if (createControl || !target.hasAttribute("data-panel-control")) return;
      send({ kind: event.type as "input" | "change", target: Number(target.dataset.panelControl), value: target.value, checked: target instanceof HTMLInputElement && target.checked });
    };
    const key = (event: KeyboardEvent) => {
      if (createControl) return;
      const target = event.target instanceof Element ? event.target.closest<HTMLElement>("[data-panel-control]") : null;
      if (!target) return;
      const recording = !!root?.querySelector(".sc-keys.recording");
      if (!recording && target.id === "settings-search-input" && event.key === "ArrowDown") {
        const first = root?.querySelector<HTMLElement>(".settings-search-result");
        if (first) { event.preventDefault(); first.focus(); return; }
      }
      if (event.key === "Tab" && !tabMode && !recording && root) {
        const items = [...root.querySelectorAll<HTMLElement>('input:not([disabled]), select:not([disabled]), textarea:not([disabled]), button:not([disabled]), summary, [tabindex="0"]')].filter(item => item.getClientRects().length && !item.closest('details:not([open]) :not(summary)'));
        const index = items.indexOf(document.activeElement as HTMLElement);
        if (!items.length || (event.shiftKey ? index <= 0 : index < 0 || index === items.length - 1)) {
          event.preventDefault(); (event.shiftKey ? items.at(-1) : items[0])?.focus();
        }
        return;
      }
      if (recording || (!tabMode && event.key === "Escape") || (target.id === "settings-search-input" && ["Enter", "ArrowDown"].includes(event.key))) {
        event.preventDefault(); event.stopPropagation();
        send({ kind: "keyboard", target: Number(target.dataset.panelControl), key: event.key, ctrl: event.ctrlKey, alt: event.altKey, shift: event.shiftKey, meta: event.metaKey });
      }
    };
    const mouse = (event: MouseEvent) => {
      if (![3, 4].includes(event.button)) return;
      const target = event.target instanceof Element ? event.target.closest<HTMLElement>("[data-panel-control]") : null;
      if (target) { event.preventDefault(); send({ kind: "mouse", target: Number(target.dataset.panelControl), button: event.button, ctrl: event.ctrlKey, alt: event.altKey, shift: event.shiftKey, meta: event.metaKey }); }
    };
    document.addEventListener("click", click, true); document.addEventListener("input", edit); document.addEventListener("change", edit); document.addEventListener("keydown", key, true); document.addEventListener("mousedown", mouse, true);
    return () => { resetProfileDialog(); clearTimeout(loadTimer); document.removeEventListener("click", click, true); document.removeEventListener("input", edit); document.removeEventListener("change", edit); document.removeEventListener("keydown", key, true); document.removeEventListener("mousedown", mouse, true); };
  });
</script>
{#if presentation}
  <div class={tabMode ? "settings-tab-surface" : "settings-backdrop settings-motion"} class:settings-closing={closing} inert={closing} role="presentation" onclick={event => { if (!tabMode && event.target === event.currentTarget) send({ kind: "dismiss", reason: "outside" }); }} bind:this={root}>
    <PanelViewNode node={presentation.tree} {pending} ack={presentation.ack} />
  </div>
{:else if tabMode}
  <div class="settings-tab-loading" role="status">
    <p>{loadTimedOut ? $t("settings.display_tab_load_error") : $t("settings.display_tab_loading")}</p>
    {#if loadTimedOut}<button onclick={() => location.reload()}>{$t("settings.display_tab_retry")}</button>{/if}
  </div>
{/if}
{#if createControl}
  <CreateProfileDialog onclose={resetProfileDialog} oncreate={createProfile} onphotofailure={() => reportCreateFailure("photo-invalid")} />
{/if}

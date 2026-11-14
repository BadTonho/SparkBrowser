(() => {
  const popover = document.getElementById("media-popover");
  const slider = document.getElementById("page-volume");
  const percentage = document.getElementById("volume-percentage");
  const title = document.getElementById("media-title");
  const items = document.getElementById("media-items");
  const empty = document.getElementById("media-empty");
  const frameNote = document.getElementById("media-frame-note");
  let state = null;
  let sequence = 0;
  let closing = false;
  let editing = false;
  let pendingFrame = 0;

  function renderVolume() {
    const value = `${slider.value}%`;
    percentage.textContent = value;
    slider.setAttribute("aria-valuetext", value);
  }

  function position() {
    if (!state) return;
    const { anchor } = state;
    const left = anchor.left + anchor.width / 2 - popover.offsetWidth / 2;
    popover.style.left = `${Math.max(8, Math.min(left, innerWidth - popover.offsetWidth - 8))}px`;
    popover.style.top = `${Math.max(8, Math.min(anchor.top + anchor.height + 6, innerHeight - popover.offsetHeight - 8))}px`;
  }

  function send(action, reason = "", targetTabId = "") {
    if (!state || closing) return;
    cancelAnimationFrame(pendingFrame);
    pendingFrame = 0;
    const url = new URL(`spark-internal://media-popover/${action}`);
    url.searchParams.set("token", state.token);
    url.searchParams.set("sequence", String(++sequence));
    if (!slider.disabled) url.searchParams.set("value", slider.value);
    if (reason) url.searchParams.set("reason", reason);
    if (targetTabId) url.searchParams.set("targetTabId", targetTabId);
    if (action === "dismiss") closing = true;
    window.location.href = url.toString();
  }

  function renderItems() {
    items.replaceChildren();
    for (const item of state.items || []) {
      const row = document.createElement("div");
      row.className = "media-item";
      row.setAttribute("role", "listitem");
      const select = document.createElement("button");
      select.className = "media-select";
      select.type = "button";
      select.setAttribute("aria-label", `${item.tabTitle} — ${item.frameLabel}`);
      const tab = document.createElement("strong");
      tab.textContent = item.tabTitle;
      const context = document.createElement("span");
      context.textContent = `${item.workspaceName ? `${item.workspaceName} · ` : ""}${item.frameLabel} · ${item.playing ? state.labels.playing : state.labels.audible}`;
      select.append(tab, context);
      select.addEventListener("click", () => send("select", "", item.tabId));
      row.append(select);
      if (item.muteAvailable) {
        const mute = document.createElement("button");
        mute.type = "button";
        mute.className = "media-mute";
        mute.textContent = item.muted ? state.labels.unmute : state.labels.mute;
        mute.setAttribute("aria-label", `${mute.textContent}: ${item.tabTitle}`);
        mute.setAttribute("aria-pressed", String(item.muted));
        mute.addEventListener("click", () => send("mute", "", item.tabId));
        row.append(mute);
      }
      items.append(row);
    }
    empty.hidden = (state.items || []).length > 0;
    empty.textContent = state.labels.noMedia;
    frameNote.hidden = state.frameTrackingSupported;
    frameNote.textContent = state.labels.frameLimit;
  }

  window.__sparkSetMediaPopover = (payload) => {
    if (!payload.visible) {
      cancelAnimationFrame(pendingFrame);
      state = null;
      closing = false;
      editing = false;
      popover.hidden = true;
      return;
    }
    const opening = state?.token !== payload.token;
    // A delayed state update must not undo a dismissal or interrupt a drag.
    if (!opening && closing) return;
    state = payload;
    if (opening) { sequence = 0; closing = false; editing = false; }
    document.documentElement.lang = payload.locale;
    document.title = payload.label;
    popover.setAttribute("aria-label", payload.label);
    title.textContent = payload.labels.title;
    renderItems();
    slider.setAttribute("aria-label", payload.label);
    slider.disabled = payload.disabled;
    if (opening || (!editing && !pendingFrame)) slider.value = String(payload.volume);
    renderVolume();
    for (const [name, value] of Object.entries(payload.theme || {})) {
      popover.style.setProperty(`--popover-${name}`, value);
    }
    popover.hidden = false;
    position();
    if (opening) (slider.disabled ? popover : slider).focus({ preventScroll: true });
  };

  slider.addEventListener("pointerdown", () => { editing = true; });
  slider.addEventListener("input", () => {
    renderVolume();
    if (!pendingFrame) pendingFrame = requestAnimationFrame(() => { pendingFrame = 0; send("volume"); });
  });
  slider.addEventListener("change", () => { editing = false; renderVolume(); send("volume"); });
  window.addEventListener("pointerup", () => { if (editing) { editing = false; send("volume"); } });
  window.addEventListener("pointercancel", () => { editing = false; send("volume"); });
  document.addEventListener("pointerdown", (event) => {
    if (!state || popover.contains(event.target)) return;
    const { anchor } = state;
    const trigger = event.clientX >= anchor.left && event.clientX <= anchor.left + anchor.width
      && event.clientY >= anchor.top && event.clientY <= anchor.top + anchor.height;
    send("dismiss", trigger ? "trigger" : "outside");
  }, true);
  window.addEventListener("keydown", (event) => {
    if (!state || !["Escape", "Tab"].includes(event.key)) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    send("dismiss", event.key === "Escape" ? "escape" : "tab");
  }, true);
  window.addEventListener("resize", position);
})();

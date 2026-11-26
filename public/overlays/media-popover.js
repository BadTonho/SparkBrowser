(() => {
  const popover = document.getElementById("media-popover");
  const slider = document.getElementById("page-volume");
  const percentage = document.getElementById("volume-percentage");
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

  function send(action, reason = "") {
    if (!state || closing) return;
    cancelAnimationFrame(pendingFrame);
    pendingFrame = 0;
    const url = new URL(`spark-internal://media-popover/${action}`);
    url.searchParams.set("token", state.token);
    url.searchParams.set("sequence", String(++sequence));
    if (!slider.disabled) url.searchParams.set("value", slider.value);
    if (reason) url.searchParams.set("reason", reason);
    if (action === "dismiss") closing = true;
    window.location.href = url.toString();
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

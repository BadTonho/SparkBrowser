import { focusAddressBar } from "../services/bridge/navigation";
import { logIgnored } from "../services/ignored-error";

/**
 * Pressing an empty (drag) area of the title bar starts a native window drag, which never gives the keyboard
 * focus to the interface: shortcuts such as F5 then go nowhere. Pressing such an area focuses the interface
 * web view first, so the shortcuts keep working. Buttons and tabs already take the focus themselves.
 */
export function focusShellOnDrag(node: HTMLElement) {
  const onPointerDown = (event: PointerEvent) => {
    if (event.button !== 0) return;
    const target = event.target;
    if (!(target instanceof Element) || !target.hasAttribute("data-tauri-drag-region")) return;
    void focusAddressBar().catch(logIgnored);
  };
  node.addEventListener("pointerdown", onPointerDown);
  return { destroy: () => node.removeEventListener("pointerdown", onPointerDown) };
}

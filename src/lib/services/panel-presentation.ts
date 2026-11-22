/** A presentation-only tree. No markup, scripts, selectors or IPC names cross the bridge. */
import { isDownloadThumbnail, DOWNLOAD_THUMBNAIL_BUDGET } from "./download-visual";
export interface PanelViewNode {
  id: number;
  tag?: string;
  text?: string;
  attributes?: Record<string, string | boolean>;
  value?: string;
  checked?: boolean;
  children?: PanelViewNode[];
}
export type PanelViewAction =
  | { kind: "click" | "submit"; target: number }
  | { kind: "profilecreate"; target: number; request: number; packet: string }
  | { kind: "input" | "change"; target: number; value: string; checked: boolean }
  | { kind: "key"; target: number; key: "ArrowLeft" | "ArrowRight" | "Home" | "End" }
  | { kind: "keyboard"; target: number; key: string; ctrl: boolean; alt: boolean; shift: boolean; meta: boolean }
  | { kind: "mouse"; target: number; button: number; ctrl: boolean; alt: boolean; shift: boolean; meta: boolean }
  | { kind: "dismiss"; reason: "escape" | "outside" | "trigger" | "blur" };
export interface PanelPresentation {
  anchor: { left: number; top: number; width: number; height: number };
  tree: PanelViewNode;
  theme: Record<string, string>;
  locale: string;
  ack: number;
  revision: number;
}
export function isPanelTrigger(anchor: PanelPresentation["anchor"], x: number, y: number): boolean {
  return x >= anchor.left && x <= anchor.left + anchor.width && y >= anchor.top && y <= anchor.top + anchor.height;
}
const tags = new Set(["div", "header", "h1", "h2", "h3", "h4", "section", "main", "aside", "nav", "button", "span", "p", "form", "input", "textarea", "select", "option", "article", "strong", "small", "label", "ul", "li", "ol", "dl", "dt", "dd", "table", "thead", "tbody", "tr", "th", "td", "details", "summary", "pre", "code", "kbd", "progress", "svg", "path", "circle", "img"]);
const attributes = new Set(["class", "id", "role", "title", "type", "name", "placeholder", "maxlength", "min", "max", "step", "rows", "required", "disabled", "open", "tabindex", "value", "width", "height", "viewBox", "fill", "stroke", "stroke-width", "stroke-linecap", "stroke-linejoin", "d", "cx", "cy", "r", "focusable"]);

/** Keeps existing Svelte controllers (including the single note writer) in the shell.
 * Only their safe presentation and opaque control handles are projected into the native surface. */
export function createPanelProjection(root: HTMLElement) {
  let serial = 0;
  let imageBytes = 0;
  const ids = new WeakMap<Node, number>();
  const handles = new Map<number, HTMLElement>();
  const id = (node: Node) => { let value = ids.get(node); if (!value) { value = ++serial; ids.set(node, value); } return value; };
  function node(source: Node): PanelViewNode | null {
    if (source.nodeType === Node.TEXT_NODE) return { id: id(source), text: source.textContent ?? "" };
    if (!(source instanceof Element) || !tags.has(source.localName)) return null;
    if (source.localName === "img") {
      const src = [...source.attributes].find(attr => attr.name === "src")?.value;
      if (!isDownloadThumbnail(src) || imageBytes + src.length > DOWNLOAD_THUMBNAIL_BUDGET) return null;
      imageBytes += src.length;
    }
    const attrs: Record<string, string | boolean> = {};
    for (const attr of source.attributes) {
      if (attributes.has(attr.name) || attr.name.startsWith("aria-") || attr.name.startsWith("data-") || (source.localName === "img" && ["src", "alt"].includes(attr.name)))
        attrs[attr.name] = ["disabled", "required", "open"].includes(attr.name) ? true : attr.value;
    }
    const result: PanelViewNode = { id: id(source), tag: source.localName, attributes: attrs,
      children: [...source.childNodes].map(node).filter((child): child is PanelViewNode => child !== null) };
    if (source instanceof HTMLElement) handles.set(result.id, source);
    if (source instanceof HTMLInputElement || source instanceof HTMLTextAreaElement || source instanceof HTMLSelectElement) result.value = source.value;
    if (source instanceof HTMLInputElement) result.checked = source.checked;
    return result;
  }
  return {
    snapshot() { handles.clear(); imageBytes = 0; return node(root)!; },
    dispatch(action: PanelViewAction) {
      if (action.kind === "dismiss") return;
      const target = handles.get(action.target);
      if (!target?.isConnected || !root.contains(target) || target.closest("[disabled]")) return;
      if (action.kind === "profilecreate" && target instanceof HTMLButtonElement && target.hasAttribute("data-profile-create-key")) {
        target.dispatchEvent(new CustomEvent("profilecreate", { detail: { request: action.request, packet: action.packet } }));
      } else if (action.kind === "click" && target.matches("button, summary")) target.click();
      else if (action.kind === "submit" && target instanceof HTMLFormElement) target.requestSubmit();
      else if ((action.kind === "input" || action.kind === "change") &&
        (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement)) {
        if (action.value.length > 1_000_000) return;
        target.value = action.value;
        if (target instanceof HTMLInputElement) target.checked = action.checked;
        target.dispatchEvent(new Event(action.kind, { bubbles: true }));
      } else if (action.kind === "keyboard") {
        target.dispatchEvent(new KeyboardEvent("keydown", { key: action.key, ctrlKey: action.ctrl, altKey: action.alt, shiftKey: action.shift, metaKey: action.meta, bubbles: true, cancelable: true }));
      } else if (action.kind === "mouse") {
        target.dispatchEvent(new MouseEvent("mousedown", { button: action.button, ctrlKey: action.ctrl, altKey: action.alt, shiftKey: action.shift, metaKey: action.meta, bubbles: true, cancelable: true }));
      } else if (action.kind === "key" && target.getAttribute("role") === "tab")
        target.dispatchEvent(new KeyboardEvent("keydown", { key: action.key, bubbles: true }));
    },
  };
}

export function panelPosition(anchor: PanelPresentation["anchor"], width: number, height: number, panel = { width: 640, maxHeight: 640 }) {
  const panelWidth = Math.max(0, Math.min(panel.width, width - 16));
  const top = Math.max(8, anchor.top + anchor.height + 8);
  return { left: Math.max(8, Math.min(anchor.left + anchor.width - panelWidth, width - panelWidth - 8)),
    top, width: panelWidth, maxHeight: Math.max(0, Math.min(panel.maxHeight, height - top - 8)) };
}

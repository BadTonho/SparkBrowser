/** A presentation-only tree. No markup, scripts, selectors or IPC names cross the bridge. */
export interface ToolsViewNode {
  id: number;
  tag?: string;
  text?: string;
  attributes?: Record<string, string | boolean>;
  value?: string;
  checked?: boolean;
  children?: ToolsViewNode[];
}
export type ToolsViewAction =
  | { kind: "click" | "submit"; target: number }
  | { kind: "input" | "change"; target: number; value: string; checked: boolean }
  | { kind: "key"; target: number; key: "ArrowLeft" | "ArrowRight" | "Home" | "End" }
  | { kind: "dismiss"; reason: "escape" | "outside" | "trigger" | "blur" };
export interface ToolsPresentation {
  anchor: { left: number; top: number; width: number; height: number };
  tree: ToolsViewNode;
  theme: Record<string, string>;
  locale: string;
  ack: number;
  revision: number;
}
const tags = new Set(["div", "header", "h2", "h3", "nav", "button", "span", "p", "form", "input", "textarea", "select", "option", "article", "strong", "small", "label", "ul", "li", "details", "summary", "pre", "svg", "path", "circle"]);
const attributes = new Set(["class", "id", "role", "title", "type", "name", "placeholder", "maxlength", "min", "max", "rows", "required", "disabled", "open", "tabindex", "value", "width", "height", "viewBox", "fill", "stroke", "stroke-width", "stroke-linecap", "stroke-linejoin", "d", "cx", "cy", "r", "focusable"]);

/** Keeps existing Svelte controllers (including the single note writer) in the shell.
 * Only their safe presentation and opaque control handles are projected into the native surface. */
export function createToolsProjection(root: HTMLElement) {
  let serial = 0;
  const ids = new WeakMap<Node, number>();
  const handles = new Map<number, HTMLElement>();
  const id = (node: Node) => { let value = ids.get(node); if (!value) { value = ++serial; ids.set(node, value); } return value; };
  function node(source: Node): ToolsViewNode | null {
    if (source.nodeType === Node.TEXT_NODE) return { id: id(source), text: source.textContent ?? "" };
    if (!(source instanceof Element) || !tags.has(source.localName)) return null;
    const attrs: Record<string, string | boolean> = {};
    for (const attr of source.attributes) {
      if (attributes.has(attr.name) || attr.name.startsWith("aria-") || attr.name.startsWith("data-"))
        attrs[attr.name] = ["disabled", "required"].includes(attr.name) ? true : attr.value;
    }
    const result: ToolsViewNode = { id: id(source), tag: source.localName, attributes: attrs,
      children: [...source.childNodes].map(node).filter((child): child is ToolsViewNode => child !== null) };
    if (source instanceof HTMLElement) handles.set(result.id, source);
    if (source instanceof HTMLInputElement || source instanceof HTMLTextAreaElement || source instanceof HTMLSelectElement) result.value = source.value;
    if (source instanceof HTMLInputElement) result.checked = source.checked;
    return result;
  }
  return {
    snapshot() { handles.clear(); return node(root)!; },
    dispatch(action: ToolsViewAction) {
      if (action.kind === "dismiss") return;
      const target = handles.get(action.target);
      if (!target?.isConnected || !root.contains(target) || target.closest("[disabled]")) return;
      if (action.kind === "click" && target.matches("button, summary")) target.click();
      else if (action.kind === "submit" && target instanceof HTMLFormElement) target.requestSubmit();
      else if ((action.kind === "input" || action.kind === "change") &&
        (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement)) {
        if (action.value.length > 1_000_000) return;
        target.value = action.value;
        if (target instanceof HTMLInputElement) target.checked = action.checked;
        target.dispatchEvent(new Event(action.kind, { bubbles: true }));
      } else if (action.kind === "key" && target.getAttribute("role") === "tab")
        target.dispatchEvent(new KeyboardEvent("keydown", { key: action.key, bubbles: true }));
    },
  };
}

export function toolsPosition(anchor: ToolsPresentation["anchor"], width: number, height: number) {
  const panelWidth = Math.max(0, Math.min(640, width - 16));
  const top = Math.max(8, anchor.top + anchor.height + 8);
  return { left: Math.max(8, Math.min(anchor.left + anchor.width - panelWidth, width - panelWidth - 8)),
    top, width: panelWidth, maxHeight: Math.max(0, Math.min(640, height - top - 8)) };
}

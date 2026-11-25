// Allowlist sanitizer for article HTML extracted from arbitrary web pages.
// Anything not explicitly allowed is removed, and unknown wrappers are unwrapped
// so their text survives. Runs in the webview, which provides DOMParser.

const DROP_WITH_CONTENT = new Set([
  "applet", "audio", "base", "button", "canvas", "dialog", "embed", "form", "frame",
  "frameset", "iframe", "input", "link", "math", "meta", "noscript", "object", "script",
  "select", "source", "style", "svg", "template", "textarea", "track", "video",
]);

const ALLOWED_TAGS = new Set([
  "b", "blockquote", "br", "caption", "cite", "code", "dd", "div", "dl", "dt", "em",
  "figcaption", "figure", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "i", "img", "li",
  "mark", "ol", "p", "pre", "q", "s", "small", "span", "strong", "sub", "sup", "table",
  "tbody", "td", "tfoot", "th", "thead", "tr", "u", "ul",
]);

const DIMENSION = /^\d{1,5}$/;
const COUNT = /^\d{1,4}$/;
// The shell's content security policy only loads `data:` images, so article images arrive from
// the tab's cache as `data:` URLs (see `images`) and any other source is removed.
const SAFE_IMAGE_SRC = /^data:image\/(png|jpe?g|gif|webp);base64,/i;

const ATTRIBUTE_RULES: Record<string, Record<string, (value: string) => boolean>> = {
  img: {
    alt: () => true,
    title: () => true,
    width: value => DIMENSION.test(value),
    height: value => DIMENSION.test(value),
    src: value => SAFE_IMAGE_SRC.test(value.trim()),
  },
  td: { colspan: value => COUNT.test(value), rowspan: value => COUNT.test(value) },
  th: {
    colspan: value => COUNT.test(value),
    rowspan: value => COUNT.test(value),
    scope: value => ["row", "col", "rowgroup", "colgroup"].includes(value),
  },
};

const DEFAULT_ATTRIBUTE_RULES: Record<string, (value: string) => boolean> = {
  title: () => true,
};

function sanitizeAttributes(element: Element, tag: string): void {
  const rules = ATTRIBUTE_RULES[tag] ?? DEFAULT_ATTRIBUTE_RULES;
  for (const attribute of Array.from(element.attributes)) {
    const name = attribute.name.toLowerCase();
    const rule = rules[name];
    if (!rule || !rule(attribute.value)) {
      element.removeAttribute(attribute.name);
    }
  }
}

function sanitizeChildren(parent: Element, images: Record<string, string>): void {
  for (const child of Array.from(parent.childNodes)) {
    if (child.nodeType === Node.TEXT_NODE) continue;
    if (!(child instanceof Element)) {
      child.remove();
      continue;
    }

    const tag = child.tagName.toLowerCase();
    if (DROP_WITH_CONTENT.has(tag)) {
      child.remove();
      continue;
    }

    if (!ALLOWED_TAGS.has(tag)) {
      // Keep the text of unknown or link wrappers, drop the element itself.
      sanitizeChildren(child, images);
      child.replaceWith(...Array.from(child.childNodes));
      continue;
    }

    if (tag === "img") {
      // Only images read from the tab's cache can be shown; the others would appear broken.
      const address = child.getAttribute("src") ?? "";
      const source = Object.hasOwn(images, address) ? images[address] : undefined;
      if (!source) {
        child.remove();
        continue;
      }
      child.setAttribute("src", source);
    }

    sanitizeAttributes(child, tag);
    if (tag === "img" && !child.hasAttribute("src")) {
      child.remove();
      continue;
    }
    sanitizeChildren(child, images);
  }
}

/** Sanitizes article HTML. `images` maps each image address in the article to its `data:` URL. */
export function sanitizeArticleHtml(html: string, images: Record<string, string> = {}): string {
  if (typeof DOMParser === "undefined") return "";
  const body = new DOMParser().parseFromString(html, "text/html").body;
  sanitizeChildren(body, images);
  return body.innerHTML;
}

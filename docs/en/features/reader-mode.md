# Immersive Reader View (Modo Leitura)

Spark Browser features a distraction-free, customizable Reader View that strips ads, navbars, sidebars, and cluttered formatting to present web articles with clean typography and focused aesthetics.

---

## Capabilities

* **Intelligent Content Extraction:** Analyzes webpage DOM to isolate main article headings, bylines, paragraphs, blockquotes, code snippets, and sanitized illustrations while discarding noisy tracking scripts, banners, and layout bloat.
* **Reading Comfort Themes:**
  * **Paper:** Soft daylight reading palette (`#FBFBF9`).
  * **Sepia:** Warm, low-contrast parchment palette ideal for long-form essays (`#F6EFE2`).
  * **Dark:** Modern slate contrast theme reducing eye strain in low-light environments (`#1E1E24`).
  * **OLED:** True pitch-black background saving battery on modern OLED displays (`#000000`).
* **Custom Typography & Sizing:**
  * **Serif:** Merriweather / Georgia editorial aesthetic.
  * **Sans-serif:** Inter / System modern typographic stack.
  * **Monospace:** Technical typography for development and programming articles.
  * **Dynamic Font Sizing:** Adjustable scaling from 14px to 28px.
  * **Adjustable Reading Width:** Narrow (640px), Medium (760px), and Wide (940px) columns.
* **Content Safety:** Scripts, iframes, forms, embedded objects, and inline event handlers are removed from extracted articles. Links in article text appear as plain text.
* **Article Images:** Images are read from the tab's own cache (the copy the page already loaded) and shown as embedded `data:` images, because the browser interface does not load remote images. Nothing is downloaded again. Images the page had not loaded yet (for example, lazy images further down), vector images, and images over 4 MB (or past 24 MB in total, up to 40 per article) are left out.
* **Platforms:** Article extraction is available on Windows. On Linux, opening Reader View shows a message, in the interface language, saying that it is available on Windows only.
* **Reading Metrics:** Displays estimated reading time ("~4 min read") and article word counts.
* **Native Viewport Protection:** Coordinates with Tauri native viewport overlays (`setOverlayVisibility(true, "reader-mode")`) to seamlessly obscure child WebViews beneath the reader canvas.

---

## How to Use Reader View

1. Navigate to any news article, blog post, or editorial web page.
2. In the Omnibox (address bar), a **Reader View (Book icon)** button appears automatically.
3. Click the button to enter Reader View.
4. Customize your theme, font family, column width, and sizing in the floating top toolbar.
5. Press **Escape (`Esc`)** or click **Close Reader View** to return to the original web document.


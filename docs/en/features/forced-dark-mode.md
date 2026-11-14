# Dark mode for websites

**Settings → Appearance & Layout → Dark mode for websites** darkens websites that are light. It is **off by default** and applies to every tab at once, without reloading.

## What it does and does not do

- It acts **only on light pages**. After a page loads, the browser reads the colour actually painted behind the content (the page background, or the white canvas when the page paints none). If it is light, the page is darkened. A page that is already dark, or that switched itself to a dark theme through the system preference, is left exactly as authored.
- The decision is re-checked when the page changes its own theme (class or style changes on the page root or body), so a site that switches to its dark theme loses the filter instead of being flipped back to light.
- Darkening inverts the page's colours and rotates the hue back, so accent colours stay close to the original. Images, video, canvas, embeds and frames are inverted back so photos keep their real colours. Side effects: logos drawn in black can end up dark on dark, and background images are inverted along with the page.
- It works the same on Windows and Linux because it is applied by a script in the page, not by an engine flag. Only top-level pages are handled; the browser's own screens are not.
- The script has no way to talk to the application; the shell switches it on with a per-view token, the same way as the automatic ad skipping.

## Storage

The choice is saved on this device (like the other appearance switches) and is not part of backups. The native side keeps it in memory only, and the interface sends it again at start-up.

## Performance

On the test machine, scrolling a long article and a heavy news page with the filter active held the same frame cadence as without it (no frame over 33 ms in either case). Lower-end graphics hardware was not measured.

## Related

The original feasibility study is in [Forced dark mode for pages: feasibility investigation](forced-dark-mode-investigation.md).

# Search engines

Settings → Search & Navigation → **Default Search Engine** is a grid of tiles instead of a drop-down: each engine shows its logo, its name and a one-line description, and the chosen one is outlined with a border (nothing else changes, so the grid never shifts). The choice is used when you type a search in the address bar, for new tabs and for the home button.

## Available engines

| Engine | Notes |
| --- | --- |
| Google | Default; most complete results |
| DuckDuckGo | Private, no profile tracking |
| Bing | Microsoft's search |
| Brave Search | Independent index, private |
| Startpage | Google results, anonymously |
| Ecosia | Plants trees with ad revenue |
| Qwant | European and private |
| Yahoo | Classic web search (its query parameter is `p`, not `q`) |
| Mojeek | Independent, no tracking |
| Kagi | Ad-free, paid subscription; results need a Kagi login |

The descriptions are short summaries meant to help choosing, not guarantees: each service's own privacy policy is what applies. Google is the default for new profiles, and it is also what any unknown or stale value falls back to.

Site search shortcuts (for example `yt cats`) still win over the default engine; see [Site search shortcuts](site-search.md).

## Logos

Brand icons sit on neutral tiles that follow the current theme. The multicolor G and Dax use artwork from the [Google brand assets](https://developers.google.com/identity/branding-guidelines#download_pre-approved_brand_icons) and [DuckDuckGo press kit](https://duckduckgo.com/press). The other marks use single-color paths from [Simple Icons](https://github.com/simple-icons/simple-icons) (CC0 1.0). Bing uses version 12, Yahoo uses version 9, and the others use version 13. Spark bundles the artwork for offline use. Each mark identifies its search engine and remains the trademark of its owner.

## Compatibility

- Engine ids (`google`, `duckduckgo`, `bing`, `brave`, `startpage`, `ecosia`, `qwant`, `yahoo`, `mojeek`, `kagi`) are stored in settings and in backups, so they never change.
- An unknown or stale id falls back to Google instead of failing, on both the native and the interface side.
- Backups that contain one of the new engines are rejected by older builds, which only knew `google` and `duckduckgo`. Importing a backup that chose a new engine into this build works, and choosing to apply preferences restores it.

## Adding an engine

1. Add an entry to `src-tauri/src/search_engines.rs` (id, label, home page, results page and query parameter).
2. Add the same entry, in the same order, to `src/lib/services/search-engines.js`, with a `color`.
3. Add `settings.engine_<id>_desc` to both settings catalogs.
4. Add the logo path to `src/lib/components/icons/searchEngineIcons.js` (24×24, path data only).

`tests/search-engines.test.mjs` checks that the lists match, that every engine has a description and a bundled icon, and that every engine uses HTTPS. Rust tests check that every engine builds a valid search URL.

## Implementation

- Native: `search_engines.rs` is the single registry used by `url_input.rs`, the new-tab preferences and backup validation.
- Interface: `search-engines.js` feeds `searchEngineStore.ts`, the offline preview in `site-search.js` and `SearchEnginePicker.svelte`. The picker uses buttons with `role="radio"`. Its icon styles and SVG fill attributes work in both native Settings surfaces.

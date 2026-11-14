# Acknowledgements and third-party licenses

[Português](../../pt-BR/legal/acknowledgements.md)

Spark is built on the work of many open-source projects and services. This page credits the ones that Spark uses directly, and explains the licensing terms that apply to them. Spark's own code is licensed under the [Apache License 2.0](../../../LICENSE).

## Data and services

| Project | What Spark uses it for | License and terms |
| --- | --- | --- |
| [Open-Meteo](https://open-meteo.com/) | Weather forecasts and city search in the new-tab weather widget | Data under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/); attribution is shown in the widget. The free endpoint is for non-commercial use, see its [pricing page](https://open-meteo.com/en/pricing). |
| [EasyList and EasyPrivacy](https://easylist.to/) | Bundled filter lists for the ad and tracker shield | GPL-3.0-or-later and CC BY-SA 3.0. Kept unmodified, with their original headers. See the [filter list notice](../../../src-tauri/resources/filterlists/README.md). |
| [NoCoin Filter List](https://github.com/hoshsadiq/adblock-nocoin-list) by Hosh | Bundled list that blocks in-browser cryptocurrency miners | MIT, see [`NOCOIN-LICENSE.txt`](../../../src-tauri/resources/filterlists/NOCOIN-LICENSE.txt). |

## Design assets

- **[Simple Icons](https://github.com/simple-icons/simple-icons)** (CC0 1.0): the single-color search engine logos. Versions and details are in [Search engines](../features/search-engines.md).
- Brand names and logos belong to their owners. Their appearance in Spark does not imply any affiliation or endorsement.

## Code included in the repository

- **[deepmerge](https://github.com/TehShrike/deepmerge)** 4.3.1 (MIT, copyright James Halliday, Josh Duff and other contributors): a private, patched copy lives in `tools/vendor/deepmerge` with its original [license](../../../tools/vendor/deepmerge/LICENSE). It only affects the build tooling, and the reason for the patch is in the [dependency notes](../security/dependency-notes.md).

## Core technologies

| Project | Role in Spark |
| --- | --- |
| [Tauri](https://tauri.app/) (with Wry and Tao) | Desktop application framework, windows and web views |
| [Svelte](https://svelte.dev/), [Vite](https://vitejs.dev/) and [TypeScript](https://www.typescriptlang.org/) | Browser interface and build tooling |
| [adblock-rust](https://github.com/brave/adblock-rust) by Brave | Network rule engine behind the shield (MPL-2.0) |
| [SQLite](https://www.sqlite.org/) through [rusqlite](https://github.com/rusqlite/rusqlite) | Local storage |
| [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) and [WebKitGTK](https://webkitgtk.org/) | Web content engines on Windows and Linux |
| [windows-rs](https://github.com/microsoft/windows-rs) and [webview2-com](https://github.com/wravery/webview2-rs) | Windows native integration |
| [gtk-rs](https://gtk-rs.org/) and [webkit2gtk-rs](https://github.com/tauri-apps/webkit2gtk-rs) | Linux native integration |
| [RustCrypto](https://github.com/RustCrypto) (`aes-gcm`, `argon2`, `sha2`) and [rustls](https://github.com/rustls/rustls) | Encryption, hashing and TLS |
| [tokio](https://tokio.rs/), [reqwest](https://github.com/seanmonstar/reqwest), [serde](https://serde.rs/) and [url](https://github.com/servo/rust-url) | Asynchronous work, HTTP and data handling |
| [window-vibrancy](https://github.com/tauri-apps/window-vibrancy) | Native window effects |
| [qrcode](https://github.com/soldair/node-qrcode) | QR code generation |

These are only the direct dependencies. Each project's own dependencies are credited by their authors in their repositories.

## Dependency licenses

All direct and transitive dependencies are open source. Rust dependencies come from `Cargo.lock` and JavaScript dependencies from `package-lock.json`.

- The large majority are licensed under MIT, Apache-2.0, or both. Others use ISC, BSD, Zlib, Unicode or similar permissive terms.
- A few crates use MPL-2.0, such as `adblock`, `cssparser` and `selectors`. MPL-2.0 applies per file: it is met by keeping those crates' sources and notices intact, which Cargo does, and it does not extend to Spark's own code.
- No dependency requires GPL, LGPL or AGPL terms. Where a crate offers a choice that includes a copyleft license (for example `r-efi`), Spark uses it under the permissive option.

You can list the exact licenses of your build with `cargo metadata --manifest-path src-tauri/Cargo.toml` and `npm ls --all`.

## Independence

Spark is an independent project. It is not affiliated with, endorsed by or sponsored by Google, Microsoft, Brave, Mozilla or any of the other companies and projects named on this page. Product and company names are trademarks of their respective owners.

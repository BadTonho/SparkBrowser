# Network privacy investigation

Status: **Tor session, DNS over HTTPS, and HTTPS-only mode are unavailable**. None is enabled or advertised as providing protection. The privacy settings display these limitations; no configuration, DNS provider, proxy, or one-time HTTP exception is active.

## Findings

- Tor: the browser has no Tor process or routing integration. The user-selected design uses an external local service and a separate disposable session with no direct fallback. Microsoft documents that WebView2's [`proxy-server` flag](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags) affects HTTP and HTTPS requests. It does not by itself prove coverage of WebSockets, WebRTC, DNS, downloads, or other traffic. WebKitGTK documents [proxy settings](https://webkitgtk.org/reference/webkit2gtk/2.41.6/ctor.NetworkProxySettings.new.html), but this alone does not prove routing and leak prevention. SOCKS support and Tor's [remote-name-resolution behavior](https://spec.torproject.org/socks-extensions) need end-to-end validation with the actual engines.
- DNS over HTTPS: the browser currently has no application-controlled secure DNS integration for its WebViews. An HTTPS request made by Rust does not configure DNS for WebView requests. Strict mode needs engine-level coverage, explicit provider selection, and proof that errors cannot silently fall back to ordinary DNS, including bootstrap and redirects.
- HTTPS-only: the current Tauri navigation callback receives a URL and can cancel navigation; the shared callback does not expose all context needed for an authenticated one-use exception and error correlation. Windows also has native navigation observers, but they currently report status rather than enforce upgrades. A complete implementation must verify frame, redirect, POST, new-window, restore and failure behavior without retries changing request semantics or allowing HTTP before confirmation.

These findings describe evidence available for the current project, not a claim that the engines can never support the features. Windows and Linux native engine behavior remains to be measured. An unavailable feature must fail closed; changes to OS-wide DNS or proxy settings are not authorized by these controls.

## Test assets

`tests/native/network-privacy-fixture.mjs` starts an HTTP service and a SOCKS5 request recorder on loopback only. The SOCKS endpoint refuses requests and never forwards them; it is not Tor. `tests/network-privacy-fixture.test.mjs` tests the fixture. The `network-privacy-baseline` native scenario checks ordinary direct navigation only: passing it does not prove Tor routing, DNS privacy, HTTPS-only behavior, or absence of leaks.

On 7 November 2026, the Windows baseline passed with WebView2 157.0.4341.42, observing a local HTTP load and redirect. The Linux baseline has not been run.

`src-tauri/src/privacy/network_policy.rs` is compiled only for unit tests. It explores URL upgrades, exact one-use permissions bound to a tab, WebView generation and navigation, and HTTPS endpoint validation. It is not wired into browser navigation or provider configuration and is not a production policy.

## Remaining validation before enabling

- Build native adapters for each engine; capture local direct and proxied traffic and DNS across documents, redirects, frames, WebSockets, WebRTC, workers and downloads. Test connection loss, restarts, session isolation, discarded storage and proxy bypass rules. Define how failure remains closed across every covered path.
- Verify DoH endpoint bootstrap and provider failure without changing the user's system resolver; confirm all claimed WebView traffic follows the selected policy. For Tor sessions resolve names through Tor, not a separate DoH request.
- Verify HTTPS upgrades and failure events correlate with the exact original navigation. Test one-time authorization, stale callbacks, redirects, non-idempotent requests, ports, fragments, subframes and recovery without silent HTTP fallback.
- Run native scenarios on Windows and Linux when each environment is available and record engine versions. A missing test environment is a pending validation, not proof of support or lack of support. Keep a feature unavailable on a platform until its claimed behavior is verified.

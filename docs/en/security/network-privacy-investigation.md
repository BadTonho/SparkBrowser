# Network privacy investigation

Status: **HTTPS-only mode and DNS over HTTPS are implemented** (off by default, see [Network privacy](../features/network-privacy.md)). **The Tor session is deferred**: it is hidden from the interface and nothing about it is active.

## Findings

- Tor (deferred): the browser has no Tor process or routing integration. The user-selected design uses an external local service and a separate disposable session with no direct fallback. Microsoft documents that WebView2's [`proxy-server` flag](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags) affects HTTP and HTTPS requests. It does not by itself prove coverage of WebSockets, WebRTC, DNS, downloads, or other traffic. WebKitGTK documents [proxy settings](https://webkitgtk.org/reference/webkit2gtk/2.41.6/ctor.NetworkProxySettings.new.html), but this alone does not prove routing and leak prevention. SOCKS support and Tor's [remote-name-resolution behavior](https://spec.torproject.org/socks-extensions) need end-to-end validation with the actual engines. The feature returns only when that validation exists.
- DNS over HTTPS: neither engine has an application-controlled secure DNS setting, and an HTTPS request made by Rust does not configure DNS for WebView requests. The implemented design is a loopback-only proxy that resolves names through DoH and is given to the engines (WebView2 start-up arguments, WebKitGTK proxy settings). It is strict (no fallback to ordinary DNS) and reaches the provider by fixed IP address, so the bootstrap never uses the system DNS. WebRTC media and the browser's own requests are outside it.
- HTTPS-only: the Tauri navigation callback receives each top-level URL and can cancel it, which is enough to cancel an HTTP address and open its HTTPS form. Failures come from the native observers (WebView2 `NavigationCompleted`, WebKitGTK `load-failed`), correlated with the tab's pending upgrade for 60 seconds. The one-use exception is bound to the tab and the exact address.

These findings describe evidence available for the current project, not a claim that the engines can never support the features. An unavailable feature must fail closed; changes to OS-wide DNS or proxy settings are not authorized by these controls.

## Test assets

`tests/native/network-privacy-fixture.mjs` starts an HTTP service and a SOCKS5 request recorder on loopback only. The SOCKS endpoint refuses requests and never forwards them; it is not Tor. `tests/network-privacy-fixture.test.mjs` tests the fixture. The `network-privacy-baseline` native scenario checks ordinary direct navigation only: passing it does not prove DNS privacy, HTTPS-only behavior, or absence of leaks.

On 7 November 2026, the Windows baseline passed with WebView2 157.0.4341.42, observing a local HTTP load and redirect. The Linux baseline has not been run.

Rust unit tests cover the HTTPS-only policy (`privacy/https_only.rs`), the DNS answer parsing, the proxy (tunnels, plain requests, failures and malformed requests, against local servers) and the saved setting (`network/`).

## Remaining validation

- HTTPS-only: run it on both engines against real redirects, subframes, POST navigation, restored tabs and pop-ups, and confirm no HTTP request leaves before the user confirms. The WebKitGTK failure hook and the WebView2 hook are covered by code review and unit tests of the policy, not yet by a native scenario.
- DNS over HTTPS: capture traffic on both engines and confirm that page traffic reaches the proxy and no name is resolved by the system DNS while it is on, including with WebSockets, workers, downloads and extensions. Confirm the Linux per-WebView proxy also covers isolated-profile WebViews. Record the engine versions.
- Tor (when resumed): build native adapters for each engine; capture local direct and proxied traffic and DNS across documents, redirects, frames, WebSockets, WebRTC, workers and downloads. Test connection loss, restarts, session isolation, discarded storage and proxy bypass rules. Resolve names through Tor, not through DoH.
- A missing test environment is a pending validation, not proof of support or lack of support.

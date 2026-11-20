# Network privacy

[Português](../../pt-BR/features/network-privacy.md)

The **Network privacy** group in **Settings → Privacy & Security** has two protections. Both are off by default. A third one, the Tor session, is [deferred](#tor-session-deferred) and is not shown in the interface.

## HTTPS-only mode

When the switch is on, every top-level `http://` navigation is cancelled and its `https://` form is opened instead (same host, port, path, query and fragment). It applies at once, with no restart.

- **When the secure version cannot be reached**, a notice appears at the top of the window: *Secure connection not available*. **Continue over HTTP** opens the original address **once**.
- **The exception is exact and used once.** It is bound to one tab and to that exact address. The next navigation (to any address, including the same one) is upgraded again, and another tab never inherits it. Turning the mode off drops every pending exception.
- **Not changed:** `https://` addresses, and addresses of the local machine (`localhost`, `*.localhost`, `127.0.0.0/8` and `::1`), which have no certificate to check.
- **Addresses with a user name or password** over `http://` are never opened (there is no safe HTTPS form of them).
- A bare host typed in the address bar (`example.com`) already opens as `https://`, with or without the mode.

What it does not cover: resources a page loads (images, scripts) and frames. Browsers block or upgrade those as mixed content, and the engines already do so for pages loaded over HTTPS. The mode acts on the address of the page itself.

How a failure is recognized: the upgrade is remembered for the tab for 60 seconds. A navigation error of that tab within the window (WebView2 `NavigationCompleted` with an error, or WebKitGTK `load-failed`) opens the notice. A navigation the browser cancelled itself, such as the replaced HTTP address, is not an error. A page that loaded correctly clears the pending upgrade.

Code: [`privacy/https_only.rs`](../../../src-tauri/src/privacy/https_only.rs) (policy and tests), the navigation callback in `window/content_factory.rs`, the failure hooks in `window/navigation_state.rs` and the notice in `HttpsOnlyNotice.svelte`.

## DNS over HTTPS

WebView2 and WebKitGTK have no setting for secure DNS. Spark therefore starts a small **proxy on the loopback address** (`127.0.0.1`, an address only this computer can reach) and points the web engines at it. The proxy resolves every host name through the chosen provider using DNS over HTTPS and then connects to the answer.

- **Providers:** Cloudflare (`1.1.1.1`) and Google (`8.8.8.8`). The provider server is reached by a fixed IP address, so asking for the provider never uses the system DNS.
- **Strict mode:** if the provider cannot be reached, or a name cannot be resolved, the page does not load (the proxy answers *502 Bad Gateway* with an explanation). There is **no fallback** to the ordinary DNS. To browse without it, choose **Off** and restart.
- **A restart applies the change.** The engine takes the proxy address when it starts, like [hardware acceleration](hardware-acceleration.md). The choice is saved in `network.json` in the data folder, which is read before any WebView exists. The setting shows **Restart now** while a change is pending. Private windows use the proxy of the window that opened them.
- **Windows:** the proxy is added to the WebView2 start-up arguments, which are shared by every WebView of the process. **Linux:** each content WebView gets the proxy through the engine's proxy settings.

What it covers: what pages load through the engine's network stack (documents, frames, scripts, images, WebSockets and downloads over HTTP and HTTPS).

What it does **not** cover:

- WebRTC media (UDP), which does not go through an HTTP proxy.
- The browser's own requests made from the application, such as updates, favicons and search suggestions.
- Anything outside Spark. The system DNS and proxy settings are never changed.
- The provider sees the names you look up. DNS over HTTPS hides them from the local network, not from the provider.

Security notes: the proxy listens on the loopback address only, refuses connections that do not come from it and forwards what it receives: every program on this computer can use it as a resolver while Spark runs. Plain `http://` requests are forwarded one per connection. A lookup failure is written to the [error log](error-log.md) as a warning without the host name.

Code: [`network/`](../../../src-tauri/src/network) (`mod.rs` for the setting and the start-up, `resolver.rs` for the lookups, `proxy.rs` for the proxy) and `commands/network.rs`.

## Tor session (deferred)

The **Tor session** is **postponed** and does not appear in Settings. Nothing about it is active. It needs a design that proves, with the real engines, that no traffic leaks outside Tor (DNS, WebRTC, WebSockets, downloads, extensions) and that the session is isolated and disposable. A switch that cannot guarantee that would be worse than none, so it will return only when that proof exists. The findings are in the [network privacy investigation](../security/network-privacy-investigation.md).

## Verification

- Rust tests: the HTTPS-only policy (`privacy::https_only`), the DNS answers, the proxy and the saved setting (`network::`). The proxy tests run tunnels, plain requests and failures against local servers, with no internet.
- The native scenario `network-privacy-baseline` still checks ordinary direct navigation only. A full end-to-end check of both protections on each operating system is a pending validation: record the engine versions when it is run.

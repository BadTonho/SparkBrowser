# Application security boundaries

## Privileged documents

Application commands require both the `main` WebView label and the configured shell document origin and entrypoint. Development trusts only the configured Vite origin; packaged Windows builds use the bundled asset origin. Auxiliary surfaces and browsing tabs receive no application-command capabilities. The shell's initial blank transition is not privileged.

Windows navigation guards prevent the shell from navigating to other documents. Internal panels stay on their packaged documents and cannot open new windows. Browsing tabs keep their existing HTTP/HTTPS navigation and explicit New Tab/PiP flows. WebView2 host objects are explicitly disabled and DevTools stay enabled only in development builds; framework IPC remains available under the application ACL. When the hardening of a content WebView fails to install, the browser terminates instead of running with a partially enforced boundary. These boundaries follow Microsoft's [WebView2 security guidance](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/security).

Internal events containing browser state are sent to the shell explicitly. Content documents are not recipients of browser-wide navigation, tab or permission data.

## Tab lifetime credentials

Each content WebView receives a fresh 256-bit bridge token at creation time. Closing, hibernating or rebuilding a WebView removes its token, its admitted sequences and its internal-page sessions, so retired transactions cannot replay through a successor document. Committed navigation revokes internal-page privileges before the new document loads, and only growing sequence numbers from one document nonce are admitted per WebView generation.

## Internal messages

Only recognized actions and fields are accepted. Query parsing rejects duplicate decoded parameter names, unknown fields, credentials, fragments and oversized requests before state changes. Typed payloads, generation/session tokens and monotonic sequences remain enforced by the owning bridge. Activity messages require literal boolean values; malformed input cannot clear a protected form/media flag. Bridge tokens contain 256 bits from the operating system's cryptographic random source.

Rejections use fixed errors and do not echo tokens or message bodies. The Notes presentation limit accommodates its existing 200 KB text limit.

## Engine maintenance

Windows distributions explicitly use the Evergreen WebView2 bootstrapper. Microsoft manages runtime installation and updates; Veloce does not change Windows update policies or download Chromium source.

The shell observes `NewBrowserVersionAvailable`, exposes the current and available runtime versions and shows a dismissible restart notice. “Later” dismisses only that available version for the current session; Settings → Updates retains the pending restart. A different available version can show a new notice. Detection errors, preview channels and explicitly configured fixed runtimes are represented as unavailable, never as proof that the engine is current. Runtime updates may include security fixes; the notice does not claim to identify particular fixes. Microsoft explains why adopting an installed version requires a [new environment or application restart](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environment#add_newbrowserversionavailable).

Restart is a separate protected-shutdown purpose. Pending notes are flushed, forms/media/downloads are reviewed and the session is checkpointed before relaunch. Cancelling or failing the restart keeps the existing browser open. The relaunch uses the current executable directly, without a shell, and waits for the old process to exit before opening the same profile.

English and Portuguese interface texts and documentation remain synchronized. Workspace content storage can additionally be scoped per workspace as an explicit opt-in (see [workspace isolation](../features/workspace-isolation.md)). Full process containment of the browser host is planned separately in the [AppContainer isolation design](app-container-isolation.md). See also the separate [future reputation protections](../roadmap/security-protections.md).

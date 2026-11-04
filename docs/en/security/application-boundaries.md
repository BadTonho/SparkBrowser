# Application security boundaries

## Privileged documents

Application commands require both the `main` WebView label and the configured shell document origin and entrypoint. Development trusts only the configured Vite origin; packaged Windows builds use the bundled asset origin. Auxiliary surfaces and browsing tabs receive no application-command capabilities. The shell's initial blank transition is not privileged.

Windows navigation guards prevent the shell from navigating to other documents. Internal panels stay on their packaged documents and cannot open new windows. Browsing tabs keep their existing HTTP/HTTPS navigation and explicit New Tab/PiP flows. WebView2 host objects are explicitly disabled; framework IPC remains available under the application ACL. These boundaries follow Microsoft's [WebView2 security guidance](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/security).

Internal events containing browser state are sent to the shell explicitly. Content documents are not recipients of browser-wide navigation, tab or permission data.

## Internal messages

Only recognized actions and fields are accepted. Query parsing rejects duplicate decoded parameter names, unknown fields, credentials, fragments and oversized requests before state changes. Typed payloads, generation/session tokens and monotonic sequences remain enforced by the owning bridge. Activity messages require literal boolean values; malformed input cannot clear a protected form/media flag. Bridge tokens contain 256 bits from the operating system's cryptographic random source.

Rejections use fixed errors and do not echo tokens or message bodies. The Notes presentation limit accommodates its existing 200 KB text limit.

## Engine maintenance

Windows distributions explicitly use the Evergreen WebView2 bootstrapper. Microsoft manages runtime installation and updates; Veloce does not change Windows update policies or download Chromium source.

The shell observes `NewBrowserVersionAvailable`, exposes the current and available runtime versions and shows a dismissible restart notice. “Later” dismisses only that available version for the current session; Settings → Updates retains the pending restart. A different available version can show a new notice. Detection errors, preview channels and explicitly configured fixed runtimes are represented as unavailable, never as proof that the engine is current. Runtime updates may include security fixes; the notice does not claim to identify particular fixes. Microsoft explains why adopting an installed version requires a [new environment or application restart](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environment#add_newbrowserversionavailable).

Restart is a separate protected-shutdown purpose. Pending notes are flushed, forms/media/downloads are reviewed and the session is checkpointed before relaunch. Cancelling or failing the restart keeps the existing browser open. The relaunch uses the current executable directly, without a shell, and waits for the old process to exit before opening the same profile.

English and Portuguese interface texts and documentation remain synchronized. See the separate [future reputation protections](../roadmap/security-protections.md).

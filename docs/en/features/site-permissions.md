# Site permissions on Windows

Click the address bar's **Site permissions** indicator to manage camera, microphone, location and notifications for the focused page's origin. **Settings → Performance & Privacy → Site permissions** lists remembered decisions. Origins include scheme, host and port; rules are shared across workspaces. Ask removes a saved rule, Allow permits future requests and Block denies them. Permission rules live in WebView2's native profile, not a separate database.

The current-site panel and permission requests use a native overlay WebView above page content. The card is opaque while the overlay outside it is transparent, so the page remains visible. Actions are forwarded to the local shell through a session-bound internal navigation bridge; the overlay does not receive application IPC permissions.

For iframe requests, the dialog also displays the native frame origin. WebView2 may apply a media permission to the embedding origin; when those origins differ, the dialog explicitly names the origin that receives the remembered decision. Frame navigation and destruction cancel its pending requests.

New requests show a browser dialog with the requesting origin and, for embedded content, the containing origin. Choose Allow once, Always allow or Block; the checkbox remembers blocking. Escape blocks only the current request. Requests are processed one at a time with at most 16 pending; overflow is denied. Navigation, closing/hibernating the view or a 60-second timeout cancels pending requests. Old request IDs cannot grant permissions to replacement documents or views. Other permission types keep their engine behavior.

Handlers are installed before external navigation. COM request objects remain on the UI thread; only request metadata reaches the local shell. One-time decisions are not saved to the profile. Changing a policy affects new requests and does not stop existing media capture; the current-site panel offers reload and asks for confirmation when detected forms/media/downloads are present. Operating-system permissions and device availability still apply.

Controls are unavailable on other platforms, in browser preview or when the required WebView2 APIs are absent. Native tests use an isolated application identifier, controlled loopback pages and Chromium's fake media device flag. See [native tests](../contributing/native-tests.md).

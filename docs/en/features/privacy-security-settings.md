# Privacy and security settings

The browser's **Privacy & Security** category brings together local browsing data, site permissions, pop-up behavior, the tracker shield, and available safe-browsing protection.

## Local browsing history

**Clear local browsing history** removes the Veloce profile's navigation history. It does not remove cookies, website storage, browser cache, or downloads.

Automatic history retention defaults to **3 months**. Choose 1, 3, 6, or 12 months, or **Always** to keep history without automatic expiration. Expired entries are removed when a visit is recorded, when the retention preference changes, and when the browser starts.

## Safe browsing

On Windows, the setting for Microsoft Defender SmartScreen controls WebView2 reputation checking and is enabled by default when the runtime supports it. WebView2 may send URLs and related information to Microsoft to check a page's reputation. Windows or organization policies can override the browser setting. See Microsoft's [WebView2 data privacy documentation](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/data-privacy?tabs=dotnetcsharp).

This control is unavailable on Linux and on Windows WebView2 runtimes that do not expose the required setting. Veloce does not substitute or simulate this protection on unsupported platforms.

## Site protection

The pop-up setting controls whether web pages may open HTTP or HTTPS destinations as new Veloce tabs. It is off by default, preserving the browser's existing blocked behavior. Veloce's own picture-in-picture window is handled separately and remains available when web pop-ups are disabled.

The tracker shield and saved site permission decisions are managed in this category as well. Site permissions cover the permission types exposed by the current platform and browser runtime.

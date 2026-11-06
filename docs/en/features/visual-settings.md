# Toolbar and content appearance

Open **Settings → Appearance & Layout → Toolbar and content** to adjust the address bar and the main WebView frame. Changes apply immediately and are saved locally for the next launch.

- **Use a wide address bar** removes the 980 px width cap so the address bar can use the free space between toolbar controls. It still shrinks to fit narrow windows and split view.
- **Always show full URLs** keeps `https://` and `www.` visible while the address bar is idle. When disabled, those prefixes are hidden for HTTP and HTTPS addresses; focusing the field always reveals the full editable address. Navigation continues to use the original URL.
- **Use compact mode** restores the current smaller toolbar sizes: 40 px toolbar height and 28 px address field. Normal mode is the default and uses 44 px and 32 px, with proportionally larger controls. The responsive action menu in narrow split view remains independent.
- **Show rounded content corners** restores the border, rounded corners, and shadow around the main WebView area. It is off by default. Zen mode remains edge-to-edge regardless of this preference.

The preferences are stored in local interface storage and do not reload tabs.

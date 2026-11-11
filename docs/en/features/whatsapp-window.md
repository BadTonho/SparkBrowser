# WhatsApp Popover

The titlebar includes a monochrome WhatsApp button, available on Windows in this release. It opens WhatsApp Web inside the browser window. Choose its display in **Settings → Appearance & Layout → WhatsApp Display**; the choice is saved across restarts.

## Behavior

* **Compact popover** is the default. It opens below the button, right-aligned when possible, at up to 560 × 680 CSS pixels and shrinks to fit the browser window.
* **Side panel** opens below the titlebar along the right edge. The page area, including split panes, resizes to make room and returns to its prior size when the panel closes.
* Click the button to show or hide WhatsApp.
* Scan the QR code once. The login is kept in the browser's default profile and survives restarts.
* Hiding the popover keeps the connection and notifications active.
* The popover is not listed in tabs, history or saved sessions, and it cannot run browser commands.
* Links that try to open new windows are blocked.

## Limits

* Clicking elsewhere in the browser does not hide WhatsApp yet; use the button.
* Available on Windows only. The button is hidden on other systems.
* WhatsApp Web requires a phone with WhatsApp linked to the account.

# WhatsApp Popover

The titlebar includes a WhatsApp button, available on Windows in this release. It opens WhatsApp Web in a popover anchored under the button, inside the browser window.

## Behavior

* Click the button to show the popover; click it again to hide it.
* Scan the QR code once. The login is kept in the browser's default profile and survives restarts.
* Hiding the popover keeps the connection and notifications active.
* The popover is not listed in tabs, history or saved sessions, and it cannot run browser commands.
* Links that try to open new windows are blocked.

## Limits

* Clicking elsewhere in the browser does not hide the popover yet; use the button.
* Available on Windows only. The button is hidden on other systems.
* WhatsApp Web requires a phone with WhatsApp linked to the account.

# Spark 0.1.5

## New features

- **Autofill in pages:** click a login, card or contact field and pick a saved item to fill the form. Logins are offered only on their own site, cards never fill the security code, and with Windows Hello set to be asked each time it is asked before a login or card is filled. Switches to turn it off and to allow it in private windows (a read-only copy of the vault).
- **Autofill & passwords** section in Settings with **Payments** (saved cards, the security code is never stored and opening a card asks for your master password or Windows Hello) and **Contact info** (names, emails, phones and addresses), both kept in the encrypted vault next to your passwords.
- Drag a tab out of the browser window and release it to open it in a new window, placed where you dropped it.

## Improvements

- Opening Spark again while it is already running now opens a new browser window in the running instance, so one window per monitor can be started from the shortcut as well.

## Fixes

- Fixed Settings tabs sometimes failing to display when opened or switched quickly.
- A second launch with the same browser profile no longer crashes and leaves the first instance running.
- Stopped reporting cleanup failures when the staged update folder or Windows Hello key was already absent.
- Stopped polling for page media while Settings is the active tab.
- Fixed **Return to tab** in the floating video player so it closes the pop-up and restores the video in its original tab.

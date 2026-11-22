# Spark 0.1.5

## New features

- **Downloads** section in Settings: choose the folder downloads are saved to (and go back to the default), keep or turn off the question about where to save each file, and choose whether the downloads panel opens when a download finishes. The question about where to save moved here from Performance.
- **Autofill in pages:** click a login, card or contact field and pick a saved item to fill the form. Logins are offered only on their own site, cards never fill the security code, and with Windows Hello set to be asked each time it is asked before a login or card is filled. Switches to turn it off and to allow it in private windows (a read-only copy of the vault).
- **Save what you type:** after you submit a login, payment or address form, Spark asks whether to save it (or update a changed password). It asks only with the vault unlocked, never in private windows, and does not repeat a question you declined.
- **Autofill & passwords** section in Settings with **Payments** (saved cards, the security code is never stored and opening a card asks for your master password or Windows Hello) and **Contact info** (names, emails, phones and addresses), both kept in the encrypted vault next to your passwords.
- Drag a tab out of the browser window and release it to open it in a new window, placed where you dropped it.
- **Profiles:** Settings has a new Profiles section to create separate profiles, each with its own history, favorites, cookies, logins, password vault and settings, and open them in their own windows (several at once). You can rename and delete them; a profile other than the Default one shows its name in the window title. When there is more than one profile, a window asks which one to open when Spark starts (with an option to always ask).
- **App lock:** each profile can ask for a secret when it opens (Settings → Profiles → App lock): a password of its own, Windows Hello or the vault master password. A locked profile shows only the lock screen, with no tab or page behind it, and wrong secrets are slowed down. It blocks access only and does not encrypt data. If the secret is forgotten, the lock screen can erase the profile and start it again empty. Updates are installed by the Default profile.
- **Optional 2FA codes in the password manager:** a login can keep its two-factor secret (paste the setup key or the `otpauth://` link the website shows, or read its QR code from a picture) and the six-digit code appears next to the password, with a countdown and a copy button. The secret is encrypted with the login, and the code is computed by the browser. Logins without it are unchanged.

## Improvements

- Glass uses neutral backgrounds, more transparency and clearer borders. In dark mode, its base is black, with no reflections.
- Quick Settings has stronger outlines and clearer labels in both themes.
- Choose the YouTube widget's thumbnail layout and preview mode in a more compact panel with example cards. The muted video example moves on hover or keyboard focus.
- See where the YouTube widget appears below the vertical tabs in the sidebar example.
- Frame images and videos for the new tab in a larger preview with a grid. Scroll to zoom, drag smaller media to the corners, and hover over the **?** icon for control tips.
- Hold Shift while dragging background media to snap it to the center, with alignment guides.
- Opening Spark again while it is already running now opens a new browser window in the running instance, so one window per monitor can be started from the shortcut as well.

## Fixes

- Fixed Settings tabs sometimes failing to display when opened or switched quickly.
- A second launch with the same browser profile no longer crashes and leaves the first instance running.
- Stopped reporting cleanup failures when the staged update folder or Windows Hello key was already absent.
- Stopped polling for page media while Settings is the active tab.
- Fixed **Return to tab** in the floating video player so it closes the pop-up and restores the video in its original tab.

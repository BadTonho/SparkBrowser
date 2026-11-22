# Profiles

[Português](../../pt-BR/features/profiles.md)

A profile is a separate copy of everything Spark keeps for you: history, favorites, cookies and site data, logins, the [password vault](./password-manager.md), downloads list and settings. Nothing is shared between profiles. Use them to keep work and personal browsing apart, or to let two people share a computer account.

Open **Settings → Profiles** to manage them.

## What you can do

- **See your profiles.** The list shows every profile and marks the one this window uses (**In use**). The profile Spark has always used is called **Default**.
- **Create** a profile by selecting **Create profile** and entering a name in the dialog. Select the avatar to choose or replace a PNG, JPEG or WebP photo. The avatar shows the name's initial when there is no photo. Enter a password of 8 to 256 characters to enable the [app lock](./app-lock.md) from the first time you open the profile. Use the eye icon inside the field to show or hide the password. Leave the password blank to open the profile without a password. A new profile starts empty. Names have 1 to 40 characters, no line breaks, and cannot repeat (upper and lower case count as the same). You can have up to 10 profiles.
- **Open** a profile in its own window. Each profile runs as its own Spark process, so you can have several open at once. Opening one that is already open brings a new window of it instead of starting it twice.
- **Rename** a profile. The Default profile cannot be renamed.
- **Delete** a profile. Spark asks you to type its name, then erases its history, favorites, cookies, logins, passwords and settings **from this computer, for good**. You cannot delete the Default profile, the profile you are using, or a profile that is open in another window (close it first).

Windows of a profile other than the Default one show the profile name in their title ("Spark - Work").

Each profile can also be locked with a password, Windows Hello or the vault master password: see [App lock](./app-lock.md).

## Choosing a profile when Spark starts

- The first time you have more than one profile, Spark asks **which profile to open**. Select a card to open that profile in its own window. Each card shows the profile photo or initial, and Spark marks the last profile you chose. Starting Spark with a single profile never asks.
- The window has **Always ask when Spark starts**. Left off, the next starts open the profile you chose last, with no window in between. Turned on, the window shows every time. You can change this later in **Settings → Profiles → Ask which profile to open when Spark starts**.
- If the profile you chose last was deleted, Spark asks again.
- Starting a profile directly (for example from a shortcut with `--profile <id>`) skips the question.
- The window only asks: it has no history, tabs or settings of its own, and it keeps its own small folder (`launcher`) away from every profile.

## Good to know

- **Updates are installed by the Default profile.** Two profiles open at once must not run the same installer, so other profiles do not install updates; their **About & Updates** page says updates are not available there. Open the Default profile now and then to keep Spark up to date.
- **Each profile has its own password vault** and its own Windows Hello key for it, so unlocking the vault of one profile never opens another one.
- **Backups, WebDAV/Drive sync and private windows belong to the profile you are in.** A backup contains only that profile's data.
- **Private windows are not profiles.** They are disposable and copy favorites and settings from the profile they were opened from. Profiles cannot be managed from a private window.
- The Default profile keeps living in the same folder as before, so nothing moves or changes for you when you update.

## Where the data lives

The Default profile stays where Spark has always stored its data. Every other profile has its own folder inside the local data folder of the Default profile, under `profiles/<id>/`, with its own database, logs, cache and web engine data. Each folder has a `profile.json` with the profile's name and its photo thumbnail if one was chosen. The app-lock password verifier is kept in the profile's database.

## How it works (for developers)

- A profile other than the Default one starts with `--profile <id>`. Before any window exists, the process redirects every application directory (config, data, local data, cache, logs) into the profile folder, the same mechanism private windows use, so the WebView2 or WebKit data is isolated too. The id is 12 lowercase hex characters made by Spark and validated before it reaches any path.
- The data folder of each profile holds its own `profile.lock`. Opening a profile asks the running process (through the instance link) for a new window and only starts a new process when none is running. Deleting takes the same lock, which is how a running profile is refused.
- Failures are written to the [error log](error-log.md) with what failed and where (`profiles` area).
- Things that were global had to become per profile: the Windows Hello key of the vault (named after the profile; the Default profile keeps the name it always had) and the updater (Default only). Backup credentials were already per profile, because their key comes from the database path.

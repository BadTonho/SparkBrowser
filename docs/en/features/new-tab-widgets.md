# New-tab widgets

Open **Customize new tab** in the bottom-right corner to configure **Weather**, **Gmail**, **Wallpaper**, and **Clock**. It opens as a compact popover right above the button, presenting a clean vertical menu of options (**Weather**, **Gmail**, **Wallpaper**, and **Clock**). Selecting an option smoothly transitions to its dedicated configuration view with a back button to return to the main list. Preferences are local to the device, shared across workspaces and restored after restart. Weather sits in the upper-left corner and Gmail in the upper-right; narrow panes stack them above search. Small windows scroll. The settings dialog supports Tab, Escape and outside clicks, returning focus to its button when closed. Search drafts and recent sites remain intact while widgets update.

## Weather

Choose a city from the search results, including its region and country, or select **Use current location**. Windows obtains a one-time position after system consent; there is no IP-based location or continuous tracking. Automatic coordinates remain in memory. Denied or unavailable location leaves manual city selection available. Other platforms support manual selection.

The widget shows current temperature, conditions, feels-like temperature and three forecast days. Celsius is the default; Fahrenheit converts locally. Forecasts update every 15 minutes while a new-tab page is visible, with one shared in-memory cache. Failed updates retain the last forecast and show an error. **Refresh** retries without reloading the document. No weather requests run with the widget disabled.

Forecasts and city search use [Open-Meteo](https://open-meteo.com/). Coordinates are sent to the forecast provider; city names are sent to its geocoding service. The free endpoint is intended here for personal, non-commercial use; [commercial use requires the appropriate plan](https://open-meteo.com/en/pricing). Icons are packaged local SVGs. Attribution opens the provider's website in the current pane.

## Gmail integration

The widget supports two connection modes:

### 1. Direct Browser Session (Default & Recommended)
Veloce configures WebViews with a standard desktop browser User-Agent, allowing seamless Google sign-in directly within browser tabs without third-party app warnings. When you are signed into Gmail (`mail.google.com`) in any normal tab, the widget automatically detects the session cookies and displays your unread count and latest messages via the inbox feed. No Google Cloud Console project or credentials are required. Clicking **Connect Gmail** navigates the current tab to `https://mail.google.com/` to sign in.

### 2. Desktop OAuth Setup (Optional / Automated Testing)
For automated native testing fixtures or developer OAuth usage:
1. Create a project in [Google Cloud Console](https://console.cloud.google.com/) and enable the Gmail API.
2. Configure the OAuth consent screen for personal testing and add your Google account as a test user. Add only `https://www.googleapis.com/auth/gmail.metadata`.
3. Create an OAuth client of type **Desktop app** and download its JSON configuration.
4. Place the downloaded file at `<application-config-directory>/oauth/google-desktop.credentials.json`. On Windows the normal application config directory is `%APPDATA%/org.veloce.browser`; development and isolated test profiles can use different directories. Restart Veloce after adding or changing this file.
5. Enable **Show Gmail** and select **Connect Gmail**. Authorization opens Microsoft Edge outside Veloce, then returns through a temporary `127.0.0.1` callback. Finish consent and return to Veloce. The connection can be cancelled from settings.

Never put the downloaded configuration, account tokens or exported user data in the repository. The app reads the standard Google `installed` configuration and uses fixed Google authorization/API endpoints. It does not ask for a Google password. Missing configuration or unavailable secure storage produces an explanatory state rather than a simulated connection.

The widget supports one account and shows the inbox's unread-message count and five newest messages, including sender, subject, date and unread indication. Only metadata is requested: bodies, attachments and remote images are not downloaded. **Open Gmail** navigates the originating pane to the account's inbox; Gmail may still require a separate website login. The widget cannot send, delete or mark messages read.

Refresh tokens use Windows Credential Manager, isolated by application/profile. Access tokens and inbox summaries stay in memory. Mail updates every 60 seconds while a new-tab page is visible; pages share requests and failures back off. Turning off the widget stops requests but preserves its connection. **Disconnect** immediately clears local account state, deletes its saved credential and attempts revocation. Expired or revoked access offers reconnection.

Google's testing mode can [expire refresh tokens after seven days](https://developers.google.com/identity/protocols/oauth2). `gmail.metadata` is a [restricted scope](https://developers.google.com/workspace/gmail/api/auth/scopes); public distribution requires a separate verification/readiness process. Other providers and public-release onboarding are outside this first version. Gmail connection currently requires Windows secure storage and Microsoft Edge.

## Wallpaper (Images and Videos)

You can set a custom background image or looping video from your device through the **Customize new tab** panel (`⚙️`).
- **Choose image or video**: Opens the system file picker to select an image (`.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`, `.bmp`, `.avif`) or video (`.mp4`, `.webm`, `.ogg`).
- **Video Playback**: Videos play automatically on loop, muted, and scaled to fill the viewport (`object-fit: cover`) behind the central search area and widgets. Files up to 50 MB are supported.
- **Image Scaling & Optimization**: High-resolution images are scaled and optimized locally via HTML Canvas to preserve smooth rendering performance and minimize memory usage.
- **Background dimming**: A slider controls the dark overlay level (0% to 90%, default 40%) to ensure all search text, widgets, and recent-site cards maintain crisp readability.
- **Glassmorphism**: When a wallpaper is active, widgets and search cards gain subtle translucency and backdrop blur.
- **Cross-tab sync & Persistence**: The chosen media is stored locally in device storage (IndexedDB Blobs and local preferences) and updates across all open new tabs in real time without entering backups.
- **Remove**: Deleting the wallpaper immediately cleans up object URLs and restores the default solid background.

## Clock

The customizable clock widget provides local, offline time presentation on the new tab page:
- **Enabled by Default**: Starts active and positioned centered in the lower area of the page below the recent shortcuts.
- **Position Options**: Selectable via the Clock settings category:
  - **Bottom (centered)** (default): Spacious display positioned below recent shortcuts.
  - **Above search**: Center dashboard style prominent time display above the search form.
  - **Widget card**: Rectangular card in the upper widgets grid alongside Weather and Gmail.
- **Font Selection**: Choose between **System default**, **Digital (Monospace)**, **Classic (Serif)**, **Modern rounded**, and **Elegance display**. All fonts render with tabular figures (`font-variant-numeric: tabular-nums`) to prevent digit jitter.
- **Format Options**: Toggle between 24-hour (`14:30`) and 12-hour (`2:30 PM`) format.
- **Seconds & Date**: Toggles to display live seconds and localized date formatting (e.g. `Thursday, November 5`).
- **Dynamic Greeting**: Optional greeting based on the local time of day (*Good morning*, *Good afternoon*, *Good evening*, or *Good night*).
- **Efficiency**: Updates align with the next whole minute when seconds are disabled, eliminating unnecessary timer overhead and conserving battery. Sychronizes in real time across open tabs through local storage events.

## Data and validation

An idempotent SQLite migration stores only widget preferences and the manually selected city. Backups remain version 4 and exclude these settings, OAuth configuration, coordinates, credentials and mail. The local document keeps `connect-src 'none'`: Rust performs remote requests and sends safe display data through the existing authenticated document bridge. Old responses cannot restore disabled widgets or disconnected mail. Opening settings cancels tab-drag previews without hiding or reloading pages.

Run `npm run validate` and the isolated `new-tab`, `keyboard`, `split-tab-groups`, `workspace-switcher` and `backup` scenarios. Widget tests use generated loopback weather/Gmail/OAuth fixtures with artificial accounts, never real authorization or the user's credential vault. Real Windows location consent and authorization with an owner-provided Google test account remain manual checks. See [native tests](../contributing/native-tests.md).

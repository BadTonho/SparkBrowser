# New-tab widgets

Open **Customize new tab** in the bottom-right corner to enable **Weather** and **Gmail**. Both start disabled. Preferences are local to the device, shared across workspaces and restored after restart. Weather sits in the upper-left corner and Gmail in the upper-right; narrow panes stack them above search. Small windows scroll. The settings dialog supports Tab, Escape and outside clicks, returning focus to its button when closed. Search drafts and recent sites remain intact while widgets update.

## Weather

Choose a city from the search results, including its region and country, or select **Use current location**. Windows obtains a one-time position after system consent; there is no IP-based location or continuous tracking. Automatic coordinates remain in memory. Denied or unavailable location leaves manual city selection available. Other platforms support manual selection.

The widget shows current temperature, conditions, feels-like temperature and three forecast days. Celsius is the default; Fahrenheit converts locally. Forecasts update every 15 minutes while a new-tab page is visible, with one shared in-memory cache. Failed updates retain the last forecast and show an error. **Refresh** retries without reloading the document. No weather requests run with the widget disabled.

Forecasts and city search use [Open-Meteo](https://open-meteo.com/). Coordinates are sent to the forecast provider; city names are sent to its geocoding service. The free endpoint is intended here for personal, non-commercial use; [commercial use requires the appropriate plan](https://open-meteo.com/en/pricing). Icons are packaged local SVGs. Attribution opens the provider's website in the current pane.

## Gmail setup for personal testing

1. Create a project in [Google Cloud Console](https://console.cloud.google.com/) and enable the Gmail API.
2. Configure the OAuth consent screen for personal testing and add your Google account as a test user. Add only `https://www.googleapis.com/auth/gmail.metadata`.
3. Create an OAuth client of type **Desktop app** and download its JSON configuration.
4. Place the downloaded file at `<application-config-directory>/oauth/google-desktop.credentials.json`. On Windows the normal application config directory is `%APPDATA%/org.veloce.browser`; development and isolated test profiles can use different directories. Restart Veloce after adding or changing this file.
5. Enable **Show Gmail** and select **Connect Gmail**. Authorization opens Microsoft Edge outside Veloce, then returns through a temporary `127.0.0.1` callback. Finish consent and return to Veloce. The connection can be cancelled from settings.

Never put the downloaded configuration, account tokens or exported user data in the repository. The app reads the standard Google `installed` configuration and uses fixed Google authorization/API endpoints. It does not ask for a Google password. Missing configuration or unavailable secure storage produces an explanatory state rather than a simulated connection.

The widget supports one account and shows the inbox's unread-message count and five newest messages, including sender, subject, date and unread indication. Only metadata is requested: bodies, attachments and remote images are not downloaded. **Open Gmail** navigates the originating pane to the account's inbox; Gmail may still require a separate website login. The widget cannot send, delete or mark messages read.

Refresh tokens use Windows Credential Manager, isolated by application/profile. Access tokens and inbox summaries stay in memory. Mail updates every 60 seconds while a new-tab page is visible; pages share requests and failures back off. Turning off the widget stops requests but preserves its connection. **Disconnect** immediately clears local account state, deletes its saved credential and attempts revocation. Expired or revoked access offers reconnection.

Google's testing mode can [expire refresh tokens after seven days](https://developers.google.com/identity/protocols/oauth2). `gmail.metadata` is a [restricted scope](https://developers.google.com/workspace/gmail/api/auth/scopes); public distribution requires a separate verification/readiness process. Other providers and public-release onboarding are outside this first version. Gmail connection currently requires Windows secure storage and Microsoft Edge.

## Data and validation

An idempotent SQLite migration stores only widget preferences and the manually selected city. Backups remain version 4 and exclude these settings, OAuth configuration, coordinates, credentials and mail. The local document keeps `connect-src 'none'`: Rust performs remote requests and sends safe display data through the existing authenticated document bridge. Old responses cannot restore disabled widgets or disconnected mail. Opening settings cancels tab-drag previews without hiding or reloading pages.

Run `npm run validate` and the isolated `new-tab`, `keyboard`, `split-tab-groups`, `workspace-switcher` and `backup` scenarios. Widget tests use generated loopback weather/Gmail/OAuth fixtures with artificial accounts, never real authorization or the user's credential vault. Real Windows location consent and authorization with an owner-provided Google test account remain manual checks. See [native tests](../contributing/native-tests.md).

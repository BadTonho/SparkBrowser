# Spotify widget

[Português](../../pt-BR/features/spotify-widget.md)

Open **Customize new tab → Spotify**, enable **Show Spotify**, and connect one Spotify account. The card appears below Weather on the left, or at the top of that column when Weather is disabled. Narrow windows stack the left column before Gmail and search.

The card shows artwork, title, artist or podcast, active device and sampled playback progress. Previous, play/pause and next control the account's active Spotify device. Open Spotify on your computer or phone and start playback first. **Open Spotify** opens the Web Player in the current pane. The widget does not stream audio, choose devices, change volume, search tracks or edit playlists. Spotify can restrict individual controls on some devices or content.

## Connect your account

1. Use a Spotify Premium account and create an application in the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard), selecting Web API.
2. Register `http://127.0.0.1/callback` as the redirect URI without a port. Spotify permits dynamically assigned ports for loopback IP literals; `localhost` is not supported. See [redirect URI requirements](https://developer.spotify.com/documentation/web-api/concepts/redirect_uri).
3. Add your account to the application's allowed users where required. Development mode currently permits up to five authenticated users and requires the application owner to have Premium. See [quota modes](https://developer.spotify.com/documentation/web-api/concepts/quota-modes).
4. Paste the application's **Client ID** into Spotify settings and select **Save Client ID**, then **Connect Spotify**. No client secret is needed.
5. Complete consent in the external browser and return to Spark. The requested scopes are `user-read-playback-state` and `user-modify-playback-state`. Authorization uses PKCE and a temporary loopback listener with state validation; it can be cancelled or time out after two minutes.

On Windows, authorization opens Microsoft Edge and refresh tokens use Windows Credential Manager. On Linux, install `secret-tool` (the distribution's libsecret tools package), run an unlocked Secret Service such as GNOME Keyring or KWallet's compatible service, and install an external browser available on PATH: Firefox, Chromium, Chrome or Edge. Missing browser or secure storage produces an error; tokens are never saved to a plaintext file.

## Refresh, disconnect and privacy

Playback is fetched every ten seconds while a normal new-tab page is visible, with shared requests across tabs and workspaces. Controls trigger an immediate refresh. Updates stop while disabled or hidden. Progress reflects the last server sample. Failed updates preserve the last display and temporarily disable controls; rate limits respect `Retry-After`. Expired authorization requests reconnection. Access-denied errors can indicate missing Premium, an account outside the allowed users, or device restrictions.

Disabling the widget preserves the connection. **Disconnect** clears local account data, removes the stored credential and records a durable disconnection marker so failed deletion cannot restore it. To revoke Spotify-side consent too, remove Spark's configured application from your Spotify account's Apps page. Changing Client ID requires reconnecting.

Preferences and Client ID are device-local and excluded from backups. Refresh tokens are isolated by application/profile and provider; access tokens and playback data stay in memory. Credentials never enter the renderer. Rust fetches bounded artwork only from Spotify's image host and passes it as image data, preserving the new-tab page's `connect-src 'none'` policy. Private windows do not connect, fetch or show Spotify account data.

Automated tests use artificial OAuth and playback fixtures; they never open real consent or use the system credential vault. Real account consent and system-vault integration require a manual check on the corresponding operating system.

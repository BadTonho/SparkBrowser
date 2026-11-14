# Spark Browser pending features roadmap

This roadmap organizes the 36 features in Parts 1 and 2 of [pending browser features](../features/browser-missing-features.md) into work stages. The proposed order groups related features and dependencies; it is not a schedule and does not establish that each feature is entirely missing.

## Execution principles

- Before implementing a feature, check what already exists and fix gaps without duplicating infrastructure.
- Keep Windows and Linux in scope. Validate WebView and system integrations on both platforms.
- Preserve project security, privacy and accessibility requirements.
- Resolve technical details during the work. Present limitations requiring a scope change or product decision.
- Do not silently reduce coverage when an API or platform cannot meet the expected scope.

## Stage 0 — Inventory and scope validation

Review the 36 items and record their current state, existing entry points, dependencies and platform feasibility. Classify each as already met, partially met, awaiting implementation or requiring technical investigation. Record the product decisions required before implementing Part 1 items.

## Stage 1 — Tab search and organization

- Keyword search shortcuts: check existing storage, command and settings flows; fix confirmed gaps only.
- Quick tab search: filter available tabs and support keyboard selection, respecting workspaces and privacy.
- Multiple tab selection: select with Ctrl/Shift and apply compatible actions within the current workspace.
- Tab list scrolling: improve scrolling in the existing layout without assuming horizontal orientation.

**Verification:** shortcut conflicts, keyboard selection, workspace boundaries, batch-action protections and confirmations, and scrolling in supported layouts.

## Stage 2 — Quick actions and local utilities

- Calculator and quick conversions: start with local arithmetic and common units; do not use `eval` or a remote service.
- Page QR code: generate locally for the current URL and offer copy or save.
- Quick bar actions: extend the existing interface while preserving confirmations for destructive actions.

**Decisions:** choose initial calculator units and bar actions considering existing features and later stages.

**Verification:** invalid and unrepresentable input, keyboard accessibility and offline behavior.

## Stage 3 — Hibernation, media and downloads

- Hibernation exceptions: configure domain exceptions without changing manual hibernation.
- Global media controls: extend the existing view to media in other tabs and frames where supported.
- Choose a destination for each download: integrate the preference and handle cancellation, collisions and native downloads.

**Decisions:** confirm domain and subdomain matching and the default download destination preference.

**Verification:** domain normalization, media outside the active tab and download flows on both platforms.

## Stage 4 — Engine and WebView dependent features

- Full-page capture: investigate long pages, fixed content and scrolling; `CapturePreview` alone does not establish complete coverage.
- Forced dark page mode: offer an opt-in setting and site exception; assess images, videos and already-dark pages.
- Caret browsing: implemented with F7 and native WebView2/WebKitGTK support, with temporary session state shared across tabs. The Windows native scenario passed; Linux native validation remains pending. See [documentation](../features/caret-browsing.md).
- Minimum font size: documentation investigation completed; WebKitGTK exposes a native setting and documented WebView2 interfaces do not list an equivalent. The Windows visual probe compiles, but its WebDriver session does not start; Linux validation is pending. Do not use `min-font-size` as a CSS property. See [investigation](../features/minimum-font-size-investigation.md).

**Feasibility gate:** investigate Windows and Linux before implementation or reduced coverage. Present limitations and alternatives for a decision if the proposed scope is infeasible.

## Stage 5 — Offline experience and local customization (deferred)

- Offline mini-game: show a simple local game after detecting a connection failure, without network access or data collection, while preserving recovery actions.
- User interface CSS: allow an optional local file and define a recovery mechanism if styling makes the interface unusable.

**Verification:** distinguish connection failures from other errors, keep reload and recovery navigation accessible, and prove the interface can be restored after CSS failure.

## Stage 6 — Identity, credentials and personal data

- Persistent account containers: define identity isolation and behavior across tabs, workspaces and restarts; prove feasibility per platform.
- Password manager: design the vault, unlock, autofill and recovery; submit the security model to audit before storing real credentials.
- Multi-device synchronization: select architecture and service, synchronized data, costs and encryption requirements. (Do not implement.)
- AI assistant: define local or external processing, providers, costs, keys and consent for content sharing. (Do not implement.)

**Decision gate:** settle identity, data and security scope before choosing persistent formats, synchronization or external integrations. Vault implementation depends on security design and audit.

## Stage 7 — Network privacy and safe browsing

- Tor browsing: define the privacy objective and window or session isolation; demonstrate leak prevention before promising anonymity.
- DNS over HTTPS: define provider, configuration, failure and exceptions; prove WebView traffic coverage on both platforms.
- HTTPS-only mode: define defaults, warnings, exceptions and HTTP fallback; validate WebView detection and coverage.

**Decision gate:** confirm protection level and failure behavior before enabling network protection by default. Check routing, DNS resolution and WebView traffic interactions.

## Stage 8 — Tracking and malicious-content protections

- Advanced rule-based blocker: define protection, lists, updates and acceptable platform differences; evaluate technical integration.
- Tracking parameter removal: define the default setting, covered navigations and exceptions for links and authentication.
- Homograph and typosquatting warning: define protection level, warning experience and false-positive tolerance.
- Global Privacy Control (GPC): define default setting, site exceptions and behavior on platforms with limited coverage.
- Cryptomining blocking: define list sources, updates, coverage and exceptions; do not assume host blocking covers every script.

**Verification:** document coverage and limitations, test exceptions and avoid unexpected blocking or warnings in legitimate flows. See [tracking protections](../features/tracking-protection.md) for implementation and outstanding work.

## Stage 9 — WebView features, extensions and languages

- Custom advanced picture-in-picture: assess gaps in existing infrastructure and which extra capabilities justify a custom solution and fallback behavior.
- Chrome Web Store extension installation: define compatibility, permissions and security commitments; prove Linux feasibility and behavior with its different engine.
- Page translation: choose local processing, external service or both; define languages, downloads, costs and consent to send text.

**Decision gate:** confirm coverage and security model before promising extension compatibility or sending content to external services. Validate engine and WebView differences on both platforms.

## Stage 10 — Organization, feeds and system integration

- Tab groups: define workspace relationship, moving and restoration behavior, and session persistence before the data model.
- RSS reader: define feed registration, update frequency, retention, privacy and remote content handling.
- Experimental flags: define activation, stable release exposure, support limits and security.
- Automatic opening by file extension: define consent and allowed types; restrict executables and potentially dangerous files.
- OS taskbar shortcuts: define the Linux experience and whether differences from Windows Jump Lists are acceptable.

**Verification:** validate group persistence and restoration, feed updates and remote content, experimental-flag isolation, file-opening confirmation and operating-system behavior.

## Roadmap completion criteria

- Each of the 36 features has been compared with the implementation and has a verifiable result, including when already met.
- Part 1 product decisions are settled before each corresponding implementation.
- Implemented flows respect workspaces, privacy, existing protections and necessary confirmations.
- Platform-dependent features have been evaluated on Windows and Linux, with relevant limitations and differences documented and presented for decision.
- Each feature's documentation describes delivered behavior and known limitations.

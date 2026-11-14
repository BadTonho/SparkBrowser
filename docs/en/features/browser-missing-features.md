# Pending Spark Browser features: decisions

This list separates features according to whether **your decision is needed before implementation**. Routine technical choices, feasibility research, and testing do not necessarily need your intervention. This classification does not authorize implementation; it is for planning discussion and work.

- **Part 1:** product, privacy, security, or support choices need your direction. The options shown are not exhaustive.
- **Part 2:** work can proceed with the proposed scope and defaults, making technical choices during implementation. If research reveals a limitation or material scope change, the decision should be brought back to you.

Windows and Linux remain in scope. Features depending on WebViews or operating-system integration require validation on both platforms; placement in part 2 does not guarantee feasibility or delivery time. This list describes proposals, not confirmed total absence from the product. Previous estimates were removed because they were not validated.

## Part 1 — Need your decision

| Feature | Decision needed before implementation |
| --- | --- |
| Multi-device synchronization | Choose a central service, decentralized architecture, or third-party service; define synchronized data, costs, and encryption requirements. |
| Advanced rule-based blocker | Define the desired protection level, filter lists and updates, and whether limitations or platform differences are acceptable. Assess technical integration afterward. |
| Persistent account containers | Define the expected identity isolation and behavior across tabs, workspaces, and restarts. Platform feasibility must be demonstrated. |
| Password manager | Define vault scope, unlocking through a master password or OS facilities, autofill, and recovery. The security model needs design and audit. |
| AI assistant | Choose local processing, external services, or both; define providers, costs, key use, and consent to send content. |
| Tor browsing | Define the privacy objective, window or session scope, and acceptable limits. Demonstrate isolation and leak prevention before promising anonymity. |
| DNS over HTTPS | Define provider, configuration, failure behavior, and exceptions. Verify whether WebView traffic can be covered on both platforms. |
| Advanced custom picture-in-picture | Define which extra capabilities justify a custom solution and behavior when the WebView does not provide them. PiP infrastructure already exists; assess its gaps first. |
| Chrome Web Store extension installation | Define the compatibility commitment and Linux behavior, where the engine differs, as well as permissions and security handling. Store integration feasibility has not been demonstrated. |
| Page translation | Choose local processing, an external service, or both; define languages, downloads, costs, and consent to send text. |
| Experimental flags | Define who can enable them, whether stable releases expose them, and their support and security limits. |
| Tab groups | Define whether groups belong to a workspace, how moving and restoring tabs works, and whether groups persist between sessions. The data model follows those choices. |
| Tracking-parameter removal | Decide whether it is enabled by default, which navigations it covers, and how exceptions work, since links and authentication flows may depend on parameters. |
| HTTPS-only mode | Define default activation, warning, and whether exceptions or fallback to HTTP are allowed. WebView coverage and failure detection require research. |
| Homograph and typosquatting warning | Define the protection level and warning experience, including tolerance for false positives; a mixed-script check alone does not cover every attack. |
| Global Privacy Control (GPC) | Define default activation and site exceptions, and behavior if a platform cannot send the signal with the expected coverage. |
| Cryptomining blocking | Define list sources and update policy, intended coverage, and exceptions. The existing host blocker does not guarantee blocking every script. |
| RSS reader | Define product scope: feed subscriptions, refresh frequency, retention, privacy, and handling of remote content. |
| Automatic opening by file extension | Define consent, permitted types, and restrictions on executable or potentially dangerous files before automatically opening downloads. |
| OS taskbar shortcuts | Decide the expected Linux experience and whether differences from Windows Jump Lists are acceptable. |

## Part 2 — Can proceed without your prior decision

The defaults below define an initial version; they are not promises of complete coverage. The team must follow project security and accessibility standards and validate the necessary integrations.

| Feature | Initial scope and verification points |
| --- | --- |
| Keyword search shortcuts | The `site_search_shortcuts` table, commands, and settings screen already exist. Check the complete flow and fix only identified gaps, without creating a second table. |
| Quick tab search | Filter available tabs in the existing interface and select results by keyboard; respect current workspace and privacy rules. Check shortcut conflicts. |
| Address-bar quick actions | Add actions to the existing interface; retain confirmations already required for destructive actions. |
| Quick calculator and conversions | Start with arithmetic and common local units, without `eval` or a remote service; choose specific units during implementation. |
| Multiple tab selection | Use Ctrl/Shift and apply compatible actions in the current workspace; retain existing safeguards and confirmations. |
| Tab-list scrolling | Improve scrolling in the existing layout; do not assume a horizontal tab bar. Add arrow buttons only if they make sense in the current layout. |
| Hibernation exceptions | Apply configurable domain exceptions to existing automatic hibernation; preserve the manual command and verify normalization and subdomains. |
| Page QR code | Generate a code locally for the current URL, with copy or save options; handle inputs that cannot be represented. |
| Full-page screenshot | First investigate APIs on Windows and Linux, including long pages, fixed content, and scrolling. `CapturePreview` alone does not establish full-page capture. If the intended coverage is infeasible, present alternatives before reducing scope. |
| Global media controls | Commands and a media panel already exist. Assess what is missing for a global view and reuse the infrastructure; verify media in other tabs and frames on both platforms. |
| Choose destination for each download | Add a preference to the existing download flow, covering cancellation, collisions, and native downloads on both platforms. |
| Offline mini-game | Show a simple local game when a connection failure is identified, with no network access or data collection; preserve access to recovery actions. |
| User interface CSS | Allow an optional local interface stylesheet, with a recovery mechanism if styling makes the interface unusable. |
| Forced dark mode for pages | Offer an option disabled by default and site exceptions; test images, videos, and already dark pages. Report limitations if results are insufficient. |
| Caret browsing | Implemented with native WebView2/WebKitGTK support and F7; session-only, shared across content tabs, and off at startup. Native validation can be run with the documented `caret-browsing` scenario. |
| Minimum font size | Investigate engine support and alternatives with accessibility and layout tests. `min-font-size` is not a standard CSS property; do not use the originally proposed rule. |

## Project references

- [Search shortcuts: SQLite repository](../../../src-tauri/src/database/site_search_repo.rs) and [settings](../../../src/lib/components/settings/SiteSearchSettings.svelte).
- [Media commands](../../../src-tauri/src/commands/media.rs) and [media panel](../../../src/lib/components/media/MediaPanel.svelte).
- [Existing tab list](../../../src/lib/components/tabs/TabList.svelte).

Consult current official documentation for platform capabilities during implementation; for example, [WebView2 `CapturePreview`](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2) and the [Media Session API](https://developer.mozilla.org/en-US/docs/Web/API/MediaSession).

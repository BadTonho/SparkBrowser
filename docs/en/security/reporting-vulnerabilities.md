# Security policy

[Português](../../pt-BR/security/reporting-vulnerabilities.md)

Spark is a desktop browser, so security reports are taken seriously. Thank you for helping keep users safe.

## Reporting a vulnerability

**Do not open a public issue, pull request or discussion for a security problem.** Use GitHub's private vulnerability reporting instead:

1. Open the [Security tab](https://github.com/BadTonho/SparkBrowser/security) of this repository.
2. Choose **Report a vulnerability**.
3. Describe the problem as completely as you can.

The report stays private between you and the maintainers until a fix is available.

### What to include

- The affected version (see **About** in the app, or the release tag) and operating system (Windows or Linux).
- A clear description of the impact: what an attacker can read, change or run.
- Step-by-step instructions or a minimal proof of concept that reproduces the problem.
- Any relevant logs or screenshots. **Remove personal data, cookies, tokens and passwords before attaching them.**

## What to expect

- We aim to acknowledge a report within **7 days**.
- We will keep you informed while we investigate, and tell you whether we consider the report valid.
- Once a fix is released, we will credit you in the release notes if you wish. Tell us how you would like to be named, or ask to stay anonymous.

Spark is maintained by volunteers, so these times are goals and not guarantees.

## Supported versions

Spark is in early development. Security fixes are made for the **latest release** and the `main` branch only. Please reproduce the problem on the latest release before reporting it.

## Scope

In scope:

- The Spark application: the Rust backend, the Svelte interface, the IPC commands and the permissions that connect them (see [Application security boundaries](./application-boundaries.md)).
- Sync and backup code, including secure credential storage.
- Release and update integrity: signed updater artifacts and the release workflow.

Out of scope:

- Vulnerabilities in websites you visit, or in the system WebView engines (WebView2 and WebKitGTK). Report those to their vendors.
- Vulnerabilities in third-party dependencies with no practical impact on Spark. Known upstream warnings are tracked in the [dependency notes](./dependency-notes.md). If you find a way to exploit one in Spark, please report it.
- Attacks that need prior control of the user's operating system account or unlocked device.
- Reports from automated scanners with no demonstrated impact.

## Good-faith research

We will not take legal action against research that follows this policy: test only against your own installation and accounts, avoid accessing other people's data, do not degrade services, and give us reasonable time to fix the issue before disclosing it.

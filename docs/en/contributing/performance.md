# Performance validation

Open **Settings → Local Diagnostics** to inspect this session's IPC counts, failures, mean duration and recent p95, along with native session-write counts and actual resident WebViews. Refresh is manual. These counters remain in memory, retain at most 256 timing samples and contain no page URLs, titles, command arguments or tokens. The p95 covers the samples still retained, not the entire session.

## Repeatable comparisons

Use the same release build, machine, display scaling and local fixture pages for both revisions. Record the commit, OS, Rust/Node versions, WebView engine version and power mode. Warm the fixture once and repeat each run at least five times.

| Scenario | Measure | Expected invariant |
| --- | --- | --- |
| Collapse/expand the sidebar repeatedly | `sync_viewport_bounds` calls and recent duration | At most one resize in flight; identical successful bounds skipped |
| Drag a tab across several neighbors | `session.persist` calls and rows written | One reorder command on release, none for pointer moves |
| Select the same tab again | Session rows written | No unchanged session rows rewritten |
| Switch among loaded tabs | IPC duration and native `tabs.switch` timing | No re-navigation of an existing resident view |
| Open 10, 50 and 100 tabs | Resident views, aggregate process-tree memory | Soft limit respected unless protected tabs require more views |
| Search more than 60 history visits | Result continuity and latency | Cursor pages have no duplicates or missing equal-timestamp visits |

`tabs.switch` measures backend work including the operation-lock wait; IPC also includes transport and frontend scheduling. Neither metric measures first paint. For first-content timing, use a local page with a visible marker and measure from the user action to that marker through native integration tooling.

## Memory and binaries

On Windows, aggregate the Rust application's working set and all descendant WebView2 processes. On other platforms, include engine helper processes as well. Let each scenario settle for the same duration. A WebView count is not a RAM measurement.

Build with `npm run build` and `cargo build --release --manifest-path src-tauri/Cargo.toml`. Compare binary size, clean build time, startup time and tab-switch p95 before enabling thin LTO, changing codegen units or stripping symbols. No custom release profile is enabled solely on an assumed improvement. A cargo release binary is not a signed installer.

The native UI helper could not initialize in the implementation environment (`failed to write kernel assets`, OS error 3). The separate Windows WebDriver suite passed its DOM focus and lifecycle assertions; see [native integration](native-tests.md). This does not establish interactive latency, aggregate RAM, multi-monitor DPI or first-paint results. Native test results must be reported separately from compilation and unit tests.

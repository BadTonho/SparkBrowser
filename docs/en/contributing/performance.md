# Performance validation

For tab switching, `tabs.switch.wait` records time waiting for the session operation lock and `tabs.switch.execute` records the subsequent backend work. The native diagnostics table shows p95 from the shared 256-sample buffer; infrequent operations may have few retained samples. Neither value measures first paint.

For list scaling, compare 100, 500 and 1,000 tabs in the same workspace, including scroll, selection and drag. Record frame times and click-to-paint, and compare against the same profile and hardware before considering virtualization. Check keyboard selection, screen reader access and drag destinations when testing any virtualization change.

Open **Settings → Local Diagnostics** to inspect IPC counts, failures, session mean duration and recent p95, plus native session-write counts and actual resident WebViews. Refresh is manual. Counters stay in memory, retain at most 256 timing samples and contain no URLs, titles, command arguments or tokens. The p95 covers retained samples, not the entire session.

The separate `get_diagnostics.tab_details` inventory includes local tab URLs/titles and workspace IDs. The response also includes the application data directory; remove these fields before sharing it. `native_resources` contains resource counts only: bridge tokens, shield scripts, media entries and Windows media views/runtime contexts. Windows context counts are `null` when unavailable. Compare settled counts with resident views when investigating tab creation/closure; bounded counts do not prove bounded WebView2 heaps.

## Reproducible comparisons

Use the same release build, computer, display scale and local fixture pages for both revisions. Record the commit, operating system, Rust/Node versions, WebView engine version and power mode. Warm up the fixture once and repeat each run at least five times.

| Scenario | Measurement | Expected behavior |
| --- | --- | --- |
| Repeatedly collapse/expand the sidebar | `sync_viewport_bounds` calls and recent duration | At most one resize in flight; identical applied bounds are skipped |
| Drag a tab across several neighbors | `session.persist` calls and rows written | One reorder command on drop, none per pointer movement |
| Select the same tab again | Session rows written | No unchanged row is rewritten |
| Switch between loaded tabs | IPC duration, native `tabs.switch` time and click-to-first-paint | Existing views keep their documents; only changed panes receive geometry or visibility updates on a resident switch |
| Open and close Settings, Quick Settings, Tools and Downloads | Auxiliary WebView count and process-tree memory before opening, immediately after closing and after at least 30 seconds idle | Idle panel views are released and can reopen from the shell's current state |
| Launch with a cold profile and with an existing profile | Time to usable window and native `startup.filters` and `startup.profile` durations | Filter compilation overlaps profile initialization; the first external navigation is protected |
| Open 10, 50 and 100 tabs | Resident views and process-tree memory | Soft cap respected unless protected tabs require more views |
| Search more than 60 history visits | Result continuity and latency | Cursor pages without duplicates or lost visits sharing a timestamp |
| Search a large notes/sessions collection | Query duration, index size and write cost | Same literal/Unicode results; bounded previews; `2+` for additional matches |

`tabs.switch` measures backend work including time waiting for the operation lock; IPC also includes transport and frontend scheduling. Neither measures first paint. To measure first content, use a local page with a visible marker and measure from the user action to that marker using native integration tools.

Run the explicit synthetic comparison with `cargo test --manifest-path src-tauri/Cargo.toml --offline --lib compare_indexed_search_with_review_baseline -- --ignored --nocapture`. It creates 5,000 notes and 1,000 three-page sessions in memory, warms each query once and prints five paired measurements against the reviewed revision's SQL. It verifies identical result IDs without an environment-dependent time threshold. A debug benchmark does not represent native release UI latency; record the compilation profile with the results. See the [optimization implementation](../architecture/optimization-work.md) for the current design.

## Memory and binaries

On Windows, sum the working set of the Rust application and all descendant WebView2 processes. On other platforms include the engine's helper processes. Wait the same settling time in each scenario. WebView counts do not measure RAM.

Build with `npm run build` and `cargo build --release --manifest-path src-tauri/Cargo.toml`. The current release profile uses thin LTO, one codegen unit and stripping; compare binary size, clean build time, startup and tab-switch p95 before changing it. `npm run test:native:build:fast` is an optional isolated test build with lower optimization, no LTO and more codegen units; it is not a substitute for the default release-profile native build used by release validation. A cargo release binary is not a signed installer.

Run `cargo test --manifest-path src-tauri/Cargo.toml --offline --lib measure_session_snapshot_scaling -- --ignored --nocapture` to measure unchanged session saves for 10, 100 and 1,000 artificial tabs in in-memory SQLite. This checks that no rows change and prints the median of ten runs. A local debug run measured approximately 0.062 ms, 0.426 ms and 4.37 ms respectively; these are not disk-backed or UI latency measurements. The incremental transactional implementation is retained until a production-like comparison justifies a more complex alternative.

The native UI helper did not initialize in the implementation environment (`failed to write kernel assets`, operating system error 3). The separate Windows WebDriver suite passed DOM-focus and lifecycle checks; see [native integration](native-tests.md). This does not establish interactive latency, aggregate RAM, cross-monitor DPI or first paint. Report native test results separately from compilation and unit tests.

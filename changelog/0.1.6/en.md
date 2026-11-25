# Spark 0.1.6

## Improvements

- With vertical tabs and split view on, you can now drag a tab from one side of the split to the other in the tab list. Drop it on a tab or on the other side's separator.
- With vertical tabs and split view on, the tab list now has a separator for each side of the split (left pane, right pane and tabs outside the split), so you can tell which tabs belong to which side.
- The error log now says how many times an identical entry was repeated and skipped, shown on the next entry written for it.
- Redesigned the About & Updates settings section with a status summary card, clearer update progress and notices, and side-by-side cards for the browser repository and WebView2 runtime.

## Fixes

- Spark no longer closes right after opening when the saved session has a tab it cannot restore (for example after going back to an older version). That tab is skipped and the problem is written to the error log.
- Errors that stop Spark from saving or closing (shown in the closing dialog) are now written to the error log with the exact failure, so they can be reported and fixed.

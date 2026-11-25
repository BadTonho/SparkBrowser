# Spark 0.1.6

## New features

- New buttons to open and close split view from the vertical tab list. The button at the top of the list opens a split with the tab you are viewing on the left and the most recently used tab on the right (a new blank tab if it is the only one). The button on the line between the panes closes the split.

## Improvements

- The header of the vertical tab list is simpler: it reads only "Tabs (n)" and its buttons (split view, ephemeral tab, new tab) are now the same size and show only an icon; hover one to see its name.
- With vertical tabs and split view on, you can now drag a tab from one side of the split to the other in the tab list. Drop it anywhere on the other side, and the list shows where it will land while you drag. You can also drop it on the other pane's page.
- With vertical tabs and split view on, the tab list now has a line separating the tabs of each side of the split (right side and tabs outside the split), so you can tell which tabs belong to which side.
- The error log now says how many times an identical entry was repeated and skipped, shown on the next entry written for it.
- Redesigned the About & Updates settings section with a status summary card, clearer update progress and notices, and side-by-side cards for the browser repository and WebView2 runtime.

## Fixes

- Dropping the tab shown in one split pane onto the other pane's page no longer swaps the two panes. The tab just moves, and the pane it left shows another of its tabs.
- Spark no longer closes right after opening when the saved session has a tab it cannot restore (for example after going back to an older version). That tab is skipped and the problem is written to the error log.
- Errors that stop Spark from saving or closing (shown in the closing dialog) are now written to the error log with the exact failure, so they can be reported and fixed.

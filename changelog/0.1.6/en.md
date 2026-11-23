# Spark 0.1.6

## Improvements

- The error log now says how many times an identical entry was repeated and skipped, shown on the next entry written for it.

## Fixes

- Spark no longer closes right after opening when the saved session has a tab it cannot restore (for example after going back to an older version). That tab is skipped and the problem is written to the error log.
- Errors that stop Spark from saving or closing (shown in the closing dialog) are now written to the error log with the exact failure, so they can be reported and fixed.

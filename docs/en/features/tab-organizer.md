# Tab organizer

Open **Tools → Tabs** or use the Command Bar. Filter the current workspace or all workspaces by title, URL and exact domain. Duplicate groups compare parsed full URLs, preserving query parameters and fragments.

Select individual eligible tabs or choose **Select extra duplicates**, which keeps the active tab or the most recently used member of each group. Review the selected titles and confirm before closing. Active, pinned, loading, edited-form, playing-media and downloading tabs are excluded. The backend rechecks these protections and reports closed and preserved tabs, including tabs that disappeared during review.

Bulk closing persists once and reconciles native WebViews once, with state restored if persistence fails. It requires the desktop application. Closing the panel returns focus; **Go to tab** closes Tools and selects that tab and its workspace.

The vertical and horizontal tab lists also support multi-selection in the current workspace: Ctrl/Meta-click toggles a tab, and Shift-click selects the inclusive range from the anchor. Keyboard users can use Ctrl/Meta or Shift with Enter or Space. The selection toolbar reviews the chosen tabs before closing; protected tabs are marked and the backend rechecks their state before applying the action. The active tab scrolls into view when it changes.

# Feed reader

The Tools panel provides locally stored RSS and Atom subscriptions. Add a name and HTTP(S) feed URL, choose an interval from 15 to 1,440 minutes (60 by default), and refresh manually or wait for periodic updates while the application runs. Each feed retains up to 200 entries. The last successful entries remain available when a network request or XML parse fails. Editing the feed URL removes items from its previous source; deleting a subscription removes its local items.

Feed responses are limited to 2 MiB. DTDs and non-HTTP(S) item URLs are rejected. Article text is rendered as text, not HTML, and the reader does not load remote images or embedded resources. Opening an item uses the regular browser navigation flow. Feed subscriptions and items are not included in the current backup format.

An optional experimental flag under Local Diagnostics hides article previews in the feed list. The flag starts off, persists locally and applies when the panel is reopened. Resetting experiments disables it.

On 7 November 2026, an isolated Windows native test passed for subscription, refresh, malformed XML retention, restart persistence and experimental flag reset. Native UI and Linux validation remain pending. Other stage 10 items, including tab groups, automatic download opening and OS shortcuts, are not implemented by this feature.

RSS titles and links support CDATA. Nested article text retains whitespace, and Atom links select reader-facing alternatives instead of self/API/enclosure resources. Refreshes share an HTTP client, use at most four concurrent requests, and reject duplicate refreshes for a subscription already in flight. Busy manual refreshes can be retried; failed or skipped scheduled updates remain eligible for a later cycle.

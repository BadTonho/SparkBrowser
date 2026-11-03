# Custom new tab page

Opening a new tab shows a local dark page with a centered search field and up to five recent-search cards. There is no logo or background image. The field receives initial focus; **Ctrl+L** still focuses the shared address bar. Enter or the search button opens the result in the same tab, using the configured Google/DuckDuckGo engine, address detection and site-search shortcuts. Empty input does nothing.

Cards come from the existing browsing history across workspaces, including searches made inside supported search sites. Google `.com` and `.com.br` results and DuckDuckGo's main, HTML and Lite results are recognized by their exact host/path and `q` parameter. The five latest distinct terms are shown, ignoring surrounding spaces and letter case. A card repeats its displayed term using its original engine. Clearing browsing history removes these cards from open new-tab pages too. A read failure leaves the search field available and offers **Try again**.

The page opens offline and loads no remote resources or suggestions. Search terms are sent to a search provider only when a search/card is submitted. Fewer than five searches produce fewer cards; an empty history shows a short message. Cards wrap on narrow panes, including split view and the 680 px minimum window width. Text and keyboard labels follow the selected English/Portuguese language.

The internal logical address is `veloce://newtab`; the address field stays empty and identifies an internal browser page. Site permissions, bookmarking and page zoom are unavailable there. **+**, **Ctrl+T**, an empty startup/workspace and replacement of the last closed tab use this page. Existing saved pages and explicit destinations are preserved. Searching inside a split pane navigates that pane only.

The internal document uses the tab's normal WebView and native history. Back, forward, reload, closed-tab reopening, hibernation, saved sessions, recovery and restart support the internal address. It is not itself a browsing-history visit. Its authenticated document bridge grants no application IPC permissions to web content; stale document/request replies are discarded. Refreshing cards preserves the search draft and does not refocus the field.

There is no database migration or separate search-history store. Backup exports remain version 4 and imports continue accepting versions 1–4. Only tabs and session pages accept this exact internal address; external-data URL restrictions remain in place. Older application builds that only accept HTTP/HTTPS tabs may reject backups containing the new internal page.

Validation: `npm run validate` and the isolated native `new-tab` scenario, with regressions for keyboard, split groups, workspace switching, closed tabs, workspace persistence, backup and recovery. See [native tests](../contributing/native-tests.md).
